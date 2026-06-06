//! Memory service for VoxVerse V1.
//!
//! Provides higher-level memory operations on top of the memory repository,
//! including memory retrieval for prompt injection and fact management.

use crate::error::AppError;
use crate::repositories::db::DbState;
use crate::repositories::memory::{self, MemoryFactRecord, RelationshipStateRecord};

/// Maximum number of memory facts to inject into a prompt.
const MAX_MEMORY_INJECTION_FACTS: usize = 20;

/// Retrieve visible memory facts for a character, formatted for prompt injection.
///
/// Returns a string block that can be appended to the system prompt to give
/// the character context about previous interactions.
pub fn build_memory_prompt_injection(
    db: &DbState,
    character_id: &str,
) -> Result<Option<String>, AppError> {
    let facts = memory::get_memory_facts(
        db,
        character_id,
        false, // exclude deleted
        None,  // all types
        MAX_MEMORY_INJECTION_FACTS,
    )?;

    if facts.is_empty() {
        return Ok(None);
    }

    let mut lines = Vec::with_capacity(facts.len() + 2);
    lines.push("You remember the following about the user:".to_string());

    for fact in &facts {
        if fact.is_visible {
            let prefix = match fact.fact_type.as_str() {
                "preference" => "Preference",
                "event" => "Event",
                "commitment" => "Promise/Commitment",
                "trait" => "Trait",
                _ => "Note",
            };
            lines.push(format!("- [{prefix}] {}", fact.content));
        }
    }

    Ok(Some(lines.join("\n")))
}

/// Build a relationship context string for prompt injection.
pub fn build_relationship_context(
    db: &DbState,
    character_id: &str,
) -> Result<Option<String>, AppError> {
    let state = memory::get_relationship_state(db, character_id)?;

    let Some(state) = state else {
        return Ok(None);
    };

    let mut parts = vec![format!(
        "Current relationship: intimacy={}, trust={}",
        state.intimacy_level, state.trust_level
    )];

    if let Some(stage) = &state.plot_stage {
        parts.push(format!("Story stage: {stage}"));
    }

    if let Some(goal) = &state.learning_goal {
        if !goal.trim().is_empty() {
            parts.push(format!("Learning goal: {goal}"));
        }
    }

    // Parse JSON fields if present
    if let Some(prefs) = &state.user_preferences {
        if !prefs.is_empty() && prefs != "null" {
            parts.push(format!("User preferences: {prefs}"));
        }
    }

    if let Some(boundaries) = &state.boundaries {
        if !boundaries.is_empty() && boundaries != "null" {
            parts.push(format!("Boundaries: {boundaries}"));
        }
    }

    Ok(Some(parts.join("\n")))
}

/// Create a new memory fact from a content string.
pub fn create_memory_fact(
    db: &DbState,
    character_id: &str,
    session_id: Option<&str>,
    fact_type: &str,
    content: &str,
    source_turn_id: Option<&str>,
) -> Result<MemoryFactRecord, AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    let fact = MemoryFactRecord {
        id: uuid::Uuid::new_v4().to_string(),
        character_id: character_id.into(),
        session_id: session_id.map(|s| s.into()),
        fact_type: fact_type.into(),
        content: content.into(),
        source_turn_id: source_turn_id.map(|s| s.into()),
        confidence: 1.0,
        is_visible: true,
        is_deleted: false,
        created_at: now.clone(),
        updated_at: now,
    };

    memory::save_memory_fact(db, &fact)?;
    Ok(fact)
}

/// Ensure a relationship state record exists for a character, creating one if needed.
pub fn ensure_relationship_state(
    db: &DbState,
    character_id: &str,
) -> Result<RelationshipStateRecord, AppError> {
    if let Some(existing) = memory::get_relationship_state(db, character_id)? {
        return Ok(existing);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let state = RelationshipStateRecord {
        id: uuid::Uuid::new_v4().to_string(),
        character_id: character_id.into(),
        intimacy_level: 0,
        trust_level: 0,
        plot_stage: None,
        learning_goal: None,
        user_preferences: None,
        boundaries: None,
        commitments: None,
        updated_at: now,
    };

    memory::upsert_relationship_state(db, &state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::db::DbState;
    use rusqlite::Connection;
    use std::sync::Mutex;

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
            "CREATE TABLE relationship_state (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL UNIQUE,
                intimacy_level INTEGER NOT NULL DEFAULT 0,
                trust_level INTEGER NOT NULL DEFAULT 0,
                plot_stage TEXT, learning_goal TEXT, user_preferences TEXT,
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
            "INSERT INTO characters (id, name, language, personality, style, avatar, system_prompt, greeting)
             VALUES ('c1', 'Test', 'en', 'friendly', 'casual', '🤖', 'Be helpful.', 'Hi!')",
            [],
        )
        .unwrap();

        DbState {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn builds_memory_injection_from_facts() {
        let db = setup_test_db();

        create_memory_fact(&db, "c1", None, "preference", "User likes coffee", None).unwrap();
        create_memory_fact(&db, "c1", None, "event", "Had a great chat yesterday", None).unwrap();

        let injection = build_memory_prompt_injection(&db, "c1").unwrap();
        assert!(injection.is_some());
        let text = injection.unwrap();
        assert!(text.contains("User likes coffee"));
        assert!(text.contains("[Preference]"));
        assert!(text.contains("[Event]"));
    }

    #[test]
    fn returns_none_for_empty_memory() {
        let db = setup_test_db();
        let injection = build_memory_prompt_injection(&db, "c1").unwrap();
        assert!(injection.is_none());
    }

    #[test]
    fn ensures_relationship_state_exists() {
        let db = setup_test_db();

        let state = ensure_relationship_state(&db, "c1").unwrap();
        assert_eq!(state.intimacy_level, 0);
        assert_eq!(state.character_id, "c1");

        // Second call returns existing
        let state2 = ensure_relationship_state(&db, "c1").unwrap();
        assert_eq!(state2.id, state.id);
    }

    #[test]
    fn builds_relationship_context() {
        let db = setup_test_db();
        let mut state = ensure_relationship_state(&db, "c1").unwrap();
        state.intimacy_level = 5;
        state.trust_level = 3;
        state.plot_stage = Some("friends".into());
        state.learning_goal = Some("Practice past tense stories".into());
        memory::upsert_relationship_state(&db, &state).unwrap();

        let context = build_relationship_context(&db, "c1").unwrap();
        assert!(context.is_some());
        let text = context.unwrap();
        assert!(text.contains("intimacy=5"));
        assert!(text.contains("trust=3"));
        assert!(text.contains("friends"));
        assert!(text.contains("Practice past tense stories"));
    }
}
