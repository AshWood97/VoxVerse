use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::error::AppError;

pub const DEFAULT_PROFILE_ID: &str = "default";

fn get_legacy_keyring_entry() -> Result<Entry, AppError> {
    Entry::new("speakmate", "api-key")
        .map_err(|e| AppError::Crypto(format!("Keychain error: {}", e)))
}

fn get_legacy_keyring_entry_for_profile(profile_id: &str) -> Result<Entry, AppError> {
    Entry::new("speakmate", &format!("api-key:{}", profile_id))
        .map_err(|e| AppError::Crypto(format!("Keychain error: {}", e)))
}

fn get_keyring_entry_for_profile(profile_id: &str) -> Result<Entry, AppError> {
    Entry::new("voxverse", &format!("api-key:{}", profile_id))
        .map_err(|e| AppError::Crypto(format!("Keychain error: {}", e)))
}

pub fn has_api_key_for_profile(profile_id: &str) -> bool {
    // Try voxverse keyring first
    if let Ok(entry) = get_keyring_entry_for_profile(profile_id) {
        if entry.get_password().is_ok() {
            return true;
        }
    }

    // Fallback: try legacy speakmate keyring per-profile
    if let Ok(entry) = get_legacy_keyring_entry_for_profile(profile_id) {
        if entry.get_password().is_ok() {
            return true;
        }
    }

    // Fallback: try legacy speakmate global key for default profile
    if profile_id == DEFAULT_PROFILE_ID {
        if let Ok(entry) = get_legacy_keyring_entry() {
            return entry.get_password().is_ok();
        }
    }

    false
}

pub fn delete_api_key_for_profile(profile_id: &str) -> Result<(), AppError> {
    let entry = get_keyring_entry_for_profile(profile_id)?;
    match entry.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Crypto(format!(
            "Failed to delete keychain entry: {}",
            e
        ))),
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiProvider {
    #[serde(rename = "openai")]
    #[default]
    OpenAi,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "custom")]
    Custom,
}

impl AiProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Ollama => "ollama",
            Self::Custom => "custom",
        }
    }

    pub fn from_stored_value(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" => Some(Self::OpenAi),
            "ollama" => Some(Self::Ollama),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub fn infer_from_base_url(base_url: &str) -> Self {
        let normalized = base_url.trim().to_ascii_lowercase();

        if normalized.contains("localhost:11434")
            || normalized.contains("127.0.0.1:11434")
            || normalized.contains("0.0.0.0:11434")
            || normalized.contains("ollama")
        {
            Self::Ollama
        } else if normalized.contains("api.openai.com") {
            Self::OpenAi
        } else {
            Self::Custom
        }
    }

    pub fn requires_api_key(&self) -> bool {
        matches!(self, Self::OpenAi)
    }

    pub fn supports_whisper_stt(&self) -> bool {
        !matches!(self, Self::Ollama)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub profile_id: String,
    pub profile_name: String,
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
    #[serde(skip)]
    pub has_keychain_entry: bool, // We track if the key exists in keychain instead of the encrypted string
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            profile_id: DEFAULT_PROFILE_ID.into(),
            profile_name: "Default".into(),
            provider: AiProvider::OpenAi,
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            has_keychain_entry: false,
        }
    }
}

impl ApiConfig {
    pub fn get_api_key(&self) -> Result<String, AppError> {
        if !self.provider.requires_api_key() {
            // Try voxverse keyring first
            if let Ok(entry) = get_keyring_entry_for_profile(&self.profile_id) {
                if let Ok(password) = entry.get_password() {
                    return Ok(password);
                }
            }

            // Try legacy speakmate per-profile keyring
            if let Ok(entry) = get_legacy_keyring_entry_for_profile(&self.profile_id) {
                if let Ok(password) = entry.get_password() {
                    return Ok(password);
                }
            }

            // Try legacy speakmate global keyring for default profile
            if self.profile_id == DEFAULT_PROFILE_ID {
                if let Ok(entry) = get_legacy_keyring_entry() {
                    if let Ok(password) = entry.get_password() {
                        return Ok(password);
                    }
                }
            }

            return Ok(String::new());
        }

        // Try voxverse keyring first
        let entry = get_keyring_entry_for_profile(&self.profile_id)?;
        match entry.get_password() {
            Ok(password) => Ok(password),
            Err(_profile_error) => {
                // Try legacy speakmate per-profile keyring
                if let Ok(legacy_entry) = get_legacy_keyring_entry_for_profile(&self.profile_id) {
                    if let Ok(password) = legacy_entry.get_password() {
                        return Ok(password);
                    }
                }

                // Try legacy speakmate global keyring for default profile
                if self.profile_id == DEFAULT_PROFILE_ID {
                    if let Ok(legacy_entry) = get_legacy_keyring_entry() {
                        if let Ok(password) = legacy_entry.get_password() {
                            return Ok(password);
                        }
                    }
                }

                Err(AppError::Config(format!(
                    "API key not found in keychain for profile '{}'",
                    self.profile_id
                )))
            }
        }
    }

    pub fn set_api_key(&mut self, plaintext_key: &str) -> Result<(), AppError> {
        let entry = get_keyring_entry_for_profile(&self.profile_id)?;
        entry
            .set_password(plaintext_key)
            .map_err(|e| AppError::Crypto(format!("Failed to save to keychain: {}", e)))?;
        self.has_keychain_entry = true;
        Ok(())
    }

    pub fn check_has_key(&mut self) {
        self.has_keychain_entry = has_api_key_for_profile(&self.profile_id);
    }
}

/// Runtime application state managed by Tauri
pub struct AppState {
    pub config: Mutex<ApiConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config: Mutex::new(ApiConfig::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AiProvider, ApiConfig, DEFAULT_PROFILE_ID};

    #[test]
    fn infers_ollama_from_local_base_url() {
        assert_eq!(
            AiProvider::infer_from_base_url("http://localhost:11434/v1"),
            AiProvider::Ollama
        );
        assert_eq!(
            AiProvider::infer_from_base_url("http://127.0.0.1:11434/v1"),
            AiProvider::Ollama
        );
    }

    #[test]
    fn infers_openai_from_official_base_url() {
        assert_eq!(
            AiProvider::infer_from_base_url("https://api.openai.com/v1"),
            AiProvider::OpenAi
        );
    }

    #[test]
    fn provider_capabilities_match_v0_1_boundaries() {
        assert!(AiProvider::OpenAi.requires_api_key());
        assert!(!AiProvider::Ollama.requires_api_key());
        assert!(AiProvider::Custom.supports_whisper_stt());
        assert!(!AiProvider::Ollama.supports_whisper_stt());
    }

    #[test]
    fn default_config_uses_default_profile_without_key_material() {
        let config = ApiConfig::default();
        assert_eq!(config.profile_id, DEFAULT_PROFILE_ID);
        assert_eq!(config.provider, AiProvider::OpenAi);
        assert!(!config.has_keychain_entry);
    }
}
