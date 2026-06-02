use reqwest::{multipart, Client};
use serde::Deserialize;
use std::time::Duration;

pub const MAX_STT_AUDIO_BYTES: usize = 25 * 1024 * 1024;
const STT_REQUEST_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Deserialize)]
struct WhisperResponse {
    text: String,
}

pub fn validate_audio_payload(audio_data: &[u8], mime_type: &str) -> Result<(), String> {
    if audio_data.is_empty() {
        return Err("Audio recording is empty.".into());
    }

    if audio_data.len() > MAX_STT_AUDIO_BYTES {
        return Err(format!(
            "Audio recording is too large ({} bytes). Please keep recordings under {} MB.",
            audio_data.len(),
            MAX_STT_AUDIO_BYTES / 1024 / 1024
        ));
    }

    let normalized_mime = mime_type.trim().to_ascii_lowercase();
    if normalized_mime.is_empty() {
        return Err("Audio MIME type is missing.".into());
    }

    if !normalized_mime.starts_with("audio/") {
        return Err(format!("Unsupported audio MIME type: {}", mime_type));
    }

    Ok(())
}

pub async fn transcribe_audio(
    base_url: &str,
    api_key: &str,
    audio_data: Vec<u8>,
    mime_type: &str,
) -> Result<String, String> {
    validate_audio_payload(&audio_data, mime_type)?;

    // Determine file extension based on mime_type.
    // Typical mime_type from MediaRecorder is "audio/webm;codecs=opus"
    let ext = if mime_type.contains("mp4") {
        "m4a"
    } else if mime_type.contains("mpeg") {
        "mp3"
    } else {
        "webm" // default for browsers
    };

    let file_name = format!("audio.{}", ext);
    let url = format!("{}/audio/transcriptions", base_url.trim_end_matches('/'));

    let client = Client::builder()
        .timeout(STT_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("Network client error: {}", e))?;

    let part = multipart::Part::bytes(audio_data)
        .file_name(file_name)
        .mime_str(mime_type)
        .map_err(|e| e.to_string())?;

    let form = multipart::Form::new()
        .part("file", part)
        .text("model", "whisper-1");

    let res = client
        .post(&url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Whisper API Error: {}", err_text));
    }

    let parsed: WhisperResponse = res
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(parsed.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_supported_audio_payload() {
        assert!(validate_audio_payload(&[1, 2, 3], "audio/webm;codecs=opus").is_ok());
    }

    #[test]
    fn rejects_empty_audio_payload() {
        assert!(validate_audio_payload(&[], "audio/webm").is_err());
    }

    #[test]
    fn rejects_oversized_audio_payload() {
        let payload = vec![0; MAX_STT_AUDIO_BYTES + 1];
        let error = validate_audio_payload(&payload, "audio/webm").unwrap_err();
        assert!(error.contains("too large"));
    }

    #[test]
    fn rejects_non_audio_mime_type() {
        let error = validate_audio_payload(&[1, 2, 3], "application/json").unwrap_err();
        assert!(error.contains("Unsupported audio MIME type"));
    }
}
