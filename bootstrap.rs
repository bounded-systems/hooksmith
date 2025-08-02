#!/usr/bin/env cargo-eval

//! ```cargo
//! [dependencies]
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! toml = "0.8"
//! anyhow = "1.0"
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    println!("🚀 Hooksmith Bootstrap");
    println!("Generating project structure...");

    // Step 1: Generate main Cargo.toml
    println!("📝 Generating main Cargo.toml...");
    generate_main_cargo_toml()?;

    // Step 2: Generate component Cargo.toml files
    println!("📝 Generating component Cargo.toml files...");
    generate_component_cargo_tomls()?;

    // Step 3: Generate xtask Cargo.toml
    println!("📝 Generating xtask Cargo.toml...");
    generate_xtask_cargo_toml()?;

    // Step 4: Build xtask
    println!("🔨 Building xtask...");
    build_xtask()?;

    // Step 5: Generate documentation using existing doc gen system
    println!("📚 Generating documentation...");
    generate_documentation()?;

    println!("\n🎉 Hooksmith project bootstrapped successfully!");
    println!("Next steps:");
    println!("  • Run 'cargo build' to build the project");
    println!("  • Run 'cargo test' to run tests");
    println!("  • Run './target/debug/xtask --help' to see available xtask commands");
    println!("  • Run './target/debug/xtask gen-docs-comprehensive --all' to regenerate all docs");

    Ok(())
}

