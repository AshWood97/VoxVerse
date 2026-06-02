use crate::services::tts::{list_voices, synthesize};
use serde::Serialize;
use tauri::command;

/// Minimal voice info sent to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceInfo {
    pub name: String,
    pub friendly_name: String,
    pub locale: String,
    pub gender: String,
}

#[command]
pub async fn synthesize_speech(text: String, voice_name: String) -> Result<Vec<u8>, String> {
    synthesize(&text, &voice_name)
        .await
        .map_err(|e| e.to_string())
}

/// Return the list of available Edge TTS voices (cached after first call).
#[command]
pub async fn get_tts_voices() -> Result<Vec<VoiceInfo>, String> {
    let voices = list_voices()?;
    Ok(voices
        .into_iter()
        .map(|v| VoiceInfo {
            name: v.name,
            friendly_name: v.friendly_name.unwrap_or_default(),
            locale: v.locale.unwrap_or_default(),
            gender: v.gender.unwrap_or_default(),
        })
        .collect())
}
