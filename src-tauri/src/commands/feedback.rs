use tauri::State;

use crate::error::AppError;
use crate::repositories::db::DbState;
use crate::services::feedback;
use crate::services::llm::stream_chat_completion;
use crate::state::{ApiConfig, AppState};
use rusqlite::params;
use serde::Deserialize;

fn load_runtime_config(state: &State<'_, AppState>) -> Result<ApiConfig, String> {
    let guard = state.config.lock().map_err(|e| e.to_string())?;
    Ok(guard.clone())
}

fn load_llm_config(state: &State<'_, AppState>) -> Result<(String, String, String), String> {
    let config = load_runtime_config(state)?;
    let api_key = config.get_api_key().map_err(|e| e.to_string())?;
    Ok((config.base_url, api_key, config.model))
}

/// Analyze user text for grammar errors and return structured JSON.
/// This is an independent LLM call that doesn't affect the main conversation.
#[tauri::command]
pub async fn analyze_text(state: State<'_, AppState>, text: String) -> Result<String, String> {
    let (base_url, api_key, model) = load_llm_config(&state)?;

    let messages = feedback::build_correction_prompt(&text);
    let response = stream_chat_completion(
        &base_url,
        &api_key,
        &model,
        messages,
        |_| {}, // No streaming needed for feedback — just collect full response
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(response)
}

/// Translate text to a target language.
#[tauri::command]
pub async fn translate_text(
    state: State<'_, AppState>,
    text: String,
    target_lang: String,
) -> Result<String, String> {
    let (base_url, api_key, model) = load_llm_config(&state)?;

    let messages = feedback::build_translation_prompt(&text, &target_lang);
    let response = stream_chat_completion(&base_url, &api_key, &model, messages, |_| {})
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

/// Polish/improve user text to sound more native.
#[tauri::command]
pub async fn polish_text(state: State<'_, AppState>, text: String) -> Result<String, String> {
    let (base_url, api_key, model) = load_llm_config(&state)?;

    let messages = feedback::build_polish_prompt(&text);
    let response = stream_chat_completion(&base_url, &api_key, &model, messages, |_| {})
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

#[derive(Debug, Deserialize)]
pub struct SaveCorrectionInput {
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
    #[serde(rename = "originalText")]
    pub original_text: String,
    #[serde(rename = "correctedText")]
    pub corrected_text: String,
    pub explanation: Option<String>,
    #[serde(rename = "betterExpression")]
    pub better_expression: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

#[tauri::command]
pub fn save_correction(
    state: State<'_, DbState>,
    correction: SaveCorrectionInput,
) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    persist_correction(&conn, correction)
}

fn persist_correction(
    conn: &rusqlite::Connection,
    correction: SaveCorrectionInput,
) -> Result<(), AppError> {
    let original_text = correction.original_text.trim();
    if original_text.is_empty() {
        return Err(AppError::Config(
            "Correction original text is required.".into(),
        ));
    }

    let corrected_text = correction.corrected_text.trim();
    if corrected_text.is_empty() {
        return Err(AppError::Config(
            "Correction corrected text is required.".into(),
        ));
    }

    let session_id = correction
        .session_id
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let Some(session_id) = session_id.as_deref() {
        let session_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM chat_sessions WHERE id = ?1)",
            params![session_id],
            |row| row.get(0),
        )?;

        if !session_exists {
            return Err(AppError::Config(format!(
                "Session '{}' was not found.",
                session_id
            )));
        }
    }

    let correction_id = format!("corr-{}", uuid::Uuid::new_v4());
    let created_at = correction
        .created_at
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let explanation = correction
        .explanation
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let better_expression = correction
        .better_expression
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    conn.execute(
        "INSERT INTO corrections
            (id, session_id, original_text, corrected_text, explanation, better_expression, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            correction_id,
            session_id,
            original_text,
            corrected_text,
            explanation,
            better_expression,
            created_at,
        ],
    )?;

    Ok(())
}

/// Generate a learning summary report for a conversation.
#[tauri::command]
pub async fn generate_summary(
    state: State<'_, AppState>,
    db: State<'_, DbState>,
    messages: Vec<(String, String)>,
    session_id: Option<String>,
) -> Result<String, String> {
    let (base_url, api_key, model) = load_llm_config(&state)?;

    let summary_context = load_summary_context(&db, session_id.as_deref())?;
    let chat_messages = feedback::build_summary_prompt(&messages, summary_context.as_ref());
    let response = stream_chat_completion(&base_url, &api_key, &model, chat_messages, |_| {})
        .await
        .map_err(|e| e.to_string())?;

    Ok(response)
}

fn load_summary_context(
    db: &DbState,
    session_id: Option<&str>,
) -> Result<Option<feedback::SummaryContext>, String> {
    let Some(session_id) = session_id.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    let conn = db.lock_conn().map_err(|e| e.to_string())?;
    let mut context = match conn.query_row(
        "SELECT
            COALESCE(pm.name, 'Free Talk'),
            COALESCE(pm.description, 'Open-ended conversation.'),
            sc.name,
            sc.description
         FROM chat_sessions s
         LEFT JOIN practice_modes pm ON pm.id = COALESCE(s.mode_id, 'free_talk')
         LEFT JOIN scenarios sc ON sc.id = s.scenario_id
         WHERE s.id = ?1",
        params![session_id],
        |row| {
            Ok(feedback::SummaryContext {
                practice_mode_name: row.get(0)?,
                practice_mode_description: row.get(1)?,
                scenario_name: row.get(2)?,
                scenario_description: row.get(3)?,
                recent_corrections: Vec::new(),
            })
        },
    ) {
        Ok(context) => context,
        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
        Err(error) => return Err(format!("Failed to load summary context: {error}")),
    };

    let mut stmt = conn
        .prepare(
            "SELECT original_text, corrected_text, explanation, better_expression
             FROM corrections
             WHERE session_id = ?1
             ORDER BY created_at DESC
             LIMIT 8",
        )
        .map_err(|error| format!("Failed to prepare correction context query: {error}"))?;

    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok(feedback::SummaryCorrection {
                original_text: row.get(0)?,
                corrected_text: row.get(1)?,
                explanation: row.get(2)?,
                better_expression: row.get(3)?,
            })
        })
        .map_err(|error| format!("Failed to load correction context: {error}"))?;

    for row in rows {
        context
            .recent_corrections
            .push(row.map_err(|error| format!("Failed to decode correction context: {error}"))?);
    }

    Ok(Some(context))
}

