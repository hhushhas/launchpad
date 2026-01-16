mod commands;
mod config;
mod fastlane;
mod templates;
mod ui;
mod xcode;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "launchpad")]
#[command(about = "iOS TestFlight deployment made easy", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy to TestFlight
    Deploy {
        /// Bump patch version (1.0.0 → 1.0.1)
        #[arg(long, conflicts_with = "minor")]
        patch: bool,

        /// Bump minor version (1.0.0 → 1.1.0)
        #[arg(long, conflicts_with = "patch")]
        minor: bool,

        /// Skip git tag creation
        #[arg(long)]
        no_tag: bool,

        /// Skip pre-flight git checks
        #[arg(long)]
        skip_git_check: bool,
    },

    /// Initialize launchpad in current project
    Init {
        /// Path to iOS project (default: auto-detect)
        #[arg(long)]
        ios_path: Option<String>,

        /// Xcode scheme to use
        #[arg(long)]
        scheme: Option<String>,

        /// Bundle identifier
        #[arg(long)]
        bundle_id: Option<String>,

        /// Non-interactive mode (accept defaults)
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Interactive first-time setup (global config)
    Setup,

    /// Check prerequisites (Xcode, fastlane, API key)
    Doctor,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match cli.command {
        Commands::Deploy {
            patch,
            minor,
            no_tag,
            skip_git_check,
        } => commands::deploy::run(patch, minor, no_tag, skip_git_check)
            .await
            .map_err(|e| e.into()),
        Commands::Init { ios_path, scheme, bundle_id, yes } => {
            commands::init::run(ios_path, scheme, bundle_id, yes)
                .await
                .map_err(|e| e.into())
        }
        Commands::Setup => commands::setup::run().await.map_err(|e| e.into()),
        Commands::Doctor => commands::doctor::run().await.map_err(|e| e.into()),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            ui::error(&e.to_string());
            ExitCode::FAILURE
        }
    }
}
