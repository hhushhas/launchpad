use crate::config::project::ProjectConfig;
use crate::templates;
use crate::ui;
use crate::xcode::Xcode;
use dialoguer::{Confirm, Input, Select};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error("No iOS project found in current directory")]
    NoIosProject,

    #[error("Could not detect Xcode scheme. Use --scheme to specify.")]
    NoSchemeDetected,

    #[error(".launchpad.toml already exists. Delete it first to reinitialize.")]
    AlreadyInitialized,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Xcode error: {0}")]
    Xcode(String),

    #[error("fastlane installation failed")]
    FastlaneInstallFailed,

    #[error("User cancelled")]
    UserCancelled,
}

pub async fn run(
    ios_path: Option<String>,
    scheme: Option<String>,
    bundle_id: Option<String>,
    non_interactive: bool,
) -> Result<(), InitError> {
    ui::header("Launchpad Init");

    // Check if already initialized
    if Path::new(".launchpad.toml").exists() {
        return Err(InitError::AlreadyInitialized);
    }

    // 1. Check and install fastlane
    check_and_install_fastlane(non_interactive)?;

    // 2. Detect iOS project path
    let detected_ios_path = ios_path.unwrap_or_else(|| detect_ios_path().unwrap_or_default());

    if detected_ios_path.is_empty() {
        return Err(InitError::NoIosProject);
    }

    ui::success(&format!("Found iOS project at: {}", detected_ios_path));

    // 3. Detect or prompt for scheme
    let schemes = Xcode::list_schemes(&detected_ios_path)
        .map_err(|e| InitError::Xcode(e.to_string()))?;

    let selected_scheme = if let Some(s) = scheme {
        s
    } else if schemes.is_empty() {
        return Err(InitError::NoSchemeDetected);
    } else if schemes.len() == 1 {
        ui::success(&format!("Detected scheme: {}", schemes[0]));
        schemes[0].clone()
    } else if non_interactive {
        // In non-interactive mode, pick the first scheme
        ui::success(&format!("Using scheme: {} (first of {})", schemes[0], schemes.len()));
        schemes[0].clone()
    } else {
        ui::step("Multiple schemes found. Please select one:");
        let selection = Select::new()
            .items(&schemes)
            .default(0)
            .interact()
            .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        schemes[selection].clone()
    };

    // 4. Detect bundle ID
    let detected_bundle_id = Xcode::get_bundle_id(&detected_ios_path, &selected_scheme)
        .unwrap_or_else(|_| "com.example.app".to_string());

    let final_bundle_id = if let Some(b) = bundle_id {
        b
    } else if non_interactive {
        ui::success(&format!("Using bundle ID: {}", detected_bundle_id));
        detected_bundle_id
    } else {
        Input::new()
            .with_prompt("Bundle identifier")
            .default(detected_bundle_id)
            .interact_text()
            .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    };

    // 5. Git tag options
    let (git_tag, push_tags) = if non_interactive {
        ui::success("Git tagging: enabled (default)");
        (true, true)
    } else {
        let git_tag = Confirm::new()
            .with_prompt("Create git tags after deploy?")
            .default(true)
            .interact()
            .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let push_tags = if git_tag {
            Confirm::new()
                .with_prompt("Push tags to remote?")
                .default(true)
                .interact()
                .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
        } else {
            false
        };

        (git_tag, push_tags)
    };

    // 6. Create config
    let config = ProjectConfig {
        project: crate::config::project::ProjectSettings {
            ios_path: detected_ios_path.clone(),
            scheme: selected_scheme.clone(),
            bundle_id: final_bundle_id,
        },
        deploy: crate::config::project::DeploySettings {
            git_tag,
            push_tags,
            clean_artifacts: true,
        },
    };

    // 7. Write config
    config
        .save()
        .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    ui::success("Created .launchpad.toml");

    // 8. Create example config for team reference
    if !Path::new(".launchpad.toml.example").exists() {
        std::fs::write(".launchpad.toml.example", templates::LAUNCHPAD_TOML_EXAMPLE)?;
        ui::success("Created .launchpad.toml.example (for team reference)");
    }

    // 9. Check and create Fastfile
    check_and_create_fastfile(&detected_ios_path, &selected_scheme, non_interactive)?;

    // 10. Offer to add to .gitignore
    if Path::new(".gitignore").exists() {
        let add_gitignore = if non_interactive {
            false // Don't modify gitignore in non-interactive mode
        } else {
            Confirm::new()
                .with_prompt("Add .launchpad.toml to .gitignore?")
                .default(false)
                .interact()
                .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
        };

        if add_gitignore {
            let mut gitignore = std::fs::read_to_string(".gitignore")?;
            if !gitignore.contains(".launchpad.toml") {
                gitignore.push_str("\n.launchpad.toml\n");
                std::fs::write(".gitignore", gitignore)?;
                ui::success("Added to .gitignore");
            }
        }
    }

    println!();
    ui::header("Setup Complete!");
    println!();
    println!("  Next steps:");
    println!("    1. Run 'launchpad doctor' to verify setup");
    println!("    2. Run 'launchpad deploy' to deploy to TestFlight");
    println!();

    Ok(())
}

