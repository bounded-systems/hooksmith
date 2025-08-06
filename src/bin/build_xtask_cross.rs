use hooksmith::{log_error, log_header, log_info, log_success, log_warning};
use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if we have at least one argument
    if args.len() < 2 {
        show_usage();
        std::process::exit(1);
    }

    let target = &args[1];

    // Handle help case
    if target == "--help" || target == "-h" {
        show_usage();
        return Ok(());
    }

    log_info(&format!("🔧 Cross-compiling xtask for target: {}", target));

    // Install target if not already installed
    if !is_target_installed(target)? {
        log_info(&format!("📦 Installing target: {}", target));
        install_target(target)?;
    } else {
        log_info(&format!("✅ Target {} is already installed", target));
    }

    // Build for the specified target
    log_info(&format!("🏗️  Building xtask for {}...", target));
    build_xtask(target, &args[2..])?;

    log_success("✅ xtask cross-compilation completed successfully!");
    log_info(&format!(
        "📁 Binary location: target/{}/debug/xtask",
        target
    ));

    Ok(())
}

fn show_usage() {
    println!("Usage: cargo run --bin build_xtask_cross -- <target-triple> [additional-cargo-args]");
    println!("Example: cargo run --bin build_xtask_cross -- aarch64-apple-darwin --release");
    println!("Example: cargo run --bin build_xtask_cross -- x86_64-unknown-linux-gnu");
}

fn is_target_installed(target: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("rustup").args(&["target", "list"]).output()?;

    let target_list = String::from_utf8(output.stdout)?;
    Ok(target_list.contains(&format!("{} (installed)", target)))
}

fn install_target(target: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("rustup")
        .args(&["target", "add", target])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(format!("Failed to install target {}: {}", target, stderr).into());
    }

    Ok(())
}

fn build_xtask(target: &str, additional_args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = vec!["build", "-p", "xtask", "--target", target];

    // Add additional cargo arguments
    for arg in additional_args {
        args.push(arg);
    }

    let output = Command::new("cargo").args(&args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        return Err(format!("Failed to build xtask for target {}: {}", target, stderr).into());
    }

    Ok(())
}
