#!/usr/bin/env cargo-eval

//! ```cargo
//! [dependencies]
//! anyhow = "1.0"
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

fn log_event(level: &str, action: &str, msg: &str, details: Option<&str>) {
    println!("[{}] {}: {} {}", level, action, msg, details.unwrap_or(""));
}

fn check_git_state() -> Result<()> {
    log_event("info", "git_check", "Checking git repository state", None);
    
    // Simple git status check using command line
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to execute git status")?;
    
    if !output.stdout.is_empty() {
        let status = String::from_utf8_lossy(&output.stdout);
        log_event("error", "git_dirty", "Working directory is not clean", Some(&status));
        anyhow::bail!("Working directory is not clean. Please commit or stash changes before running bootstrap.");
    }
    
    log_event("info", "git_clean", "Git working directory is clean", None);
    Ok(())
}

fn ensure_xtask_structure() -> Result<()> {
    log_event("info", "check_xtask", "Checking xtask structure", None);
    
    if !Path::new("xtask").exists() {
        log_event("info", "create_xtask_dir", "Creating xtask directory", None);
        fs::create_dir_all("xtask/src")?;
        
        // Create a basic xtask/Cargo.toml
        let cargo_toml = r#"[package]
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
        fs::write("xtask/Cargo.toml", cargo_toml)?;
        
        // Create a basic xtask/src/main.rs
        let main_rs = r#"use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Build system for Hooksmith")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bootstrap the project
    Bootstrap,
    /// Build the project
    Build,
    /// Test the project
    Test,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Bootstrap => {
            println!("🚀 Bootstrapping project...");
            // TODO: Implement bootstrap logic
            println!("✅ Bootstrap completed!");
        }
        Commands::Build => {
            println!("🔨 Building project...");
            // TODO: Implement build logic
            println!("✅ Build completed!");
        }
        Commands::Test => {
            println!("🧪 Testing project...");
            // TODO: Implement test logic
            println!("✅ Tests completed!");
        }
    }
    
    Ok(())
}
"#;
        fs::write("xtask/src/main.rs", main_rs)?;
    }
    
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
    log_event("info", "build_xtask", "Building xtask binary", None);
    
    let output = Command::new("cargo")
        .args(["build", "--manifest-path", "xtask/Cargo.toml"])
        .output()
        .context("Failed to execute cargo build")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log_event("error", "build_failed", "Failed to build xtask", Some(&stderr));
        anyhow::bail!("Failed to build xtask: {}", stderr);
    }
    
    log_event("info", "build_success", "Xtask built successfully", None);
    Ok(())
}

fn generate_version_info() -> Result<()> {
    log_event("info", "generate_version", "Generating version information", None);
    
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.1.0".to_string());
    let authors = std::env::var("CARGO_PKG_AUTHORS").unwrap_or_else(|_| "Hooksmith Team".to_string());
    let name = std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "hooksmith".to_string());
    let description = std::env::var("CARGO_PKG_DESCRIPTION").unwrap_or_else(|_| {
        "CLI tool for building Rust binaries into Lefthook hooks with WASM components".to_string()
    });

    let version_info = format!(
        r#"
/// Auto-generated version information
pub const VERSION: &str = "{}";
pub const AUTHORS: &str = "{}";
pub const NAME: &str = "{}";
pub const DESCRIPTION: &str = "{}";
pub const BUILD_TIMESTAMP: &str = "{}";
"#,
        version,
        authors,
        name,
        description,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );

    // Create src/generated directory if it doesn't exist
    fs::create_dir_all("src/generated")?;
    fs::write("src/generated/version.rs", version_info)?;
    
    log_event("info", "version_generated", "Version information generated", None);
    Ok(())
}

fn generate_feature_flags() -> Result<()> {
    log_event("info", "generate_features", "Generating feature flags", None);
    
    let features = std::env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    let feature_flags = format!(
        r#"
/// Auto-generated feature flags
pub const TARGET_FEATURES: &str = "{}";
pub const TARGET_ARCH: &str = "{}";
pub const TARGET_OS: &str = "{}";
pub const IS_WASM: bool = {};
pub const IS_WASI: bool = {};
pub const IS_NATIVE: bool = {};
"#,
        features,
        target_arch,
        target_os,
        target_arch == "wasm32",
        target_os == "wasi",
        target_arch != "wasm32" && target_os != "wasi"
    );

    fs::write("src/generated/features.rs", feature_flags)?;
    
    log_event("info", "features_generated", "Feature flags generated", None);
    Ok(())
}

fn generate_doc_constants() -> Result<()> {
    log_event("info", "generate_docs", "Generating documentation constants", None);
    
    let doc_constants = r#"
/// Auto-generated documentation constants
pub const DOCS_URL: &str = "https://docs.rs/hooksmith";
pub const REPOSITORY_URL: &str = "https://github.com/bdelanghe/hooksmith";
pub const LICENSE: &str = "MIT";
pub const KEYWORDS: &[&str] = &["cli", "hooks", "lefthook", "wasm", "git"];
pub const CATEGORIES: &[&str] = &["command-line-utilities", "development-tools"];

/// Component information
pub const COMPONENTS: &[&str] = &[
    "cli-core",
    "git-filter",
    "hook-builder",
    "worktree-runner",
    "lefthook-rs",
    "xtask",
];
"#;

    fs::write("src/generated/docs.rs", doc_constants)?;
    
    log_event("info", "docs_generated", "Documentation constants generated", None);
    Ok(())
}

fn main() -> Result<()> {
    log_event("info", "bootstrap_start", "Starting comprehensive bootstrap process", None);
    
    // Step 1: Check git state
    check_git_state()?;
    
    // Step 2: Ensure xtask structure exists
    ensure_xtask_structure()?;
    
    // Step 3: Generate build-time files
    generate_version_info()?;
    generate_feature_flags()?;
    generate_doc_constants()?;
    
    // Step 4: Build xtask
    build_xtask()?;
    
    log_event("info", "bootstrap_complete", "Bootstrap completed successfully", None);
    println!("\n🎉 Hooksmith project bootstrapped successfully!");
    println!("Next steps:");
    println!("  • Run 'cargo build' to build the project");
    println!("  • Run 'cargo test' to run tests");
    println!("  • Run './target/debug/xtask --help' to see available xtask commands");
    
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
