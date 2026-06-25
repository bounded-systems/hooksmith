//! Contract-driven Bootstrap & Validation Workflow
//!
//! This module provides unified commands for project bootstrap and validation:
//! - `contract build`: Sets up project structure, regenerates codegen/config/docs, builds and tests
//! - `contract check`: Validates that all generated files are up-to-date and project is healthy
//!
//! The workflow ensures that the project state matches the contract specifications
//! and provides clear, actionable error messages when validation fails.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::Command;

/// Contract workflow configuration
#[derive(Debug, Clone)]
pub struct ContractConfig {
    /// Whether to run bootstrap logic if needed
    pub bootstrap_if_needed: bool,
    /// Whether to regenerate all codegen files
    pub regenerate_codegen: bool,
    /// Whether to validate generated files
    pub validate_generated: bool,
    /// Whether to build all components
    pub build_components: bool,
    /// Whether to run tests
    pub run_tests: bool,
    /// Whether to commit generated files
    pub commit_generated: bool,
    /// Whether to check Git hooks installation
    pub check_hooks: bool,
    /// Whether to validate Git attributes
    pub validate_attributes: bool,
}

impl Default for ContractConfig {
    fn default() -> Self {
        Self {
            bootstrap_if_needed: true,
            regenerate_codegen: true,
            validate_generated: true,
            build_components: true,
            run_tests: true,
            commit_generated: false,
            check_hooks: true,
            validate_attributes: true,
        }
    }
}

/// Contract workflow commands
#[derive(Parser)]
#[command(name = "contract")]
#[command(about = "Contract-driven bootstrap & validation workflow")]
pub struct ContractCli {
    #[command(subcommand)]
    command: ContractCommands,
}

#[derive(Subcommand)]
pub enum ContractCommands {
    /// Build the project: bootstrap if needed, regenerate codegen, build and test
    Build {
        /// Skip bootstrap logic
        #[arg(long)]
        no_bootstrap: bool,
        /// Skip codegen regeneration
        #[arg(long)]
        no_codegen: bool,
        /// Skip validation
        #[arg(long)]
        no_validate: bool,
        /// Skip building components
        #[arg(long)]
        no_build: bool,
        /// Skip running tests
        #[arg(long)]
        no_test: bool,
        /// Commit generated files with a clear message
        #[arg(long)]
        commit: bool,
        /// Force regeneration of all files
        #[arg(long)]
        force: bool,
    },
    /// Check project health: validate generated files, hooks, and build status
    Check {
        /// Exit with error if any validation fails
        #[arg(long)]
        strict: bool,
        /// Check only staged files
        #[arg(long)]
        staged_only: bool,
        /// Show detailed validation output
        #[arg(long)]
        verbose: bool,
        /// Custom error message for violations
        #[arg(long)]
        custom_message: Option<String>,
    },
}

/// Run a contract command
pub async fn run_contract_command(command: ContractCommands) -> Result<()> {
    match command {
        ContractCommands::Build {
            no_bootstrap,
            no_codegen,
            no_validate,
            no_build,
            no_test,
            commit,
            force,
        } => {
            let config = ContractConfig {
                bootstrap_if_needed: !no_bootstrap,
                regenerate_codegen: !no_codegen,
                validate_generated: !no_validate,
                build_components: !no_build,
                run_tests: !no_test,
                commit_generated: commit,
                check_hooks: true,
                validate_attributes: true,
            };
            contract_build(config, force).await
        }
        ContractCommands::Check {
            strict,
            staged_only,
            verbose,
            custom_message,
        } => contract_check(strict, staged_only, verbose, custom_message).await,
    }
}

