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
    pub run: String,
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
}

impl Default for LefthookConfig {
    fn default() -> Self {
        Self {
            pre_commit: None,
            pre_push: None,
            commit_msg: None,
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
/// * `validate_schema` - Whether to validate against schema (currently ignored)
///
/// # Returns
///
/// Returns `Ok(())` if the configuration was generated successfully.
pub async fn generate_lefthook_config(
    output_path: &Path,
    _hooks_dir: &str,
    _wasm_components: Option<Vec<String>>,
    _validate_schema: bool,
) -> Result<()> {
    let mut config = LefthookConfig::default();

    // Add pre-commit hooks
    let mut pre_commit_hooks = HashMap::new();
    pre_commit_hooks.insert(
        "hooksmith-fmt".to_string(),
        LefthookHook {
            run: "cargo fmt --all -- --check".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "hooksmith-clippy".to_string(),
        LefthookHook {
            run: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "hooksmith-test".to_string(),
        LefthookHook {
            run: "cargo test --all-targets --all-features".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "hooksmith-gen-wit".to_string(),
        LefthookHook {
            run: "cargo xtask gen-wit".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );

    config.pre_commit = Some(pre_commit_hooks);

    // Add pre-push hooks
    let mut pre_push_hooks = HashMap::new();
    pre_push_hooks.insert(
        "hooksmith-audit".to_string(),
        LefthookHook {
            run: "cargo audit".to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );
    pre_push_hooks.insert(
        "hooksmith-check-generated".to_string(),
        LefthookHook {
            run: "cargo xtask check --strict".to_string(),
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

/// Generate comprehensive Lefthook configuration
///
/// This function creates a comprehensive lefthook.yml file with all available
/// Git hooks for documentation or as a starting point.
///
/// # Arguments
///
/// * `output_path` - Path where the lefthook.yml file should be written
/// * `validate_schema` - Whether to validate against schema (currently ignored)
///
/// # Returns
///
/// Returns `Ok(())` if the configuration was generated successfully.
pub async fn generate_comprehensive_config(
    output_path: &Path,
    _validate_schema: bool,
) -> Result<()> {
    let mut config = LefthookConfig::default();

    // Add all pre-commit hooks
    let mut pre_commit_hooks = HashMap::new();
    pre_commit_hooks.insert(
        "rustfmt".to_string(),
        LefthookHook {
            run: "cargo fmt --all -- --check".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "clippy".to_string(),
        LefthookHook {
            run: "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "test".to_string(),
        LefthookHook {
            run: "cargo test --all-targets --all-features".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "gen-wit".to_string(),
        LefthookHook {
            run: "cargo xtask gen-wit --overwrite".to_string(),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        },
    );
    pre_commit_hooks.insert(
        "check-generated".to_string(),
        LefthookHook {
            run: "cargo xtask check --strict".to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );

    config.pre_commit = Some(pre_commit_hooks);

    // Add pre-push hooks
    let mut pre_push_hooks = HashMap::new();
    pre_push_hooks.insert(
        "audit".to_string(),
        LefthookHook {
            run: "cargo audit".to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );
    pre_push_hooks.insert(
        "build".to_string(),
        LefthookHook {
            run: "cargo xtask build --target all --release".to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );

    config.pre_push = Some(pre_push_hooks);

    // Add commit-msg hooks
    let mut commit_msg_hooks = HashMap::new();
    commit_msg_hooks.insert(
        "conventional-commits".to_string(),
        LefthookHook {
            run: r#"if ! echo "$1" | grep -qE "^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+"; then
  echo "Commit message must follow conventional commit format:"
  echo "  <type>(<scope>): <description>"
  echo "  Examples:"
  echo "    feat(cli): add new command"
  echo "    fix(wasm): correct parsing bug"
  echo "    docs: update README"
  exit 1
fi"#.to_string(),
            files: None,
            parallel: Some(false),
            env: None,
        },
    );

    config.commit_msg = Some(commit_msg_hooks);

    // Write the configuration to file
    let yaml_content = serde_yaml::to_string(&config)?;
    fs::write(output_path, yaml_content)?;

    Ok(())
}

/// Validate an existing Lefthook configuration file
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
pub async fn validate_existing_config(config_path: &Path) -> Result<()> {
    let content = fs::read_to_string(config_path)?;
    let _config: LefthookConfig = serde_yaml::from_str(&content)?;
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
        assert!(content.contains("hooksmith-fmt"));

        Ok(())
    }

    #[tokio::test]
    async fn test_validate_lefthook_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let mut config = LefthookConfig::default();
        let mut pre_commit_hooks = HashMap::new();
        pre_commit_hooks.insert(
            "test-hook".to_string(),
            LefthookHook {
                run: "echo 'test'".to_string(),
                files: None,
                parallel: None,
                env: None,
            },
        );
        config.pre_commit = Some(pre_commit_hooks);

        let yaml_content = serde_yaml::to_string(&config)?;
        fs::write(temp_file.path(), yaml_content)?;

        validate_existing_config(temp_file.path()).await?;
        Ok(())
    }
}
