//!
//! # Hooksmith Minimal Bootstrapper
//!
//! This script bootstraps the Hooksmith project from scratch, using JSONC config templates.
//!
//! ## Usage
//!
//! ```bash
//! cargo eval bootstrap.rs
//! ```
//!
//! - Ensures a clean git state
//! - Reads config templates from `bootstrap-config/*.jsonc`
//! - Converts JSONC to TOML or other formats as needed
//! - Writes files to the correct locations
//! - Emits JSONL logs and SARIF errors
//! - Self-tests can be run with `cargo eval --test bootstrap.rs`
//!
//! ## Requirements
//! - Rust
//! - cargo-eval
//! - JSONC config files in `bootstrap-config/`
//!
//! ## Example Config (bootstrap-config/xtask_cargo.jsonc)
//! ```jsonc
//! {
//!   // Package metadata
//!   "package": {
//!     "name": "xtask",
//!     "version": "0.1.0",
//!     "edition": "2021"
//!   },
//!   // Dependencies
//!   "dependencies": {
//!     "anyhow": "1.0",
//!     "clap": { "version": "4.0", "features": ["derive"] },
//!     "serde": { "version": "1.0", "features": ["derive"] },
//!     "serde_json": "1.0",
//!     "tokio": { "version": "1.0", "features": ["full"] },
//!     "chrono": { "version": "0.4", "features": ["serde"] }
//!   }
//! }
//! ```
//!
//! ## Troubleshooting
//! - If git state is dirty, commit or stash changes.
//! - If a config is missing, add it to `bootstrap-config/`.
//!
//! ---

//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! jsonc-parser = "0.1"
//! toml = "0.8"
//! chrono = { version = "0.4", features = ["serde"] }
//! anyhow = "1.0"
//! git2 = "0.18"
//! tempfile = "3.0"
//! ```

use anyhow::{Context, Result};
use chrono::Utc;
use git2::Repository;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use jsonc_parser::parse_to_value;

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

fn check_git_state() -> Result<()> {
    log_event!("info", "git_check", "Checking git repository state", None);
    let repo = Repository::open(".").context("Failed to open git repository")?;
    let statuses = repo.statuses(Some(git2::StatusOptions::new().include_untracked(true)))?;
    if !statuses.is_empty() {
        let mut changed_files = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                changed_files.push(path.to_string());
            }
        }
        let details = format!("Uncommitted changes found: {}", changed_files.join(", "));
        log_event!("error", "git_dirty", "Working directory is not clean", Some(&details));
        anyhow::bail!("Working directory is not clean. Please commit or stash changes before running bootstrap.");
    }
    log_event!("info", "git_clean", "Git working directory is clean", None);
    Ok(())
}

fn read_jsonc_config(path: &str) -> Result<serde_json::Value> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read config: {}", path))?;
    let value = parse_to_value(&content, &Default::default())
        .ok_or_else(|| anyhow::anyhow!("Failed to parse JSONC: {}", path))?;
    Ok(value)
}

fn write_toml_from_jsonc(jsonc: &serde_json::Value, out_path: &str) -> Result<()> {
    let toml_value: toml::Value = toml::Value::try_from(jsonc.clone())
        .context("Failed to convert JSONC to TOML")?;
    let toml_str = toml::to_string_pretty(&toml_value)?;
    fs::write(out_path, toml_str).with_context(|| format!("Failed to write TOML: {}", out_path))?;
    Ok(())
}

fn ensure_xtask_structure() -> Result<()> {
    log_event!("info", "check_xtask", "Checking xtask structure", None);
    if !Path::new("xtask").exists() {
        log_event!("info", "create_xtask_dir", "Creating xtask directory", None);
        fs::create_dir_all("xtask/src")?;
    }
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
        anyhow::bail!("Failed to build xtask: {}", stderr);
    }
    log_event!("info", "build_success", "Xtask built successfully", None);
    Ok(())
}

fn main() -> Result<()> {
    log_event!("info", "bootstrap_start", "Starting minimal bootstrap process", None);
    check_git_state()?;
    ensure_xtask_structure()?;
    // Example: Read and write xtask/Cargo.toml from JSONC
    let xtask_cargo_jsonc = read_jsonc_config("bootstrap-config/xtask_cargo.jsonc")?;
    write_toml_from_jsonc(&xtask_cargo_jsonc, "xtask/Cargo.toml")?;
    build_xtask()?;
    log_event!("info", "bootstrap_complete", "Minimal bootstrap completed successfully", None);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    #[test]
    fn test_jsonc_parse_and_toml_write() {
        let jsonc = r#"{
            // comment
            "package": { "name": "foo", "version": "0.1.0" },
            "dependencies": { "serde": "1.0" }
        }"#;
        let value = parse_to_value(jsonc, &Default::default()).expect("parse");
        let dir = tempdir().unwrap();
        let out_path = dir.path().join("Cargo.toml");
        write_toml_from_jsonc(&value, out_path.to_str().unwrap()).unwrap();
        let written = std::fs::read_to_string(out_path).unwrap();
        assert!(written.contains("[package]"));
        assert!(written.contains("name = \"foo\""));
    }
}
