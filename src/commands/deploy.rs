use crate::config::{global::GlobalConfig, project::ProjectConfig};
use crate::fastlane::Fastlane;
use crate::ui;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeployError {
    #[error("Global config not found. Run 'launchpad setup' first.")]
    NoGlobalConfig,

    #[error("Project config not found. Run 'launchpad init' first.")]
    NoProjectConfig,

    #[error("Apple API key not found at: {0}")]
    ApiKeyNotFound(String),

    #[error("Git working directory is not clean. Commit or stash changes first.")]
    DirtyWorkingDirectory,

    #[error("Fastlane failed: {0}")]
    FastlaneFailed(String),

    #[error("Failed to create git tag: {0}")]
    GitTagFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),
}

pub async fn run(
    patch: bool,
    minor: bool,
    no_tag: bool,
    skip_git_check: bool,
) -> Result<(), DeployError> {
    ui::header("Launchpad Deploy");

    // Load configs
    let global_config =
        GlobalConfig::load().map_err(|e| DeployError::Config(e.to_string()))?;
    let global_config = global_config.ok_or(DeployError::NoGlobalConfig)?;

    let project_config =
        ProjectConfig::load().map_err(|e| DeployError::Config(e.to_string()))?;
    let project_config = project_config.ok_or(DeployError::NoProjectConfig)?;

    // Validate API key exists
    let key_path = shellexpand::tilde(&global_config.apple.key_path).to_string();
    if !std::path::Path::new(&key_path).exists() {
        return Err(DeployError::ApiKeyNotFound(key_path));
    }

    // Git checks
    if !skip_git_check {
        ui::step("Checking git status...");
        if !is_git_clean()? {
            return Err(DeployError::DirtyWorkingDirectory);
        }
        ui::success("Working directory clean");
    }

    // Determine version bump type
    let version_bump = if patch {
        Some("patch")
    } else if minor {
        Some("minor")
    } else {
        None // Build number only
    };

    let action = match version_bump {
        Some("patch") => "patch version bump",
        Some("minor") => "minor version bump",
        _ => "build number increment",
    };
    ui::step(&format!("Deploying with {}...", action));

    // Build fastlane command
    let fastlane = Fastlane::new(&global_config, &project_config);

    // Run fastlane
    let spinner = ui::spinner("Building and uploading to TestFlight...");
    let result = fastlane.deploy(version_bump).await;
    spinner.finish_and_clear();

    match result {
        Ok(version) => {
            ui::success(&format!("Successfully deployed version {}", version));

            // Create git tag if configured and not disabled
            let should_tag = !no_tag && project_config.deploy.git_tag;
            if should_tag {
                let tag = format!("v{}", version);
                ui::step(&format!("Creating git tag {}...", tag));

                if let Err(e) = create_git_tag(&tag) {
                    ui::warn(&format!("Failed to create tag: {}", e));
                } else {
                    ui::success(&format!("Created tag {}", tag));

                    if project_config.deploy.push_tags {
                        if let Err(e) = push_git_tags() {
                            ui::warn(&format!("Failed to push tags: {}", e));
                        } else {
                            ui::success("Pushed tags to remote");
                        }
                    }
                }
            }

            ui::header("Deploy Complete!");
            println!();
            println!("  Version: {}", version);
            println!("  TestFlight: Processing (usually 10-30 minutes)");
            println!();

            Ok(())
        }
        Err(e) => Err(DeployError::FastlaneFailed(e.to_string())),
    }
}

fn is_git_clean() -> Result<bool, std::io::Error> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    Ok(output.stdout.is_empty())
}

fn create_git_tag(tag: &str) -> Result<(), DeployError> {
    let output = Command::new("git")
        .args(["tag", "-a", tag, "-m", &format!("Release {}", tag)])
        .output()
        .map_err(DeployError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DeployError::GitTagFailed(stderr.to_string()));
    }

    Ok(())
}

fn push_git_tags() -> Result<(), DeployError> {
    let output = Command::new("git")
        .args(["push", "--tags"])
        .output()
        .map_err(DeployError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DeployError::GitTagFailed(stderr.to_string()));
    }

    Ok(())
}