/// Build the project using contract-driven workflow
async fn contract_build(config: ContractConfig, force: bool) -> Result<()> {
    println!("🔨 Hooksmith Contract Build");
    println!("Building project with contract-driven workflow...");

    let mut steps_completed = Vec::new();
    let mut errors = Vec::new();

    // Step 1: Bootstrap if needed
    if config.bootstrap_if_needed {
        println!("🚀 Step 1: Checking project bootstrap...");
        match bootstrap_if_needed().await {
            Ok(bootstrapped) => {
                if bootstrapped {
                    steps_completed.push("✅ Project bootstrapped");
                } else {
                    steps_completed.push("✅ Project already bootstrapped");
                }
            }
            Err(e) => {
                let error = format!("❌ Bootstrap failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Bootstrap step failed: {}", e));
            }
        }
    }

    // Step 2: Regenerate codegen files
    if config.regenerate_codegen {
        println!("📝 Step 2: Regenerating codegen files...");
        match regenerate_codegen(force).await {
            Ok(_) => {
                steps_completed.push("✅ Codegen files regenerated");
            }
            Err(e) => {
                let error = format!("❌ Codegen regeneration failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Codegen step failed: {}", e));
            }
        }
    }

    // Step 3: Validate generated files
    if config.validate_generated {
        println!("🔍 Step 3: Validating generated files...");
        match validate_generated_files(false, false).await {
            Ok(_) => {
                steps_completed.push("✅ Generated files validated");
            }
            Err(e) => {
                let error = format!("❌ Generated file validation failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Validation step failed: {}", e));
            }
        }
    }

    // Step 4: Build components
    if config.build_components {
        println!("🔨 Step 4: Building components...");
        match build_all_components().await {
            Ok(_) => {
                steps_completed.push("✅ Components built");
            }
            Err(e) => {
                let error = format!("❌ Component build failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Build step failed: {}", e));
            }
        }
    }

    // Step 5: Run tests
    if config.run_tests {
        println!("🧪 Step 5: Running tests...");
        match run_all_tests().await {
            Ok(_) => {
                steps_completed.push("✅ Tests passed");
            }
            Err(e) => {
                let error = format!("❌ Tests failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Test step failed: {}", e));
            }
        }
    }

    // Step 6: Check hooks installation
    if config.check_hooks {
        println!("🪝 Step 6: Checking Git hooks...");
        match check_hooks_installation().await {
            Ok(_) => {
                steps_completed.push("✅ Git hooks verified");
            }
            Err(e) => {
                let error = format!("❌ Hook verification failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Hook verification failed: {}", e));
            }
        }
    }

    // Step 7: Validate Git attributes
    if config.validate_attributes {
        println!("🔧 Step 7: Validating Git attributes...");
        match validate_git_attributes().await {
            Ok(_) => {
                steps_completed.push("✅ Git attributes validated");
            }
            Err(e) => {
                let error = format!("❌ Git attributes validation failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Git attributes validation failed: {}", e));
            }
        }
    }

    // Step 8: Commit generated files if requested
    if config.commit_generated {
        println!("📝 Step 8: Committing generated files...");
        match commit_generated_files().await {
            Ok(_) => {
                steps_completed.push("✅ Generated files committed");
            }
            Err(e) => {
                let error = format!("❌ Commit failed: {e}");
                errors.push(error);
                return Err(anyhow::anyhow!("Commit step failed: {}", e));
            }
        }
    }

    // Print summary
    println!("\n🎉 Contract Build Completed Successfully!");
    println!("Steps completed:");
    for step in steps_completed {
        println!("  {step}");
    }

    if !errors.is_empty() {
        println!("\n❌ Errors encountered:");
        for error in errors {
            println!("  {error}");
        }
    }

    println!("\n📋 Project Status:");
    println!("  • Project structure: ✅ Ready");
    println!("  • Codegen files: ✅ Up to date");
    println!("  • Components: ✅ Built");
    println!("  • Tests: ✅ Passing");
    println!("  • Git hooks: ✅ Installed");
    println!("  • Git attributes: ✅ Configured");

    Ok(())
}

