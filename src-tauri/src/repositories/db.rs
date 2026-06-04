use crate::error::AppError;
use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::AppHandle;
use tauri::Manager;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

impl DbState {
    pub fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|_| AppError::Db("Database connection lock was poisoned.".into()))
    }
}

pub fn init(app: &AppHandle) -> Result<(), AppError> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| AppError::Config(format!("Failed to get local data dir: {error}")))?;
    std::fs::create_dir_all(&app_dir).map_err(|error| {
        AppError::Store(format!(
            "Failed to create app data directory '{}': {error}",
            app_dir.display()
        ))
    })?;

    let db_path = app_dir.join("voxverse.db");

    // Auto-migrate from older VoxVerse/SpeakMate locations if the new DB doesn't exist yet.
    if !db_path.exists() {
        for legacy_db_path in legacy_database_candidates(&app_dir) {
            if !legacy_db_path.exists() {
                continue;
            }

            if let Err(e) = std::fs::copy(&legacy_db_path, &db_path) {
                log::warn!(
                    "Failed to migrate legacy database from '{}' to '{}': {}",
                    legacy_db_path.display(),
                    db_path.display(),
                    e
                );
            } else {
                log::info!(
                    "Migrated legacy database from '{}' to '{}'.",
                    legacy_db_path.display(),
                    db_path.display()
                );
                break;
            }
        }
    }

    let conn = Connection::open(db_path)?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    // ── Lv.2: Characters ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            language TEXT NOT NULL,
            personality TEXT NOT NULL,
            style TEXT NOT NULL,
            avatar TEXT NOT NULL,
            system_prompt TEXT NOT NULL,
            greeting TEXT NOT NULL,
            voice_lang TEXT,
            voice_name TEXT
        )",
        [],
    )?;

    // ── Lv.3: API config persistence ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // ── Lv.5: Provider profiles ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS provider_profiles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            base_url TEXT NOT NULL,
            model TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )?;
    seed_default_provider_profile(&conn)?;

    // V0.2: preset practice modes that shape the coaching prompt.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS practice_modes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            prompt_suffix TEXT NOT NULL,
            is_preset INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        )",
        [],
    )?;
    seed_preset_practice_modes(&conn)?;

    // ── Lv.3: Chat history ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chat_sessions (
            id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL,
            title TEXT,
            mode_id TEXT,
            scenario_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
        )",
        [],
    )?;
    ensure_column(&conn, "chat_sessions", "mode_id", "TEXT")?;
    ensure_column(&conn, "chat_sessions", "scenario_id", "TEXT")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // ── Lv.3: Vocabulary collection ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS vocabulary (
            id TEXT PRIMARY KEY,
            word_or_phrase TEXT NOT NULL,
            translation TEXT,
            context TEXT,
            notes TEXT,
            character_id TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE SET NULL
        )",
        [],
    )?;

    // ── Lv.3: Learning corrections ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS corrections (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            original_text TEXT NOT NULL,
            corrected_text TEXT NOT NULL,
            explanation TEXT,
            better_expression TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // ── Lv.3: Scenarios ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scenarios (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL DEFAULT '💬',
            learning_goals TEXT,
            is_preset INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    // Seed preset scenarios if none exist
    seed_preset_scenarios(&conn)?;

    // ── V1: Relationship state ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS relationship_state (
            id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL UNIQUE,
            intimacy_level INTEGER NOT NULL DEFAULT 0,
            trust_level INTEGER NOT NULL DEFAULT 0,
            plot_stage TEXT,
            user_preferences TEXT,
            boundaries TEXT,
            commitments TEXT,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // ── V1: Memory facts ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_facts (
            id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL,
            session_id TEXT,
            fact_type TEXT NOT NULL,
            content TEXT NOT NULL,
            source_turn_id TEXT,
            confidence REAL NOT NULL DEFAULT 1.0,
            is_visible INTEGER NOT NULL DEFAULT 1,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // ── V1: Memory embeddings (placeholder for future vector search) ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS memory_embeddings (
            id TEXT PRIMARY KEY,
            fact_id TEXT NOT NULL,
            embedding_model TEXT NOT NULL,
            vector_json TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (fact_id) REFERENCES memory_facts(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // ── V1: Tool invocations ──
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tool_invocations (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            input_json TEXT,
            output_json TEXT,
            error_message TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT,
            FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
        )",
        [],
    )?;

    ensure_v0_3_indexes(&conn)?;

    app.manage(DbState {
        conn: Mutex::new(conn),
    });

    Ok(())
}

