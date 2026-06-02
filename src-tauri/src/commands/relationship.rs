//! Tauri commands for relationship state management.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::repositories::db::DbState;
use crate::repositories::memory;
use crate::services::memory_service;

#[derive(Debug, Clone, Serialize)]
pub struct RelationshipInfo {
    pub id: String,
    pub character_id: String,
    pub intimacy_level: i64,
    pub trust_level: i64,
    pub plot_stage: Option<String>,
    pub user_preferences: Option<serde_json::Value>,
    pub boundaries: Option<serde_json::Value>,
    pub commitments: Option<serde_json::Value>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct RelationshipUpdate {
    pub intimacy_level: Option<i64>,
    pub trust_level: Option<i64>,
    pub plot_stage: Option<String>,
    pub user_preferences: Option<serde_json::Value>,
    pub boundaries: Option<serde_json::Value>,
    pub commitments: Option<serde_json::Value>,
}

/// Get the relationship state for a character, creating one if it doesn't exist.
#[tauri::command]
pub async fn get_relationship_state(
    db: State<'_, DbState>,
    character_id: String,
) -> Result<RelationshipInfo, String> {
    let state = memory_service::ensure_relationship_state(&db, &character_id)
        .map_err(|e| e.to_string())?;

    Ok(record_to_info(state))
}

/// Update specific fields of a character's relationship state.
#[tauri::command]
pub async fn update_relationship_state(
    db: State<'_, DbState>,
    character_id: String,
    updates: RelationshipUpdate,
) -> Result<RelationshipInfo, String> {
    let mut state = memory_service::ensure_relationship_state(&db, &character_id)
        .map_err(|e| e.to_string())?;

    if let Some(intimacy) = updates.intimacy_level {
        state.intimacy_level = intimacy.clamp(0, 100);
    }
    if let Some(trust) = updates.trust_level {
        state.trust_level = trust.clamp(0, 100);
    }
    if let Some(stage) = updates.plot_stage {
        state.plot_stage = if stage.is_empty() { None } else { Some(stage) };
    }
    if let Some(prefs) = updates.user_preferences {
        state.user_preferences = Some(serde_json::to_string(&prefs).unwrap_or_default());
    }
    if let Some(boundaries) = updates.boundaries {
        state.boundaries = Some(serde_json::to_string(&boundaries).unwrap_or_default());
    }
    if let Some(commitments) = updates.commitments {
        state.commitments = Some(serde_json::to_string(&commitments).unwrap_or_default());
    }

    state.updated_at = chrono::Utc::now().to_rfc3339();
    memory::upsert_relationship_state(&db, &state).map_err(|e| e.to_string())?;

    Ok(record_to_info(state))
}

fn record_to_info(state: memory::RelationshipStateRecord) -> RelationshipInfo {
    RelationshipInfo {
        id: state.id,
        character_id: state.character_id,
        intimacy_level: state.intimacy_level,
        trust_level: state.trust_level,
        plot_stage: state.plot_stage,
        user_preferences: state
            .user_preferences
            .and_then(|s| serde_json::from_str(&s).ok()),
        boundaries: state
            .boundaries
            .and_then(|s| serde_json::from_str(&s).ok()),
        commitments: state
            .commitments
            .and_then(|s| serde_json::from_str(&s).ok()),
        updated_at: state.updated_at,
    }
}
