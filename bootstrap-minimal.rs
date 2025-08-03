#!/usr/bin/env cargo-eval

//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! chrono = { version = "0.4", features = ["serde"] }
//! anyhow = "1.0"
//! git2 = "0.18"
//! ```

use anyhow::{Context, Result};
use chrono::Utc;
use git2::Repository;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Serialize)]
struct BootstrapEvent {
    timestamp: String,
    level: String,
    action: String,
    message: String,
    details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<u32>,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResultItem>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifToolComponent,
}

#[derive(Serialize)]
struct SarifToolComponent {
    name: String,
    version: String,
}

#[derive(Serialize)]
struct SarifResultItem {
    level: String,
    message: SarifMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    locations: Option<Vec<SarifLocation>>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_location: Option<SarifPhysicalLocation>,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_location: Option<SarifArtifactLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    #[serde(skip_serializing_if = "Option::is_none")]
    start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<u32>,
}

macro_rules! log_event {
    ($level:expr, $action:expr, $msg:expr, $details:expr) => {{
        let event = BootstrapEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: $level.to_string(),
            action: $action.to_string(),
            message: $msg.to_string(),
            details: $details.map(|s| s.to_string()),
            file: None,
            line: None,
        };
        println!("{}", serde_json::to_string(&event).unwrap());
    }};
}

macro_rules! log_event_with_location {
    ($level:expr, $action:expr, $msg:expr, $details:expr, $file:expr, $line:expr) => {{
        let event = BootstrapEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: $level.to_string(),
            action: $action.to_string(),
            message: $msg.to_string(),
            details: $details.map(|s| s.to_string()),
            file: Some($file.to_string()),
            line: Some($line),
        };
        println!("{}", serde_json::to_string(&event).unwrap());
    }};
}

fn emit_sarif_error(file: &str, line: u32, msg: &str) {
    let sarif = SarifResult {
        schema: "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifToolComponent {
                    name: "hooksmith-bootstrap".to_string(),
                    version: "0.1.0".to_string(),
                },
            },
            results: vec![SarifResultItem {
                level: "error".to_string(),
                message: SarifMessage {
                    text: msg.to_string(),
                },
                locations: Some(vec![SarifLocation {
                    physical_location: Some(SarifPhysicalLocation {
                        artifact_location: Some(SarifArtifactLocation {
                            uri: file.to_string(),
                        }),
                        region: Some(SarifRegion {
                            start_line: Some(line),
                            start_column: Some(1),
                        }),
                    }),
                }]),
            }],
        }],
    };

    println!("{}", serde_json::to_string_pretty(&sarif).unwrap());
}

fn check_git_state() -> Result<()> {
    log_event!("info", "git_check", "Checking git repository state", None);
    
    let repo = Repository::open(".").context("Failed to open git repository")?;
    
    // Check if working directory is clean
    let statuses = repo.statuses(Some(git2::StatusOptions::new().include_untracked(true)))?;
    
    if !statuses.is_empty() {
        let mut changed_files = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                changed_files.push(path.to_string_lossy().to_string());
            }
        }
        
        let details = format!("Uncommitted changes found: {}", changed_files.join(", "));
        log_event!("error", "git_dirty", "Working directory is not clean", Some(&details));
        emit_sarif_error("bootstrap-minimal.rs", 1, "Git working directory is not clean");
        
        anyhow::bail!("Working directory is not clean. Please commit or stash changes before running bootstrap.");
    }
    
    log_event!("info", "git_clean", "Git working directory is clean", None);
    Ok(())
}

fn ensure_xtask_structure() -> Result<()> {
    log_event!("info", "check_xtask", "Checking xtask structure", None);
    
    // Create xtask directory if it doesn't exist
    if !Path::new("xtask").exists() {
        log_event!("info", "create_xtask_dir", "Creating xtask directory", None);
        fs::create_dir_all("xtask/src")?;
    }
    
    // Create minimal xtask/Cargo.toml if it doesn't exist
    let xtask_cargo_toml = "xtask/Cargo.toml";
    if !Path::new(xtask_cargo_toml).exists() {
        log_event!("info", "create_xtask_cargo", "Creating minimal xtask/Cargo.toml", None);
        
        let minimal_cargo_toml = r#"[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
"#;
        
        fs::write(xtask_cargo_toml, minimal_cargo_toml)
            .context("Failed to write minimal xtask/Cargo.toml")?;
    }
    
    // Create minimal xtask/src/main.rs if it doesn't exist
    let xtask_main_rs = "xtask/src/main.rs";
    if !Path::new(xtask_main_rs).exists() {
        log_event!("info", "create_xtask_main", "Creating minimal xtask/src/main.rs", None);
        
        let minimal_main_rs = r#"use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Bootstrap the project with all generated files
    Bootstrap {
        /// Whether to validate after bootstrap
        #[arg(long)]
        validate: bool,
        /// Whether to commit generated files
        #[arg(long)]
        commit: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Bootstrap { validate, commit } => {
            println!("Bootstrap command called with validate={}, commit={}", validate, commit);
            // TODO: Implement full bootstrap logic
            Ok(())
        }
    }
}
"#;
        
        fs::write(xtask_main_rs, minimal_main_rs)
            .context("Failed to write minimal xtask/src/main.rs")?;
    }
    
    log_event!("info", "xtask_ready", "Xtask structure is ready", None);
    Ok(())
}

fn build_xtask() -> Result<()> {
    log_event!("info", "build_xtask", "Building xtask binary", None);
    
    let output = Command::new("cargo")
        .args(&["build", "--manifest-path", "xtask/Cargo.toml"])
        .output()
        .context("Failed to execute cargo build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_event!("error", "build_failed", "Failed to build xtask", Some(&stderr));
        emit_sarif_error("xtask/Cargo.toml", 1, &format!("Build failed: {}", stderr));
        anyhow::bail!("Failed to build xtask: {}", stderr);
    }
    
    log_event!("info", "build_success", "Xtask built successfully", None);
    Ok(())
}

fn run_xtask_bootstrap() -> Result<()> {
    log_event!("info", "delegate_bootstrap", "Delegating to xtask bootstrap", None);
    
    let output = Command::new("cargo")
        .args(&["run", "-p", "xtask", "--", "bootstrap"])
        .output()
        .context("Failed to execute xtask bootstrap")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_event!("error", "bootstrap_failed", "Xtask bootstrap failed", Some(&stderr));
        emit_sarif_error("xtask/src/main.rs", 1, &format!("Bootstrap failed: {}", stderr));
        anyhow::bail!("Xtask bootstrap failed: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    log_event!("info", "bootstrap_success", "Xtask bootstrap completed", Some(&stdout));
    Ok(())
}

fn main() -> Result<()> {
    log_event!("info", "bootstrap_start", "Starting minimal bootstrap process", None);
    
    // Step 1: Check git state
    check_git_state()?;
    
    // Step 2: Ensure xtask structure exists
    ensure_xtask_structure()?;
    
    // Step 3: Build xtask
    build_xtask()?;
    
    // Step 4: Delegate to xtask bootstrap
    run_xtask_bootstrap()?;
    
    log_event!("info", "bootstrap_complete", "Minimal bootstrap completed successfully", None);
    Ok(())
} 