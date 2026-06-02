use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PracticeMode {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "promptSuffix")]
    pub prompt_suffix: String,
    #[serde(rename = "isPreset")]
    pub is_preset: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[tauri::command]
pub fn get_practice_modes(state: State<'_, DbState>) -> Result<Vec<PracticeMode>, AppError> {
    let conn = state.lock_conn()?;
    list_practice_modes(&conn)
}

pub fn list_practice_modes(conn: &Connection) -> Result<Vec<PracticeMode>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, prompt_suffix, is_preset, created_at
         FROM practice_modes
         ORDER BY
            CASE id
                WHEN 'free_talk' THEN 0
                WHEN 'roleplay' THEN 1
                WHEN 'scenario_drill' THEN 2
                WHEN 'ielts_speaking' THEN 3
                ELSE 10
            END,
            name ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        let is_preset_int: i32 = row.get(4)?;
        Ok(PracticeMode {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            prompt_suffix: row.get(3)?,
            is_preset: is_preset_int != 0,
            created_at: row.get(5)?,
        })
    })?;

    let mut modes = Vec::new();
    for row in rows {
        modes.push(row?);
    }
    Ok(modes)
}

pub fn practice_mode_exists(conn: &Connection, mode_id: &str) -> Result<bool, AppError> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM practice_modes WHERE id = ?1)",
        [mode_id],
        |row| row.get(0),
    )?;
    Ok(exists)
}
