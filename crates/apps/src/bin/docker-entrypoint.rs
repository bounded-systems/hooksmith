use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

/// Hooksmith Docker Entrypoint - Pure Rust Implementation
///
/// This binary replaces the shell entrypoint script with:
/// - Clean argument passing
/// - Environment setup for GitHub Actions and act
/// - Binary validation and auto-build
/// - Proper workspace handling
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    println!("🔧 Hooksmith entrypoint starting...");
    println!("📦 Arguments: {}", args.len() - 1);
    println!("🔍 Command: {}", args[1..].join(" "));

    // Set up workspace directory
    if let Ok(workspace) = env::var("GITHUB_WORKSPACE") {
        println!("📁 Setting workspace: {}", workspace);
        env::set_current_dir(&workspace)?;
    }

    // Set up Rust environment
    let cargo_env = Path::new("/root/.cargo/env");
    if cargo_env.exists() {
        println!("🦀 Loading Rust environment...");
        // Note: In Rust, we don't need to source the env file
        // The PATH is already set in the Docker image
    }

    // Validate and locate Hooksmith binary
    let hooksmith_bin = locate_hooksmith_binary()?;
    env::set_var("HOOKSMITH_BIN", &hooksmith_bin);

    // Execute the command
    println!("🚀 Executing: {}", args[1..].join(" "));

    if args.len() > 1 {
        let status = Command::new(&args[1])
            .args(&args[2..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        std::process::exit(status.code().unwrap_or(1));
    } else {
        // Default command
        let status = Command::new("hooksmith")
            .arg("--help")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        std::process::exit(status.code().unwrap_or(1));
    }
}

fn locate_hooksmith_binary() -> Result<String> {
    // Check for binary in system path
    let system_path = "/usr/local/bin/hooksmith";
    if Path::new(system_path).exists() {
        println!("✅ Hooksmith binary found in system path");
        return Ok(system_path.to_string());
    }

    // Check for binary in container path
    let container_path = "/hooksmith/target/release/hooksmith";
    if Path::new(container_path).exists() {
        println!("✅ Hooksmith binary found in container");
        return Ok(container_path.to_string());
    }

    // Check for binary in workspace
    let workspace_path = "./target/release/hooksmith";
    if Path::new(workspace_path).exists() {
        println!("✅ Hooksmith binary found in workspace");
        return Ok(workspace_path.to_string());
    }

    // Try to build the binary (only if source is available)
    println!("⚠️  Hooksmith binary not found, attempting to build...");
    let build_status = Command::new("cargo")
        .args(&["build", "--release"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match build_status {
        Ok(status) if status.success() => {
            println!("✅ Hooksmith binary built successfully");
            Ok("./target/release/hooksmith".to_string())
        }
        _ => {
            println!("❌ Hooksmith binary not found and cannot be built");
            Err(anyhow::anyhow!("Hooksmith binary not available"))
        }
    }
}
