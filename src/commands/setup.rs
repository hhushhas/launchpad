use crate::config::global::{AppleConfig, GlobalConfig};
use crate::ui;
use dialoguer::{Confirm, Input};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Setup cancelled")]
    Cancelled,
}

pub async fn run() -> Result<(), SetupError> {
    ui::header("Launchpad Setup");
    println!();
    println!("This will configure your Apple App Store Connect API credentials.");
    println!("You'll need an API key from: https://appstoreconnect.apple.com/access/api");
    println!();

    // Check for existing config
    if GlobalConfig::load()
        .map_err(|e| SetupError::Config(e.to_string()))?
        .is_some()
    {
        let overwrite = Confirm::new()
            .with_prompt("Existing config found. Overwrite?")
            .default(false)
            .interact()
            .map_err(|e| SetupError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if !overwrite {
            return Err(SetupError::Cancelled);
        }
    }

    // Get API key details
    let key_id: String = Input::new()
        .with_prompt("API Key ID")
        .interact_text()
        .map_err(|e| SetupError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let issuer_id: String = Input::new()
        .with_prompt("Issuer ID")
        .interact_text()
        .map_err(|e| SetupError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let key_path: String = Input::new()
        .with_prompt("Path to .p8 key file")
        .interact_text()
        .map_err(|e| SetupError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // Expand and validate key path
    let expanded_path = shellexpand::tilde(&key_path).to_string();
    if !Path::new(&expanded_path).exists() {
        ui::warn(&format!("Warning: Key file not found at {}", expanded_path));
        let proceed = Confirm::new()
            .with_prompt("Continue anyway?")
            .default(false)
            .interact()
            .map_err(|e| SetupError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if !proceed {
            return Err(SetupError::Cancelled);
        }
    }

    // Create config directory
    let config_dir = GlobalConfig::config_dir()
        .ok_or_else(|| SetupError::Config("Could not determine config directory".to_string()))?;
    std::fs::create_dir_all(&config_dir)?;

    // Copy key file to config directory
    let keys_dir = config_dir.join("keys");
    std::fs::create_dir_all(&keys_dir)?;

    let key_filename = format!("AuthKey_{}.p8", key_id);
    let dest_key_path = keys_dir.join(&key_filename);

    if Path::new(&expanded_path).exists() {
        std::fs::copy(&expanded_path, &dest_key_path)?;
        ui::success(&format!("Copied key to {}", dest_key_path.display()));
    }

    // Determine final key path (use copied location if it exists, otherwise original)
    let final_key_path = if dest_key_path.exists() {
        format!("~/.launchpad/keys/{}", key_filename)
    } else {
        key_path
    };

    // Create and save config
    let config = GlobalConfig {
        apple: AppleConfig {
            key_id,
            issuer_id,
            key_path: final_key_path,
        },
    };

    config
        .save()
        .map_err(|e| SetupError::Config(e.to_string()))?;

    ui::success("Configuration saved");
    println!();

    // Run doctor
    ui::step("Running diagnostics...");
    println!();

    if let Err(e) = crate::commands::doctor::run().await {
        ui::warn(&format!("Some checks failed: {}", e));
    }

    println!();
    ui::header("Setup Complete!");
    println!();
    println!("  Next steps:");
    println!("    1. cd into your iOS project");
    println!("    2. Run 'launchpad init'");
    println!("    3. Run 'launchpad deploy'");
    println!();

    Ok(())
}
