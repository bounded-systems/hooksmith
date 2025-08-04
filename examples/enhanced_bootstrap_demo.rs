//! Enhanced Bootstrap Demo
//! 
//! This example demonstrates how to use the enhanced bootstrap command
//! with all its new features.

use std::process::Command;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🚀 Enhanced Bootstrap Demo");
    println!("========================\n");

    // Example 1: Basic bootstrap with validation
    println!("1️⃣ Basic bootstrap with validation:");
    run_bootstrap(&["--validate"])?;

    // Example 2: Full bootstrap with all features
    println!("\n2️⃣ Full bootstrap with all features:");
    run_bootstrap(&["--validate", "--commit", "--clean", "--verbose"])?;

    // Example 3: Dry-run to see what would be done
    println!("\n3️⃣ Dry-run to see what would be done:");
    run_bootstrap(&["--validate", "--commit", "--clean", "--dry-run", "--verbose"])?;

    // Example 4: CI/CD friendly bootstrap
    println!("\n4️⃣ CI/CD friendly bootstrap:");
    run_bootstrap(&["--validate", "--clean", "--verbose"])?;

    println!("\n✅ Enhanced bootstrap demo completed!");
    Ok(())
}

fn run_bootstrap(args: &[&str]) -> Result<()> {
    let mut command = Command::new("cargo");
    command.args(["xtask", "bootstrap"]);
    command.args(args);

    println!("   Running: cargo xtask bootstrap {}", args.join(" "));
    
    let output = command.output()?;
    
    if output.status.success() {
        println!("   ✅ Success");
        if !output.stdout.is_empty() {
            println!("   Output: {}", String::from_utf8_lossy(&output.stdout));
        }
    } else {
        println!("   ❌ Failed");
        if !output.stderr.is_empty() {
            println!("   Error: {}", String::from_utf8_lossy(&output.stderr));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootstrap_dry_run() -> Result<()> {
        // Test that dry-run works without making changes
        run_bootstrap(&["--dry-run", "--verbose"])?;
        Ok(())
    }

    #[test]
    fn test_bootstrap_help() -> Result<()> {
        // Test that help works
        let output = Command::new("cargo")
            .args(["xtask", "bootstrap", "--help"])
            .output()?;
        
        assert!(output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("Bootstrap"));
        
        Ok(())
    }
} 