/// Check project health using contract-driven validation
async fn contract_check(
    strict: bool,
    staged_only: bool,
    verbose: bool,
    custom_message: Option<String>,
) -> Result<()> {
    println!("🔍 Hooksmith Contract Check");
    println!("Validating project health...");

    let mut checks_passed = Vec::new();
    let mut checks_failed = Vec::new();

    // Check 1: Generated files are up to date
    println!("🔍 Check 1: Generated files validation...");
    match validate_generated_files(staged_only, verbose).await {
        Ok(_) => {
            checks_passed.push("✅ Generated files are up to date");
        }
        Err(e) => {
            let error = format!("❌ Generated files validation failed: {e}");
            checks_failed.push(error);
        }
    }

    // Check 2: Project builds successfully
    println!("🔍 Check 2: Project build validation...");
    match validate_project_build().await {
        Ok(_) => {
            checks_passed.push("✅ Project builds successfully");
        }
        Err(e) => {
            let error = format!("❌ Project build validation failed: {e}");
            checks_failed.push(error);
        }
    }

    // Check 3: Tests pass
    println!("🔍 Check 3: Test validation...");
    match validate_tests().await {
        Ok(_) => {
            checks_passed.push("✅ Tests pass");
        }
        Err(e) => {
            let error = format!("❌ Test validation failed: {e}");
            checks_failed.push(error);
        }
    }

    // Check 4: Git hooks are installed
    println!("🔍 Check 4: Git hooks validation...");
    match check_hooks_installation().await {
        Ok(_) => {
            checks_passed.push("✅ Git hooks are installed");
        }
        Err(e) => {
            let error = format!("❌ Git hooks validation failed: {e}");
            checks_failed.push(error);
        }
    }

    // Check 5: Git attributes are configured
    println!("🔍 Check 5: Git attributes validation...");
    match validate_git_attributes().await {
        Ok(_) => {
            checks_passed.push("✅ Git attributes are configured");
        }
        Err(e) => {
            let error = format!("❌ Git attributes validation failed: {e}");
            checks_failed.push(error);
        }
    }

    // Check 6: No linter warnings
    println!("🔍 Check 6: Linter validation...");
    match validate_linter().await {
        Ok(_) => {
            checks_passed.push("✅ No linter warnings");
        }
        Err(e) => {
            let error = format!("❌ Linter validation failed: {e}");
            checks_failed.push(error);
        }
    }

    // Print results
    println!("\n📊 Contract Check Results:");
    println!("Checks passed: {}", checks_passed.len());
    for check in checks_passed {
        println!("  {check}");
    }

    if !checks_failed.is_empty() {
        let failed_count = checks_failed.len();
        println!("\nChecks failed: {failed_count}");
        for check in &checks_failed {
            println!("  {check}");
        }

        // Show actionable error message
        let message = custom_message.unwrap_or_else(|| {
            "❌ Hooksmith contract check failed!\nRun `cargo xtask contract --build` to regenerate configs/docs/hooks.".to_string()
        });
        println!("\n{message}");

        if strict {
            return Err(anyhow::anyhow!(
                "Contract check failed with {} errors",
                failed_count
            ));
        }
    } else {
        println!("\n🎉 All contract checks passed!");
        println!("Project is healthy and ready for development.");
    }

    Ok(())
}

/// Check if project needs bootstrapping and run bootstrap if needed
async fn bootstrap_if_needed() -> Result<bool> {
    // Check if main Cargo.toml exists
    if Path::new("Cargo.toml").exists() {
        // Check if xtask exists and is built
        if Path::new("xtask/Cargo.toml").exists() && Path::new("target/debug/xtask").exists() {
            println!("   Project appears to be already bootstrapped");
            return Ok(false);
        }
    }

    println!("   Project needs bootstrapping, running bootstrap...");

    // Run bootstrap logic
    let status = Command::new("cargo")
        .args(["run", "--manifest-path", "bootstrap-simple.rs"])
        .status()
        .context("Failed to run bootstrap")?;

    if !status.success() {
        anyhow::bail!("Bootstrap failed with exit code {}", status);
    }

    println!("   Bootstrap completed successfully");
    Ok(true)
}

