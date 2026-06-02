use crate::commands::practice::practice_mode_exists;
use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

const ALLOWED_MESSAGE_ROLES: [&str; 3] = ["user", "assistant", "system"];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatSession {
    pub id: String,
    #[serde(rename = "characterId")]
    pub character_id: String,
    pub title: Option<String>,
    #[serde(rename = "modeId")]
    pub mode_id: Option<String>,
    #[serde(rename = "scenarioId")]
    pub scenario_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionContext {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "modeId")]
    pub mode_id: Option<String>,
    #[serde(rename = "scenarioId")]
    pub scenario_id: Option<String>,
}

#[tauri::command]
pub fn get_sessions(
    character_id: String,
    state: State<'_, DbState>,
) -> Result<Vec<ChatSession>, AppError> {
    let conn = state.lock_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, character_id, title, mode_id, scenario_id, created_at, updated_at FROM chat_sessions WHERE character_id = ?1 ORDER BY updated_at DESC"
    )?;

    let iter = stmt.query_map(params![character_id], |row| {
        Ok(ChatSession {
            id: row.get(0)?,
            character_id: row.get(1)?,
            title: row.get(2)?,
            mode_id: row.get(3)?,
            scenario_id: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            messages: vec![], // Messages loaded separately or populated defensively
        })
    })?;

    let mut sessions = Vec::new();
    for result in iter {
        sessions.push(result?);
    }
    Ok(sessions)
}

#[tauri::command]
pub fn get_session_messages(
    session_id: String,
    state: State<'_, DbState>,
) -> Result<Vec<Message>, AppError> {
    let conn = state.lock_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, content, timestamp FROM messages WHERE session_id = ?1 ORDER BY timestamp ASC"
    )?;

    let iter = stmt.query_map(params![session_id], |row| {
        Ok(Message {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            content: row.get(3)?,
            timestamp: row.get(4)?,
        })
    })?;

    let mut msgs = Vec::new();
    for result in iter {
        msgs.push(result?);
    }
    Ok(msgs)
}

#[tauri::command]
pub fn create_session(mut session: ChatSession, state: State<'_, DbState>) -> Result<(), AppError> {
    session.mode_id = normalize_optional_id(session.mode_id);
    session.scenario_id = normalize_optional_id(session.scenario_id);

    let conn = state.lock_conn()?;
    validate_session_for_create(&conn, &session)?;
    conn.execute(
        "INSERT INTO chat_sessions (id, character_id, title, mode_id, scenario_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![session.id, session.character_id, session.title, session.mode_id, session.scenario_id, session.created_at, session.updated_at],
    )?;
    Ok(())
}

#[tauri::command]
pub fn get_session_context(
    id: String,
    state: State<'_, DbState>,
) -> Result<SessionContext, AppError> {
    let conn = state.lock_conn()?;
    conn.query_row(
        "SELECT id, mode_id, scenario_id FROM chat_sessions WHERE id = ?1",
        params![id],
        |row| {
            Ok(SessionContext {
                session_id: row.get(0)?,
                mode_id: row.get(1)?,
                scenario_id: row.get(2)?,
            })
        },
    )
    .map_err(AppError::from)
}

#[tauri::command]
pub fn update_session_context(
    id: String,
    mode_id: Option<String>,
    scenario_id: Option<String>,
    state: State<'_, DbState>,
) -> Result<SessionContext, AppError> {
    let conn = state.lock_conn()?;
    let normalized_mode_id = normalize_optional_id(mode_id);
    let normalized_scenario_id = normalize_optional_id(scenario_id);

    ensure_session_exists(&conn, &id)?;
    validate_session_context(
        &conn,
        normalized_mode_id.as_deref(),
        normalized_scenario_id.as_deref(),
    )?;

    conn.execute(
        "UPDATE chat_sessions SET mode_id = ?1, scenario_id = ?2, updated_at = ?3 WHERE id = ?4",
        params![
            normalized_mode_id.clone(),
            normalized_scenario_id.clone(),
            chrono::Utc::now().to_rfc3339(),
            &id,
        ],
    )?;

    Ok(SessionContext {
        session_id: id,
        mode_id: normalized_mode_id,
        scenario_id: normalized_scenario_id,
    })
}

