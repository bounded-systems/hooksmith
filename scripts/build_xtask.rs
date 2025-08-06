#!/usr/bin/env rust-script
//! Build xtask with platform-specific optimizations
//! Auto-detects platform and uses appropriate target

use std::process::{Command, Stdio};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get rustc version info to detect platform
    let rustc_output = Command::new("rustc")
        .args(&["-vV"])
        .stdout(Stdio::piped())
        .output()?;

    let rustc_info = String::from_utf8(rustc_output.stdout)?;
    let target = rustc_info
        .lines()
        .find(|line| line.contains("host"))
        .and_then(|line| line.split_whitespace().nth(2))
        .unwrap_or("unknown");

    println!("🔧 Building xtask for platform: {}", target);

    // Get additional cargo arguments
    let args: Vec<String> = env::args().skip(1).collect();

    // Build command based on platform
    let mut cmd = Command::new("cargo");
    cmd.args(&["build", "-p", "xtask"]);

    // Add platform-specific target if needed
    match target {
        "aarch64-apple-darwin" => {
            println!("📱 Detected Apple Silicon Mac - using native target");
            cmd.args(&["--target", "aarch64-apple-darwin"]);
        }
        "x86_64-apple-darwin" => {
            println!("🖥️  Detected Intel Mac - using native target");
            cmd.args(&["--target", "x86_64-apple-darwin"]);
        }
        "x86_64-unknown-linux-gnu" => {
            println!("🐧 Detected Linux x86_64 - using native target");
        }
        "aarch64-unknown-linux-gnu" => {
            println!("🐧 Detected Linux ARM64 - using native target");
        }
        _ => {
            println!("🖥️  Using default target for platform: {}", target);
        }
    }

    // Add any additional arguments passed to the script
    if !args.is_empty() {
        cmd.args(&args);
    }

    // Execute the build command
    let status = cmd.status()?;
    
    if status.success() {
        println!("✅ xtask build completed successfully!");
        Ok(())
    } else {
        eprintln!("❌ xtask build failed with exit code: {}", status);
        std::process::exit(status.code().unwrap_or(1));
    }
} 
