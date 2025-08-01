//! Lefthook configuration generation
//!
//! This module provides functionality for generating Lefthook configuration files
//! that integrate with the built hooks and WASM components.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Lefthook hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookHook {
    /// Command to execute
    pub command: String,
    /// Files to run on
    pub files: Option<String>,
    /// Whether to run in parallel
    pub parallel: Option<bool>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
}

/// Lefthook configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookConfig {
    /// Pre-commit hooks
    #[serde(rename = "pre-commit")]
    pub pre_commit: Option<HashMap<String, LefthookHook>>,
    /// Pre-push hooks
    #[serde(rename = "pre-push")]
    pub pre_push: Option<HashMap<String, LefthookHook>>,
    /// Commit-msg hooks
    #[serde(rename = "commit-msg")]
    pub commit_msg: Option<HashMap<String, LefthookHook>>,
    /// Global configuration
    pub config: Option<HashMap<String, serde_json::Value>>,
}

impl Default for LefthookConfig {
    fn default() -> Self {
        Self {
            pre_commit: None,
            pre_push: None,
            commit_msg: None,
            config: None,
        }
    }
}

/// Generate a Lefthook configuration file
///
/// This function creates a lefthook.yml file that integrates the built hooks
/// with the Git workflow.
///
/// # Arguments
///
/// * `output_path` - Path where the lefthook.yml file should be written
/// * `hooks_dir` - Directory containing the built hooks
/// * `wasm_components` - Optional list of WASM component paths to include
///
/// # Returns
///
/// Returns `Ok(())` if the configuration was generated successfully.
pub fn generate_lefthook_config(
    output_path: &Path,
    hooks_dir: &str,
    wasm_components: Option<Vec<String>>,
) -> Result<()> {
    let mut config = LefthookConfig::default();

    // Add pre-commit hooks
    let mut pre_commit_hooks = HashMap::new();
    pre_commit_hooks.insert(
        "hooksmith-build".to_string(),
        LefthookHook {
            command: format!("{}/build-hook", hooks_dir),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );

    // Add WASM component hooks if provided
    if let Some(components) = wasm_components {
        for (i, component) in components.iter().enumerate() {
            pre_commit_hooks.insert(
                format!("hooksmith-wasm-{}", i),
                LefthookHook {
                    command: format!("{}/wasm-runner {}", hooks_dir, component),
                    files: Some("*.wit".to_string()),
                    parallel: Some(true),
                    env: None,
                },
            );
        }
    }

    config.pre_commit = Some(pre_commit_hooks);

    // Add pre-push hooks
    let mut pre_push_hooks = HashMap::new();
    pre_push_hooks.insert(
        "hooksmith-test".to_string(),
        LefthookHook {
            command: format!("{}/test-hook", hooks_dir),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );

    config.pre_push = Some(pre_push_hooks);

    // Write the configuration to file
    let yaml_content = serde_yaml::to_string(&config)?;
    fs::write(output_path, yaml_content)?;

    Ok(())
}

/// Validate a Lefthook configuration file
///
/// This function validates that a lefthook.yml file is properly formatted
/// and contains valid hook configurations.
///
/// # Arguments
///
/// * `config_path` - Path to the lefthook.yml file to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid.
pub fn validate_lefthook_config(config_path: &Path) -> Result<()> {
    let content = fs::read_to_string(config_path)?;
    let _config: LefthookConfig = serde_yaml::from_str(&content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_lefthook_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let hooks_dir = "target/hooks";
        let wasm_components = Some(vec!["components/worktree-runner".to_string()]);

        generate_lefthook_config(temp_file.path(), hooks_dir, wasm_components)?;

        // Verify the file was created and contains expected content
        let content = fs::read_to_string(temp_file.path())?;
        assert!(content.contains("pre-commit"));
        assert!(content.contains("hooksmith-build"));

        Ok(())
    }

    #[test]
    fn test_validate_lefthook_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let config = LefthookConfig::default();
        let yaml_content = serde_yaml::to_string(&config)?;
        fs::write(temp_file.path(), yaml_content)?;

        validate_lefthook_config(temp_file.path())?;
        Ok(())
    }
}
