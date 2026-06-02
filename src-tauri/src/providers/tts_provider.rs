//! TTS (Text-to-Speech) provider trait and implementations.
//!
//! Extracts the existing Edge TTS logic into a trait-based abstraction,
//! enabling future providers (ElevenLabs, Cartesia, OpenAI audio, etc.).

use serde::{Deserialize, Serialize};

use super::provider_error::ProviderError;

/// Configuration for a specific voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceConfig {
    /// Voice identifier (provider-specific).
    pub voice_id: String,
    /// Optional language code.
    pub language: Option<String>,
}

/// Information about an available voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInfo {
    /// Provider-specific voice ID.
    pub id: String,
    /// Human-readable display name.
    pub name: String,
    /// Language code (e.g., "en-US").
    pub language: String,
    /// Gender, if available.
    pub gender: Option<String>,
}

/// Synthesized audio output.
#[derive(Debug, Clone)]
pub struct AudioOutput {
    /// Raw audio bytes (typically MP3 or WAV).
    pub data: Vec<u8>,
    /// MIME type of the audio data.
    pub mime_type: String,
}

/// Trait for TTS providers.
#[allow(async_fn_in_trait)]
pub trait TtsProvider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Synthesize speech from text.
    async fn synthesize(&self, text: &str, voice: &VoiceConfig) -> Result<AudioOutput, ProviderError>;

    /// List available voices.
    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, ProviderError>;
}

/// Microsoft Edge TTS provider.
///
/// Wraps the existing `services::tts` module into the `TtsProvider` trait.
pub struct EdgeTtsProvider;

impl TtsProvider for EdgeTtsProvider {
    fn name(&self) -> &str {
        "Edge TTS"
    }

    async fn synthesize(&self, text: &str, voice: &VoiceConfig) -> Result<AudioOutput, ProviderError> {
        let bytes = crate::services::tts::synthesize(text, &voice.voice_id)
            .await
            .map_err(|e| ProviderError::Internal(e.to_string()))?;

        Ok(AudioOutput {
            data: bytes,
            mime_type: "audio/mpeg".into(),
        })
    }

    async fn list_voices(&self) -> Result<Vec<VoiceInfo>, ProviderError> {
        let voices = crate::services::tts::list_voices()
            .map_err(ProviderError::Unavailable)?;

        Ok(voices
            .into_iter()
            .map(|v| VoiceInfo {
                id: v.name.clone(),
                name: v.short_name.unwrap_or_default(),
                language: v.locale.unwrap_or_default(),
                gender: v.gender,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_tts_provider_has_correct_name() {
        let provider = EdgeTtsProvider;
        assert_eq!(provider.name(), "Edge TTS");
    }

    #[test]
    fn voice_config_can_be_constructed() {
        let config = VoiceConfig {
            voice_id: "en-US-AriaNeural".into(),
            language: Some("en-US".into()),
        };
        assert_eq!(config.voice_id, "en-US-AriaNeural");
    }

    #[test]
    fn audio_output_stores_data() {
        let output = AudioOutput {
            data: vec![0xFF, 0xFB],
            mime_type: "audio/mpeg".into(),
        };
        assert_eq!(output.data.len(), 2);
        assert_eq!(output.mime_type, "audio/mpeg");
    }
}
