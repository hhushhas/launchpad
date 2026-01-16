use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum XcodeError {
    #[error("Xcode command failed: {0}")]
    CommandFailed(String),

    #[error("No Xcode project found at: {0}")]
    NoProjectFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Xcode;

impl Xcode {
    /// List available schemes in an Xcode project
    pub fn list_schemes(ios_path: &str) -> Result<Vec<String>, XcodeError> {
        let path = Path::new(ios_path);

        // Find workspace or project file
        let workspace = find_workspace(path);
        let project = find_project(path);

        let mut cmd = Command::new("xcodebuild");
        cmd.arg("-list");

        if let Some(ws) = workspace {
            cmd.arg("-workspace").arg(ws);
        } else if let Some(proj) = project {
            cmd.arg("-project").arg(proj);
        } else {
            return Err(XcodeError::NoProjectFound(ios_path.to_string()));
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(XcodeError::CommandFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let schemes = parse_schemes(&stdout);

        Ok(schemes)
    }

    /// Get bundle identifier for a scheme
    pub fn get_bundle_id(ios_path: &str, scheme: &str) -> Result<String, XcodeError> {
        let path = Path::new(ios_path);
        let workspace = find_workspace(path);
        let project = find_project(path);

        let mut cmd = Command::new("xcodebuild");
        cmd.arg("-showBuildSettings").arg("-scheme").arg(scheme);

        if let Some(ws) = workspace {
            cmd.arg("-workspace").arg(ws);
        } else if let Some(proj) = project {
            cmd.arg("-project").arg(proj);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(XcodeError::CommandFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse PRODUCT_BUNDLE_IDENTIFIER
        for line in stdout.lines() {
            if line.contains("PRODUCT_BUNDLE_IDENTIFIER") {
                if let Some(value) = line.split('=').nth(1) {
                    return Ok(value.trim().to_string());
                }
            }
        }

        Err(XcodeError::CommandFailed(
            "Could not find bundle identifier".to_string(),
        ))
    }

    /// Check if Xcode is installed
    pub fn is_installed() -> bool {
        Command::new("xcode-select")
            .arg("-p")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get Xcode version
    pub fn version() -> Option<String> {
        let output = Command::new("xcodebuild").arg("-version").output().ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().next().map(|s| s.to_string())
        } else {
            None
        }
    }
}

fn find_workspace(path: &Path) -> Option<String> {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".xcworkspace") && !name_str.starts_with("project.") {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }
    None
}

fn find_project(path: &Path) -> Option<String> {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".xcodeproj") {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }
    None
}

fn parse_schemes(output: &str) -> Vec<String> {
    let mut schemes = Vec::new();
    let mut in_schemes = false;

    for line in output.lines() {
        let trimmed = line.trim();

        if trimmed == "Schemes:" {
            in_schemes = true;
            continue;
        }

        if in_schemes {
            if trimmed.is_empty() || trimmed.ends_with(':') {
                break;
            }
            schemes.push(trimmed.to_string());
        }
    }

    schemes
}
