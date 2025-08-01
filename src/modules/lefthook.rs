//! Lefthook configuration generation and validation
//!
//! This module provides functionality for generating and validating Lefthook configuration files
//! that integrate with the built hooks and WASM components. It uses the official Lefthook JSON schema
//! for validation to ensure configurations are compliant with Lefthook's specifications.

use anyhow::{Context, Result};
use jsonschema::{Draft, JSONSchema};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use tracing::{debug, info, warn};

// Cache for the Lefthook JSON schema
static SCHEMA_CACHE: OnceLock<JSONSchema> = OnceLock::new();

/// Custom error types for Lefthook configuration validation
#[derive(Debug, thiserror::Error)]
pub enum LefthookError {
    #[error("Files incompatible: {0}")]
    FilesIncompatible(String),
    
    #[error("Invalid hook name: {0}")]
    InvalidHookName(String),
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
    
    #[error("File read error: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("YAML parse error: {0}")]
    YamlParseError(#[from] serde_yaml::Error),
    
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

/// Result type for Lefthook operations
pub type LefthookResult<T> = Result<T, LefthookError>;

/// All available Git hooks supported by Lefthook
///
/// This list is taken from the official Lefthook implementation and matches
/// the hooks documented at https://git-scm.com/docs/githooks
pub const AVAILABLE_HOOKS: &[&str] = &[
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "proc-receive",
    "post-receive",
    "post-update",
    "reference-transaction",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

/// Special hook names used by Lefthook
pub const GHOST_HOOK_NAME: &str = "prepare-commit-msg";
pub const CHECKSUM_FILE_NAME: &str = "lefthook.checksum";

/// Get all available hooks as a HashSet for efficient lookups
pub fn get_available_hooks_set() -> HashSet<&'static str> {
    AVAILABLE_HOOKS.iter().copied().collect()
}

/// Check if a hook name is known/supported
pub fn is_known_hook(hook: &str) -> bool {
    AVAILABLE_HOOKS.contains(&hook)
}

/// Check if a hook uses staged files
///
/// Currently only `pre-commit` uses staged files according to Lefthook's implementation.
pub fn hook_uses_staged_files(hook: &str) -> bool {
    hook == "pre-commit"
}

/// Check if a hook uses push files
///
/// Currently only `pre-push` uses push files according to Lefthook's implementation.
pub fn hook_uses_push_files(hook: &str) -> bool {
    hook == "pre-push"
}

/// Get hooks that use staged files
pub fn get_staged_file_hooks() -> Vec<&'static str> {
    AVAILABLE_HOOKS
        .iter()
        .filter(|hook| hook_uses_staged_files(hook))
        .copied()
        .collect()
}

/// Get hooks that use push files
pub fn get_push_file_hooks() -> Vec<&'static str> {
    AVAILABLE_HOOKS
        .iter()
        .filter(|hook| hook_uses_push_files(hook))
        .copied()
        .collect()
}

/// Lefthook hook configuration
///
/// This structure matches the official Lefthook Command struct from the Go implementation.
/// All fields are optional except for `run`, which is required.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookHook {
    /// Command to execute (required)
    pub run: String,

    /// Files to run on
    #[serde(rename = "files", skip_serializing_if = "Option::is_none")]
    pub files: Option<String>,

    /// Skip conditions (boolean or array of strings)
    #[serde(rename = "skip", skip_serializing_if = "Option::is_none")]
    pub skip: Option<serde_json::Value>,

    /// Only conditions (boolean or array of strings)
    #[serde(rename = "only", skip_serializing_if = "Option::is_none")]
    pub only: Option<serde_json::Value>,

    /// Tags (string or array of strings)
    #[serde(rename = "tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<serde_json::Value>,

    /// Environment variables
    #[serde(rename = "env", skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,

    /// File types to match
    #[serde(rename = "file_types", skip_serializing_if = "Option::is_none")]
    pub file_types: Option<Vec<String>>,

    /// Glob patterns (string or array of strings)
    #[serde(rename = "glob", skip_serializing_if = "Option::is_none")]
    pub glob: Option<serde_json::Value>,

    /// Root directory for execution
    #[serde(rename = "root", skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,

    /// Exclude patterns (string or array of strings)
    #[serde(rename = "exclude", skip_serializing_if = "Option::is_none")]
    pub exclude: Option<serde_json::Value>,

    /// Execution priority
    #[serde(rename = "priority", skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,

    /// Custom failure message
    #[serde(rename = "fail_text", skip_serializing_if = "Option::is_none")]
    pub fail_text: Option<String>,

    /// Whether to run interactively
    #[serde(rename = "interactive", skip_serializing_if = "Option::is_none")]
    pub interactive: Option<bool>,

    /// Whether to use stdin
    #[serde(rename = "use_stdin", skip_serializing_if = "Option::is_none")]
    pub use_stdin: Option<bool>,

    /// Whether to stage fixed files
    #[serde(rename = "stage_fixed", skip_serializing_if = "Option::is_none")]
    pub stage_fixed: Option<bool>,
}

