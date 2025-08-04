//! Rust wrapper for Lefthook Git hooks manager
//!
//! This crate provides a type-safe, Rust-native API for configuring and managing
//! Lefthook Git hooks. It eliminates the need for shell scripts and echo-based
//! YAML generation by providing structured Rust data models.
//!
//! ## Features
//!
//! - **Type-safe configuration**: Rust structs for all Lefthook configuration options
//! - **Structured YAML generation**: No more echo-based templating
//! - **Binary management**: Automatic Lefthook binary detection and installation
//! - **CLI integration**: Optional command-line interface
//! - **Async support**: Full async/await support for all operations
//!
//! ## Quick Start
//!
//! ```rust
//! use lefthook_rs::{HookConfig, HookSection, JobConfig};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create a new Lefthook configuration
//!     let mut config = HookConfig::default();
//!
//!     // Configure pre-commit hooks
//!     let mut pre_commit_jobs = HashMap::new();
//!     pre_commit_jobs.insert(
//!         "fmt".to_string(),
//!         JobConfig::new("cargo fmt --all -- --check")
//!             .with_files("*.rs")
//!             .with_stage_fixed(true),
//!     );
//!     pre_commit_jobs.insert(
//!         "clippy".to_string(),
//!         JobConfig::new("cargo clippy --all-targets --all-features -- -D warnings")
//!             .with_files("*.rs"),
//!     );
//!
//!     config.pre_commit = Some(HookSection::new(pre_commit_jobs).with_parallel(true));
//!
//!     // Write configuration to file
//!     config.write_to_file("lefthook.yml").await?;
//!
//!     // Install Lefthook hooks
//!     lefthook_rs::install().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod binary;
pub mod cli;
pub mod config;
pub mod error;

pub use config::{GlobalConfig, HookConfig, HookSection, JobConfig};
pub use error::{LefthookError, Result};

use anyhow::Context;
use std::path::Path;
use std::process::Command;

/// Install Lefthook hooks in the current repository
///
/// This function runs `lefthook install` to set up Git hooks in the current
/// repository. It will automatically detect the Lefthook binary and install
/// the hooks based on the current `lefthook.yml` configuration.
///
/// # Returns
///
/// Returns `Ok(())` if the installation was successful.
///
/// # Errors
///
/// Returns an error if:
/// - Lefthook binary is not found
/// - Installation command fails
/// - Current directory is not a Git repository
pub async fn install() -> Result<()> {
    let binary = binary::find_lefthook_binary().await?;

    let status = Command::new(binary)
        .arg("install")
        .status()
        .context("Failed to execute lefthook install")?;

    if !status.success() {
        return Err(LefthookError::Installation(format!(
            "Lefthook install failed with status: {status}"
        )));
    }

    Ok(())
}

/// Run a specific Lefthook hook
///
/// This function runs `lefthook run <hook_name>` to execute a specific hook.
///
/// # Arguments
///
/// * `hook_name` - Name of the hook to run (e.g., "pre-commit", "pre-push")
///
/// # Returns
///
/// Returns `Ok(())` if the hook execution was successful.
///
/// # Errors
///
/// Returns an error if:
/// - Lefthook binary is not found
/// - Hook execution fails
/// - Hook name is invalid
pub async fn run_hook(hook_name: &str) -> Result<()> {
    let binary = binary::find_lefthook_binary().await?;

    let status = Command::new(binary)
        .arg("run")
        .arg(hook_name)
        .status()
        .context(format!("Failed to execute lefthook run {hook_name}"))?;

    if !status.success() {
        return Err(LefthookError::CommandExecution(format!(
            "Lefthook run {hook_name} failed with status: {status}"
        )));
    }

    Ok(())
}

/// Validate a Lefthook configuration file
///
/// This function validates that a `lefthook.yml` file is properly formatted
/// and contains valid configuration.
///
/// # Arguments
///
/// * `config_path` - Path to the lefthook.yml file to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid.
///
/// # Errors
///
/// Returns an error if:
/// - File cannot be read
/// - YAML parsing fails
/// - Configuration structure is invalid
pub async fn validate_config(config_path: &Path) -> Result<()> {
    let content = tokio::fs::read_to_string(config_path)
        .await
        .context("Failed to read configuration file")?;

    let _config: HookConfig =
        serde_yaml::from_str(&content).context("Failed to parse configuration YAML")?;

    Ok(())
}

/// Check if Lefthook is installed and available
///
/// This function checks if the Lefthook binary is available in the system PATH
/// or can be downloaded.
///
/// # Returns
///
/// Returns `Ok(())` if Lefthook is available.
///
/// # Errors
///
/// Returns an error if Lefthook is not found and cannot be downloaded.
pub async fn check_installation() -> Result<()> {
    binary::find_lefthook_binary().await?;
    Ok(())
}

/// Get the version of the installed Lefthook binary
///
/// This function runs `lefthook version` to get the version information.
///
/// # Returns
///
/// Returns the version string if successful.
///
/// # Errors
///
/// Returns an error if:
/// - Lefthook binary is not found
/// - Version command fails
pub async fn get_version() -> Result<String> {
    let binary = binary::find_lefthook_binary().await?;

    let output = Command::new(binary)
        .arg("version")
        .output()
        .context("Failed to execute lefthook version")?;

    if !output.status.success() {
        return Err(LefthookError::Version(format!(
            "Lefthook version failed with status: {}",
            output.status
        )));
    }

    let version = String::from_utf8(output.stdout).context("Failed to parse version output")?;

    Ok(version.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validate_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("lefthook.yml");

        // Create a minimal valid configuration
        let config = HookConfig::default();
        config.write_to_file(&config_path).await.unwrap();

        // Validate the configuration
        validate_config(&config_path).await.unwrap();
    }

    #[tokio::test]
    async fn test_config_serialization() {
        let mut config = HookConfig::default();
        let mut pre_commit_jobs = std::collections::HashMap::new();
        pre_commit_jobs.insert("test".to_string(), JobConfig::new("echo 'test'"));
        config.pre_commit = Some(HookSection::new(pre_commit_jobs));

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("pre-commit"));
        assert!(yaml.contains("test"));
    }
}
