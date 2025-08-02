#!/usr/bin/env cargo-script

//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! serde_yaml = "0.9"
//! toml = "0.8"
//! anyhow = "1.0"
//! clap = { version = "4.0", features = ["derive"] }
//! tokio = { version = "1.0", features = ["full"] }
//! console = "0.15"
//! indicatif = "0.17"
//! git2 = "0.18"
//! which = "5.0"
//! tracing = "0.1"
//! tracing-subscriber = "0.3"
//! rustdoc-stripper = "0.1"
//! wasmtime = "18.0"
//! wit-bindgen = "0.20"
//! wasmtime-wasi = "18.0"
//! wit-parser = "0.12"
//! wit-component = "0.12"
//! wat = "1.0"
//! wasmparser = "0.120"
//! chrono = { version = "0.4", features = ["serde"] }
//! jsonschema = "0.17"
//! reqwest = { version = "0.11", features = ["json"] }
//! sha2 = "0.10"
//! tempfile = "3.0"
//! ```

use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
struct CargoToml {
    package: Package,
    workspace: Workspace,
    #[serde(rename = "workspace.dependencies")]
    workspace_dependencies: HashMap<String, String>,
    bin: Vec<Bin>,
    lib: Lib,
    dependencies: HashMap<String, Dependency>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, String>,
    #[serde(rename = "package.metadata.docs.rs")]
    package_metadata_docs_rs: PackageMetadataDocsRs,
}

#[derive(Debug, Serialize, Deserialize)]
struct Package {
    name: String,
    version: String,
    edition: String,
    authors: Vec<String>,
    description: String,
    license: String,
    documentation: String,
    repository: String,
    keywords: Vec<String>,
    categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Workspace {
    members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Bin {
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Lib {
    name: String,
    path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Dependency {
    Workspace { workspace: bool },
    Path { path: String },
    Version { version: String },
}

#[derive(Debug, Serialize, Deserialize)]
struct PackageMetadataDocsRs {
    #[serde(rename = "all-features")]
    all_features: bool,
    #[serde(rename = "rustdoc-args")]
    rustdoc_args: Vec<String>,
}

fn main() -> Result<()> {
    println!("{}", style("🚀 Hooksmith Bootstrap Test").bold().blue());
    println!("{}", style("Checking project structure...").dim());

    let progress = ProgressBar::new(4);
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    // Step 1: Check if main Cargo.toml exists
    progress.set_message("Checking main Cargo.toml...");
    if Path::new("Cargo.toml").exists() {
        println!("{}", style("✓ Main Cargo.toml already exists").green());
    } else {
        println!("{}", style("✗ Main Cargo.toml missing - would generate it").red());
    }
    progress.inc(1);

    // Step 2: Check component directories
    progress.set_message("Checking component directories...");
    let components = [
        "components/cli-core",
        "components/worktree-runner", 
        "components/git-filter",
        "components/hook-builder",
        "lefthook-rs",
    ];
    
    for component in &components {
        if Path::new(&format!("{}/Cargo.toml", component)).exists() {
            println!("{}", style(&format!("✓ {} Cargo.toml exists", component)).green());
        } else {
            println!("{}", style(&format!("✗ {} Cargo.toml missing", component)).red());
        }
    }
    progress.inc(1);

    // Step 3: Check xtask
    progress.set_message("Checking xtask...");
    if Path::new("xtask/Cargo.toml").exists() {
        println!("{}", style("✓ xtask/Cargo.toml exists").green());
    } else {
        println!("{}", style("✗ xtask/Cargo.toml missing").red());
    }
    progress.inc(1);

    // Step 4: Check if xtask builds
    progress.set_message("Checking xtask build...");
    if Path::new("xtask/Cargo.toml").exists() {
        let output = Command::new("cargo")
            .args(&["check", "--manifest-path", "xtask/Cargo.toml"])
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                println!("{}", style("✓ xtask builds successfully").green());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("{}", style("✗ xtask build failed").red());
                println!("{}", style(&format!("Error: {}", stderr)).red());
            }
            Err(e) => {
                println!("{}", style(&format!("✗ Failed to run cargo check: {}", e)).red());
            }
        }
    } else {
        println!("{}", style("✗ Cannot check xtask build - Cargo.toml missing").red());
    }
    progress.inc(1);

    progress.finish_with_message("✅ Bootstrap test complete!");

    println!("\n{}", style("📊 Project Status Summary:").bold());
    println!("This test shows what the bootstrap script would generate.");
    println!("Run 'cargo script bootstrap.rs' to actually generate missing files.");

    Ok(())
} 