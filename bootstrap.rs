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
    println!("{}", style("🚀 Hooksmith Bootstrap").bold().blue());
    println!("{}", style("Generating project structure...").dim());

    let progress = ProgressBar::new(4);
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    // Step 1: Generate main Cargo.toml
    progress.set_message("Generating main Cargo.toml...");
    generate_main_cargo_toml()?;
    progress.inc(1);

    // Step 2: Generate component Cargo.toml files
    progress.set_message("Generating component Cargo.toml files...");
    generate_component_cargo_tomls()?;
    progress.inc(1);

    // Step 3: Generate xtask Cargo.toml
    progress.set_message("Generating xtask Cargo.toml...");
    generate_xtask_cargo_toml()?;
    progress.inc(1);

    // Step 4: Build xtask
    progress.set_message("Building xtask...");
    build_xtask()?;
    progress.inc(1);

    progress.finish_with_message("✅ Bootstrap complete!");

    println!("\n{}", style("🎉 Hooksmith project bootstrapped successfully!").bold().green());
    println!("{}", style("Next steps:").bold());
    println!("  • Run 'cargo build' to build the project");
    println!("  • Run 'cargo test' to run tests");
    println!("  • Run './target/debug/xtask --help' to see available xtask commands");

    Ok(())
}

fn generate_main_cargo_toml() -> Result<()> {
    let cargo_toml = CargoToml {
        package: Package {
            name: "hooksmith".to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            authors: vec!["Hooksmith Team".to_string()],
            description: "CLI tool for building Rust binaries into Lefthook hooks with WASM components".to_string(),
            license: "MIT".to_string(),
            documentation: "https://docs.rs/hooksmith".to_string(),
            repository: "https://github.com/bdelanghe/hooksmith".to_string(),
            keywords: vec!["cli".to_string(), "hooks".to_string(), "lefthook".to_string(), "wasm".to_string(), "git".to_string()],
            categories: vec!["command-line-utilities".to_string(), "development-tools".to_string()],
        },
        workspace: Workspace {
            members: vec![
                ".".to_string(),
                "components/cli-core".to_string(),
                "components/worktree-runner".to_string(),
                "components/git-filter".to_string(),
                "components/hook-builder".to_string(),
                "lefthook-rs".to_string(),
                "xtask".to_string(),
            ],
        },
        workspace_dependencies: {
            let mut deps = HashMap::new();
            deps.insert("clap".to_string(), "{ version = \"4.0\", features = [\"derive\"] }".to_string());
            deps.insert("tokio".to_string(), "{ version = \"1.0\", features = [\"full\"] }".to_string());
            deps.insert("anyhow".to_string(), "\"1.0\"".to_string());
            deps.insert("console".to_string(), "\"0.15\"".to_string());
            deps.insert("indicatif".to_string(), "\"0.17\"".to_string());
            deps.insert("serde".to_string(), "{ version = \"1.0\", features = [\"derive\"] }".to_string());
            deps.insert("serde_json".to_string(), "\"1.0\"".to_string());
            deps.insert("serde_yaml".to_string(), "\"0.9\"".to_string());
            deps.insert("toml".to_string(), "\"0.8\"".to_string());
            deps.insert("git2".to_string(), "\"0.18\"".to_string());
            deps.insert("which".to_string(), "\"5.0\"".to_string());
            deps.insert("tracing".to_string(), "\"0.1\"".to_string());
            deps.insert("tracing-subscriber".to_string(), "\"0.3\"".to_string());
            deps.insert("rustdoc-stripper".to_string(), "\"0.1\"".to_string());
            deps.insert("wasmtime".to_string(), "\"18.0\"".to_string());
            deps.insert("wit-bindgen".to_string(), "\"0.20\"".to_string());
            deps.insert("wasmtime-wasi".to_string(), "\"18.0\"".to_string());
            deps.insert("wit-parser".to_string(), "\"0.12\"".to_string());
            deps.insert("wit-component".to_string(), "\"0.12\"".to_string());
            deps.insert("wat".to_string(), "\"1.0\"".to_string());
            deps.insert("wasmparser".to_string(), "\"0.120\"".to_string());
            deps.insert("chrono".to_string(), "{ version = \"0.4\", features = [\"serde\"] }".to_string());
            deps.insert("jsonschema".to_string(), "\"0.17\"".to_string());
            deps.insert("reqwest".to_string(), "{ version = \"0.11\", features = [\"json\"] }".to_string());
            deps.insert("sha2".to_string(), "\"0.10\"".to_string());
            deps
        },
        bin: vec![Bin {
            name: "hooksmith".to_string(),
            path: "src/main.rs".to_string(),
        }],
        lib: Lib {
            name: "hooksmith".to_string(),
            path: "src/lib.rs".to_string(),
        },
        dependencies: {
            let mut deps = HashMap::new();
            deps.insert("clap".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("tokio".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("anyhow".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("console".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("indicatif".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("serde".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("serde_json".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("serde_yaml".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("toml".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("git2".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("which".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("tracing".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("tracing-subscriber".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("cli-core".to_string(), Dependency::Path { path: "components/cli-core".to_string() });
            deps.insert("git-filter".to_string(), Dependency::Path { path: "components/git-filter".to_string() });
            deps.insert("wasmtime".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("wit-bindgen".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("wasmtime-wasi".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("chrono".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("jsonschema".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("reqwest".to_string(), Dependency::Workspace { workspace: true });
            deps.insert("sha2".to_string(), Dependency::Workspace { workspace: true });
            deps
        },
        dev_dependencies: {
            let mut deps = HashMap::new();
            deps.insert("tempfile".to_string(), "\"3.0\"".to_string());
            deps
        },
        package_metadata_docs_rs: PackageMetadataDocsRs {
            all_features: true,
            rustdoc_args: vec!["--cfg".to_string(), "docsrs".to_string()],
        },
    };

    let toml_string = toml::to_string_pretty(&cargo_toml)
        .context("Failed to serialize Cargo.toml")?;
    
    fs::write("Cargo.toml", toml_string)
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

    // Generate git-filter Cargo.toml
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
tracing.workspace = true
jsonschema.workspace = true
sha2.workspace = true
chrono.workspace = true
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
