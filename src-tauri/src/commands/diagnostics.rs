use serde::Serialize;
use tauri::State;

use crate::services::{llm::check_chat_endpoint, tts::list_voices};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticCheck {
    pub status: String,
    pub summary: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDiagnostics {
    pub chat: DiagnosticCheck,
    pub stt: DiagnosticCheck,
    pub tts: DiagnosticCheck,
}

#[tauri::command]
pub async fn run_runtime_diagnostics(
    state: State<'_, AppState>,
) -> Result<RuntimeDiagnostics, String> {
    let config = {
        let mut guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.check_has_key();
        guard.clone()
    };

    let chat = if config.provider.requires_api_key() && !config.has_keychain_entry {
        DiagnosticCheck {
            status: "error".into(),
            summary: "Chat endpoint is not ready yet.".into(),
            details: "This provider requires an API key. Save a key in Settings, then run diagnostics again.".into(),
        }
    } else {
        let api_key = config.get_api_key().map_err(|e| e.to_string())?;
        match check_chat_endpoint(&config.base_url, &api_key, &config.model).await {
            Ok(()) => DiagnosticCheck {
                status: "success".into(),
                summary: "Live chat request succeeded.".into(),
                details: format!(
                    "The configured model '{}' responded from '{}'.",
                    config.model, config.base_url
                ),
            },
            Err(error) => DiagnosticCheck {
                status: "error".into(),
                summary: "Live chat request failed.".into(),
                details: error.to_string(),
            },
        }
    };

    let stt = if !config.provider.supports_whisper_stt() {
        DiagnosticCheck {
            status: "warning".into(),
            summary: "Whisper STT is disabled for this provider.".into(),
            details: "The current Ollama path is treated as chat-only for Whisper. The frontend can still try browser speech recognition when the WebView supports it.".into(),
        }
    } else if config.provider.requires_api_key() && !config.has_keychain_entry {
        DiagnosticCheck {
            status: "error".into(),
            summary: "Whisper STT is blocked by missing credentials.".into(),
            details: "Save an API key before testing voice input with the microphone.".into(),
        }
    } else if !config.has_keychain_entry {
        DiagnosticCheck {
            status: "warning".into(),
            summary: "Whisper STT is configuration-ready, but auth is uncertain.".into(),
            details: "This endpoint may still require an API key. If microphone transcription fails later, save credentials and retry.".into(),
        }
    } else {
        DiagnosticCheck {
            status: "success".into(),
            summary: "Whisper STT is ready for a live microphone test.".into(),
            details: "The provider supports speech transcription and a key is already stored. Use the mic button once to verify end-to-end recording and upload.".into(),
        }
    };

    let tts = match list_voices() {
        Ok(voices) => DiagnosticCheck {
            status: "success".into(),
            summary: "Edge TTS voice catalog loaded.".into(),
            details: format!(
                "Loaded {} voices from Edge TTS. Use the preview button in Settings to confirm playback.",
                voices.len()
            ),
        },
        Err(error) => DiagnosticCheck {
            status: "error".into(),
            summary: "Edge TTS voice catalog could not be loaded.".into(),
            details: error,
        },
    };

    Ok(RuntimeDiagnostics { chat, stt, tts })
}
