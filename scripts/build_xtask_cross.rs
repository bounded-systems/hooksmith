#!/usr/bin/env rust-script
//! Cross-compilation script for xtask
//! Usage: ./scripts/build_xtask_cross.rs <target-triple> [additional-cargo-args]

use std::process::{Command, Stdio};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <target-triple> [additional-cargo-args]", args[0]);
        eprintln!("Example: {} aarch64-apple-darwin --release", args[0]);
        eprintln!("Example: {} x86_64-unknown-linux-gnu", args[0]);
        std::process::exit(1);
    }

    let target = &args[1];
    let additional_args: Vec<&String> = args.iter().skip(2).collect();

    // Handle help case
    if target == "--help" || target == "-h" {
        println!("Usage: {} <target-triple> [additional-cargo-args]", args[0]);
        println!("Example: {} aarch64-apple-darwin --release", args[0]);
        println!("Example: {} x86_64-unknown-linux-gnu", args[0]);
        return Ok(());
    }

    println!("🔧 Cross-compiling xtask for target: {}", target);

    // Check if target is installed
    let target_list = Command::new("rustup")
        .args(&["target", "list"])
        .stdout(Stdio::piped())
        .output()?;

    let target_list_str = String::from_utf8(target_list.stdout)?;
    let target_installed = target_list_str
        .lines()
        .any(|line| line.contains(target) && line.contains("(installed)"));

    if !target_installed {
        println!("📦 Installing target: {}", target);
        let install_status = Command::new("rustup")
            .args(&["target", "add", target])
            .status()?;

        if !install_status.success() {
            eprintln!("❌ Failed to install target: {}", target);
            std::process::exit(install_status.code().unwrap_or(1));
        }
    }

    // Build for the specified target
    println!("🏗️  Building xtask for {}...", target);
    
    let mut cmd = Command::new("cargo");
    cmd.args(&["build", "-p", "xtask", "--target", target]);
    
    // Add any additional arguments
    if !additional_args.is_empty() {
        cmd.args(additional_args);
    }

    let build_status = cmd.status()?;
    
    if build_status.success() {
        println!("✅ xtask cross-compilation completed successfully!");
        println!("📁 Binary location: target/{}/debug/xtask", target);
        Ok(())
    } else {
        eprintln!("❌ xtask cross-compilation failed with exit code: {}", build_status);
        std::process::exit(build_status.code().unwrap_or(1));
    }
} 
