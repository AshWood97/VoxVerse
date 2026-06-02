//! LLM provider trait and implementations.
//!
//! Extracts the existing OpenAI-compatible chat completion logic into a
//! trait-based abstraction. Future providers can implement this trait
//! for different LLM backends.

use serde::{Deserialize, Serialize};

use super::provider_error::ProviderError;

/// A chat message for LLM completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Configuration for a chat completion request.
#[derive(Debug, Clone)]
pub struct CompletionConfig {
    /// Model name.
    pub model: String,
    /// Whether to stream tokens.
    pub stream: bool,
    /// Maximum tokens to generate (None = provider default).
    pub max_tokens: Option<u32>,
    /// Temperature (None = provider default).
    pub temperature: Option<f32>,
}

impl Default for CompletionConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o-mini".into(),
            stream: true,
            max_tokens: None,
            temperature: None,
        }
    }
}

/// Result from a chat completion.
#[derive(Debug, Clone)]
pub struct CompletionResult {
    /// The full response text.
    pub text: String,
    /// Tool calls requested by the LLM, if any.
    pub tool_calls: Vec<LlmToolCall>,
    /// Token usage information, if available.
    pub usage: Option<TokenUsage>,
}

/// An LLM-requested tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Token usage statistics from a completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

/// Trait for LLM providers.
#[allow(async_fn_in_trait)]
pub trait LlmProvider: Send + Sync {
    /// Human-readable provider name.
    fn name(&self) -> &str;

    /// Run a streaming chat completion, calling `on_token` for each token.
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &CompletionConfig,
        on_token: &mut dyn FnMut(String),
    ) -> Result<CompletionResult, ProviderError>;

    /// Quick health check (ping the endpoint).
    async fn health_check(&self) -> Result<(), ProviderError>;
}

/// OpenAI-compatible LLM provider.
///
/// Wraps the existing `services::llm` module into the `LlmProvider` trait.
pub struct OpenAiCompatibleProvider {
    pub base_url: String,
    pub api_key: String,
}

impl LlmProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        "OpenAI Compatible"
    }

    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        config: &CompletionConfig,
        on_token: &mut dyn FnMut(String),
    ) -> Result<CompletionResult, ProviderError> {
        let llm_messages: Vec<crate::services::llm::ChatMessage> = messages
            .into_iter()
            .map(|m| crate::services::llm::ChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        let text = crate::services::llm::stream_chat_completion(
            &self.base_url,
            &self.api_key,
            &config.model,
            llm_messages,
            on_token,
        )
        .await
        .map_err(|e| ProviderError::Network(e.to_string()))?;

        Ok(CompletionResult {
            text,
            tool_calls: vec![],
            usage: None,
        })
    }

    async fn health_check(&self) -> Result<(), ProviderError> {
        crate::services::llm::check_chat_endpoint(&self.base_url, &self.api_key, "gpt-4o-mini")
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_provider_has_correct_name() {
        let provider = OpenAiCompatibleProvider {
            base_url: "https://api.openai.com/v1".into(),
            api_key: "test".into(),
        };
        assert_eq!(provider.name(), "OpenAI Compatible");
    }

    #[test]
    fn completion_config_has_sensible_defaults() {
        let config = CompletionConfig::default();
        assert_eq!(config.model, "gpt-4o-mini");
        assert!(config.stream);
        assert!(config.max_tokens.is_none());
    }

    #[test]
    fn chat_message_serializes() {
        let msg = ChatMessage {
            role: "user".into(),
            content: "Hello".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
    }
}
