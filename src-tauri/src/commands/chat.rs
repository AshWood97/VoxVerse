use serde::{Deserialize, Serialize};
use tauri::{ipc::Channel, State};
use uuid::Uuid;

use crate::repositories::config::ConfigProfileRecord;
use crate::repositories::db::DbState;
use crate::services::llm::{stream_chat_completion, ChatMessage};
use crate::services::memory_service;
use crate::state::{
    delete_api_key_for_profile, has_api_key_for_profile, AiProvider, ApiConfig, AppState,
    DEFAULT_PROFILE_ID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePayload {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamEvent {
    pub token: String,
    pub done: bool,
}

/// Send a message to the LLM and stream the response back via a channel.
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    db: State<'_, DbState>,
    system_prompt: String,
    messages: Vec<MessagePayload>,
    on_event: Channel<StreamEvent>,
    character_id: Option<String>,
    session_id: Option<String>,
) -> Result<String, String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let api_key = config.get_api_key().map_err(|e| e.to_string())?;

    let final_system_prompt = build_contextual_system_prompt(
        &db,
        &system_prompt,
        character_id.as_deref(),
        session_id.as_deref(),
    )?;

    // Build messages array with system prompt
    let mut chat_messages = vec![ChatMessage {
        role: "system".into(),
        content: final_system_prompt,
    }];

    for msg in &messages {
        chat_messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    let on_event_clone = on_event.clone();
    let full_response = stream_chat_completion(
        &config.base_url,
        &api_key,
        &config.model,
        chat_messages,
        move |token| {
            let _ = on_event_clone.send(StreamEvent { token, done: false });
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    // Send done signal
    let _ = on_event.send(StreamEvent {
        token: String::new(),
        done: true,
    });

    Ok(full_response)
}

/// Session-aware send path that injects per-character memory and relationship context.
#[tauri::command]
pub async fn send_session_message(
    state: State<'_, AppState>,
    db: State<'_, DbState>,
    system_prompt: String,
    messages: Vec<MessagePayload>,
    on_event: Channel<StreamEvent>,
    character_id: String,
    session_id: String,
) -> Result<String, String> {
    send_message(
        state,
        db,
        system_prompt,
        messages,
        on_event,
        Some(character_id),
        Some(session_id),
    )
    .await
}

fn build_contextual_system_prompt(
    db: &DbState,
    system_prompt: &str,
    character_id: Option<&str>,
    _session_id: Option<&str>,
) -> Result<String, String> {
    let Some(character_id) = character_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return Ok(system_prompt.to_string());
    };

    let mut context_blocks = Vec::new();

    if let Some(relationship_context) =
        memory_service::build_relationship_context(db, character_id).map_err(|e| e.to_string())?
    {
        context_blocks.push(format!("[Relationship Context]\n{relationship_context}"));
    }

    if let Some(memory_context) = memory_service::build_memory_prompt_injection(db, character_id)
        .map_err(|e| e.to_string())?
    {
        context_blocks.push(format!("[Memory Context]\n{memory_context}"));
    }

    if context_blocks.is_empty() {
        return Ok(system_prompt.to_string());
    }

    Ok(format!(
        "{}\n\n{}",
        system_prompt.trim_end(),
        context_blocks.join("\n\n")
    ))
}

/// Save API configuration and persist non-secret fields to SQLite.
#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    db: State<'_, crate::repositories::db::DbState>,
    provider: AiProvider,
    base_url: String,
    api_key: String,
    model: String,
) -> Result<(), String> {
    let (normalized_base_url, normalized_model) = validate_provider_fields(&base_url, &model)?;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.provider = provider;
    config.base_url = normalized_base_url;
    config.model = normalized_model;
    if !api_key.is_empty() {
        config.set_api_key(&api_key).map_err(|e| e.to_string())?;
    }

    // Persist to SQLite so config survives app restarts
    crate::repositories::config::save_api_config(
        &db,
        &config.profile_id,
        &config.profile_name,
        config.provider.as_str(),
        &config.base_url,
        &config.model,
        true,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<ConfigInfo, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.check_has_key(); // Ensure we have the latest state from keyring

    Ok(config_info_from_runtime(&config))
}

#[tauri::command]
pub async fn list_config_profiles(
    db: State<'_, crate::repositories::db::DbState>,
) -> Result<Vec<ConfigProfileInfo>, String> {
    let profiles =
        crate::repositories::config::list_config_profiles(&db).map_err(|e| e.to_string())?;

    Ok(profiles.into_iter().map(profile_info_from_record).collect())
}

#[tauri::command]
pub async fn switch_config_profile(
    state: State<'_, AppState>,
    db: State<'_, crate::repositories::db::DbState>,
    profile_id: String,
) -> Result<ConfigInfo, String> {
    let profile = crate::repositories::config::get_config_profile(&db, &profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Config profile '{}' was not found.", profile_id))?;

    crate::repositories::config::set_active_config_profile(&db, &profile)
        .map_err(|e| e.to_string())?;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    apply_profile_to_runtime(&mut config, &profile);
    config.check_has_key();

    Ok(config_info_from_runtime(&config))
}

#[tauri::command]
pub async fn create_config_profile(
    state: State<'_, AppState>,
    db: State<'_, crate::repositories::db::DbState>,
    name: String,
    provider: AiProvider,
    base_url: String,
    api_key: String,
    model: String,
) -> Result<ConfigInfo, String> {
    let normalized_name = validate_profile_name(&name)?;
    ensure_unique_profile_name(&db, normalized_name, None)?;
    let (normalized_base_url, normalized_model) = validate_provider_fields(&base_url, &model)?;

    if provider.requires_api_key() && api_key.trim().is_empty() {
        return Err("This provider requires an API key for a new profile.".into());
    }

    let profile_id = Uuid::new_v4().to_string();
    let mut config = ApiConfig {
        profile_id: profile_id.clone(),
        profile_name: normalized_name.into(),
        provider,
        base_url: normalized_base_url,
        model: normalized_model,
        has_keychain_entry: false,
    };

    if !api_key.trim().is_empty() {
        config
            .set_api_key(api_key.trim())
            .map_err(|e| e.to_string())?;
    }

    crate::repositories::config::save_api_config(
        &db,
        &config.profile_id,
        &config.profile_name,
        config.provider.as_str(),
        &config.base_url,
        &config.model,
        true,
    )
    .map_err(|e| e.to_string())?;

    config.check_has_key();

    let mut runtime_config = state.config.lock().map_err(|e| e.to_string())?;
    *runtime_config = config.clone();

    Ok(config_info_from_runtime(&config))
}

#[tauri::command]
pub async fn rename_config_profile(
    state: State<'_, AppState>,
    db: State<'_, crate::repositories::db::DbState>,
    profile_id: String,
    name: String,
) -> Result<ConfigInfo, String> {
    let profile = crate::repositories::config::get_config_profile(&db, &profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Config profile '{}' was not found.", profile_id))?;
    let normalized_name = validate_profile_name(&name)?;
    ensure_unique_profile_name(&db, normalized_name, Some(&profile_id))?;

    crate::repositories::config::rename_config_profile(&db, &profile_id, normalized_name)
        .map_err(|e| e.to_string())?;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if config.profile_id == profile.id {
        config.profile_name = normalized_name.into();
    }
    config.check_has_key();

    Ok(config_info_from_runtime(&config))
}

#[tauri::command]
pub async fn delete_config_profile(
    state: State<'_, AppState>,
    db: State<'_, crate::repositories::db::DbState>,
    profile_id: String,
) -> Result<ConfigInfo, String> {
    let profile = crate::repositories::config::get_config_profile(&db, &profile_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Config profile '{}' was not found.", profile_id))?;

    if profile.id == DEFAULT_PROFILE_ID {
        return Err("The default config profile cannot be deleted.".into());
    }

    let fallback_profile = if profile.is_default {
        Some(
            crate::repositories::config::get_fallback_config_profile(
                &db,
                &profile.id,
                DEFAULT_PROFILE_ID,
            )
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Cannot delete the only remaining config profile.".to_string())?,
        )
    } else {
        None
    };

    crate::repositories::config::delete_config_profile(&db, &profile.id)
        .map_err(|e| e.to_string())?;
    delete_api_key_for_profile(&profile.id).map_err(|e| e.to_string())?;

    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    if let Some(fallback) = fallback_profile {
        crate::repositories::config::set_active_config_profile(&db, &fallback)
            .map_err(|e| e.to_string())?;
        apply_profile_to_runtime(&mut config, &fallback);
    }
    config.check_has_key();

    Ok(config_info_from_runtime(&config))
}

#[tauri::command]
pub async fn clear_active_api_key(state: State<'_, AppState>) -> Result<ConfigInfo, String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    delete_api_key_for_profile(&config.profile_id).map_err(|e| e.to_string())?;
    config.has_keychain_entry = false;
    Ok(config_info_from_runtime(&config))
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigInfo {
    pub active_profile_id: String,
    pub active_profile_name: String,
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
    pub requires_api_key: bool,
    pub supports_stt: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigProfileInfo {
    pub id: String,
    pub name: String,
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
    pub is_default: bool,
    pub has_api_key: bool,
    pub requires_api_key: bool,
    pub supports_stt: bool,
}

fn apply_profile_to_runtime(config: &mut ApiConfig, profile: &ConfigProfileRecord) {
    config.profile_id = profile.id.clone();
    config.profile_name = profile.name.clone();
    config.provider = AiProvider::from_stored_value(&profile.provider)
        .unwrap_or_else(|| AiProvider::infer_from_base_url(&profile.base_url));
    config.base_url = profile.base_url.clone();
    config.model = profile.model.clone();
}

fn config_info_from_runtime(config: &ApiConfig) -> ConfigInfo {
    ConfigInfo {
        active_profile_id: config.profile_id.clone(),
        active_profile_name: config.profile_name.clone(),
        provider: config.provider,
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        has_api_key: config.has_keychain_entry,
        requires_api_key: config.provider.requires_api_key(),
        supports_stt: config.provider.supports_whisper_stt(),
    }
}

fn profile_info_from_record(profile: ConfigProfileRecord) -> ConfigProfileInfo {
    let provider = AiProvider::from_stored_value(&profile.provider)
        .unwrap_or_else(|| AiProvider::infer_from_base_url(&profile.base_url));
    let has_api_key = has_api_key_for_profile(&profile.id);

    ConfigProfileInfo {
        id: profile.id,
        name: profile.name,
        provider,
        base_url: profile.base_url,
        model: profile.model,
        is_default: profile.is_default,
        has_api_key,
        requires_api_key: provider.requires_api_key(),
        supports_stt: provider.supports_whisper_stt(),
    }
}

fn validate_profile_name(name: &str) -> Result<&str, String> {
    let normalized_name = name.trim();
    if normalized_name.is_empty() {
        return Err("Profile name is required.".into());
    }

    if normalized_name.chars().count() > 80 {
        return Err("Profile name must be 80 characters or fewer.".into());
    }

    Ok(normalized_name)
}

fn validate_provider_fields(base_url: &str, model: &str) -> Result<(String, String), String> {
    let normalized_base_url = base_url.trim();
    if normalized_base_url.is_empty() {
        return Err("API base URL is required.".into());
    }

    if !normalized_base_url.starts_with("https://") && !normalized_base_url.starts_with("http://") {
        return Err("API base URL must start with http:// or https://.".into());
    }

    let normalized_model = model.trim();
    if normalized_model.is_empty() {
        return Err("Model is required.".into());
    }

    Ok((normalized_base_url.into(), normalized_model.into()))
}

fn ensure_unique_profile_name(
    db: &crate::repositories::db::DbState,
    profile_name: &str,
    current_profile_id: Option<&str>,
) -> Result<(), String> {
    let existing = crate::repositories::config::get_config_profile_by_name(db, profile_name)
        .map_err(|e| e.to_string())?;

    if let Some(existing) = existing {
        if current_profile_id != Some(existing.id.as_str()) {
            return Err(format!(
                "A config profile named '{}' already exists.",
                profile_name
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_contextual_system_prompt, validate_profile_name, validate_provider_fields};
    use crate::repositories::db::DbState;
    use rusqlite::Connection;
    use std::sync::Mutex;

    #[test]
    fn validates_profile_name_boundaries() {
        assert_eq!(validate_profile_name("  Work  ").unwrap(), "Work");
        assert!(validate_profile_name("   ").is_err());
        assert!(validate_profile_name(&"a".repeat(81)).is_err());
    }

    #[test]
    fn validates_provider_fields() {
        let (base_url, model) =
            validate_provider_fields("  http://localhost:11434/v1  ", " llama3.2 ").unwrap();
        assert_eq!(base_url, "http://localhost:11434/v1");
        assert_eq!(model, "llama3.2");
        assert!(validate_provider_fields("", "gpt-4o-mini").is_err());
        assert!(validate_provider_fields("api.openai.com/v1", "gpt-4o-mini").is_err());
        assert!(validate_provider_fields("https://api.openai.com/v1", "").is_err());
    }

    #[test]
    fn contextual_prompt_injects_relationship_and_visible_memory() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE relationship_state (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL UNIQUE,
                intimacy_level INTEGER NOT NULL DEFAULT 0,
                trust_level INTEGER NOT NULL DEFAULT 0,
                plot_stage TEXT, learning_goal TEXT, user_preferences TEXT,
                boundaries TEXT, commitments TEXT, updated_at TEXT NOT NULL
            );
            CREATE TABLE memory_facts (
                id TEXT PRIMARY KEY, character_id TEXT NOT NULL,
                session_id TEXT, fact_type TEXT NOT NULL, content TEXT NOT NULL,
                source_turn_id TEXT, confidence REAL NOT NULL DEFAULT 1.0,
                is_visible INTEGER NOT NULL DEFAULT 1, is_deleted INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL, updated_at TEXT NOT NULL
            );",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO relationship_state
                (id, character_id, intimacy_level, trust_level, updated_at)
             VALUES ('rel-1', 'c1', 4, 7, 'now')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO memory_facts
                (id, character_id, fact_type, content, created_at, updated_at)
             VALUES ('mem-1', 'c1', 'preference', 'User likes short answers.', 'now', 'now')",
            [],
        )
        .unwrap();

        let db = DbState {
            conn: Mutex::new(conn),
        };

        let prompt =
            build_contextual_system_prompt(&db, "You are Emily.", Some("c1"), Some("s1")).unwrap();

        assert!(prompt.starts_with("You are Emily."));
        assert!(prompt.contains("[Relationship Context]"));
        assert!(prompt.contains("intimacy=4"));
        assert!(prompt.contains("[Memory Context]"));
        assert!(prompt.contains("User likes short answers."));
    }
}
