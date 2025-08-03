//! Development workflow script for Hooksmith
//!
//! This script provides a unified interface for common development tasks,
//! integrating all the Rust tooling and Git hooks we've set up.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "dev-workflow",
    about = "Development workflow script for Hooksmith",
    version,
    long_about = "Provides a unified interface for common development tasks, integrating all Rust tooling and Git hooks."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, short, default_value = "false")]
    verbose: bool,

    #[arg(long, short, default_value = "false")]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up the development environment
    Setup {
        #[arg(long, short, default_value = "false")]
        force: bool,
    },

    /// Run all quality checks
    Quality {
        #[arg(long, short, default_value = "false")]
        strict: bool,

        #[arg(long, short, default_value = "false")]
        fix: bool,
    },

    /// Build the project
    Build {
        #[arg(long, short, default_value = "false")]
        release: bool,

        #[arg(long, short, default_value = "false")]
        wasm: bool,

        #[arg(long, short, default_value = "false")]
        wasi: bool,
    },

    /// Run tests
    Test {
        #[arg(long, short, default_value = "false")]
        all_targets: bool,

        #[arg(long, short, default_value = "false")]
        all_features: bool,

        #[arg(long, short)]
        filter: Option<String>,
    },

    /// Generate documentation
    Docs {
        #[arg(long, short, default_value = "false")]
        open: bool,

        #[arg(long, short, default_value = "false")]
        serve: bool,
    },

    /// Run Git hooks manually
    Hooks {
        #[arg(long, short, default_value = "pre-commit")]
        hook: HookType,
    },

    /// Generate missing Cargo.toml files
    GenerateCargoToml,

    /// Update dependencies
    Update {
        #[arg(long, short, default_value = "false")]
        aggressive: bool,
    },

    /// Clean build artifacts
    Clean {
        #[arg(long, short, default_value = "false")]
        all: bool,
    },

    /// Show project status
    Status,

    /// Run development server
    Dev {
        #[arg(long, short, default_value = "false")]
        watch: bool,
    },
}

#[derive(ValueEnum, Clone)]
enum HookType {
    PreCommit,
    PrePush,
    CommitMsg,
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::PreCommit => write!(f, "pre-commit"),
            HookType::PrePush => write!(f, "pre-push"),
            HookType::CommitMsg => write!(f, "commit-msg"),
        }
    }
}

#[derive(Debug)]
struct WorkflowResult {
    success: bool,
    message: String,
    details: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let term = Term::stdout();

    if cli.verbose {
        println!("🔧 Hooksmith Development Workflow");
        println!("Verbose mode: enabled");
        println!(
            "Dry run mode: {}",
            if cli.dry_run { "enabled" } else { "disabled" }
        );
    }