fn generate_main_cargo_toml() -> Result<()> {
    let cargo_toml = r#"[package]
name = "hooksmith"
version = "0.1.0"
edition = "2021"
authors = ["Hooksmith Team"]
description = "CLI tool for building Rust binaries into Lefthook hooks with WASM components"
license = "MIT"
documentation = "https://docs.rs/hooksmith"
repository = "https://github.com/bdelanghe/hooksmith"
keywords = ["cli", "hooks", "lefthook", "wasm", "git"]
categories = ["command-line-utilities", "development-tools"]

[workspace]
members = [
    ".",
    "components/cli-core",
    "components/worktree-runner",
    "components/git-filter",
    "components/hook-builder",
    "lefthook-rs",
    "xtask",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Hooksmith Team"]
license = "MIT"

[workspace.dependencies]
# CLI framework
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
console = "0.15"
indicatif = "0.17"

# File system and paths
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"

# Git operations
git2 = "0.18"
gix-filter = "0.8"

# Process and command execution
which = "5.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Documentation
rustdoc-stripper = "0.1"

# WASM and WIT support
wasmtime = "18.0"
wit-bindgen = "0.20"
wasmtime-wasi = "18.0"
wit-parser = "0.12"
wit-component = "0.12"
wat = "1.0"
wasmparser = "0.120"

# Time and date handling
chrono = { version = "0.4", features = ["serde"] }

# JSON Schema validation
jsonschema = "0.17"
reqwest = { version = "0.11", features = ["json"] }

# Cryptographic hashing
sha2 = "0.10"

# Error handling
thiserror = "1.0"

# Async runtime
once_cell = "1.0"
regex = "1.0"
futures-io = "0.3"

[[bin]]
name = "hooksmith"
path = "src/main.rs"

[lib]
name = "hooksmith"
path = "src/lib.rs"

[dependencies]
# CLI dependencies
clap.workspace = true
tokio.workspace = true
anyhow.workspace = true
console.workspace = true
indicatif.workspace = true
serde.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
toml.workspace = true
git2.workspace = true
which.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true

# Internal component dependencies
cli-core = { path = "components/cli-core" }
git-filter = { path = "components/git-filter" }

# WASM dependencies
wasmtime.workspace = true
wit-bindgen.workspace = true
wasmtime-wasi.workspace = true

# Time and date handling
chrono.workspace = true

# JSON Schema validation
jsonschema.workspace = true
reqwest.workspace = true

# Cryptographic hashing
sha2.workspace = true

[dev-dependencies]
tempfile = "3.0"

[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
"#;
    
    fs::write("Cargo.toml", cargo_toml)
        .context("Failed to write Cargo.toml")?;

    Ok(())
}

fn generate_component_cargo_tomls() -> Result<()> {
    // Create components directory if it doesn't exist
    fs::create_dir_all("components/cli-core/src")?;
    fs::create_dir_all("components/worktree-runner/src")?;
    fs::create_dir_all("components/git-filter/src")?;
    fs::create_dir_all("components/hook-builder/src")?;
    fs::create_dir_all("lefthook-rs/src")?;

    // Generate cli-core Cargo.toml
    let cli_core_toml = r#"[package]
name = "cli-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
clap.workspace = true
anyhow.workspace = true
console.workspace = true
indicatif.workspace = true
serde.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
toml.workspace = true
tracing.workspace = true
"#;
    fs::write("components/cli-core/Cargo.toml", cli_core_toml)?;

    // Generate worktree-runner Cargo.toml
    let worktree_runner_toml = r#"[package]
name = "worktree-runner"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
git2.workspace = true
which.workspace = true
tracing.workspace = true
"#;
    fs::write("components/worktree-runner/Cargo.toml", worktree_runner_toml)?;

    // Generate git-filter Cargo.toml with all required dependencies
    let git_filter_toml = r#"[package]
name = "git-filter"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
git2.workspace = true
gix-filter.workspace = true
tracing.workspace = true
jsonschema.workspace = true
sha2.workspace = true
chrono.workspace = true
thiserror.workspace = true
once_cell.workspace = true
regex.workspace = true
futures-io.workspace = true
"#;
    fs::write("components/git-filter/Cargo.toml", git_filter_toml)?;

    // Generate hook-builder Cargo.toml
    let hook_builder_toml = r#"[package]
name = "hook-builder"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
tokio.workspace = true
wasmtime.workspace = true
wit-bindgen.workspace = true
wit-parser.workspace = true
wit-component.workspace = true
wat.workspace = true
wasmparser.workspace = true
tracing.workspace = true
"#;
    fs::write("components/hook-builder/Cargo.toml", hook_builder_toml)?;

    // Generate lefthook-rs Cargo.toml
    let lefthook_rs_toml = r#"[package]
name = "lefthook-rs"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
anyhow.workspace = true
serde.workspace = true
serde_yaml.workspace = true
tokio.workspace = true
which.workspace = true
tracing.workspace = true
"#;
    fs::write("lefthook-rs/Cargo.toml", lefthook_rs_toml)?;

    Ok(())
}

fn generate_xtask_cargo_toml() -> Result<()> {
    let xtask_toml = r#"[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
clap = { version = "4.0", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
regex = "1.0"
chrono = { version = "0.4", features = ["serde"] }
cargo_metadata = "0.18"
sha2 = "0.10"
tempfile = "3.0"
jsonschema = "0.17"
schemars = "0.8"
toml = "0.8"
hooksmith = { path = ".." }

[dev-dependencies]
tempfile = "3.0"
"#;
    
    fs::write("xtask/Cargo.toml", xtask_toml)?;
    Ok(())
}

fn build_xtask() -> Result<()> {
    let output = Command::new("cargo")
        .args(&["build", "--manifest-path", "xtask/Cargo.toml"])
        .output()
        .context("Failed to execute cargo build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to build xtask: {}", stderr);
    }

    Ok(())
}

fn generate_documentation() -> Result<()> {
    // Generate README using the existing doc gen system
    let output = Command::new("./target/debug/xtask")
        .args(&["gen-readme", "--overwrite"])
        .output()
        .context("Failed to execute xtask gen-readme")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("⚠️  Warning: Failed to generate README: {}", stderr);
        // Don't fail the bootstrap if doc generation fails
    } else {
        println!("   ✅ Generated README.md");
    }

    // Try to generate comprehensive documentation if possible
    let output = Command::new("./target/debug/xtask")
        .args(&["gen-docs-comprehensive", "--all"])
        .output()
        .context("Failed to execute xtask gen-docs-comprehensive")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("⚠️  Warning: Failed to generate comprehensive docs: {}", stderr);
        println!("   You can run './target/debug/xtask gen-docs-comprehensive --all' manually later");
    } else {
        println!("   ✅ Generated comprehensive documentation");
    }

    Ok(())
}