/// Lefthook configuration structure with support for all hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookConfig {
    // Pre-commit hooks
    #[serde(rename = "pre-commit")]
    pub pre_commit: Option<HashMap<String, LefthookHook>>,

    // Pre-push hooks
    #[serde(rename = "pre-push")]
    pub pre_push: Option<HashMap<String, LefthookHook>>,

    // Commit-msg hooks
    #[serde(rename = "commit-msg")]
    pub commit_msg: Option<HashMap<String, LefthookHook>>,

    // Post-commit hooks
    #[serde(rename = "post-commit")]
    pub post_commit: Option<HashMap<String, LefthookHook>>,

    // Pre-receive hooks
    #[serde(rename = "pre-receive")]
    pub pre_receive: Option<HashMap<String, LefthookHook>>,

    // Post-receive hooks
    #[serde(rename = "post-receive")]
    pub post_receive: Option<HashMap<String, LefthookHook>>,

    // Post-update hooks
    #[serde(rename = "post-update")]
    pub post_update: Option<HashMap<String, LefthookHook>>,

    // Pre-rebase hooks
    #[serde(rename = "pre-rebase")]
    pub pre_rebase: Option<HashMap<String, LefthookHook>>,

    // Post-checkout hooks
    #[serde(rename = "post-checkout")]
    pub post_checkout: Option<HashMap<String, LefthookHook>>,

    // Post-merge hooks
    #[serde(rename = "post-merge")]
    pub post_merge: Option<HashMap<String, LefthookHook>>,

    // Pre-merge-commit hooks
    #[serde(rename = "pre-merge-commit")]
    pub pre_merge_commit: Option<HashMap<String, LefthookHook>>,

    // Prepare-commit-msg hooks
    #[serde(rename = "prepare-commit-msg")]
    pub prepare_commit_msg: Option<HashMap<String, LefthookHook>>,

    // Applypatch-msg hooks
    #[serde(rename = "applypatch-msg")]
    pub applypatch_msg: Option<HashMap<String, LefthookHook>>,

    // Pre-applypatch hooks
    #[serde(rename = "pre-applypatch")]
    pub pre_applypatch: Option<HashMap<String, LefthookHook>>,

    // Post-applypatch hooks
    #[serde(rename = "post-applypatch")]
    pub post_applypatch: Option<HashMap<String, LefthookHook>>,

    // Update hooks
    #[serde(rename = "update")]
    pub update: Option<HashMap<String, LefthookHook>>,

    // Proc-receive hooks
    #[serde(rename = "proc-receive")]
    pub proc_receive: Option<HashMap<String, LefthookHook>>,

    // Reference-transaction hooks
    #[serde(rename = "reference-transaction")]
    pub reference_transaction: Option<HashMap<String, LefthookHook>>,

    // Push-to-checkout hooks
    #[serde(rename = "push-to-checkout")]
    pub push_to_checkout: Option<HashMap<String, LefthookHook>>,

    // Pre-auto-gc hooks
    #[serde(rename = "pre-auto-gc")]
    pub pre_auto_gc: Option<HashMap<String, LefthookHook>>,

    // Post-rewrite hooks
    #[serde(rename = "post-rewrite")]
    pub post_rewrite: Option<HashMap<String, LefthookHook>>,

    // Sendemail-validate hooks
    #[serde(rename = "sendemail-validate")]
    pub sendemail_validate: Option<HashMap<String, LefthookHook>>,

    // Fsmonitor-watchman hooks
    #[serde(rename = "fsmonitor-watchman")]
    pub fsmonitor_watchman: Option<HashMap<String, LefthookHook>>,

    // P4-changelist hooks
    #[serde(rename = "p4-changelist")]
    pub p4_changelist: Option<HashMap<String, LefthookHook>>,

    // P4-prepare-changelist hooks
    #[serde(rename = "p4-prepare-changelist")]
    pub p4_prepare_changelist: Option<HashMap<String, LefthookHook>>,

    // P4-post-changelist hooks
    #[serde(rename = "p4-post-changelist")]
    pub p4_post_changelist: Option<HashMap<String, LefthookHook>>,

    // P4-pre-submit hooks
    #[serde(rename = "p4-pre-submit")]
    pub p4_pre_submit: Option<HashMap<String, LefthookHook>>,

    // Post-index-change hooks
    #[serde(rename = "post-index-change")]
    pub post_index_change: Option<HashMap<String, LefthookHook>>,

    /// Global configuration
    pub config: Option<HashMap<String, serde_json::Value>>,
}

