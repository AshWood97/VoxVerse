//! Unified error type for all VoxVerse providers (STT, TTS, LLM, etc.).

use std::fmt;

/// Error categories shared across all provider types.
#[derive(Debug, Clone)]
pub enum ProviderError {
    /// Authentication failed (invalid key, expired token, etc.).
    Auth(String),
    /// Network-level error (DNS, connection refused, etc.).
    Network(String),
    /// Provider rate-limited the request.
    RateLimit {
        retry_after_ms: Option<u64>,
        message: String,
    },
    /// Request timed out.
    Timeout(String),
    /// Invalid input (bad audio format, text too long, etc.).
    InvalidInput(String),
    /// Provider returned an unexpected response.
    InvalidResponse(String),
    /// Internal provider error (bug, misconfiguration).
    Internal(String),
    /// Provider is not available or not configured.
    Unavailable(String),
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth(msg) => write!(f, "Authentication error: {msg}"),
            Self::Network(msg) => write!(f, "Network error: {msg}"),
            Self::RateLimit { message, .. } => write!(f, "Rate limited: {message}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
            Self::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
            Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            Self::Unavailable(msg) => write!(f, "Provider unavailable: {msg}"),
        }
    }
}

impl std::error::Error for ProviderError {}

impl From<reqwest::Error> for ProviderError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(err.to_string())
        } else {
            Self::Network(err.to_string())
        }
    }
}

impl From<ProviderError> for crate::error::AppError {
    fn from(err: ProviderError) -> Self {
        crate::error::AppError::Provider(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_error_displays_category() {
        let err = ProviderError::Auth("bad key".into());
        assert!(err.to_string().contains("Authentication"));

        let err = ProviderError::RateLimit {
            retry_after_ms: Some(5000),
            message: "too fast".into(),
        };
        assert!(err.to_string().contains("Rate limited"));
    }

    #[test]
    fn provider_error_converts_to_app_error() {
        let provider_err = ProviderError::Timeout("took too long".into());
        let app_err: crate::error::AppError = provider_err.into();
        let msg = app_err.to_string();
        assert!(msg.contains("Provider"));
        assert!(msg.contains("Timeout"));
    }
}
