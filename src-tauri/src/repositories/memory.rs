//! Memory data access layer for VoxVerse V1.
//!
//! Provides CRUD operations for memory facts, relationship state,
//! and tool invocations stored in SQLite.

use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

// ── Memory Facts ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFactRecord {
    pub id: String,
    pub character_id: String,
    pub session_id: Option<String>,
    pub fact_type: String,
    pub content: String,
    pub source_turn_id: Option<String>,
    pub confidence: f64,
    pub is_visible: bool,
    pub is_deleted: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub fn save_memory_fact(db: &DbState, fact: &MemoryFactRecord) -> Result<(), AppError> {
    let conn = db.lock_conn()?;
    conn.execute(
        "INSERT INTO memory_facts
            (id, character_id, session_id, fact_type, content, source_turn_id,
             confidence, is_visible, is_deleted, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET
            content = excluded.content,
            confidence = excluded.confidence,
            is_visible = excluded.is_visible,
            is_deleted = excluded.is_deleted,
            updated_at = excluded.updated_at",
        params![
            fact.id,
            fact.character_id,
            fact.session_id,
            fact.fact_type,
            fact.content,
            fact.source_turn_id,
            fact.confidence,
            fact.is_visible as i32,
            fact.is_deleted as i32,
            fact.created_at,
            fact.updated_at,
        ],
    )?;
    Ok(())
}

pub fn get_memory_facts(
    db: &DbState,
    character_id: &str,
    include_deleted: bool,
    fact_type: Option<&str>,
    limit: usize,
) -> Result<Vec<MemoryFactRecord>, AppError> {
    let conn = db.lock_conn()?;

    let mut sql = String::from(
        "SELECT id, character_id, session_id, fact_type, content, source_turn_id,
                confidence, is_visible, is_deleted, created_at, updated_at
         FROM memory_facts
         WHERE character_id = ?1",
    );

    if !include_deleted {
        sql.push_str(" AND is_deleted = 0");
    }

    if fact_type.is_some() {
        sql.push_str(" AND fact_type = ?2");
    }

    sql.push_str(" ORDER BY updated_at DESC LIMIT ?3");

    let mut stmt = conn.prepare(&sql)?;
    let limit_i64 = limit as i64;

    let rows = if let Some(ft) = fact_type {
        stmt.query_map(params![character_id, ft, limit_i64], row_to_memory_fact)?
    } else {
        // When fact_type filter is not used, ?2 is skipped — we need to bind limit as ?2
        drop(stmt);
        let sql_no_type = format!(
            "SELECT id, character_id, session_id, fact_type, content, source_turn_id,
                    confidence, is_visible, is_deleted, created_at, updated_at
             FROM memory_facts
             WHERE character_id = ?1{}
             ORDER BY updated_at DESC LIMIT ?2",
            if include_deleted { "" } else { " AND is_deleted = 0" }
        );
        let mut stmt2 = conn.prepare(&sql_no_type)?;
        let rows = stmt2.query_map(params![character_id, limit_i64], row_to_memory_fact)?;
        return rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from);
    };

    rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
}

pub fn soft_delete_memory_fact(db: &DbState, fact_id: &str) -> Result<bool, AppError> {
    let conn = db.lock_conn()?;
    let now = chrono::Utc::now().to_rfc3339();
    let changed = conn.execute(
        "UPDATE memory_facts SET is_deleted = 1, updated_at = ?2 WHERE id = ?1",
        params![fact_id, now],
    )?;
    Ok(changed > 0)
}

pub fn toggle_memory_visibility(
    db: &DbState,
    fact_id: &str,
    visible: bool,
) -> Result<bool, AppError> {
    let conn = db.lock_conn()?;
    let now = chrono::Utc::now().to_rfc3339();
    let changed = conn.execute(
        "UPDATE memory_facts SET is_visible = ?2, updated_at = ?3 WHERE id = ?1 AND is_deleted = 0",
        params![fact_id, visible as i32, now],
    )?;
    Ok(changed > 0)
}