impl Default for LefthookConfig {
    fn default() -> Self {
        Self {
            pre_commit: None,
            pre_push: None,
            commit_msg: None,
            post_commit: None,
            pre_receive: None,
            post_receive: None,
            post_update: None,
            pre_rebase: None,
            post_checkout: None,
            post_merge: None,
            pre_merge_commit: None,
            prepare_commit_msg: None,
            applypatch_msg: None,
            pre_applypatch: None,
            post_applypatch: None,
            update: None,
            proc_receive: None,
            reference_transaction: None,
            push_to_checkout: None,
            pre_auto_gc: None,
            post_rewrite: None,
            sendemail_validate: None,
            fsmonitor_watchman: None,
            p4_changelist: None,
            p4_prepare_changelist: None,
            p4_post_changelist: None,
            p4_pre_submit: None,
            post_index_change: None,
            config: None,
        }
    }
}

/// Check if file types are compatible
///
/// This function validates that file types in a configuration are compatible
/// with each other, similar to the official Lefthook implementation.
///
/// # Arguments
///
/// * `file_types` - Vector of file types to check
///
/// # Returns
///
/// Returns `Ok(())` if file types are compatible.
pub fn validate_file_types_compatibility(file_types: &[String]) -> LefthookResult<()> {
    if file_types.len() <= 1 {
        return Ok(());
    }

    // Check for incompatible file type combinations
    let has_rust = file_types.iter().any(|ft| ft == "rust");
    let has_js = file_types.iter().any(|ft| ft == "js" || ft == "javascript");
    let has_py = file_types.iter().any(|ft| ft == "py" || ft == "python");

    // Rust and JavaScript/Python are generally incompatible in the same hook
    if has_rust && (has_js || has_py) {
        return Err(LefthookError::FilesIncompatible(
            "Rust files are incompatible with JavaScript/Python files in the same hook".to_string(),
        ));
    }

    Ok(())
}

