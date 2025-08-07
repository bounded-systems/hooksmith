use anyhow::Result;
use std::env;
use std::process::Command;

/// Hooksmith FSMonitor Setup
///
/// This binary configures file system monitoring for optimal Git performance:
/// 1. Enables Git's built-in FSMonitor daemon if available
/// 2. Configures the Rust-based fsmonitor-watchman hook
/// 3. Sets up performance monitoring and validation
fn main() -> Result<()> {
    println!("🔧 Setting up Hooksmith FSMonitor...");

    // Check Git version for FSMonitor support
    let git_version = get_git_version()?;
    println!("📦 Git version: {}", git_version);

    if supports_builtin_fsmonitor(&git_version)? {
        println!("✅ Git supports built-in FSMonitor daemon");
        enable_builtin_fsmonitor()?;
    } else {
        println!("⚠️  Git version doesn't support built-in FSMonitor, using Rust implementation");
    }

    // Configure the fsmonitor-watchman hook
    configure_fsmonitor_hook()?;

    // Set up performance monitoring
    setup_performance_monitoring()?;

    println!("🎉 FSMonitor setup complete!");
    println!("💡 Performance tip: Run 'git status' to see the speed improvement");

    Ok(())
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

fn enable_builtin_fsmonitor() -> Result<()> {
    println!("🚀 Enabling Git's built-in FSMonitor daemon...");

    // Enable core.fsmonitor
    let status = Command::new("git")
        .args(&["config", "core.fsmonitor", "true"])
        .status()?;

    if status.success() {
        println!("✅ Built-in FSMonitor enabled");
    } else {
        println!("❌ Failed to enable built-in FSMonitor");
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
}
