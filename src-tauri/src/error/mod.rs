use serde::Serialize;

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub enum AppError {
    Api(String),
    Network(String),
    Config(String),
    Crypto(String),
    Store(String),
    Db(String),
    Provider(String),
    Memory(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Api(msg) => write!(f, "API error: {}", msg),
            AppError::Network(msg) => write!(f, "Network error: {}", msg),
            AppError::Config(msg) => write!(f, "Config error: {}", msg),
            AppError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            AppError::Store(msg) => write!(f, "Store error: {}", msg),
            AppError::Db(msg) => write!(f, "Database error: {}", msg),
            AppError::Provider(msg) => write!(f, "Provider error: {}", msg),
            AppError::Memory(msg) => write!(f, "Memory error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Network(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Api(format!("JSON parse error: {}", err))
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Db(err.to_string())
    }
}
