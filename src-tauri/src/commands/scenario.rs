use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Scenario {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    #[serde(rename = "learningGoals")]
    pub learning_goals: Option<String>,
    #[serde(rename = "isPreset")]
    pub is_preset: bool,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[tauri::command]
pub fn get_scenarios(state: State<'_, DbState>) -> Result<Vec<Scenario>, AppError> {
    let conn = state.lock_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, description, icon, learning_goals, is_preset, created_at FROM scenarios ORDER BY is_preset DESC, name ASC"
    )?;

    let iter = stmt.query_map([], |row| {
        let is_preset_int: i32 = row.get(5)?;
        Ok(Scenario {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            icon: row.get(3)?,
            learning_goals: row.get(4)?,
            is_preset: is_preset_int != 0,
            created_at: row.get(6)?,
        })
    })?;

    let mut scenarios = Vec::new();
    for result in iter {
        scenarios.push(result?);
    }
    Ok(scenarios)
}

#[tauri::command]
pub fn save_scenario(scenario: Scenario, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    let is_preset_int: i32 = if scenario.is_preset { 1 } else { 0 };

    conn.execute(
        "INSERT INTO scenarios (id, name, description, icon, learning_goals, is_preset, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            name=excluded.name,
            description=excluded.description,
            icon=excluded.icon,
            learning_goals=excluded.learning_goals",
        params![
            scenario.id,
            scenario.name,
            scenario.description,
            scenario.icon,
            scenario.learning_goals,
            is_preset_int,
            scenario.created_at,
        ],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_scenario(id: String, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    // Don't allow deleting preset scenarios
    let is_preset: i32 = conn
        .query_row(
            "SELECT is_preset FROM scenarios WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if is_preset != 0 {
        return Err(AppError::Config("Cannot delete preset scenarios".into()));
    }

    conn.execute("DELETE FROM scenarios WHERE id = ?1", params![id])?;
    Ok(())
}