fn row_to_memory_fact(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryFactRecord> {
    let is_visible: i32 = row.get(7)?;
    let is_deleted: i32 = row.get(8)?;
    Ok(MemoryFactRecord {
        id: row.get(0)?,
        character_id: row.get(1)?,
        session_id: row.get(2)?,
        fact_type: row.get(3)?,
        content: row.get(4)?,
        source_turn_id: row.get(5)?,
        confidence: row.get(6)?,
        is_visible: is_visible != 0,
        is_deleted: is_deleted != 0,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

// ── Relationship State ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipStateRecord {
    pub id: String,
    pub character_id: String,
    pub intimacy_level: i64,
    pub trust_level: i64,
    pub plot_stage: Option<String>,
    pub user_preferences: Option<String>,
    pub boundaries: Option<String>,
    pub commitments: Option<String>,
    pub updated_at: String,
}

pub fn get_relationship_state(
    db: &DbState,
    character_id: &str,
) -> Result<Option<RelationshipStateRecord>, AppError> {
    let conn = db.lock_conn()?;
    conn.query_row(
        "SELECT id, character_id, intimacy_level, trust_level, plot_stage,
                user_preferences, boundaries, commitments, updated_at
         FROM relationship_state
         WHERE character_id = ?1",
        params![character_id],
        row_to_relationship,
    )
    .optional()
    .map_err(AppError::from)
}

pub fn upsert_relationship_state(
    db: &DbState,
    state: &RelationshipStateRecord,
) -> Result<(), AppError> {
    let conn = db.lock_conn()?;
    conn.execute(
        "INSERT INTO relationship_state
            (id, character_id, intimacy_level, trust_level, plot_stage,
             user_preferences, boundaries, commitments, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(character_id) DO UPDATE SET
            intimacy_level = excluded.intimacy_level,
            trust_level = excluded.trust_level,
            plot_stage = excluded.plot_stage,
            user_preferences = excluded.user_preferences,
            boundaries = excluded.boundaries,
            commitments = excluded.commitments,
            updated_at = excluded.updated_at",
        params![
            state.id,
            state.character_id,
            state.intimacy_level,
            state.trust_level,
            state.plot_stage,
            state.user_preferences,
            state.boundaries,
            state.commitments,
            state.updated_at,
        ],
    )?;
    Ok(())
}

fn row_to_relationship(row: &rusqlite::Row<'_>) -> rusqlite::Result<RelationshipStateRecord> {
    Ok(RelationshipStateRecord {
        id: row.get(0)?,
        character_id: row.get(1)?,
        intimacy_level: row.get(2)?,
        trust_level: row.get(3)?,
        plot_stage: row.get(4)?,
        user_preferences: row.get(5)?,
        boundaries: row.get(6)?,
        commitments: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

// ── Tool Invocations ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocationRecord {
    pub id: String,
    pub session_id: String,
    pub tool_name: String,
    pub status: String,
    pub input_json: Option<String>,
    pub output_json: Option<String>,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

pub fn save_tool_invocation(db: &DbState, record: &ToolInvocationRecord) -> Result<(), AppError> {
    let conn = db.lock_conn()?;
    conn.execute(
        "INSERT INTO tool_invocations
            (id, session_id, tool_name, status, input_json, output_json,
             error_message, started_at, completed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            record.id,
            record.session_id,
            record.tool_name,
            record.status,
            record.input_json,
            record.output_json,
            record.error_message,
            record.started_at,
            record.completed_at,
        ],
    )?;
    Ok(())
}

pub fn update_tool_invocation_status(
    db: &DbState,
    invocation_id: &str,
    status: &str,
    output_json: Option<&str>,
    error_message: Option<&str>,
) -> Result<bool, AppError> {
    let conn = db.lock_conn()?;
    let now = chrono::Utc::now().to_rfc3339();
    let changed = conn.execute(
        "UPDATE tool_invocations
         SET status = ?2, output_json = ?3, error_message = ?4, completed_at = ?5
         WHERE id = ?1",
        params![invocation_id, status, output_json, error_message, now],
    )?;
    Ok(changed > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_test_db() -> DbState {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

        conn.execute(
            "CREATE TABLE characters (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, language TEXT NOT NULL,
                personality TEXT NOT NULL, style TEXT NOT NULL, avatar TEXT NOT NULL,
                system_prompt TEXT NOT NULL, greeting TEXT NOT NULL,
                voice_lang TEXT, voice_name TEXT
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE chat_sessions (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL,
                title TEXT, mode_id TEXT, scenario_id TEXT,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE relationship_state (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL UNIQUE,
                intimacy_level INTEGER NOT NULL DEFAULT 0,
                trust_level INTEGER NOT NULL DEFAULT 0,
                plot_stage TEXT, user_preferences TEXT,
                boundaries TEXT, commitments TEXT, updated_at TEXT NOT NULL,
                FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE memory_facts (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL,
                session_id TEXT, fact_type TEXT NOT NULL, content TEXT NOT NULL,
                source_turn_id TEXT, confidence REAL NOT NULL DEFAULT 1.0,
                is_visible INTEGER NOT NULL DEFAULT 1, is_deleted INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                FOREIGN KEY (character_id) REFERENCES characters(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE tool_invocations (
                id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
                tool_name TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending',
                input_json TEXT, output_json TEXT, error_message TEXT,
                started_at TEXT NOT NULL, completed_at TEXT,
                FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE
            )",
            [],
        )
        .unwrap();

        // Seed a test character
        conn.execute(
            "INSERT INTO characters (id, name, language, personality, style, avatar, system_prompt, greeting)
             VALUES ('c1', 'Test', 'en', 'friendly', 'casual', '🤖', 'Be helpful.', 'Hi!')",
            [],
        )
        .unwrap();

        // Seed a test session
        conn.execute(
            "INSERT INTO chat_sessions (id, character_id, title, created_at, updated_at)
             VALUES ('s1', 'c1', 'Test Session', '2026-01-01', '2026-01-01')",
            [],
        )
        .unwrap();

        DbState {
            conn: std::sync::Mutex::new(conn),
        }
    }

    #[test]
    fn saves_and_retrieves_memory_fact() {
        let db = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        let fact = MemoryFactRecord {
            id: "f1".into(),
            character_id: "c1".into(),
            session_id: Some("s1".into()),
            fact_type: "preference".into(),
            content: "User likes coffee".into(),
            source_turn_id: None,
            confidence: 0.9,
            is_visible: true,
            is_deleted: false,
            created_at: now.clone(),
            updated_at: now,
        };

        save_memory_fact(&db, &fact).unwrap();

        let facts = get_memory_facts(&db, "c1", false, None, 10).unwrap();
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].content, "User likes coffee");
        assert_eq!(facts[0].fact_type, "preference");
    }

    #[test]
    fn soft_deletes_memory_fact() {
        let db = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        let fact = MemoryFactRecord {
            id: "f2".into(),
            character_id: "c1".into(),
            session_id: None,
            fact_type: "trait".into(),
            content: "User is quiet".into(),
            source_turn_id: None,
            confidence: 1.0,
            is_visible: true,
            is_deleted: false,
            created_at: now.clone(),
            updated_at: now,
        };
        save_memory_fact(&db, &fact).unwrap();

        let deleted = soft_delete_memory_fact(&db, "f2").unwrap();
        assert!(deleted);

        let facts = get_memory_facts(&db, "c1", false, None, 10).unwrap();
        assert!(facts.is_empty());

        let all_facts = get_memory_facts(&db, "c1", true, None, 10).unwrap();
        assert_eq!(all_facts.len(), 1);
    }

    #[test]
    fn toggles_memory_visibility() {
        let db = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        let fact = MemoryFactRecord {
            id: "f3".into(),
            character_id: "c1".into(),
            session_id: None,
            fact_type: "event".into(),
            content: "Had dinner together".into(),
            source_turn_id: None,
            confidence: 1.0,
            is_visible: true,
            is_deleted: false,
            created_at: now.clone(),
            updated_at: now,
        };
        save_memory_fact(&db, &fact).unwrap();

        toggle_memory_visibility(&db, "f3", false).unwrap();
        let facts = get_memory_facts(&db, "c1", false, None, 10).unwrap();
        assert!(!facts[0].is_visible);
    }

    #[test]
    fn upserts_relationship_state() {
        let db = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        let state = RelationshipStateRecord {
            id: "r1".into(),
            character_id: "c1".into(),
            intimacy_level: 5,
            trust_level: 3,
            plot_stage: Some("introduction".into()),
            user_preferences: None,
            boundaries: None,
            commitments: None,
            updated_at: now.clone(),
        };

        upsert_relationship_state(&db, &state).unwrap();
        let loaded = get_relationship_state(&db, "c1").unwrap().unwrap();
        assert_eq!(loaded.intimacy_level, 5);

        // Update
        let updated = RelationshipStateRecord {
            intimacy_level: 8,
            updated_at: chrono::Utc::now().to_rfc3339(),
            ..state
        };
        upsert_relationship_state(&db, &updated).unwrap();
        let reloaded = get_relationship_state(&db, "c1").unwrap().unwrap();
        assert_eq!(reloaded.intimacy_level, 8);
    }

    #[test]
    fn saves_and_updates_tool_invocation() {
        let db = setup_test_db();
        let now = chrono::Utc::now().to_rfc3339();
        let record = ToolInvocationRecord {
            id: "t1".into(),
            session_id: "s1".into(),
            tool_name: "save_memory".into(),
            status: "pending".into(),
            input_json: Some(r#"{"content":"test"}"#.into()),
            output_json: None,
            error_message: None,
            started_at: now,
            completed_at: None,
        };

        save_tool_invocation(&db, &record).unwrap();

        let updated = update_tool_invocation_status(
            &db,
            "t1",
            "success",
            Some(r#"{"saved":true}"#),
            None,
        )
        .unwrap();
        assert!(updated);
    }
}
