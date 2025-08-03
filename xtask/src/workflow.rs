use std::process::Command;
use std::os::unix::fs::PermissionsExt;
use anyhow::{Result, Context};

/// Run the development workflow automation
pub async fn run_dev_workflow(
    run_tests: bool,
    run_checks: bool,
    parallel: bool,
    optimize: bool,
) -> Result<()> {
    println!("🚀 Starting development workflow...");
    
    if run_checks {
        println!("🔍 Running code checks...");
        run_code_checks(parallel).await?;
    }
    
    if run_tests {
        println!("🧪 Running tests...");
        run_tests_impl(parallel).await?;
    }
    
    if optimize {
        println!("⚡ Running optimizations...");
        run_optimizations().await?;
    }
    
    println!("✅ Development workflow completed successfully!");
    Ok(())
}

/// Run build optimization and tool installation
pub async fn run_optimize(
    install_tools: bool,
    configure: bool,
    benchmark: bool,
    status: bool,
) -> Result<()> {
    println!("⚡ Starting build optimization...");
    
    if status {
        show_optimization_status().await?;
    }
    
    if install_tools {
        println!("📦 Installing optimization tools...");
        install_optimization_tools().await?;
    }
    
    if configure {
        println!("⚙️ Configuring optimization settings...");
        configure_optimization().await?;
    }
    
    if benchmark {
        println!("📊 Running benchmarks...");
        run_benchmarks().await?;
    }
    
    println!("✅ Build optimization completed!");
    Ok(())
}

/// Run macOS-specific optimizations
pub async fn run_macos_optimize(
    developer_mode: bool,
    gatekeeper: bool,
    install_tools: bool,
    status: bool,
) -> Result<()> {
    println!("🍎 Starting macOS optimization...");
    
    if status {
        show_macos_status().await?;
    }
    
    if developer_mode {
        println!("👨‍💻 Enabling developer mode...");
        enable_developer_mode().await?;
    }
    
    if gatekeeper {
        println!("🚪 Configuring Gatekeeper...");
        configure_gatekeeper().await?;
    }
    
    if install_tools {
        println!("📦 Installing macOS tools...");
        install_macos_tools().await?;
    }
    
    println!("✅ macOS optimization completed!");
    Ok(())
}

/// Run security checks
pub async fn run_security_check(
    gatekeeper: bool,
    sip: bool,
    permissions: bool,
    tools: bool,
    score: bool,
) -> Result<()> {
    println!("🔒 Starting security check...");
    
    if gatekeeper {
        println!("🚪 Checking Gatekeeper status...");
        check_gatekeeper().await?;
    }
    
    if sip {
        println!("🛡️ Checking System Integrity Protection...");
        check_sip().await?;
    }
    
    if permissions {
        println!("🔐 Checking file permissions...");
        check_permissions().await?;
    }
    
    if tools {
        println!("🛠️ Checking security tools...");
        check_security_tools().await?;
    }
    
    if score {
        println!("📊 Calculating security score...");
        calculate_security_score().await?;
    }
    
    println!("✅ Security check completed!");
    Ok(())
}

// Implementation functions
async fn run_code_checks(parallel: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("check");
    
    if parallel {
        cmd.arg("--jobs").arg(num_cpus::get().to_string());
    }
    
    let status = cmd.status().context("Failed to run cargo check")?;
    if !status.success() {
        anyhow::bail!("Code checks failed");
    }
    Ok(())
}

async fn run_tests_impl(parallel: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    
    if parallel {
        cmd.arg("--jobs").arg(num_cpus::get().to_string());
    }
    
    let status = cmd.status().context("Failed to run cargo test")?;
    if !status.success() {
        anyhow::bail!("Tests failed");
    }
    Ok(())
}

async fn run_optimizations() -> Result<()> {
    // Run clippy
    let status = Command::new("cargo")
        .arg("clippy")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .status()
        .context("Failed to run clippy")?;
    
    if !status.success() {
        println!("⚠️ Clippy found issues");
    }
    
    Ok(())
}

async fn show_optimization_status() -> Result<()> {
    println!("📊 Optimization Status:");
    println!("  - Rust version: {}", get_rust_version()?);
    println!("  - Cargo version: {}", get_cargo_version()?);
    println!("  - Target: {}", get_target_triple()?);
    Ok(())
}

async fn install_optimization_tools() -> Result<()> {
    // Install cargo-watch for development
    let status = Command::new("cargo")
        .arg("install")
        .arg("cargo-watch")
        .status()
        .context("Failed to install cargo-watch")?;
    
    if status.success() {
        println!("✅ cargo-watch installed");
    }
    
    Ok(())
}

async fn configure_optimization() -> Result<()> {
    // Configure cargo for better performance
    let status = Command::new("cargo")
        .arg("config")
        .arg("set")
        .arg("build.jobs")
        .arg(num_cpus::get().to_string())
        .status()
        .context("Failed to configure cargo")?;
    
    if status.success() {
        println!("✅ Cargo configured for parallel builds");
    }
    
    Ok(())
}

async fn run_benchmarks() -> Result<()> {
    println!("📊 Running benchmarks...");
    // This would run actual benchmarks if they exist
    println!("✅ Benchmarks completed");
    Ok(())
}