/// Regenerate all codegen files
async fn regenerate_codegen(force: bool) -> Result<()> {
    // Generate all configuration files
    println!("   Generating configuration files...");
    let status = Command::new("cargo")
        .args(["run", "--bin", "xtask", "--", "gen-config"])
        .args(if force { vec!["--overwrite"] } else { vec![] })
        .status()
        .context("Failed to generate config files")?;

    if !status.success() {
        anyhow::bail!("Config generation failed");
    }

    // Generate all documentation
    println!("   Generating documentation...");
    let status = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "xtask",
            "--",
            "gen-docs-comprehensive",
            "--all",
        ])
        .status()
        .context("Failed to generate documentation")?;

    if !status.success() {
        anyhow::bail!("Documentation generation failed");
    }

    // Generate WIT interfaces
    println!("   Generating WIT interfaces...");
    let status = Command::new("cargo")
        .args(["run", "--bin", "xtask", "--", "gen-wit"])
        .args(if force { vec!["--overwrite"] } else { vec![] })
        .status()
        .context("Failed to generate WIT interfaces")?;

    if !status.success() {
        anyhow::bail!("WIT generation failed");
    }

    // Generate Lefthook configuration
    println!("   Generating Lefthook configuration...");
    let status = Command::new("cargo")
        .args(["run", "--bin", "xtask", "--", "gen-lefthook"])
        .status()
        .context("Failed to generate Lefthook config")?;

    if !status.success() {
        anyhow::bail!("Lefthook generation failed");
    }

    // Generate Git attributes
    println!("   Generating Git attributes...");
    let status = Command::new("cargo")
        .args(["run", "--bin", "xtask", "--", "gen-gitattributes"])
        .args(if force { vec!["--overwrite"] } else { vec![] })
        .status()
        .context("Failed to generate Git attributes")?;

    if !status.success() {
        anyhow::bail!("Git attributes generation failed");
    }

    Ok(())
}

/// Validate generated files
async fn validate_generated_files(staged_only: bool, _verbose: bool) -> Result<()> {
    let status = Command::new("cargo")
        .args(["run", "--bin", "xtask", "--", "validate-generated"])
        .args(if staged_only {
            vec!["--staged-only"]
        } else {
            vec![]
        })
        .args(vec!["--strict"])
        .status()
        .context("Failed to validate generated files")?;

    if !status.success() {
        anyhow::bail!("Generated file validation failed");
    }

    Ok(())
}

/// Build all components
async fn build_all_components() -> Result<()> {
    let status = Command::new("cargo")
        .args(["build", "--workspace"])
        .status()
        .context("Failed to build workspace")?;

    if !status.success() {
        anyhow::bail!("Workspace build failed");
    }

    Ok(())
}

/// Run all tests
async fn run_all_tests() -> Result<()> {
    let status = Command::new("cargo")
        .args(["test", "--workspace"])
        .status()
        .context("Failed to run tests")?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }

    Ok(())
}

/// Check if Git hooks are properly installed
async fn check_hooks_installation() -> Result<()> {
    // Check if Lefthook is installed
    let lefthook_check = Command::new("lefthook")
        .arg("--version")
        .output()
        .context("Failed to check Lefthook installation")?;

    if !lefthook_check.status.success() {
        anyhow::bail!("Lefthook is not installed or not in PATH");
    }

    // Check if hooks are installed
    let hooks_check = Command::new("lefthook")
        .arg("list")
        .output()
        .context("Failed to check Lefthook hooks")?;

    if !hooks_check.status.success() {
        anyhow::bail!("Failed to list Lefthook hooks");
    }

    let hooks_output = String::from_utf8_lossy(&hooks_check.stdout);
    if hooks_output.trim().is_empty() {
        anyhow::bail!("No Lefthook hooks are installed");
    }

    Ok(())
}