#[tauri::command]
pub fn update_session_title(
    id: String,
    title: String,
    state: State<'_, DbState>,
) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    conn.execute(
        "UPDATE chat_sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
        params![title, chrono::Utc::now().to_rfc3339(), id],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_session(id: String, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    conn.execute("DELETE FROM chat_sessions WHERE id = ?1", params![id])?;
    Ok(())
}

#[tauri::command]
pub fn save_message(message: Message, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    validate_message_for_save(&conn, &message)?;
    conn.execute(
        "INSERT INTO messages (id, session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![message.id, message.session_id, message.role, message.content, message.timestamp],
    )?;
    // Update session timestamp
    conn.execute(
        "UPDATE chat_sessions SET updated_at = ?1 WHERE id = ?2",
        params![chrono::Utc::now().to_rfc3339(), message.session_id],
    )?;
    Ok(())
}

fn validate_session_for_create(
    conn: &rusqlite::Connection,
    session: &ChatSession,
) -> Result<(), AppError> {
    if session.id.trim().is_empty() {
        return Err(AppError::Config("Session id is required.".into()));
    }

    if session.character_id.trim().is_empty() {
        return Err(AppError::Config("Session character id is required.".into()));
    }

    let character_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM characters WHERE id = ?1)",
        params![session.character_id],
        |row| row.get(0),
    )?;

    if !character_exists {
        return Err(AppError::Config(format!(
            "Character '{}' was not found.",
            session.character_id
        )));
    }

    validate_session_context(
        conn,
        session.mode_id.as_deref(),
        session.scenario_id.as_deref(),
    )?;

    Ok(())
}

fn normalize_optional_id(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn ensure_session_exists(conn: &rusqlite::Connection, session_id: &str) -> Result<(), AppError> {
    let session_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM chat_sessions WHERE id = ?1)",
        params![session_id],
        |row| row.get(0),
    )?;

    if session_exists {
        Ok(())
    } else {
        Err(AppError::Config(format!(
            "Session '{}' was not found.",
            session_id
        )))
    }
}

fn validate_session_context(
    conn: &rusqlite::Connection,
    mode_id: Option<&str>,
    scenario_id: Option<&str>,
) -> Result<(), AppError> {
    if let Some(mode_id) = mode_id {
        if !practice_mode_exists(conn, mode_id)? {
            return Err(AppError::Config(format!(
                "Practice mode '{}' was not found.",
                mode_id
            )));
        }
    }

    if let Some(scenario_id) = scenario_id {
        let scenario_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM scenarios WHERE id = ?1)",
            params![scenario_id],
            |row| row.get(0),
        )?;

        if !scenario_exists {
            return Err(AppError::Config(format!(
                "Scenario '{}' was not found.",
                scenario_id
            )));
        }
    }

    Ok(())
}

fn validate_message_role(role: &str) -> Result<(), AppError> {
    if ALLOWED_MESSAGE_ROLES.contains(&role) {
        return Ok(());
    }

    Err(AppError::Config(format!(
        "Unsupported message role '{}'.",
        role
    )))
}

fn validate_message_for_save(
    conn: &rusqlite::Connection,
    message: &Message,
) -> Result<(), AppError> {
    if message.id.trim().is_empty() {
        return Err(AppError::Config("Message id is required.".into()));
    }

    if message.session_id.trim().is_empty() {
        return Err(AppError::Config("Message session id is required.".into()));
    }

    validate_message_role(message.role.trim())?;

    if message.content.trim().is_empty() {
        return Err(AppError::Config("Message content is required.".into()));
    }

    let session_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM chat_sessions WHERE id = ?1)",
        params![message.session_id],
        |row| row.get(0),
    )?;

    if !session_exists {
        return Err(AppError::Config(format!(
            "Session '{}' was not found.",
            message.session_id
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_optional_id, validate_message_role, validate_session_context};
    use crate::error::AppError;
    use rusqlite::Connection;

    #[test]
    fn accepts_supported_message_roles() {
        assert!(validate_message_role("user").is_ok());
        assert!(validate_message_role("assistant").is_ok());
        assert!(validate_message_role("system").is_ok());
    }

    #[test]
    fn rejects_unsupported_message_roles() {
        assert!(validate_message_role("tool").is_err());
        assert!(validate_message_role("").is_err());
    }

    #[test]
    fn normalizes_optional_session_context_ids() {
        assert_eq!(
            normalize_optional_id(Some(" roleplay ".into())),
            Some("roleplay".into())
        );
        assert_eq!(normalize_optional_id(Some("   ".into())), None);
        assert_eq!(normalize_optional_id(None), None);
    }

    #[test]
    fn validates_session_context_against_known_mode_and_scenario() -> Result<(), AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE practice_modes (id TEXT PRIMARY KEY);
             CREATE TABLE scenarios (id TEXT PRIMARY KEY);",
        )?;
        conn.execute("INSERT INTO practice_modes (id) VALUES ('roleplay')", [])?;
        conn.execute("INSERT INTO scenarios (id) VALUES ('interview')", [])?;

        assert!(validate_session_context(&conn, Some("roleplay"), Some("interview")).is_ok());
        assert!(validate_session_context(&conn, Some("missing"), Some("interview")).is_err());
        assert!(validate_session_context(&conn, Some("roleplay"), Some("missing")).is_err());

        Ok(())
    }
}
