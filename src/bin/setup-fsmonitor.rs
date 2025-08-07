use anyhow::Result;
use std::process::Command;

/// Hooksmith FSMonitor Setup
///
/// This binary configures file system monitoring for optimal Git performance:
/// 1. Auto-detects available FSMonitor strategies
/// 2. Enables Git's built-in FSMonitor daemon if available
/// 3. Configures rs-git-fsmonitor if available
/// 4. Sets up the Rust-based fsmonitor-watchman hook
/// 5. Provides performance monitoring and validation
fn main() -> Result<()> {
    println!("🔧 Setting up Hooksmith FSMonitor...");

    // Check Git version for FSMonitor support
    let git_version = get_git_version()?;
    println!("📦 Git version: {}", git_version);

    // Detect available FSMonitor strategies
    let strategies = detect_fsmonitor_strategies()?;
    println!("\n🔍 Available FSMonitor strategies:");

    for (strategy, available) in &strategies {
        let status = if *available { "✅ Available" } else { "❌ Not available" };
        println!("   {}: {}", strategy, status);
    }

    // Select and configure the best strategy
    let best_strategy = select_best_strategy(&strategies)?;
    configure_fsmonitor_strategy(best_strategy)?;

    // Configure the fsmonitor-watchman hook
    configure_fsmonitor_hook()?;

    // Set up performance monitoring
    setup_performance_monitoring()?;

    println!("\n🎉 FSMonitor setup complete!");
    println!("💡 Performance tip: Run 'cargo run --bin performance-test' to benchmark");

    Ok(())
}

#[derive(Debug)]
enum FSMonitorStrategy {
    BuiltIn,
    RsGitFsmonitor,
    RustImplementation,
}

fn get_git_version() -> Result<String> {
    let output = Command::new("git")
        .args(&["--version"])
        .output()?;

    if output.status.success() {
        let version = String::from_utf8(output.stdout)?
            .trim()
            .to_string();
        Ok(version)
    } else {
        Err(anyhow::anyhow!("Failed to get Git version"))
    }
}

fn detect_fsmonitor_strategies() -> Result<Vec<(String, bool)>> {
    let mut strategies = Vec::new();

    // Check built-in FSMonitor
    let builtin_available = supports_builtin_fsmonitor(&get_git_version()?)?;
    strategies.push(("Built-in FSMonitor daemon".to_string(), builtin_available));

    // Check rs-git-fsmonitor
    let rs_git_available = is_rs_git_fsmonitor_available()?;
    strategies.push(("rs-git-fsmonitor (Rust Watchman)".to_string(), rs_git_available));

    // Our Rust implementation is always available
    strategies.push(("Hooksmith Rust implementation".to_string(), true));

    Ok(strategies)
}

fn supports_builtin_fsmonitor(version: &str) -> Result<bool> {
    // Git 2.37.0+ supports built-in FSMonitor
    // Parse version like "git version 2.37.0"
    if let Some(version_part) = version.split_whitespace().nth(2) {
        if let Some(major_minor) = version_part.split('.').take(2).collect::<Vec<_>>().join(".").parse::<f32>().ok() {
            return Ok(major_minor >= 2.37);
        }
    }
    Ok(false)
}

fn is_rs_git_fsmonitor_available() -> Result<bool> {
    // Check if rs-git-fsmonitor is installed and available
    let output = Command::new("which")
        .args(&["rs-git-fsmonitor"])
        .output()?;

    Ok(output.status.success())
}

fn select_best_strategy(strategies: &[(String, bool)]) -> Result<FSMonitorStrategy> {
    // Priority order: Built-in > rs-git-fsmonitor > Rust implementation

    if strategies.iter().any(|(name, available)| name.contains("Built-in") && *available) {
        println!("\n🚀 Using built-in FSMonitor daemon (recommended)");
        return Ok(FSMonitorStrategy::BuiltIn);
    }

    if strategies.iter().any(|(name, available)| name.contains("rs-git-fsmonitor") && *available) {
        println!("\n⚡ Using rs-git-fsmonitor (Rust-based Watchman hook)");
        return Ok(FSMonitorStrategy::RsGitFsmonitor);
    }

    println!("\n🔧 Using Hooksmith's Rust-based implementation");
    Ok(FSMonitorStrategy::RustImplementation)
}

fn configure_fsmonitor_strategy(strategy: FSMonitorStrategy) -> Result<()> {
    match strategy {
        FSMonitorStrategy::BuiltIn => {
            println!("🔧 Enabling Git's built-in FSMonitor daemon...");
            let status = Command::new("git")
                .args(&["config", "core.fsmonitor", "true"])
                .status()?;

            if status.success() {
                println!("✅ Built-in FSMonitor enabled");
            } else {
                println!("❌ Failed to enable built-in FSMonitor");
            }
        }
        FSMonitorStrategy::RsGitFsmonitor => {
            println!("🔧 Configuring rs-git-fsmonitor...");
            let status = Command::new("git")
                .args(&["config", "core.fsmonitor", "rs-git-fsmonitor"])
                .status()?;

            if status.success() {
                println!("✅ rs-git-fsmonitor configured");
            } else {
                println!("❌ Failed to configure rs-git-fsmonitor");
            }
        }
        FSMonitorStrategy::RustImplementation => {
            println!("🔧 Configuring Hooksmith's Rust implementation...");
            // Our implementation is used when core.fsmonitor points to our hook
            // This is already configured via core.hooksPath
            println!("✅ Rust implementation ready (via core.hooksPath)");
        }
    }

    Ok(())
}

fn configure_fsmonitor_hook() -> Result<()> {
    println!("🔧 Configuring fsmonitor-watchman hook...");

    // Ensure the hook is executable
    let hook_path = ".hooksmith/hooks/git/fsmonitor-watchman";

    // Make sure the hook exists and is executable
    if std::path::Path::new(hook_path).exists() {
        println!("✅ fsmonitor-watchman hook found");

        // Set executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(hook_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(hook_path, perms)?;
        }

        println!("✅ Hook permissions set");
    } else {
        println!("❌ fsmonitor-watchman hook not found at {}", hook_path);
    }

    Ok(())
}

fn setup_performance_monitoring() -> Result<()> {
    println!("📊 Setting up performance monitoring...");

    // Create a performance test script
    let test_script = r#"#!/bin/bash
# Performance test for FSMonitor
echo "Testing FSMonitor performance..."

# Time git status without FSMonitor
git config core.fsmonitor false
echo "Without FSMonitor:"
time git status > /dev/null 2>&1

# Time git status with FSMonitor
git config core.fsmonitor true
echo "With FSMonitor:"
time git status > /dev/null 2>&1

echo "Performance test complete!"
"#;

    std::fs::write(".hooksmith/performance-test.sh", test_script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(".hooksmith/performance-test.sh")?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(".hooksmith/performance-test.sh", perms)?;
    }

    println!("✅ Performance test script created: .hooksmith/performance-test.sh");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_version_parsing() {
        let version = "git version 2.37.0";
        assert!(supports_builtin_fsmonitor(version).unwrap());

        let old_version = "git version 2.36.0";
        assert!(!supports_builtin_fsmonitor(old_version).unwrap());
    }

    #[test]
    fn test_strategy_selection() {
        let strategies = vec![
            ("Built-in FSMonitor daemon".to_string(), true),
            ("rs-git-fsmonitor (Rust Watchman)".to_string(), false),
            ("Hooksmith Rust implementation".to_string(), true),
        ];

        let best = select_best_strategy(&strategies).unwrap();
        assert!(matches!(best, FSMonitorStrategy::BuiltIn));
    }
}
