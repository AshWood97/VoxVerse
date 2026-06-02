use msedge_tts::{tts::client::connect, tts::SpeechConfig, voice::get_voices_list, voice::Voice};
use std::error::Error;
use std::sync::OnceLock;
use std::time::Duration;

pub const MAX_TTS_TEXT_CHARS: usize = 4_000;
const TTS_SYNTHESIS_TIMEOUT: Duration = Duration::from_secs(45);

/// Global voice list cache, fetched once and reused for the process lifetime.
static VOICES_CACHE: OnceLock<Vec<Voice>> = OnceLock::new();

pub fn validate_tts_text(text: &str) -> Result<&str, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err("TTS text is empty.".into());
    }

    let char_count = trimmed.chars().count();
    if char_count > MAX_TTS_TEXT_CHARS {
        return Err(format!(
            "TTS text is too long ({} characters). Please keep it under {} characters.",
            char_count, MAX_TTS_TEXT_CHARS
        ));
    }

    Ok(trimmed)
}

/// Fetch the voice list, using the in-process cache if already loaded.
fn load_voices() -> Result<&'static Vec<Voice>, String> {
    if let Some(voices) = VOICES_CACHE.get() {
        return Ok(voices);
    }
    let voices =
        get_voices_list().map_err(|e| format!("Failed to fetch Edge TTS voices: {}", e))?;
    Ok(VOICES_CACHE.get_or_init(|| voices))
}

/// Return all cached voices (fetches on first call).
/// Used by the `get_tts_voices` Tauri command.
pub fn list_voices() -> Result<Vec<Voice>, String> {
    load_voices().cloned()
}

/// Build a SpeechConfig for the given voice name.
/// Falls back to the first available voice if name is not found.
fn config_for_voice(voice_name: &str) -> Result<SpeechConfig, String> {
    let voices = load_voices()?;
    let matched = voices.iter().find(|v| v.name == voice_name);
    match matched {
        Some(v) => Ok(SpeechConfig::from(v)),
        None => voices
            .first()
            .map(|first| {
                let mut cfg = SpeechConfig::from(first);
                cfg.voice_name = voice_name.to_string();
                cfg
            })
            .ok_or_else(|| "No voices available from Edge TTS".to_string()),
    }
}

fn boxed_io_error(message: impl Into<String>) -> Box<dyn Error + Send + Sync> {
    Box::new(std::io::Error::other(message.into()))
}

/// Synthesize speech using Microsoft Edge TTS and return raw MP3 bytes.
///
/// Runs the synchronous msedge-tts client in a blocking thread to avoid
/// conflicts between msedge-tts's async-std runtime and Tauri's tokio runtime.
pub async fn synthesize(
    text: &str,
    voice_name: &str,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let text = validate_tts_text(text).map_err(boxed_io_error)?.to_string();
    let voice_name = voice_name.to_string();

    let result = tokio::time::timeout(
        TTS_SYNTHESIS_TIMEOUT,
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>, String> {
            let speech_config = config_for_voice(&voice_name)?;
            let mut tts = connect().map_err(|e| format!("TTS connect error: {}", e))?;
            let audio = tts
                .synthesize(&text, &speech_config)
                .map_err(|e| format!("TTS synthesize error: {}", e))?;
            Ok(audio.audio_bytes)
        }),
    )
    .await
    .map_err(|_| boxed_io_error("TTS synthesis timed out."))?
    .map_err(|e| boxed_io_error(e.to_string()))?;

    result.map_err(boxed_io_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_trimmed_tts_text() {
        assert_eq!(validate_tts_text("  hello  ").unwrap(), "hello");
    }

    #[test]
    fn rejects_empty_tts_text() {
        assert!(validate_tts_text("   ").is_err());
    }

    #[test]
    fn rejects_oversized_tts_text() {
        let long_text = "a".repeat(MAX_TTS_TEXT_CHARS + 1);
        let error = validate_tts_text(&long_text).unwrap_err();
        assert!(error.contains("too long"));
    }
}
