//! Tauri commands for memory management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::repositories::db::DbState;
use crate::repositories::memory;
use crate::services::memory_service;

#[derive(Debug, Clone, Serialize)]
pub struct MemoryFactInfo {
    pub id: String,
    pub character_id: String,
    pub session_id: Option<String>,
    pub fact_type: String,
    pub content: String,
    pub confidence: f64,
    pub is_visible: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct MemoryFactFilters {
    pub fact_type: Option<String>,
    pub include_deleted: Option<bool>,
    pub limit: Option<usize>,
}

/// Get memory facts for a character with optional filters.
#[tauri::command]
pub async fn get_memory_facts(
    db: State<'_, DbState>,
    character_id: String,
    filters: Option<MemoryFactFilters>,
) -> Result<Vec<MemoryFactInfo>, String> {
    let filters = filters.unwrap_or(MemoryFactFilters {
        fact_type: None,
        include_deleted: None,
        limit: None,
    });

    let facts = memory::get_memory_facts(
        &db,
        &character_id,
        filters.include_deleted.unwrap_or(false),
        filters.fact_type.as_deref(),
        filters.limit.unwrap_or(100),
    )
    .map_err(|e| e.to_string())?;

    Ok(facts
        .into_iter()
        .map(|f| MemoryFactInfo {
            id: f.id,
            character_id: f.character_id,
            session_id: f.session_id,
            fact_type: f.fact_type,
            content: f.content,
            confidence: f.confidence,
            is_visible: f.is_visible,
            created_at: f.created_at,
            updated_at: f.updated_at,
        })
        .collect())
}

/// Create a new memory fact for a character.
#[tauri::command]
pub async fn create_memory_fact(
    db: State<'_, DbState>,
    character_id: String,
    fact_type: String,
    content: String,
) -> Result<MemoryFactInfo, String> {
    let content = content.trim();
    if content.is_empty() {
        return Err("Memory content cannot be empty.".into());
    }

    let valid_types = ["event", "preference", "commitment", "trait", "custom"];
    if !valid_types.contains(&fact_type.as_str()) {
        return Err(format!(
            "Invalid fact type '{}'. Must be one of: {}",
            fact_type,
            valid_types.join(", ")
        ));
    }

    let fact =
        memory_service::create_memory_fact(&db, &character_id, None, &fact_type, content, None)
            .map_err(|e| e.to_string())?;

    Ok(MemoryFactInfo {
        id: fact.id,
        character_id: fact.character_id,
        session_id: fact.session_id,
        fact_type: fact.fact_type,
        content: fact.content,
        confidence: fact.confidence,
        is_visible: fact.is_visible,
        created_at: fact.created_at,
        updated_at: fact.updated_at,
    })
}

/// Soft-delete a memory fact.
#[tauri::command]
pub async fn delete_memory_fact(db: State<'_, DbState>, fact_id: String) -> Result<bool, String> {
    memory::soft_delete_memory_fact(&db, &fact_id).map_err(|e| e.to_string())
}

/// Toggle whether a memory fact is injected into prompts.
#[tauri::command]
pub async fn toggle_memory_visibility(
    db: State<'_, DbState>,
    fact_id: String,
    visible: bool,
) -> Result<bool, String> {
    memory::toggle_memory_visibility(&db, &fact_id, visible).map_err(|e| e.to_string())
}
