use crate::config::{global::GlobalConfig, project::ProjectConfig};
use crate::ui;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoctorError {
    #[error("Prerequisites check failed")]
    ChecksFailed,
}

struct CheckResult {
    name: String,
    passed: bool,
    message: String,
}

pub async fn run() -> Result<(), DoctorError> {
    ui::header("Launchpad Doctor");
    println!();

    let mut checks: Vec<CheckResult> = Vec::new();

    // Check Xcode
    checks.push(check_xcode());

    // Check fastlane
    checks.push(check_fastlane());

    // Check global config
    checks.push(check_global_config());

    // Check project config (if in a project)
    if let Some(project_check) = check_project_config() {
        checks.push(project_check);
    }

    // Check Fastfile (if project config exists)
    if let Some(fastfile_check) = check_fastfile() {
        checks.push(fastfile_check);
    }

    // Display results
    let mut failed = 0;
    for check in &checks {
        if check.passed {
            ui::check_pass(&check.name, &check.message);
        } else {
            ui::check_fail(&check.name, &check.message);
            failed += 1;
        }
    }

    println!();

    if failed > 0 {
        println!(
            "{} issue{} found",
            failed,
            if failed == 1 { "" } else { "s" }
        );
        return Err(DoctorError::ChecksFailed);
    }

    ui::success("All checks passed!");
    Ok(())
}

fn check_xcode() -> CheckResult {
    let output = Command::new("xcodebuild").arg("-version").output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout);
            let version_line = version.lines().next().unwrap_or("Unknown");
            CheckResult {
                name: "Xcode".to_string(),
                passed: true,
                message: version_line.to_string(),
            }
        }
        _ => CheckResult {
            name: "Xcode".to_string(),
            passed: false,
            message: "Not installed (run: xcode-select --install)".to_string(),
        },
    }
}

fn check_fastlane() -> CheckResult {
    match which::which("fastlane") {
        Ok(_) => {
            let output = Command::new("fastlane").arg("--version").output();
            let version = match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // fastlane outputs version in format "fastlane X.Y.Z"
                    stdout
                        .lines()
                        .find(|l| l.contains("fastlane"))
                        .and_then(|l| l.split_whitespace().last())
                        .unwrap_or("installed")
                        .to_string()
                }
                Err(_) => "installed".to_string(),
            };
            CheckResult {
                name: "fastlane".to_string(),
                passed: true,
                message: version,
            }
        }
        Err(_) => CheckResult {
            name: "fastlane".to_string(),
            passed: false,
            message: "Not installed (run: brew install fastlane)".to_string(),
        },
    }
}

fn check_global_config() -> CheckResult {
    match GlobalConfig::load() {
        Ok(Some(config)) => {
            let key_path = shellexpand::tilde(&config.apple.key_path).to_string();
            if Path::new(&key_path).exists() {
                CheckResult {
                    name: "Apple API key".to_string(),
                    passed: true,
                    message: format!("Configured ({})", config.apple.key_id),
                }
            } else {
                CheckResult {
                    name: "Apple API key".to_string(),
                    passed: false,
                    message: format!("Key file not found: {}", key_path),
                }
            }
        }
        Ok(None) => CheckResult {
            name: "Apple API key".to_string(),
            passed: false,
            message: "Not configured (run: launchpad setup)".to_string(),
        },
        Err(e) => CheckResult {
            name: "Apple API key".to_string(),
            passed: false,
            message: format!("Config error: {}", e),
        },
    }
}

fn check_project_config() -> Option<CheckResult> {
    if !Path::new(".launchpad.toml").exists() {
        return None;
    }

    match ProjectConfig::load() {
        Ok(Some(config)) => {
            let ios_path = Path::new(&config.project.ios_path);
            if ios_path.exists() {
                Some(CheckResult {
                    name: "Project".to_string(),
                    passed: true,
                    message: format!(
                        "{} (scheme: {})",
                        config.project.ios_path, config.project.scheme
                    ),
                })
            } else {
                Some(CheckResult {
                    name: "Project".to_string(),
                    passed: false,
                    message: format!("iOS path not found: {}", config.project.ios_path),
                })
            }
        }
        Ok(None) => None,
        Err(e) => Some(CheckResult {
            name: "Project".to_string(),
            passed: false,
            message: format!("Config error: {}", e),
        }),
    }
}

fn check_fastfile() -> Option<CheckResult> {
    let project_config = ProjectConfig::load().ok()??;
    let ios_path = &project_config.project.ios_path;

    let fastfile_paths = [
        format!("{}/fastlane/Fastfile", ios_path),
        format!("{}/Fastfile", ios_path),
        "fastlane/Fastfile".to_string(),
        "Fastfile".to_string(),
    ];

    for path in &fastfile_paths {
        if Path::new(path).exists() {
            return Some(CheckResult {
                name: "Fastfile".to_string(),
                passed: true,
                message: path.clone(),
            });
        }
    }

    Some(CheckResult {
        name: "Fastfile".to_string(),
        passed: false,
        message: "Not found (run: fastlane init in ios directory)".to_string(),
    })
}
