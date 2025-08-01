//! Lefthook configuration generation and validation
//!
//! This module provides functionality for generating and validating Lefthook configuration files
//! that integrate with the built hooks and WASM components. It uses the official Lefthook JSON schema
//! for validation to ensure configurations are compliant with Lefthook's specifications.

use anyhow::{Context, Result};
use jsonschema::{Draft, JSONSchema};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use tracing::{debug, info, warn};

// Cache for the Lefthook JSON schema
static SCHEMA_CACHE: OnceLock<JSONSchema> = OnceLock::new();

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

/// Fetch the official Lefthook JSON schema from the repository
///
/// This function downloads the schema from the official Lefthook repository
/// and caches it for subsequent validations.
///
/// # Returns
///
/// Returns the compiled JSONSchema for Lefthook configurations.
async fn fetch_lefthook_schema() -> Result<JSONSchema> {
    let schema_url = "https://raw.githubusercontent.com/evilmartians/lefthook/refs/heads/master/schema.json";

    info!("Fetching Lefthook JSON schema from {}", schema_url);

    let response = reqwest::get(schema_url)
        .await
        .context("Failed to fetch Lefthook schema")?;

    let schema_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse schema JSON")?;

    let schema = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema_json)
        .context("Failed to compile JSON schema")?;

    info!("Successfully loaded Lefthook JSON schema");
    Ok(schema)
}
}

/// Get the cached Lefthook schema, fetching it if necessary
///
/// This function ensures the schema is loaded only once and cached
/// for subsequent validations.
///
/// # Returns
///
/// Returns the compiled JSONSchema for Lefthook configurations.
async fn get_lefthook_schema() -> Result<&'static JSONSchema> {
    SCHEMA_CACHE.get_or_try_init(|| {
        tokio::runtime::Handle::current().block_on(fetch_lefthook_schema())
    })
}

/// Validate a Lefthook configuration against the official schema
///
/// This function validates that a lefthook.yml file conforms to the
/// official Lefthook JSON schema specification.
///
/// # Arguments
///
/// * `config_json` - The configuration as a JSON value
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid according to the schema.
pub async fn validate_against_schema(config_json: &serde_json::Value) -> Result<()> {
    let schema = get_lefthook_schema().await?;

    match schema.validate(config_json) {
        Ok(_) => {
            debug!("Configuration passed schema validation");
            Ok(())
        }
        Err(errors) => {
            let error_messages: Vec<String> = errors
                .map(|error| {
                    format!(
                        "Schema validation error at {}: {}",
                        error.instance_path,
                        error.to_string()
                    )
                })
                .collect();

            let error_summary = error_messages.join("\n");
            anyhow::bail!("Configuration failed schema validation:\n{}", error_summary);
        }
    }
}

/// Generate a Lefthook configuration file
///
/// This function creates a lefthook.yml file that integrates the built hooks
/// with the Git workflow. The generated configuration is validated against
/// the official Lefthook JSON schema.
///
/// # Arguments
///
/// * `output_path` - Path where the lefthook.yml file should be written
/// * `hooks_dir` - Directory containing the built hooks
/// * `wasm_components` - Optional list of WASM component paths to include
/// * `validate_schema` - Whether to validate against the official schema
///
/// # Returns
///
/// Returns `Ok(())` if the configuration was generated successfully.
pub async fn generate_lefthook_config(
    output_path: &Path,
    hooks_dir: &str,
    wasm_components: Option<Vec<String>>,
    validate_schema: bool,
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

    // Convert to JSON for schema validation
    let config_json = serde_json::to_value(&config)
        .context("Failed to serialize configuration to JSON")?;

    // Validate against schema if requested
    if validate_schema {
        validate_against_schema(&config_json).await?;
        info!("Configuration validated against official Lefthook schema");
    }

    // Write the configuration to file
    let yaml_content = serde_yaml::to_string(&config)?;
    fs::write(output_path, yaml_content)?;

    info!("Generated Lefthook configuration at {:?}", output_path);
    Ok(())
}

/// Validate a Lefthook configuration file
///
/// This function validates that a lefthook.yml file is properly formatted
/// and contains valid hook configurations. It can optionally validate
/// against the official Lefthook JSON schema.
///
/// # Arguments
///
/// * `config_path` - Path to the lefthook.yml file to validate
/// * `validate_schema` - Whether to validate against the official schema
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid.
pub async fn validate_lefthook_config(
    config_path: &Path,
    validate_schema: bool,
) -> Result<()> {
    let content = fs::read_to_string(config_path)
        .context("Failed to read configuration file")?;

    // Parse YAML
    let config: LefthookConfig = serde_yaml::from_str(&content)
        .context("Failed to parse YAML configuration")?;

    info!("Configuration file parsed successfully");

    // Validate against schema if requested
    if validate_schema {
        let config_json = serde_json::to_value(&config)
            .context("Failed to serialize configuration to JSON")?;

        validate_against_schema(&config_json).await?;
        info!("Configuration validated against official Lefthook schema");
    }

    Ok(())
}

/// Validate an existing lefthook.yml file against the official schema
///
/// This is a convenience function for validating existing configuration files
/// without modifying them.
///
/// # Arguments
///
/// * `config_path` - Path to the lefthook.yml file to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid.
pub async fn validate_existing_config(config_path: &Path) -> Result<()> {
    let content = fs::read_to_string(config_path)
        .context("Failed to read configuration file")?;

    // Parse YAML to JSON
    let config_json: serde_json::Value = serde_yaml::from_str(&content)
        .context("Failed to parse YAML configuration")?;

    // Validate against schema
    validate_against_schema(&config_json).await?;

    info!("Existing configuration validated against official Lefthook schema");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_generate_lefthook_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let hooks_dir = "target/hooks";
        let wasm_components = Some(vec!["components/worktree-runner".to_string()]);

        generate_lefthook_config(temp_file.path(), hooks_dir, wasm_components, false).await?;

        // Verify the file was created and contains expected content
        let content = fs::read_to_string(temp_file.path())?;
        assert!(content.contains("pre-commit"));
        assert!(content.contains("hooksmith-build"));

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_lefthook_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let config = LefthookConfig::default();
        let yaml_content = serde_yaml::to_string(&config)?;
        fs::write(temp_file.path(), yaml_content)?;

        validate_lefthook_config(temp_file.path(), false).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_schema_validation() -> Result<()> {
        // Test with a valid configuration
        let valid_config = serde_json::json!({
            "pre-commit": {
                "commands": {
                    "test": {
                        "run": "cargo test"
                    }
                }
            }
        });

        validate_against_schema(&valid_config).await?;
        Ok(())
    }
}