fn legacy_database_candidates(app_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = vec![app_dir.join("speakmate.db")];

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        candidates.push(local_app_data.join("com.voxverse.app").join("voxverse.db"));
        candidates.push(
            local_app_data
                .join("com.ai-speaking.desktop")
                .join("voxverse.db"),
        );
        candidates.push(
            local_app_data
                .join("com.ai-speaking.desktop")
                .join("speakmate.db"),
        );
        candidates.push(
            local_app_data
                .join("com.ai-speaking.app")
                .join("speakmate.db"),
        );
        candidates.push(local_app_data.join("SpeakMate").join("speakmate.db"));
    }

    candidates
}

fn ensure_column(
    conn: &Connection,
    table_name: &str,
    column_name: &str,
    column_definition: &str,
) -> Result<(), AppError> {
    if column_exists(conn, table_name, column_name)? {
        return Ok(());
    }

    let sql = format!("ALTER TABLE {table_name} ADD COLUMN {column_name} {column_definition}");
    conn.execute(&sql, [])?;
    Ok(())
}

fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> Result<bool, AppError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table_name})"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

    for row in rows {
        if row? == column_name {
            return Ok(true);
        }
    }

    Ok(false)
}

fn ensure_v0_3_indexes(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_chat_sessions_character_updated
            ON chat_sessions(character_id, updated_at DESC);
         CREATE INDEX IF NOT EXISTS idx_messages_session_timestamp
            ON messages(session_id, timestamp ASC);
         CREATE INDEX IF NOT EXISTS idx_corrections_session_created
            ON corrections(session_id, created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_vocabulary_character_created
            ON vocabulary(character_id, created_at DESC);
         CREATE INDEX IF NOT EXISTS idx_memory_facts_character_visible
            ON memory_facts(character_id, is_deleted, is_visible, updated_at DESC);
         CREATE INDEX IF NOT EXISTS idx_tool_invocations_session_started
            ON tool_invocations(session_id, started_at DESC);",
    )?;
    Ok(())
}

fn seed_preset_practice_modes(conn: &Connection) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let presets: Vec<(&str, &str, &str, &str)> = vec![
        (
            "free_talk",
            "Free Talk",
            "Open-ended conversation for relaxed speaking practice.",
            "Let the conversation flow naturally. Keep the tone friendly, ask follow-up questions, and gently model better English without over-correcting every sentence.",
        ),
        (
            "roleplay",
            "Roleplay",
            "A focused roleplay where the learner practices real-life interaction.",
            "Stay in a clear roleplay setup. Drive the scene with realistic turns, ask context-aware questions, and help the learner practice useful phrases for the situation.",
        ),
        (
            "scenario_drill",
            "Scenario Drill",
            "Structured repetition for one practical situation.",
            "Keep the learner inside the selected scenario. Repeat and vary the same practical language pattern so the learner can improve accuracy and fluency.",
        ),
        (
            "ielts_speaking",
            "IELTS Speaking",
            "IELTS-style spoken answer practice with coaching.",
            "Act as an IELTS speaking examiner. Ask concise Part 1, Part 2, or Part 3 style prompts, then give practical feedback on fluency, vocabulary, grammar, and coherence.",
        ),
        (
            "interview_practice",
            "Interview Practice",
            "Professional interview rehearsal with follow-up questions and expression coaching.",
            "Act as a professional interviewer. Ask focused behavioral and experience-based questions, follow up naturally, and coach the learner toward concise, confident answers.",
        ),
    ];

    for (id, name, description, prompt_suffix) in presets {
        conn.execute(
            "INSERT OR IGNORE INTO practice_modes
                (id, name, description, prompt_suffix, is_preset, created_at)
             VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![id, name, description, prompt_suffix, now],
        )?;
    }

    Ok(())
}

