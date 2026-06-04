//! STT (Speech-to-Text) provider trait and implementations.
//!
//! This module extracts the existing Whisper STT logic into a trait-based
//! abstraction, allowing future providers (Deepgram, AssemblyAI, etc.) to
//! be added without changing the session orchestrator.

use serde::{Deserialize, Serialize};

use super::provider_error::ProviderError;

/// Audio payload for STT transcription.
#[derive(Debug, Clone)]
pub struct AudioPayload {
    /// Raw audio bytes.
    pub data: Vec<u8>,
    /// MIME type (e.g., "audio/webm;codecs=opus").
    pub mime_type: String,
    /// Optional language hint for the STT engine.
    pub language: Option<String>,
}

/// Result of an STT transcription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptResult {
    /// The transcribed text.
    pub text: String,
    /// Confidence score (0.0–1.0), if available.
    pub confidence: Option<f32>,
    /// Whether this is a final or partial transcript.
    pub is_final: bool,
    /// Detected language, if available.
    pub detected_language: Option<String>,
}

/// Trait for STT providers.
///
/// Implementors process audio data and return text transcripts.
/// The default `supports_streaming` returns false; streaming providers
/// should override this and provide streaming via WebSocket or similar.
#[allow(async_fn_in_trait)]
pub trait SttProvider: Send + Sync {
    /// Human-readable provider name (e.g., "Whisper", "Deepgram").
    fn name(&self) -> &str;

    /// Transcribe a complete audio payload.
    async fn transcribe(&self, audio: AudioPayload) -> Result<TranscriptResult, ProviderError>;

    /// Whether this provider supports streaming (partial) transcription.
    fn supports_streaming(&self) -> bool {
        false
    }
}

/// OpenAI Whisper-compatible STT provider.
///
/// Wraps the existing `services::stt::transcribe_audio` logic into the
/// `SttProvider` trait interface.
pub struct WhisperSttProvider {
    pub base_url: String,
    pub api_key: String,
}

impl SttProvider for WhisperSttProvider {
    fn name(&self) -> &str {
        "Whisper"
    }

    async fn transcribe(&self, audio: AudioPayload) -> Result<TranscriptResult, ProviderError> {
        crate::services::stt::validate_audio_payload(&audio.data, &audio.mime_type)
            .map_err(ProviderError::InvalidInput)?;

        let text = crate::services::stt::transcribe_audio(
            &self.base_url,
            &self.api_key,
            audio.data,
            &audio.mime_type,
        )
        .await
        .map_err(ProviderError::Network)?;

        Ok(TranscriptResult {
            text,
            confidence: None,
            is_final: true,
            detected_language: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whisper_provider_has_correct_name() {
        let provider = WhisperSttProvider {
            base_url: "https://api.openai.com/v1".into(),
            api_key: "test".into(),
        };
        assert_eq!(provider.name(), "Whisper");
        assert!(!provider.supports_streaming());
    }

    #[test]
    fn audio_payload_can_be_constructed() {
        let payload = AudioPayload {
            data: vec![1, 2, 3],
            mime_type: "audio/webm".into(),
            language: Some("en".into()),
        };
        assert_eq!(payload.data.len(), 3);
        assert!(payload.language.is_some());
    }
}
