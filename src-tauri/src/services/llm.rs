use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::AppError;

const CHAT_COMPLETION_TIMEOUT: Duration = Duration::from_secs(120);
const CHAT_HEALTH_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Option<DeltaContent>,
}

#[derive(Debug, Deserialize)]
struct DeltaContent {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize)]
struct ChatHealthRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    max_tokens: u32,
}

fn build_chat_request(client: &Client, base_url: &str, api_key: &str) -> reqwest::RequestBuilder {
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
    let request = client.post(url).header("Content-Type", "application/json");

    if api_key.trim().is_empty() {
        request
    } else {
        request.header("Authorization", format!("Bearer {}", api_key))
    }
}

fn build_timed_client(timeout: Duration) -> Result<Client, AppError> {
    Client::builder()
        .timeout(timeout)
        .build()
        .map_err(AppError::from)
}

/// Send a chat completion request to an OpenAI-compatible API
/// and stream tokens back via the provided callback.
pub async fn stream_chat_completion<F>(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    mut on_token: F,
) -> Result<String, AppError>
where
    F: FnMut(String),
{
    let client = build_timed_client(CHAT_COMPLETION_TIMEOUT)?;

    let request_body = ChatRequest {
        model: model.to_string(),
        messages,
        stream: true,
    };

    let response = build_chat_request(&client, base_url, api_key).json(&request_body);

    let response = response.send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Api(format!("API returned {}: {}", status, body)));
    }

    let mut stream = response.bytes_stream();
    let mut full_response = String::new();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Network(e.to_string()))?;
        let chunk_str = String::from_utf8_lossy(&chunk);
        buffer.push_str(&chunk_str);

        // Process complete SSE lines
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].trim().to_string();
            buffer = buffer[line_end + 1..].to_string();

            if line.is_empty() || line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    return Ok(full_response);
                }

                if let Ok(parsed) = serde_json::from_str::<StreamResponse>(data) {
                    for choice in &parsed.choices {
                        if let Some(delta) = &choice.delta {
                            if let Some(content) = &delta.content {
                                full_response.push_str(content);
                                on_token(content.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(full_response)
}

pub async fn check_chat_endpoint(
    base_url: &str,
    api_key: &str,
    model: &str,
) -> Result<(), AppError> {
    let client = build_timed_client(CHAT_HEALTH_TIMEOUT)?;
    let request_body = ChatHealthRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: "user".into(),
            content: "ping".into(),
        }],
        stream: false,
        max_tokens: 1,
    };

    let response = build_chat_request(&client, base_url, api_key)
        .json(&request_body)
        .send()
        .await?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(AppError::Api(format!("API returned {}: {}", status, body)))
}
