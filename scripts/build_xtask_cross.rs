#!/usr/bin/env rustc
//! Cross-compilation script for xtask
//! Usage: ./scripts/build_xtask_cross.rs <target-triple> [additional-cargo-args]

use std::process::{Command, Stdio};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: {} <target-triple> [additional-cargo-args]", env::args().next().unwrap());
        println!("Example: {} aarch64-apple-darwin --release", env::args().next().unwrap());
        println!("Example: {} x86_64-unknown-linux-gnu", env::args().next().unwrap());
        std::process::exit(1);
    }

    let target = &args[0];

    // Handle help case
    if target == "--help" || target == "-h" {
        println!("Usage: {} <target-triple> [additional-cargo-args]", env::args().next().unwrap());
        println!("Example: {} aarch64-apple-darwin --release", env::args().next().unwrap());
        println!("Example: {} x86_64-unknown-linux-gnu", env::args().next().unwrap());
        std::process::exit(0);
    }

    println!("🔧 Cross-compiling xtask for target: {}", target);

    // Check if target is installed
    let rustup_output = Command::new("rustup")
        .args(["target", "list"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let installed_targets = String::from_utf8(rustup_output.stdout)?;
    let target_installed = installed_targets
        .lines()
        .any(|line| line.contains(target) && line.contains("(installed)"));

    if !target_installed {
        println!("📦 Installing target: {}", target);
        let install_status = Command::new("rustup")
            .args(["target", "add", target])
            .status()?;

        if !install_status.success() {
            eprintln!("❌ Failed to install target: {}", target);
            std::process::exit(install_status.code().unwrap_or(1));
        }
    }

    // Build for the specified target
    println!("🏗️  Building xtask for {}...", target);

    let mut cargo_args = vec!["build", "-p", "xtask", "--target", target];
    cargo_args.extend(args.iter().skip(1).map(|s| s.as_str()));

    let status = Command::new("cargo")
        .args(&cargo_args)
        .status()?;

    if status.success() {
        println!("✅ xtask cross-compilation completed successfully!");
        println!("📁 Binary location: target/{}/debug/xtask", target);
        Ok(())
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
}
