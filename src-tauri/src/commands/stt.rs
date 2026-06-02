use crate::services::stt::transcribe_audio;
use crate::state::AppState;
use tauri::command;

#[command]
pub async fn transcribe(
    state: tauri::State<'_, AppState>,
    audio_data: Vec<u8>,
    mime_type: String,
) -> Result<String, String> {
    let config = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    if !config.provider.supports_whisper_stt() {
        return Err(
            "Whisper STT is unavailable when the provider is set to Ollama. Switch to OpenAI or another compatible speech endpoint to use voice input."
                .to_string(),
        );
    }

    let api_key = config.get_api_key().map_err(|e| e.to_string())?;

    transcribe_audio(&config.base_url, &api_key, audio_data, &mime_type).await
}
