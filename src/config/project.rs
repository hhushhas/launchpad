use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

const CONFIG_FILENAME: &str = ".launchpad.toml";

#[derive(Error, Debug)]
pub enum ProjectConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectSettings,
    pub deploy: DeploySettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub ios_path: String,
    pub scheme: String,
    pub bundle_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeploySettings {
    #[serde(default = "default_true")]
    pub git_tag: bool,

    #[serde(default = "default_true")]
    pub push_tags: bool,

    #[serde(default = "default_true")]
    pub clean_artifacts: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DeploySettings {
    fn default() -> Self {
        Self {
            git_tag: true,
            push_tags: true,
            clean_artifacts: true,
        }
    }
}

impl ProjectConfig {
    pub fn load() -> Result<Option<Self>, ProjectConfigError> {
        let config_path = Path::new(CONFIG_FILENAME);

        if !config_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;

        Ok(Some(config))
    }

    pub fn save(&self) -> Result<(), ProjectConfigError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(CONFIG_FILENAME, content)?;
        Ok(())
    }
}