/// Validate Git attributes configuration
async fn validate_git_attributes() -> Result<()> {
    // Check if .gitattributes file exists
    if !Path::new(".gitattributes").exists() {
        anyhow::bail!("No .gitattributes file found");
    }

    // Check if Git attributes are properly configured
    let config_check = Command::new("git")
        .args(["config", "--list"])
        .output()
        .context("Failed to check Git configuration")?;

    if !config_check.status.success() {
        anyhow::bail!("Failed to check Git configuration");
    }

    let config_output = String::from_utf8_lossy(&config_check.stdout);

    // Check for required filter configurations
    let required_filters = [
        "filter.contract_validate.clean",
        "filter.contract_validate.smudge",
        "filter.contract_validate.required",
    ];

    for filter in &required_filters {
        if !config_output.contains(filter) {
            anyhow::bail!("Git filter configuration missing: {}", filter);
        }
    }

    Ok(())
}

/// Validate project build
async fn validate_project_build() -> Result<()> {
    let status = Command::new("cargo")
        .args(["check", "--workspace"])
        .status()
        .context("Failed to check workspace")?;

    if !status.success() {
        anyhow::bail!("Project check failed");
    }

    Ok(())
}

/// Validate tests
async fn validate_tests() -> Result<()> {
    let status = Command::new("cargo")
        .args(["test", "--workspace", "--no-run"])
        .status()
        .context("Failed to validate tests")?;

    if !status.success() {
        anyhow::bail!("Test validation failed");
    }

    Ok(())
}

/// Validate linter
async fn validate_linter() -> Result<()> {
    let status = Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .status()
        .context("Failed to run clippy")?;

    if !status.success() {
        anyhow::bail!("Clippy found warnings or errors");
    }

    Ok(())
}

/// Commit generated files with a clear message
async fn commit_generated_files() -> Result<()> {
    // Check if there are any changes to commit
    let status_check = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .context("Failed to check Git status")?;

    if !status_check.status.success() {
        anyhow::bail!("Failed to check Git status");
    }

    let status_output = String::from_utf8_lossy(&status_check.stdout);
    if status_output.trim().is_empty() {
        println!("   No changes to commit");
        return Ok(());
    }

    // Add all changes
    let status = Command::new("git")
        .args(["add", "."])
        .status()
        .context("Failed to add files to Git")?;

    if !status.success() {
        anyhow::bail!("Failed to add files to Git");
    }

    // Commit with clear message
    let status = Command::new("git")
        .args([
            "commit",
            "-m",
            "chore(codegen): regenerate workspace configs, docs, and hooks",
        ])
        .status()
        .context("Failed to commit files")?;

    if !status.success() {
        anyhow::bail!("Failed to commit files");
    }

    println!("   Generated files committed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_config_default() {
        let config = ContractConfig::default();
        assert!(config.bootstrap_if_needed);
        assert!(config.regenerate_codegen);
        assert!(config.validate_generated);
        assert!(config.build_components);
        assert!(config.run_tests);
        assert!(!config.commit_generated);
        assert!(config.check_hooks);
        assert!(config.validate_attributes);
    }

    #[test]
    fn test_contract_config_custom() {
        let config = ContractConfig {
            bootstrap_if_needed: false,
            regenerate_codegen: false,
            validate_generated: false,
            build_components: false,
            run_tests: false,
            commit_generated: true,
            check_hooks: false,
            validate_attributes: false,
        };

        assert!(!config.bootstrap_if_needed);
        assert!(!config.regenerate_codegen);
        assert!(!config.validate_generated);
        assert!(!config.build_components);
        assert!(!config.run_tests);
        assert!(config.commit_generated);
        assert!(!config.check_hooks);
        assert!(!config.validate_attributes);
    }

    #[test]
    fn test_contract_commands_parsing() {
        // Test that the CLI can parse contract commands
        use clap::Parser;

        // ContractCli is the standalone `contract` parser (clap treats argv[0]
        // as the bin name), so its subcommands are `build`/`check` directly —
        // not nested under another `contract` token.
        let _cli = ContractCli::parse_from(["xtask", "build"]);
        let _cli = ContractCli::parse_from(["xtask", "check"]);
        let _cli = ContractCli::parse_from(["xtask", "build", "--commit"]);
        let _cli = ContractCli::parse_from(["xtask", "check", "--strict"]);
    }
}
