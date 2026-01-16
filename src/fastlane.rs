use crate::config::{global::GlobalConfig, project::ProjectConfig};
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Error, Debug)]
pub enum FastlaneError {
    #[error("Fastlane command failed: {0}")]
    CommandFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not parse version from output")]
    VersionParseFailed,
}

pub struct Fastlane {
    key_id: String,
    issuer_id: String,
    key_path: String,
    ios_path: String,
    scheme: String,
}

impl Fastlane {
    pub fn new(global_config: &GlobalConfig, project_config: &ProjectConfig) -> Self {
        let key_path = shellexpand::tilde(&global_config.apple.key_path).to_string();

        Self {
            key_id: global_config.apple.key_id.clone(),
            issuer_id: global_config.apple.issuer_id.clone(),
            key_path,
            ios_path: project_config.project.ios_path.clone(),
            scheme: project_config.project.scheme.clone(),
        }
    }

    pub async fn deploy(&self, version_bump: Option<&str>) -> Result<String, FastlaneError> {
        // Build the fastlane command
        let lane = match version_bump {
            Some("patch") => "beta_patch",
            Some("minor") => "beta_minor",
            _ => "beta",
        };

        let mut cmd = Command::new("fastlane");
        cmd.current_dir(&self.ios_path)
            .arg(lane)
            .env("APP_STORE_CONNECT_API_KEY_KEY_ID", &self.key_id)
            .env("APP_STORE_CONNECT_API_KEY_ISSUER_ID", &self.issuer_id)
            .env("APP_STORE_CONNECT_API_KEY_KEY_FILEPATH", &self.key_path)
            .env("FASTLANE_XCODEBUILD_SETTINGS_TIMEOUT", "180")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take().expect("stdout not captured");
        let stderr = child.stderr.take().expect("stderr not captured");

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        let mut last_version = String::new();
        let mut output_lines = Vec::new();

        // Stream output and capture version
        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            output_lines.push(line.clone());
                            // Look for version in output
                            if line.contains("Version:") || line.contains("version:") {
                                if let Some(v) = extract_version(&line) {
                                    last_version = v;
                                }
                            }
                            // Also check for build number
                            if line.contains("Successfully uploaded") || line.contains("Build") {
                                if let Some(v) = extract_version(&line) {
                                    last_version = v;
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(line)) => {
                            output_lines.push(line);
                        }
                        Ok(None) => {}
                        Err(_) => {}
                    }
                }
            }
        }

        let status = child.wait().await?;

        if !status.success() {
            // Get last few lines for error context
            let error_context: Vec<_> = output_lines.iter().rev().take(10).collect();
            let error_msg = error_context
                .into_iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .join("\n");
            return Err(FastlaneError::CommandFailed(error_msg));
        }

        // If we couldn't extract version, use a placeholder
        if last_version.is_empty() {
            last_version = "unknown".to_string();
        }

        Ok(last_version)
    }
}

fn extract_version(line: &str) -> Option<String> {
    // Try to find version patterns like "1.0.0", "1.0.0 (123)", etc.
    let re = regex_lite::Regex::new(r"(\d+\.\d+\.\d+)(?:\s*\((\d+)\))?").ok()?;

    if let Some(caps) = re.captures(line) {
        let version = caps.get(1)?.as_str();
        if let Some(build) = caps.get(2) {
            return Some(format!("{} ({})", version, build.as_str()));
        }
        return Some(version.to_string());
    }

    None
}
