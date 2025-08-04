//! Component Status Demo
//!
//! This example demonstrates how to use the component status system
//! to check the health of all WIT components and native crates.

use anyhow::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 Hooksmith Component Status Demo");
    println!("==================================\n");

    // Get workspace root
    let workspace_root = std::env::current_dir()?;
    println!("Workspace root: {}", workspace_root.display());

    // Check if component registry exists
    let registry_path = workspace_root.join("config/component-registry.jsonc");
    if !registry_path.exists() {
        println!("❌ Component registry not found at: {}", registry_path.display());
        println!("Please run: cargo run -p xtask -- component-status");
        return Ok(());
    }

    println!("✅ Component registry found\n");

    // Demonstrate different output formats
    println!("📊 Status Report - Table Format");
    println!("-------------------------------");
    run_component_status(&workspace_root, false, "table").await?;

    println!("\n📊 Status Report - JSON Format");
    println!("-----------------------------");
    run_component_status(&workspace_root, false, "json").await?;

    println!("\n📊 Status Report - CSV Format");
    println!("----------------------------");
    run_component_status(&workspace_root, false, "csv").await?;

    println!("\n📊 Status Report - Verbose Table");
    println!("--------------------------------");
    run_component_status(&workspace_root, true, "table").await?;

    // Demonstrate registry validation
    println!("\n🔍 Registry Validation");
    println!("---------------------");
    validate_registry(&workspace_root).await?;

    // Demonstrate individual component checks
    println!("\n🔍 Individual Component Checks");
    println!("-----------------------------");
    check_individual_components(&workspace_root).await?;

    println!("\n🎉 Component Status Demo Complete!");
    println!("\n💡 Usage Tips:");
    println!("  • cargo run -p xtask -- component-status");
    println!("  • cargo run -p xtask -- component-status --verbose");
    println!("  • cargo run -p xtask -- component-status --format json");
    println!("  • cargo run -p xtask -- component-status --format csv");

    Ok(())
}

async fn run_component_status(workspace_root: &PathBuf, verbose: bool, format: &str) -> Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["run", "-p", "xtask", "--", "component-status"]);
    
    if verbose {
        cmd.arg("--verbose");
    }
    
    cmd.args(["--format", format]);
    cmd.current_dir(workspace_root);

    let output = cmd.output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("❌ Error: {}", stderr);
    }

    Ok(())
}

async fn validate_registry(workspace_root: &PathBuf) -> Result<()> {
    let registry_path = workspace_root.join("config/component-registry.jsonc");
    let content = tokio::fs::read_to_string(&registry_path).await?;

    // Basic validation
    if !content.contains("\"metadata\"") {
        println!("❌ Invalid registry format - missing metadata");
        return Ok(());
    }

    if !content.contains("\"wit_components\"") {
        println!("❌ Invalid registry format - missing wit_components");
        return Ok(());
    }

    if !content.contains("\"native_crates\"") {
        println!("❌ Invalid registry format - missing native_crates");
        return Ok(());
    }

    println!("✅ Registry format validation passed");

    // Count components
    let wit_count = content.matches("\"name\"").count() / 2; // Divide by 2 because both sections have "name" fields
    println!("📦 Total components in registry: {}", wit_count);

    Ok(())
}

async fn check_individual_components(workspace_root: &PathBuf) -> Result<()> {
    let components = [
        ("git-filter", "crates/components/git-filter"),
        ("hook-builder", "crates/components/hook-builder"),
        ("validation-handler", "crates/components/validation-handler"),
        ("worktree-runner", "crates/components/worktree-runner"),
    ];

    let native_crates = [
        ("cli-core", "crates/components/cli-core"),
        ("file-operations", "crates/file-operations"),
        ("git-operations", "crates/git-operations"),
        ("lefthook-rs", "crates/lefthook-rs"),
        ("xtask", "crates/xtask"),
        ("event-types", "crates/event-types"),
        ("file-system-handler", "crates/file-system-handler"),
    ];

    println!("📦 WIT Components:");
    for (name, path) in &components {
        let component_path = workspace_root.join(path);
        if component_path.exists() {
            println!("  ✅ {}: {}", name, path);
            
            // Check for Cargo.toml
            let cargo_toml = component_path.join("Cargo.toml");
            if cargo_toml.exists() {
                println!("    📄 Cargo.toml: ✅");
            } else {
                println!("    📄 Cargo.toml: ❌");
            }

            // Check for WIT file
            let wit_file = component_path.join("wit").join(format!("{}.wit", name));
            if wit_file.exists() {
                println!("    🔧 WIT file: ✅");
            } else {
                println!("    🔧 WIT file: ❌");
            }
        } else {
            println!("  ❌ {}: {} (not found)", name, path);
        }
    }

    println!("\n🦀 Native Crates:");
    for (name, path) in &native_crates {
        let crate_path = workspace_root.join(path);
        if crate_path.exists() {
            println!("  ✅ {}: {}", name, path);
            
            // Check for Cargo.toml
            let cargo_toml = crate_path.join("Cargo.toml");
            if cargo_toml.exists() {
                println!("    📄 Cargo.toml: ✅");
            } else {
                println!("    📄 Cargo.toml: ❌");
            }

            // Check for src directory
            let src_dir = crate_path.join("src");
            if src_dir.exists() {
                println!("    📁 src/: ✅");
            } else {
                println!("    📁 src/: ❌");
            }
        } else {
            println!("  ❌ {}: {} (not found)", name, path);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_validation() {
        let workspace_root = std::env::current_dir().unwrap();
        let result = validate_registry(&workspace_root).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_individual_components() {
        let workspace_root = std::env::current_dir().unwrap();
        let result = check_individual_components(&workspace_root).await;
        assert!(result.is_ok());
    }
} 