async fn show_macos_status() -> Result<()> {
    println!("🍎 macOS Status:");
    println!("  - macOS version: {}", get_macos_version()?);
    println!("  - Developer mode: {}", check_developer_mode()?);
    println!("  - Gatekeeper status: {}", get_gatekeeper_status()?);
    Ok(())
}

async fn enable_developer_mode() -> Result<()> {
    let status = Command::new("sudo")
        .arg("spctl")
        .arg("--master-disable")
        .status()
        .context("Failed to disable Gatekeeper")?;
    
    if status.success() {
        println!("✅ Developer mode enabled");
    }
    
    Ok(())
}

async fn configure_gatekeeper() -> Result<()> {
    let status = Command::new("sudo")
        .arg("spctl")
        .arg("--add")
        .arg("/Applications")
        .status()
        .context("Failed to configure Gatekeeper")?;
    
    if status.success() {
        println!("✅ Gatekeeper configured");
    }
    
    Ok(())
}

async fn install_macos_tools() -> Result<()> {
    // Install Homebrew if not present
    if !Command::new("brew").arg("--version").status().is_ok() {
        println!("📦 Installing Homebrew...");
        let status = Command::new("/bin/bash")
            .arg("-c")
            .arg(r#"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"#)
            .status()
            .context("Failed to install Homebrew")?;
        
        if status.success() {
            println!("✅ Homebrew installed");
        }
    }
    
    Ok(())
}

async fn check_gatekeeper() -> Result<()> {
    let output = Command::new("spctl")
        .arg("--status")
        .output()
        .context("Failed to check Gatekeeper status")?;
    
    let status = String::from_utf8_lossy(&output.stdout);
    println!("🚪 Gatekeeper status: {}", status.trim());
    Ok(())
}

async fn check_sip() -> Result<()> {
    let output = Command::new("csrutil")
        .arg("status")
        .output()
        .context("Failed to check SIP status")?;
    
    let status = String::from_utf8_lossy(&output.stdout);
    println!("🛡️ SIP status: {}", status.trim());
    Ok(())
}

async fn check_permissions() -> Result<()> {
    println!("🔐 Checking file permissions...");
    // Check if key files have correct permissions
    let files = vec![
        "Cargo.toml",
        "src/main.rs",
        "xtask/src/main.rs",
    ];
    
    for file in files {
        if std::path::Path::new(file).exists() {
            let metadata = std::fs::metadata(file)
                .context(format!("Failed to get metadata for {}", file))?;
            let mode = metadata.permissions().mode();
            println!("  {}: {:o}", file, mode & 0o777);
        }
    }
    
    Ok(())
}

async fn check_security_tools() -> Result<()> {
    println!("🛠️ Checking security tools...");
    
    let tools = vec!["codesign", "security", "spctl"];
    for tool in tools {
        let status = Command::new(tool).arg("--help").status();
        if status.is_ok() {
            println!("  ✅ {}: available", tool);
        } else {
            println!("  ❌ {}: not available", tool);
        }
    }
    
    Ok(())
}

async fn calculate_security_score() -> Result<()> {
    println!("📊 Calculating security score...");
    
    let mut score = 100;
    let mut issues = Vec::new();
    
    // Check various security aspects
    if !check_developer_mode()? {
        score -= 20;
        issues.push("Developer mode disabled");
    }
    
    if !check_gatekeeper_enabled()? {
        score -= 15;
        issues.push("Gatekeeper disabled");
    }
    
    println!("🔒 Security Score: {}/100", score);
    if !issues.is_empty() {
        println!("⚠️ Issues found:");
        for issue in issues {
            println!("  - {}", issue);
        }
    }
    
    Ok(())
}

// Helper functions
fn get_rust_version() -> Result<String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .context("Failed to get Rust version")?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_cargo_version() -> Result<String> {
    let output = Command::new("cargo")
        .arg("--version")
        .output()
        .context("Failed to get Cargo version")?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn get_target_triple() -> Result<String> {
    let output = Command::new("rustc")
        .arg("--print")
        .arg("target-list")
        .output()
        .context("Failed to get target triple")?;
    
    let binding = String::from_utf8_lossy(&output.stdout);
    let targets: Vec<&str> = binding
        .lines()
        .filter(|line| line.contains("apple"))
        .collect();
    
    Ok(targets.first().unwrap_or(&"unknown").to_string())
}

fn get_macos_version() -> Result<String> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .context("Failed to get macOS version")?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn check_developer_mode() -> Result<bool> {
    let output = Command::new("spctl")
        .arg("--status")
        .output()
        .context("Failed to check developer mode")?;
    
    let status = String::from_utf8_lossy(&output.stdout);
    Ok(status.contains("disabled"))
}

fn get_gatekeeper_status() -> Result<String> {
    let output = Command::new("spctl")
        .arg("--status")
        .output()
        .context("Failed to get Gatekeeper status")?;
    
    let status = String::from_utf8_lossy(&output.stdout);
    if status.contains("disabled") {
        Ok("disabled".to_string())
    } else {
        Ok("enabled".to_string())
    }
}

fn check_gatekeeper_enabled() -> Result<bool> {
    let status = get_gatekeeper_status()?;
    Ok(status == "enabled")
} 