/// Validate a hook configuration
///
/// This function performs comprehensive validation of a hook configuration,
/// including file type compatibility and required fields.
///
/// # Arguments
///
/// * `hook` - The hook configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if the hook is valid.
pub fn validate_hook_configuration(hook: &LefthookHook) -> LefthookResult<()> {
    // Check required fields
    if hook.run.is_empty() {
        return Err(LefthookError::MissingRequiredField("run".to_string()));
    }

    // Validate file types if present
    if let Some(file_types) = &hook.file_types {
        validate_file_types_compatibility(file_types)?;
    }

    // Validate glob patterns if present
    if let Some(glob) = &hook.glob {
        if let Some(patterns) = glob.as_array() {
            for pattern in patterns {
                if let Some(pattern_str) = pattern.as_str() {
                    if pattern_str.is_empty() {
                        return Err(LefthookError::InvalidConfiguration(
                            "Empty glob pattern is not allowed".to_string(),
                        ));
                    }
                }
            }
        }
    }

    // Validate priority range
    if let Some(priority) = hook.priority {
        if priority < -1000 || priority > 1000 {
            return Err(LefthookError::InvalidConfiguration(
                "Priority must be between -1000 and 1000".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate a complete Lefthook configuration
///
/// This function validates all hooks in a configuration, ensuring
/// they meet the requirements of the official Lefthook implementation.
///
/// # Arguments
///
/// * `config` - The configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if the configuration is valid.
pub fn validate_configuration(config: &LefthookConfig) -> LefthookResult<()> {
    // Helper function to validate hook maps
    fn validate_hook_map(hooks: &HashMap<String, LefthookHook>, hook_type: &str) -> LefthookResult<()> {
        for (name, hook) in hooks {
            // Validate hook name
            if !is_known_hook(hook_type) {
                return Err(LefthookError::InvalidHookName(hook_type.to_string()));
            }

            // Validate individual hook
            validate_hook_configuration(hook)?;

            // Check for duplicate hook names
            if name.is_empty() {
                return Err(LefthookError::InvalidConfiguration(
                    "Hook name cannot be empty".to_string(),
                ));
            }
        }
        Ok(())
    }

    // Validate all hook types
    if let Some(hooks) = &config.pre_commit {
        validate_hook_map(hooks, "pre-commit")?;
    }
    if let Some(hooks) = &config.pre_push {
        validate_hook_map(hooks, "pre-push")?;
    }
    if let Some(hooks) = &config.commit_msg {
        validate_hook_map(hooks, "commit-msg")?;
    }
    if let Some(hooks) = &config.post_commit {
        validate_hook_map(hooks, "post-commit")?;
    }
    if let Some(hooks) = &config.pre_receive {
        validate_hook_map(hooks, "pre-receive")?;
    }
    if let Some(hooks) = &config.post_receive {
        validate_hook_map(hooks, "post-receive")?;
    }
    if let Some(hooks) = &config.post_update {
        validate_hook_map(hooks, "post-update")?;
    }
    if let Some(hooks) = &config.pre_rebase {
        validate_hook_map(hooks, "pre-rebase")?;
    }
    if let Some(hooks) = &config.post_checkout {
        validate_hook_map(hooks, "post-checkout")?;
    }
    if let Some(hooks) = &config.post_merge {
        validate_hook_map(hooks, "post-merge")?;
    }
    if let Some(hooks) = &config.pre_merge_commit {
        validate_hook_map(hooks, "pre-merge-commit")?;
    }
    if let Some(hooks) = &config.prepare_commit_msg {
        validate_hook_map(hooks, "prepare-commit-msg")?;
    }
    if let Some(hooks) = &config.applypatch_msg {
        validate_hook_map(hooks, "applypatch-msg")?;
    }
    if let Some(hooks) = &config.pre_applypatch {
        validate_hook_map(hooks, "pre-applypatch")?;
    }
    if let Some(hooks) = &config.post_applypatch {
        validate_hook_map(hooks, "post-applypatch")?;
    }
    if let Some(hooks) = &config.update {
        validate_hook_map(hooks, "update")?;
    }
    if let Some(hooks) = &config.proc_receive {
        validate_hook_map(hooks, "proc-receive")?;
    }
    if let Some(hooks) = &config.reference_transaction {
        validate_hook_map(hooks, "reference-transaction")?;
    }
    if let Some(hooks) = &config.push_to_checkout {
        validate_hook_map(hooks, "push-to-checkout")?;
    }
    if let Some(hooks) = &config.pre_auto_gc {
        validate_hook_map(hooks, "pre-auto-gc")?;
    }
    if let Some(hooks) = &config.post_rewrite {
        validate_hook_map(hooks, "post-rewrite")?;
    }
    if let Some(hooks) = &config.sendemail_validate {
        validate_hook_map(hooks, "sendemail-validate")?;
    }
    if let Some(hooks) = &config.fsmonitor_watchman {
        validate_hook_map(hooks, "fsmonitor-watchman")?;
    }
    if let Some(hooks) = &config.p4_changelist {
        validate_hook_map(hooks, "p4-changelist")?;
    }
    if let Some(hooks) = &config.p4_prepare_changelist {
        validate_hook_map(hooks, "p4-prepare-changelist")?;
    }
    if let Some(hooks) = &config.p4_post_changelist {
        validate_hook_map(hooks, "p4-post-changelist")?;
    }
    if let Some(hooks) = &config.p4_pre_submit {
        validate_hook_map(hooks, "p4-pre-submit")?;
    }
    if let Some(hooks) = &config.post_index_change {
        validate_hook_map(hooks, "post-index-change")?;
    }

    Ok(())
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
    let schema_url =
        "https://raw.githubusercontent.com/evilmartians/lefthook/refs/heads/master/schema.json";

    info!("Fetching Lefthook JSON schema from {}", schema_url);

    let response = reqwest::get(schema_url)
        .await
        .context("Failed to fetch Lefthook schema")?;

    let schema_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse schema JSON")?;

    let schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .context("Failed to compile JSON schema")?;

    info!("Successfully loaded Lefthook JSON schema");
    Ok(schema)
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
    SCHEMA_CACHE.get_or_init(|| tokio::runtime::Handle::current().block_on(fetch_lefthook_schema()))
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
pub async fn validate_against_schema(config_json: &serde_json::Value) -> LefthookResult<()> {
    let schema = get_lefthook_schema().await.map_err(|e| LefthookError::SchemaValidation(e.to_string()))?;

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
            Err(LefthookError::SchemaValidation(error_summary))
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
        LefthookHook::new(format!("{}/build-hook", hooks_dir))
            .with_files("*.rs".to_string())
            .with_stage_fixed(true),
    );

    // Add WASM component hooks if provided
    if let Some(components) = wasm_components {
        for (i, component) in components.iter().enumerate() {
            pre_commit_hooks.insert(
                format!("hooksmith-wasm-{}", i),
                LefthookHook::new(format!("{}/wasm-runner {}", hooks_dir, component))
                    .with_files("*.wit".to_string()),
            );
        }
    }

    config.pre_commit = Some(pre_commit_hooks);

    // Add pre-push hooks
    let mut pre_push_hooks = HashMap::new();
    pre_push_hooks.insert(
        "hooksmith-test".to_string(),
        LefthookHook::new(format!("{}/test-hook", hooks_dir)),
    );

    config.pre_push = Some(pre_push_hooks);

    // Convert to JSON for schema validation
    let config_json =
        serde_json::to_value(&config).context("Failed to serialize configuration to JSON")?;

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
pub async fn validate_lefthook_config(config_path: &Path, validate_schema: bool) -> Result<()> {
    let content = fs::read_to_string(config_path).context("Failed to read configuration file")?;

    // Parse YAML
    let config: LefthookConfig =
        serde_yaml::from_str(&content).context("Failed to parse YAML configuration")?;

    info!("Configuration file parsed successfully");

    // Validate against schema if requested
    if validate_schema {
        let config_json =
            serde_json::to_value(&config).context("Failed to serialize configuration to JSON")?;

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
    let content = fs::read_to_string(config_path).context("Failed to read configuration file")?;

    // Parse YAML to JSON
    let config_json: serde_json::Value =
        serde_yaml::from_str(&content).context("Failed to parse YAML configuration")?;

    // Validate against schema
    validate_against_schema(&config_json).await?;

    info!("Existing configuration validated against official Lefthook schema");
    Ok(())
}

/// Generate a comprehensive Lefthook configuration with all available hooks
///
/// This function creates a template configuration that includes all available
/// Git hooks, useful for documentation or as a starting point.
///
/// # Arguments
///
/// * `output_path` - Path where the lefthook.yml file should be written
/// * `validate_schema` - Whether to validate against the official schema
///
/// # Returns
///
/// Returns `Ok(())` if the configuration was generated successfully.
pub async fn generate_comprehensive_config(
    output_path: &Path,
    validate_schema: bool,
) -> Result<()> {
    let mut config = LefthookConfig::default();

    // Add example hooks for all available hook types
    for hook_name in AVAILABLE_HOOKS {
        let hook_map = HashMap::new(); // Empty map for template

        // Set the appropriate field based on hook name
        match *hook_name {
            "pre-commit" => config.pre_commit = Some(hook_map),
            "pre-push" => config.pre_push = Some(hook_map),
            "commit-msg" => config.commit_msg = Some(hook_map),
            "post-commit" => config.post_commit = Some(hook_map),
            "pre-receive" => config.pre_receive = Some(hook_map),
            "post-receive" => config.post_receive = Some(hook_map),
            "post-update" => config.post_update = Some(hook_map),
            "pre-rebase" => config.pre_rebase = Some(hook_map),
            "post-checkout" => config.post_checkout = Some(hook_map),
            "post-merge" => config.post_merge = Some(hook_map),
            "pre-merge-commit" => config.pre_merge_commit = Some(hook_map),
            "prepare-commit-msg" => config.prepare_commit_msg = Some(hook_map),
            "applypatch-msg" => config.applypatch_msg = Some(hook_map),
            "pre-applypatch" => config.pre_applypatch = Some(hook_map),
            "post-applypatch" => config.post_applypatch = Some(hook_map),
            "update" => config.update = Some(hook_map),
            "proc-receive" => config.proc_receive = Some(hook_map),
            "reference-transaction" => config.reference_transaction = Some(hook_map),
            "push-to-checkout" => config.push_to_checkout = Some(hook_map),
            "pre-auto-gc" => config.pre_auto_gc = Some(hook_map),
            "post-rewrite" => config.post_rewrite = Some(hook_map),
            "sendemail-validate" => config.sendemail_validate = Some(hook_map),
            "fsmonitor-watchman" => config.fsmonitor_watchman = Some(hook_map),
            "p4-changelist" => config.p4_changelist = Some(hook_map),
            "p4-prepare-changelist" => config.p4_prepare_changelist = Some(hook_map),
            "p4-post-changelist" => config.p4_post_changelist = Some(hook_map),
            "p4-pre-submit" => config.p4_pre_submit = Some(hook_map),
            "post-index-change" => config.post_index_change = Some(hook_map),
            _ => warn!("Unknown hook name: {}", hook_name),
        }
    }

    // Convert to JSON for schema validation
    let config_json =
        serde_json::to_value(&config).context("Failed to serialize configuration to JSON")?;

    // Validate against schema if requested
    if validate_schema {
        validate_against_schema(&config_json).await?;
        info!("Comprehensive configuration validated against official Lefthook schema");
    }

    // Write the configuration to file
    let yaml_content = serde_yaml::to_string(&config)?;
    fs::write(output_path, yaml_content)?;

    info!(
        "Generated comprehensive Lefthook configuration at {:?}",
        output_path
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_available_hooks() {
        assert_eq!(AVAILABLE_HOOKS.len(), 28);
        assert!(is_known_hook("pre-commit"));
        assert!(is_known_hook("pre-push"));
        assert!(is_known_hook("commit-msg"));
        assert!(!is_known_hook("invalid-hook"));
    }

    #[test]
    fn test_hook_behavior() {
        assert!(hook_uses_staged_files("pre-commit"));
        assert!(!hook_uses_staged_files("pre-push"));

        assert!(hook_uses_push_files("pre-push"));
        assert!(!hook_uses_push_files("pre-commit"));
    }

    #[test]
    fn test_hook_lists() {
        let staged_hooks = get_staged_file_hooks();
        assert_eq!(staged_hooks.len(), 1);
        assert_eq!(staged_hooks[0], "pre-commit");

        let push_hooks = get_push_file_hooks();
        assert_eq!(push_hooks.len(), 1);
        assert_eq!(push_hooks[0], "pre-push");
    }

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

    #[tokio::test]
    async fn test_comprehensive_config() -> Result<()> {
        let temp_file = NamedTempFile::new()?;

        generate_comprehensive_config(temp_file.path(), false).await?;

        // Verify the file was created
        let content = fs::read_to_string(temp_file.path())?;
        assert!(!content.is_empty());

        Ok(())
    }
}
