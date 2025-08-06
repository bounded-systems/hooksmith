#!/usr/bin/env rustc
//! Build script for xtask with platform auto-detection

use std::process::{Command, Stdio};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    // Get rustc version info to detect platform
    let rustc_output = Command::new("rustc")
        .args(["-vV"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let rustc_info = String::from_utf8(rustc_output.stdout)?;
    let target = rustc_info
        .lines()
        .find(|line| line.contains("host"))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("unknown");

    println!("🔧 Building xtask for platform: {}", target);

    // Build command based on platform
    let mut cargo_args = vec!["build", "-p", "xtask"];
    cargo_args.extend(args.iter().map(|s| s.as_str()));

    let target_arg = match target {
        "aarch64-apple-darwin" => {
            println!("📱 Detected Apple Silicon Mac - using native target");
            Some("aarch64-apple-darwin")
        },
        "x86_64-apple-darwin" => {
            println!("🖥️  Detected Intel Mac - using native target");
            Some("x86_64-apple-darwin")
        },
        "x86_64-unknown-linux-gnu" => {
            println!("🐧 Detected Linux x86_64 - using native target");
            None
        },
        "aarch64-unknown-linux-gnu" => {
            println!("🐧 Detected Linux ARM64 - using native target");
            None
        },
        _ => {
            println!("🖥️  Using default target for platform: {}", target);
            None
        }
    };

    if let Some(target_triple) = target_arg {
        cargo_args.extend_from_slice(&["--target", target_triple]);
    }

    let status = Command::new("cargo")
        .args(&cargo_args)
        .status()?;

    if status.success() {
        println!("✅ xtask build completed successfully!");
        Ok(())
    } else {
        std::process::exit(status.code().unwrap_or(1));
    }
}