fn check_and_install_fastlane(non_interactive: bool) -> Result<(), InitError> {
    if which::which("fastlane").is_ok() {
        ui::success("fastlane found");
        return Ok(());
    }

    ui::error("fastlane not found");

    let install = if non_interactive {
        ui::step("Installing fastlane (--yes mode)...");
        true
    } else {
        Confirm::new()
            .with_prompt("Install fastlane?")
            .default(true)
            .interact()
            .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    };

    if !install {
        return Err(InitError::UserCancelled);
    }

    ui::step("Running: brew install fastlane");

    let spinner = ui::spinner("Installing fastlane...");

    let status = Command::new("brew")
        .args(["install", "fastlane"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    spinner.finish_and_clear();

    if !status.success() {
        ui::error("Failed to install fastlane via brew");
        ui::step("Try manually: brew install fastlane");
        return Err(InitError::FastlaneInstallFailed);
    }

    ui::success("fastlane installed");
    Ok(())
}

fn check_and_create_fastfile(ios_path: &str, scheme: &str, non_interactive: bool) -> Result<(), InitError> {
    let fastfile_paths = [
        format!("{}/fastlane/Fastfile", ios_path),
        format!("{}/Fastfile", ios_path),
        "fastlane/Fastfile".to_string(),
        "Fastfile".to_string(),
    ];

    for path in &fastfile_paths {
        if Path::new(path).exists() {
            ui::success(&format!("Fastfile found at: {}", path));
            return Ok(());
        }
    }

    ui::warn(&format!("Fastfile not found in {}/fastlane/", ios_path));

    let create = if non_interactive {
        ui::step("Creating Fastfile (--yes mode)...");
        true
    } else {
        Confirm::new()
            .with_prompt("Create Fastfile with required lanes?")
            .default(true)
            .interact()
            .map_err(|e| InitError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    };

    if !create {
        ui::warn("Skipping Fastfile creation. You'll need to create it manually.");
        return Ok(());
    }

    // Create fastlane directory if it doesn't exist
    let fastlane_dir = format!("{}/fastlane", ios_path);
    std::fs::create_dir_all(&fastlane_dir)?;

    // Generate and write Fastfile
    let fastfile_content = templates::generate_fastfile(scheme);
    let fastfile_path = format!("{}/Fastfile", fastlane_dir);
    std::fs::write(&fastfile_path, fastfile_content)?;

    ui::success(&format!("Created {}", fastfile_path));

    Ok(())
}

fn detect_ios_path() -> Option<String> {
    let candidates = ["ios", ".", "App", "app"];

    for candidate in candidates {
        let path = Path::new(candidate);

        // Check for .xcworkspace or .xcodeproj
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".xcworkspace") || name_str.ends_with(".xcodeproj") {
                    return Some(candidate.to_string());
                }
            }
        }
    }

    None
}
