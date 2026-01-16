use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Could not determine config directory")]
    NoConfigDir,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub apple: AppleConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppleConfig {
    pub key_id: String,
    pub issuer_id: String,
    pub key_path: String,
}

impl GlobalConfig {
    pub fn config_dir() -> Option<PathBuf> {
        // Check for custom location via env var
        if let Ok(path) = std::env::var("LAUNCHPAD_CONFIG_DIR") {
            return Some(PathBuf::from(path));
        }

        // Default to ~/.launchpad
        dirs::home_dir().map(|h| h.join(".launchpad"))
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn load() -> Result<Option<Self>, ConfigError> {
        // Check environment variables first
        let key_id = std::env::var("APPLE_API_KEY_ID");
        let issuer_id = std::env::var("APPLE_API_ISSUER_ID");
        let key_path = std::env::var("APPLE_API_KEY_PATH");

        if let (Ok(key_id), Ok(issuer_id), Ok(key_path)) = (key_id, issuer_id, key_path) {
            return Ok(Some(GlobalConfig {
                apple: AppleConfig {
                    key_id,
                    issuer_id,
                    key_path,
                },
            }));
        }

        // Fall back to config file
        let config_path = Self::config_path().ok_or(ConfigError::NoConfigDir)?;

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: GlobalConfig = toml::from_str(&content)?;

        Ok(Some(config))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = Self::config_path().ok_or(ConfigError::NoConfigDir)?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }
}
