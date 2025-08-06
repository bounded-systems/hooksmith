use std::process::Command;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header};

fn detect_platform() -> Result<String, String> {
    let output = Command::new("rustc")
        .args(&["-vV"])
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if line.contains("host") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(parts[1].to_string());
                }
            }
        }
    }
    
    Err("Could not detect platform".to_string())
}

fn build_xtask(target: &str, args: &[String]) -> Result<(), String> {
    let mut cargo_args = vec!["build", "-p", "xtask"];
    
    // Add target-specific arguments
    match target {
        "aarch64-apple-darwin" => {
            log_info("📱 Detected Apple Silicon Mac - using native target");
            cargo_args.extend_from_slice(&["--target", "aarch64-apple-darwin"]);
        }
        "x86_64-apple-darwin" => {
            log_info("🖥️  Detected Intel Mac - using native target");
            cargo_args.extend_from_slice(&["--target", "x86_64-apple-darwin"]);
        }
        "x86_64-unknown-linux-gnu" => {
            log_info("🐧 Detected Linux x86_64 - using native target");
        }
        "aarch64-unknown-linux-gnu" => {
            log_info("🐧 Detected Linux ARM64 - using native target");
        }
        _ => {
            log_info(&format!("🖥️  Using default target for platform: {}", target));
        }
    }
    
    // Add any additional arguments passed to the script
    cargo_args.extend(args.iter().map(|s| s.as_str()));
    
    log_info(&format!("Running: cargo {}", cargo_args.join(" ")));
    
    let output = Command::new("cargo")
        .args(&cargo_args)
        .output()
        .map_err(|e| format!("Failed to run cargo build: {}", e))?;
    
    if output.status.success() {
        log_success("xtask build completed successfully!");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Build failed: {}", stderr))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("BUILD XTASK");
    println!();
    
    // Detect platform
    let target = detect_platform()?;
    log_info(&format!("🔧 Building xtask for platform: {}", target));
    println!();
    
    // Get command line arguments (skip the first one which is the binary name)
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    // Build xtask
    match build_xtask(&target, &args) {
        Ok(()) => {
            log_success("Build completed successfully!");
            Ok(())
        }
        Err(e) => {
            log_error(&format!("Build failed: {}", e));
            Err(e.into())
        }
    }
}
