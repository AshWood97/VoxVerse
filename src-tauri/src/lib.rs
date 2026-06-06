#![allow(dead_code)]
#![allow(unused_imports)]

mod commands;
mod error;
mod protocol;
mod providers;
mod repositories;
mod services;
mod state;

use repositories::db::DbState;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .setup(|app| {
            // Initialize database (creates tables, seeds scenarios)
            repositories::db::init(app.handle())?;

            // Load persisted API config profile from DB into runtime AppState
            let db_state: tauri::State<'_, DbState> = app.state();
            if let Ok(Some(profile)) = repositories::config::load_active_config_profile(&db_state) {
                let app_state: tauri::State<'_, AppState> = app.state();
                let mut config = app_state.config.lock().map_err(|_| {
                    error::AppError::Config("Runtime config lock was poisoned.".into())
                })?;
                config.profile_id = profile.id;
                config.profile_name = profile.name;
                config.base_url = profile.base_url;
                config.model = profile.model;
                config.provider = state::AiProvider::from_stored_value(&profile.provider)
                    .or_else(|| Some(state::AiProvider::infer_from_base_url(&config.base_url)))
                    .unwrap_or_default();
                config.check_has_key();
            } else if let Ok((provider, base_url, model)) =
                repositories::config::load_api_config(&db_state)
            {
                let app_state: tauri::State<'_, AppState> = app.state();
                let mut config = app_state.config.lock().map_err(|_| {
                    error::AppError::Config("Runtime config lock was poisoned.".into())
                })?;
                if let Some(url) = base_url {
                    config.base_url = url;
                }
                if let Some(m) = model {
                    config.model = m;
                }
                config.provider = provider
                    .as_deref()
                    .and_then(state::AiProvider::from_stored_value)
                    .unwrap_or_else(|| state::AiProvider::infer_from_base_url(&config.base_url));
                config.check_has_key();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Chat
            commands::chat::send_message,
            commands::chat::send_session_message,
            commands::chat::save_config,
            commands::chat::get_config,
            commands::chat::list_config_profiles,
            commands::chat::create_config_profile,
            commands::chat::switch_config_profile,
            commands::chat::rename_config_profile,
            commands::chat::delete_config_profile,
            commands::chat::clear_active_api_key,
            // Characters
            commands::character::get_characters,
            commands::character::save_character,
            commands::character::delete_character,
            // Scenarios
            commands::scenario::get_scenarios,
            commands::scenario::save_scenario,
            commands::scenario::delete_scenario,
            // Practice modes
            commands::practice::get_practice_modes,
            // Vocabulary
            commands::vocabulary::get_vocabulary,
            commands::vocabulary::save_vocabulary,
            commands::vocabulary::delete_vocabulary,
            // Feedback
            commands::feedback::analyze_text,
            commands::feedback::translate_text,
            commands::feedback::polish_text,
            commands::feedback::save_correction,
            commands::feedback::generate_summary,
            // Session
            commands::session::get_sessions,
            commands::session::get_session_messages,
            commands::session::create_session,
            commands::session::get_session_context,
            commands::session::update_session_context,
            commands::session::update_session_title,
            commands::session::delete_session,
            commands::session::save_message,
            // TTS & STT
            commands::tts::synthesize_speech,
            commands::tts::get_tts_voices,
            commands::stt::transcribe,
            commands::diagnostics::run_runtime_diagnostics,
            // Stats
            commands::stats::get_learning_stats,
            // Memory (V1)
            commands::memory::get_memory_facts,
            commands::memory::create_memory_fact,
            commands::memory::delete_memory_fact,
            commands::memory::clear_character_memory,
            commands::memory::toggle_memory_visibility,
            // Relationship (V1)
            commands::relationship::get_relationship_state,
            commands::relationship::update_relationship_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