    let result = match cli.command {
        Commands::Setup { force } => {
            println!("Note: Setup is now handled by the dedicated setup script");
            println!("Run: cargo run --bin setup");
            Ok(WorkflowResult {
                success: true,
                message: "Setup command redirected".to_string(),
                details: Some("Use 'cargo run --bin setup' for environment setup".to_string()),
            })
        }
        Commands::Quality { strict, fix } => {
            run_quality_checks(strict, fix, cli.verbose, cli.dry_run)
        }
        Commands::Build {
            release,
            wasm,
            wasi,
        } => build_project(release, wasm, wasi, cli.verbose, cli.dry_run),
        Commands::Test {
            all_targets,
            all_features,
            filter,
        } => run_tests(all_targets, all_features, filter, cli.verbose, cli.dry_run),
        Commands::Docs { open, serve } => generate_docs(open, serve, cli.verbose, cli.dry_run),
        Commands::Hooks { hook } => run_hooks(hook, cli.verbose, cli.dry_run),
        Commands::GenerateCargoToml => generate_cargo_toml(cli.verbose, cli.dry_run),
        Commands::Update { aggressive } => {
            update_dependencies(aggressive, cli.verbose, cli.dry_run)
        }
        Commands::Clean { all } => clean_project(all, cli.verbose, cli.dry_run),
        Commands::Status => show_status(cli.verbose),
        Commands::Dev { watch } => run_dev_server(watch, cli.verbose, cli.dry_run),
    }?;

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

fn run_quality_checks(
    strict: bool,
    fix: bool,
    verbose: bool,
    dry_run: bool,
) -> Result<WorkflowResult> {
    println!("🔍 Running quality checks...");

    let mut checks = vec![
        ("Formatting", "cargo fmt --all -- --check"),
        (
            "Clippy",
            "cargo clippy --all-targets --all-features -- -D warnings",
        ),
        ("Tests", "cargo test --all-targets --all-features"),
        ("Audit", "cargo audit"),
        ("Deny", "cargo deny check"),
    ];

    if fix {
        checks.insert(0, ("Fixing formatting", "cargo fmt --all"));
        checks.insert(
            1,
            (
                "Fixing clippy",
                "cargo clippy --all-targets --all-features --fix",
            ),
        );
    }

    if strict {
        checks.push((
            "Contract validation",
            "cargo run -p xtask -- contract-validate --validate-generated --comprehensive",
        ));
        checks.push((
            "Generated file validation",
            "cargo run -p xtask -- validate-generated --strict",
        ));
    }

    let mut failed_checks = Vec::new();

    if !dry_run {
        let progress = ProgressBar::new(checks.len() as u64);
        progress.set_style(ProgressStyle::default_spinner());

        for (check_name, command) in checks {
            progress.set_message(check_name);

            if verbose {
                println!("Running: {}", command);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() {
                failed_checks.push(check_name);
                if verbose {
                    println!("❌ {} failed", check_name);
                    if !output.stdout.is_empty() {
                        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
            }

            progress.inc(1);
        }

        progress.finish_with_message("✅ Quality checks complete");
    }

    if failed_checks.is_empty() {
        Ok(WorkflowResult {
            success: true,
            message: "All quality checks passed".to_string(),
            details: None,
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: format!("Quality checks failed: {}", failed_checks.join(", ")),
            details: Some("Run with --verbose for detailed output".to_string()),
        })
    }
}

fn build_project(
    release: bool,
    wasm: bool,
    wasi: bool,
    verbose: bool,
    dry_run: bool,
) -> Result<WorkflowResult> {
    println!("🔨 Building project...");

    let mut builds = vec![("Debug build", "cargo build --all-targets --all-features")];

    if release {
        builds.push((
            "Release build",
            "cargo build --release --all-targets --all-features",
        ));
    }

    if wasm {
        builds.push((
            "WASM build",
            "cargo build --target wasm32-unknown-unknown --release",
        ));
    }

    if wasi {
        builds.push(("WASI build", "cargo build --target wasm32-wasi --release"));
    }

    let mut failed_builds = Vec::new();

    if !dry_run {
        let progress = ProgressBar::new(builds.len() as u64);
        progress.set_style(ProgressStyle::default_spinner());

        for (build_name, command) in builds {
            progress.set_message(build_name);

            if verbose {
                println!("Running: {}", command);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() {
                failed_builds.push(build_name);
                if verbose {
                    println!("❌ {} failed", build_name);
                    if !output.stderr.is_empty() {
                        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
            }

            progress.inc(1);
        }

        progress.finish_with_message("✅ Build complete");
    }

    if failed_builds.is_empty() {
        Ok(WorkflowResult {
            success: true,
            message: "All builds successful".to_string(),
            details: None,
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: format!("Builds failed: {}", failed_builds.join(", ")),
            details: Some("Run with --verbose for detailed output".to_string()),
        })
    }
}

fn run_tests(
    all_targets: bool,
    all_features: bool,
    filter: Option<String>,
    verbose: bool,
    dry_run: bool,
) -> Result<WorkflowResult> {
    println!("🧪 Running tests...");

    let mut command = "cargo test".to_string();
    if all_targets {
        command.push_str(" --all-targets");
    }
    if all_features {
        command.push_str(" --all-features");
    }
    if let Some(filter) = filter {
        command.push_str(&format!(" {}", filter));
    }

    if dry_run {
        println!("Would run: {}", command);
        return Ok(WorkflowResult {
            success: true,
            message: "Test command prepared".to_string(),
            details: Some(command),
        });
    }

    if verbose {
        println!("Running: {}", command);
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(WorkflowResult {
            success: true,
            message: "All tests passed".to_string(),
            details: None,
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: "Tests failed".to_string(),
            details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

fn generate_docs(open: bool, serve: bool, verbose: bool, dry_run: bool) -> Result<WorkflowResult> {
    println!("📚 Generating documentation...");

    let mut command = "cargo doc --all-features --no-deps".to_string();
    if open {
        command.push_str(" --open");
    }

    if dry_run {
        println!("Would run: {}", command);
        return Ok(WorkflowResult {
            success: true,
            message: "Documentation command prepared".to_string(),
            details: Some(command),
        });
    }

    if verbose {
        println!("Running: {}", command);
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        let mut details = "Documentation generated successfully".to_string();
        if serve {
            details.push_str("\nStarting documentation server...");
            // Start a simple HTTP server for the docs
            Command::new("python3")
                .arg("-m")
                .arg("http.server")
                .arg("8000")
                .arg("-d")
                .arg("target/doc")
                .spawn()?;
            details.push_str("\nDocumentation available at: http://localhost:8000");
        }

        Ok(WorkflowResult {
            success: true,
            message: "Documentation generated".to_string(),
            details: Some(details),
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: "Documentation generation failed".to_string(),
            details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

fn run_hooks(hook: HookType, verbose: bool, dry_run: bool) -> Result<WorkflowResult> {
    println!("🎣 Running Git hooks...");

    let command = format!("lefthook run {}", hook);

    if dry_run {
        println!("Would run: {}", command);
        return Ok(WorkflowResult {
            success: true,
            message: "Hook command prepared".to_string(),
            details: Some(command),
        });
    }

    if verbose {
        println!("Running: {}", command);
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(WorkflowResult {
            success: true,
            message: format!("{} hook completed successfully", hook),
            details: None,
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: format!("{} hook failed", hook),
            details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

fn generate_cargo_toml(verbose: bool, dry_run: bool) -> Result<WorkflowResult> {
    println!("📝 Generating Cargo.toml files...");

    let command = "cargo run --bin generate-cargo-toml";

    if dry_run {
        println!("Would run: {}", command);
        return Ok(WorkflowResult {
            success: true,
            message: "Cargo.toml generation command prepared".to_string(),
            details: Some(command.to_string()),
        });
    }

    if verbose {
        println!("Running: {}", command);
    }

    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if output.status.success() {
        Ok(WorkflowResult {
            success: true,
            message: "Cargo.toml files generated".to_string(),
            details: Some(String::from_utf8_lossy(&output.stdout).to_string()),
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: "Cargo.toml generation failed".to_string(),
            details: Some(String::from_utf8_lossy(&output.stderr).to_string()),
        })
    }
}

fn update_dependencies(aggressive: bool, verbose: bool, dry_run: bool) -> Result<WorkflowResult> {
    println!("🔄 Updating dependencies...");

    let mut commands = vec!["cargo update"];

    if aggressive {
        commands.push("cargo outdated -R");
        commands.push("cargo audit --fix");
    }

    let mut failed_updates = Vec::new();

    if !dry_run {
        let progress = ProgressBar::new(commands.len() as u64);
        progress.set_style(ProgressStyle::default_spinner());

        for command in commands {
            progress.set_message(command);

            if verbose {
                println!("Running: {}", command);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() {
                failed_updates.push(command);
                if verbose {
                    println!("❌ {} failed", command);
                }
            }

            progress.inc(1);
        }

        progress.finish_with_message("✅ Dependencies updated");
    }

    if failed_updates.is_empty() {
        Ok(WorkflowResult {
            success: true,
            message: "Dependencies updated successfully".to_string(),
            details: None,
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: format!("Some updates failed: {}", failed_updates.join(", ")),
            details: Some("Run with --verbose for detailed output".to_string()),
        })
    }
}

fn clean_project(all: bool, verbose: bool, dry_run: bool) -> Result<WorkflowResult> {
    println!("🧹 Cleaning project...");

    let mut commands = vec!["cargo clean"];

    if all {
        commands.push("rm -rf target/");
        commands.push("rm -rf .cargo/");
        commands.push("find . -name '*.rs.bk' -delete");
    }

    if !dry_run {
        let progress = ProgressBar::new(commands.len() as u64);
        progress.set_style(ProgressStyle::default_spinner());

        for command in commands {
            progress.set_message(command);

            if verbose {
                println!("Running: {}", command);
            }

            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()?;

            if !output.status.success() && verbose {
                println!("⚠️  {} had issues", command);
            }

            progress.inc(1);
        }

        progress.finish_with_message("✅ Clean complete");
    }

    Ok(WorkflowResult {
        success: true,
        message: "Project cleaned".to_string(),
        details: None,
    })
}

fn show_status(verbose: bool) -> Result<WorkflowResult> {
    println!("📊 Project Status");

    let mut status_info = Vec::new();

    // Check Rust version
    let rust_output = Command::new("rustc").arg("--version").output()?;
    if rust_output.status.success() {
        status_info.push(format!(
            "Rust: {}",
            String::from_utf8_lossy(&rust_output.stdout).trim()
        ));
    }

    // Check Cargo version
    let cargo_output = Command::new("cargo").arg("--version").output()?;
    if cargo_output.status.success() {
        status_info.push(format!(
            "Cargo: {}",
            String::from_utf8_lossy(&cargo_output.stdout).trim()
        ));
    }

    // Check workspace status
    let workspace_output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()?;
    if workspace_output.status.success() {
        let metadata: serde_json::Value = serde_json::from_slice(&workspace_output.stdout)?;
        if let Some(workspace_members) = metadata["workspace_members"].as_array() {
            status_info.push(format!("Workspace members: {}", workspace_members.len()));
        }
    }

    // Check Git status
    let git_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;
    if git_output.status.success() {
        let changes = String::from_utf8_lossy(&git_output.stdout).lines().count();
        status_info.push(format!("Git changes: {}", changes));
    }

    let details = status_info.join("\n");

    Ok(WorkflowResult {
        success: true,
        message: "Status check complete".to_string(),
        details: Some(details),
    })
}

fn run_dev_server(watch: bool, verbose: bool, dry_run: bool) -> Result<WorkflowResult> {
    println!("🚀 Starting development server...");

    let command = if watch {
        "cargo watch -x run -x test"
    } else {
        "cargo run"
    };

    if dry_run {
        println!("Would run: {}", command);
        return Ok(WorkflowResult {
            success: true,
            message: "Development server command prepared".to_string(),
            details: Some(command.to_string()),
        });
    }

    if verbose {
        println!("Running: {}", command);
    }

    // For development server, we want to run it in the foreground
    let status = Command::new("sh").arg("-c").arg(command).status()?;

    if status.success() {
        Ok(WorkflowResult {
            success: true,
            message: "Development server completed".to_string(),
            details: None,
        })
    } else {
        Ok(WorkflowResult {
            success: false,
            message: "Development server failed".to_string(),
            details: None,
        })
    }
}
