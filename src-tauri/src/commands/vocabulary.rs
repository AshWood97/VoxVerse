use crate::error::AppError;
use crate::repositories::db::DbState;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VocabularyItem {
    pub id: String,
    #[serde(rename = "wordOrPhrase")]
    pub word_or_phrase: String,
    pub translation: Option<String>,
    pub context: Option<String>,
    pub notes: Option<String>,
    #[serde(rename = "characterId")]
    pub character_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[tauri::command]
pub fn get_vocabulary(state: State<'_, DbState>) -> Result<Vec<VocabularyItem>, AppError> {
    let conn = state.lock_conn()?;
    let mut stmt = conn.prepare(
        "SELECT id, word_or_phrase, translation, context, notes, character_id, created_at FROM vocabulary ORDER BY created_at DESC"
    )?;

    let iter = stmt.query_map([], |row| {
        Ok(VocabularyItem {
            id: row.get(0)?,
            word_or_phrase: row.get(1)?,
            translation: row.get(2)?,
            context: row.get(3)?,
            notes: row.get(4)?,
            character_id: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut items = Vec::new();
    for result in iter {
        items.push(result?);
    }
    Ok(items)
}

#[tauri::command]
pub fn save_vocabulary(item: VocabularyItem, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    conn.execute(
        "INSERT INTO vocabulary (id, word_or_phrase, translation, context, notes, character_id, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET
            word_or_phrase=excluded.word_or_phrase,
            translation=excluded.translation,
            context=excluded.context,
            notes=excluded.notes",
        params![
            item.id,
            item.word_or_phrase,
            item.translation,
            item.context,
            item.notes,
            item.character_id,
            item.created_at,
        ],
    )?;
    Ok(())
}

#[tauri::command]
pub fn delete_vocabulary(id: String, state: State<'_, DbState>) -> Result<(), AppError> {
    let conn = state.lock_conn()?;
    conn.execute("DELETE FROM vocabulary WHERE id = ?1", params![id])?;
    Ok(())
}