#[cfg(test)]
mod tests {
    use super::{persist_correction, SaveCorrectionInput};
    use crate::error::AppError;
    use rusqlite::Connection;

    fn setup_conn() -> Result<Connection, AppError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE chat_sessions (id TEXT PRIMARY KEY);
             CREATE TABLE corrections (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                original_text TEXT NOT NULL,
                corrected_text TEXT NOT NULL,
                explanation TEXT,
                better_expression TEXT,
                created_at TEXT NOT NULL
             );",
        )?;
        conn.execute("INSERT INTO chat_sessions (id) VALUES ('session-1')", [])?;
        Ok(conn)
    }

    #[test]
    fn persists_correction_with_trimmed_content_and_uuid_ids() -> Result<(), AppError> {
        let conn = setup_conn()?;

        for text in ["I has apple", "She go home"] {
            persist_correction(
                &conn,
                SaveCorrectionInput {
                    session_id: Some(" session-1 ".into()),
                    original_text: format!(" {text} "),
                    corrected_text: "Corrected sentence.".into(),
                    explanation: Some(" Subject verb agreement. ".into()),
                    better_expression: Some("A more natural expression.".into()),
                    created_at: Some("2026-05-16T00:00:00Z".into()),
                },
            )?;
        }

        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM corrections", [], |row| row.get(0))?;
        assert_eq!(count, 2);

        let original: String = conn.query_row(
            "SELECT original_text FROM corrections ORDER BY original_text ASC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(original, "I has apple");

        let unique_ids: i64 =
            conn.query_row("SELECT COUNT(DISTINCT id) FROM corrections", [], |row| {
                row.get(0)
            })?;
        assert_eq!(unique_ids, 2);

        Ok(())
    }

    #[test]
    fn rejects_correction_for_unknown_session() -> Result<(), AppError> {
        let conn = setup_conn()?;
        let result = persist_correction(
            &conn,
            SaveCorrectionInput {
                session_id: Some("missing-session".into()),
                original_text: "I has apple".into(),
                corrected_text: "I have an apple.".into(),
                explanation: None,
                better_expression: None,
                created_at: None,
            },
        );

        assert!(result.is_err());
        Ok(())
    }
}
