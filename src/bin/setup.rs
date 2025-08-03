//! Hooksmith Development Environment Setup Script
//!
//! This script sets up the complete development environment for Hooksmith
//! using Rust for better error handling and cross-platform compatibility.

use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "setup",
    about = "Hooksmith Development Environment Setup",
    version,
    long_about = "Sets up the complete development environment for Hooksmith including Rust toolchain, cargo tools, and project configuration."
)]
struct Cli {
    /// Force reinstallation of tools
    #[arg(long, short, action = ArgAction::SetTrue)]
    force: bool,

    /// Skip tool installation (only setup project config)
    #[arg(long, short, action = ArgAction::SetTrue)]
    skip_tools: bool,

    /// Verbose output
    #[arg(long, short, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Dry run (show what would be done)
    #[arg(long, short, action = ArgAction::SetTrue)]
    dry_run: bool,
}

#[derive(Debug, Clone)]
struct SetupResult {
    success: bool,
    message: String,
    details: Option<String>,
}

#[derive(Debug)]
struct ToolInfo {
    name: String,
    install_command: String,
    check_command: String,
    description: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("🔧 Hooksmith Development Environment Setup");
        println!("==========================================");
        println!("Force: {}", cli.force);
        println!("Skip tools: {}", cli.skip_tools);
        println!("Verbose: {}", cli.verbose);
        println!("Dry run: {}", cli.dry_run);
        println!();
    }

    let result = setup_environment(&cli)?;

    if result.success {
        println!("{} {}", style("✅").green(), result.message);
        if let Some(details) = result.details {
            println!("{}", details);
        }
    } else {
        println!("{} {}", style("❌").red(), result.message);
        if let Some(details) = result.details {
            eprintln!("{}", details);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn setup_environment(cli: &Cli) -> Result<SetupResult> {
    // Check if we're in the project root
    let current_dir = env::current_dir()?;
    if !current_dir.join("Cargo.toml").exists() {
        return Ok(SetupResult {
            success: false,
            message: "This script must be run from the project root directory (where Cargo.toml is located)".to_string(),
            details: None,
        });
    }

    let mut steps = Vec::new();

    if !cli.skip_tools {
        steps.push((
            "Installing Rust toolchain",
            install_rust_toolchain as fn(&Cli) -> Result<SetupResult>,
        ));
        steps.push((
            "Installing Cargo tools",
            install_cargo_tools as fn(&Cli) -> Result<SetupResult>,
        ));
        steps.push((
            "Verifying installation",
            verify_installation as fn(&Cli) -> Result<SetupResult>,
        ));
    }

    steps.push((
        "Setting up project configuration",
        setup_project_config as fn(&Cli) -> Result<SetupResult>,
    ));
    steps.push((
        "Running initial build",
        run_initial_build as fn(&Cli) -> Result<SetupResult>,
    ));

    let mut results = Vec::new();

    if !cli.dry_run {
        let progress = ProgressBar::new(steps.len() as u64);
        progress.set_style(ProgressStyle::default_spinner());

        for (step_name, step_fn) in steps {
            progress.set_message(step_name);

            if cli.verbose {
                println!("Running: {}", step_name);
            }

            let result = step_fn(cli)?;
            results.push((step_name, result.clone()));

            if cli.verbose {
                match &result {
                    SetupResult { success: true, .. } => println!("✅ {} completed", step_name),
                    SetupResult {
                        success: false,
                        message,
                        ..
                    } => println!("❌ {} failed: {}", step_name, message),
                }
            }

            progress.inc(1);
        }

        progress.finish_with_message("✅ Setup complete");
    } else {
        for (step_name, _) in steps {
            println!("Would run: {}", step_name);
        }
    }

    // Check if any steps failed
    let failed_steps: Vec<_> = results
        .iter()
        .filter(|(_, result)| !result.success)
        .map(|(name, result)| format!("{}: {}", name, result.message))
        .collect();

    if failed_steps.is_empty() {
        Ok(SetupResult {
            success: true,
            message: "Development environment setup complete".to_string(),
            details: Some(show_next_steps()),
        })
    } else {
        Ok(SetupResult {
            success: false,
            message: "Setup failed".to_string(),
            details: Some(format!("Failed steps:\n{}", failed_steps.join("\n"))),
        })
    }
}

fn install_rust_toolchain(cli: &Cli) -> Result<SetupResult> {
    // Check if rustup is available
    if !command_exists("rustup")? {
        if cli.dry_run {
            return Ok(SetupResult {
                success: true,
                message: "Would install Rust toolchain".to_string(),
                details: None,
            });
        }

        // Install rustup
        let output = Command::new("curl")
            .args([
                "--proto",
                "=https",
                "--tlsv1.2",
                "-sSf",
                "https://sh.rustup.rs",
            ])
            .output()
            .context("Failed to download rustup installer")?;

        if !output.status.success() {
            return Ok(SetupResult {
                success: false,
                message: "Failed to download rustup installer".to_string(),
                details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            });
        }

        // Note: In a real implementation, you'd need to handle the interactive installation
        // For now, we'll assume rustup is already installed
        return Ok(SetupResult {
            success: false,
            message: "Rust installation requires manual setup".to_string(),
            details: Some("Please install Rust manually: https://rustup.rs/".to_string()),
        });
    }

    // Install required components
    let components = [
        "rustfmt",
        "clippy",
        "rust-analyzer",
        "rust-src",
        "rustc-dev",
        "llvm-tools-preview",
    ];

    for component in &components {
        if cli.dry_run {
            println!("Would install component: {}", component);
            continue;
        }

        let output = Command::new("rustup")
            .args(["component", "add", component])
            .output()
            .with_context(|| format!("Failed to install component: {}", component))?;

        if !output.status.success() && cli.verbose {
            println!(
                "Warning: Failed to install component {}: {}",
                component,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    // Install WASM targets
    let targets = ["wasm32-unknown-unknown", "wasm32-wasi"];

    for target in &targets {
        if cli.dry_run {
            println!("Would install target: {}", target);
            continue;
        }

        let output = Command::new("rustup")
            .args(["target", "add", target])
            .output()
            .with_context(|| format!("Failed to install target: {}", target))?;

        if !output.status.success() && cli.verbose {
            println!(
                "Warning: Failed to install target {}: {}",
                target,
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    Ok(SetupResult {
        success: true,
        message: "Rust toolchain configured".to_string(),
        details: None,
    })
}

fn install_cargo_tools(cli: &Cli) -> Result<SetupResult> {
    let tools = vec![
        ToolInfo {
            name: "cargo-watch".to_string(),
            install_command: "cargo install cargo-watch".to_string(),
            check_command: "cargo watch --version".to_string(),
            description: "File watching for development".to_string(),
        },
        ToolInfo {
            name: "cargo-audit".to_string(),
            install_command: "cargo install cargo-audit".to_string(),
            check_command: "cargo audit --version".to_string(),
            description: "Security vulnerability scanning".to_string(),
        },
        ToolInfo {
            name: "cargo-deny".to_string(),
            install_command: "cargo install cargo-deny".to_string(),
            check_command: "cargo deny --version".to_string(),
            description: "Dependency policy enforcement".to_string(),
        },
        ToolInfo {
            name: "cargo-outdated".to_string(),
            install_command: "cargo install cargo-outdated".to_string(),
            check_command: "cargo outdated --version".to_string(),
            description: "Check for outdated dependencies".to_string(),
        },
        ToolInfo {
            name: "cargo-tree".to_string(),
            install_command: "cargo install cargo-tree".to_string(),
            check_command: "cargo tree --version".to_string(),
            description: "Dependency tree visualization".to_string(),
        },
        ToolInfo {
            name: "lefthook".to_string(),
            install_command: "cargo install lefthook".to_string(),
            check_command: "lefthook --version".to_string(),
            description: "Git hooks management".to_string(),
        },
    ];

    let mut installed_tools = Vec::new();
    let mut failed_tools = Vec::new();

    for tool in tools {
        if cli.dry_run {
            println!("Would install: {} - {}", tool.name, tool.description);
            continue;
        }

        // Check if tool is already installed
        if command_exists(&tool.name)? {
            if cli.verbose {
                println!("{} is already installed", tool.name);
            }
            installed_tools.push(tool.name.clone());
            continue;
        }

        // Install tool
        let output = Command::new("sh")
            .arg("-c")
            .arg(&tool.install_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .with_context(|| format!("Failed to install {}", tool.name))?;

        if output.status.success() {
            installed_tools.push(tool.name.clone());
            if cli.verbose {
                println!("✅ {} installed", tool.name);
            }
        } else {
            failed_tools.push((
                tool.name.clone(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
            if cli.verbose {
                println!("❌ {} failed to install", tool.name);
            }
        }
    }

    if failed_tools.is_empty() {
        Ok(SetupResult {
            success: true,
            message: format!("Installed {} tools", installed_tools.len()),
            details: Some(format!("Installed: {}", installed_tools.join(", "))),
        })
    } else {
        Ok(SetupResult {
            success: false,
            message: "Some tools failed to install".to_string(),
            details: Some(format!("Failed: {:?}", failed_tools)),
        })
    }
}

fn verify_installation(cli: &Cli) -> Result<SetupResult> {
    let tools = [
        "rustc",
        "cargo",
        "rustfmt",
        "clippy",
        "cargo-watch",
        "cargo-audit",
        "cargo-deny",
        "cargo-outdated",
        "cargo-tree",
        "lefthook",
    ];

    let mut available_tools = Vec::new();
    let mut missing_tools = Vec::new();

    for tool in &tools {
        if command_exists(tool)? {
            available_tools.push(*tool);
            if cli.verbose {
                println!("✅ {} is available", tool);
            }
        } else {
            missing_tools.push(*tool);
            if cli.verbose {
                println!("❌ {} is not available", tool);
            }
        }
    }

    if missing_tools.is_empty() {
        Ok(SetupResult {
            success: true,
            message: "All tools are properly installed".to_string(),
            details: Some(format!("Available tools: {}", available_tools.join(", "))),
        })
    } else {
        Ok(SetupResult {
            success: false,
            message: "Some tools are missing".to_string(),
            details: Some(format!("Missing: {}", missing_tools.join(", "))),
        })
    }
}

fn setup_project_config(cli: &Cli) -> Result<SetupResult> {
    let config_files = [
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "rustfmt.toml",
        "clippy.toml",
        "build.rs",
    ];

    let mut existing_files = Vec::new();
    let mut missing_files = Vec::new();

    for file in &config_files {
        if Path::new(file).exists() {
            existing_files.push(*file);
            if cli.verbose {
                println!("✅ {} exists", file);
            }
        } else {
            missing_files.push(*file);
            if cli.verbose {
                println!("⚠️  {} is missing", file);
            }
        }
    }

    // Setup Git hooks if lefthook is available
    let mut hook_status = "Skipped";
    if command_exists("lefthook")? {
        if cli.dry_run {
            hook_status = "Would setup";
        } else {
            let output = Command::new("lefthook")
                .arg("install")
                .output()
                .context("Failed to setup Git hooks")?;

            if output.status.success() {
                hook_status = "Configured";
                if cli.verbose {
                    println!("✅ Git hooks configured");
                }
            } else {
                hook_status = "Failed";
                if cli.verbose {
                    println!("❌ Git hooks setup failed");
                }
            }
        }
    }

    let details = format!(
        "Configuration files: {}/{} present\nGit hooks: {}",
        existing_files.len(),
        config_files.len(),
        hook_status
    );

    Ok(SetupResult {
        success: missing_files.is_empty(),
        message: "Project configuration setup".to_string(),
        details: Some(details),
    })
}

fn run_initial_build(cli: &Cli) -> Result<SetupResult> {
    if cli.dry_run {
        return Ok(SetupResult {
            success: true,
            message: "Would run initial build".to_string(),
            details: None,
        });
    }

    let output = Command::new("cargo")
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to run cargo build")?;

    if output.status.success() {
        Ok(SetupResult {
            success: true,
            message: "Initial build successful".to_string(),
            details: None,
        })
    } else {
        Ok(SetupResult {
            success: false,
            message: "Initial build failed".to_string(),
            details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

fn command_exists(command: &str) -> Result<bool> {
    let output = Command::new("which")
        .arg(command)
        .output()
        .context("Failed to check if command exists")?;

    Ok(output.status.success())
}

fn show_next_steps() -> String {
    r#"
Next steps:
1. Review the configuration files:
   - rust-toolchain.toml
   - .cargo/config.toml
   - rustfmt.toml
   - clippy.toml
   - build.rs

2. Try the development workflow:
   cargo run --bin dev-workflow -- status
   cargo run --bin dev-workflow -- quality

3. Generate documentation:
   cargo run --bin dev-workflow -- docs --open

4. Check available commands:
   cargo run --bin dev-workflow -- --help

For more information, see RUST_TOOLING_SETUP.md
"#
    .to_string()
}