fn seed_preset_scenarios(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM scenarios WHERE is_preset = 1",
        [],
        |row| row.get(0),
    )?;

    if count > 0 {
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339();
    let presets: Vec<(&str, &str, &str, &str)> = vec![
        ("coffee",     "☕", "Coffee Shop",    "You are at a cozy coffee shop. The user is a customer trying to order drinks and food. Guide them through the ordering process naturally. Use casual, friendly language."),
        ("hotel",      "🏨", "Hotel Check-in", "You are a hotel front desk clerk. The user is checking in. Handle their reservation, room preferences, and any questions about the hotel facilities."),
        ("airport",    "🛫", "Airport",        "You are an airport staff member. The user is going through check-in or security. Guide them through the process, ask about luggage, and handle any travel questions."),
        ("doctor",     "🏥", "Doctor Visit",   "You are a doctor at a clinic. The user is describing their symptoms. Ask relevant follow-up questions, show empathy, and give general health advice."),
        ("interview",  "💼", "Job Interview",  "You are an interviewer at a company. Conduct a casual but professional job interview. Ask about the user's experience, skills, and career goals."),
        ("party",      "🎉", "Social Party",   "You are at a house party. The user just arrived. Make small talk, introduce yourself, ask about their interests, and help them feel welcome."),
        ("shopping",   "🛍️", "Shopping",      "You are a helpful sales associate at a clothing store. Help the user find what they're looking for, suggest sizes, colors, and make recommendations."),
        ("restaurant", "🍽️", "Restaurant",    "You are a waiter at a nice restaurant. Take the user's order, explain menu items, make recommendations, and handle any special dietary requests."),
        ("taxi",       "🚕", "Taxi Ride",      "You are a friendly taxi driver. The user just got in your cab. Chat naturally about the city, weather, or local recommendations while driving."),
        ("movie",      "🎬", "Movie Chat",     "You are a movie enthusiast friend. Discuss recent movies, share recommendations, debate opinions, and talk about actors and directors."),
    ];

    for (id, icon, name, description) in presets {
        conn.execute(
            "INSERT INTO scenarios (id, name, description, icon, is_preset, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![id, name, description, icon, now],
        )?;
    }

    Ok(())
}

fn get_app_config_value(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
}

fn seed_default_provider_profile(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM provider_profiles", [], |row| {
        row.get(0)
    })?;

    if count > 0 {
        return Ok(());
    }

    let provider = get_app_config_value(conn, "provider").unwrap_or_else(|| "openai".into());
    let base_url = get_app_config_value(conn, "base_url")
        .unwrap_or_else(|| "https://api.openai.com/v1".into());
    let model = get_app_config_value(conn, "model").unwrap_or_else(|| "gpt-4o-mini".into());
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO provider_profiles
            (id, name, provider, base_url, model, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6, ?6)",
        params!["default", "Default", provider, base_url, model, now],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{column_exists, ensure_column, ensure_v0_3_indexes, seed_preset_practice_modes};
    use crate::error::AppError;
    use rusqlite::Connection;

    fn create_practice_modes_table(conn: &Connection) -> Result<(), AppError> {
        conn.execute(
            "CREATE TABLE practice_modes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                prompt_suffix TEXT NOT NULL,
                is_preset INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    #[test]
    fn seeds_all_v0_2_practice_modes_idempotently() -> Result<(), AppError> {
        let conn = Connection::open_in_memory()?;
        create_practice_modes_table(&conn)?;

        seed_preset_practice_modes(&conn)?;
        seed_preset_practice_modes(&conn)?;

        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM practice_modes", [], |row| row.get(0))?;
        assert_eq!(count, 5);

        let has_ielts: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM practice_modes WHERE id = 'ielts_speaking')",
            [],
            |row| row.get(0),
        )?;
        assert!(has_ielts);

        let has_interview: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM practice_modes WHERE id = 'interview_practice')",
            [],
            |row| row.get(0),
        )?;
        assert!(has_interview);

        Ok(())
    }

    #[test]
    fn migrates_legacy_chat_sessions_with_context_columns() -> Result<(), AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute(
            "CREATE TABLE chat_sessions (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                title TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        ensure_column(&conn, "chat_sessions", "mode_id", "TEXT")?;
        ensure_column(&conn, "chat_sessions", "scenario_id", "TEXT")?;

        assert!(column_exists(&conn, "chat_sessions", "mode_id")?);
        assert!(column_exists(&conn, "chat_sessions", "scenario_id")?);

        // Re-running the migration should be a no-op for existing columns.
        ensure_column(&conn, "chat_sessions", "mode_id", "TEXT")?;
        ensure_column(&conn, "chat_sessions", "scenario_id", "TEXT")?;

        Ok(())
    }

    #[test]
    fn creates_v0_3_performance_indexes_idempotently() -> Result<(), AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE chat_sessions (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL
             );
             CREATE TABLE corrections (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                created_at TEXT NOT NULL
             );
             CREATE TABLE vocabulary (
                id TEXT PRIMARY KEY,
                character_id TEXT,
                created_at TEXT NOT NULL
             );
             CREATE TABLE memory_facts (
                id TEXT PRIMARY KEY,
                character_id TEXT NOT NULL,
                is_deleted INTEGER NOT NULL,
                is_visible INTEGER NOT NULL,
                updated_at TEXT NOT NULL
             );
             CREATE TABLE tool_invocations (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                started_at TEXT NOT NULL
             );",
        )?;

        ensure_v0_3_indexes(&conn)?;
        ensure_v0_3_indexes(&conn)?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master
             WHERE type = 'index' AND name LIKE 'idx_%'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 6);

        Ok(())
    }
}
