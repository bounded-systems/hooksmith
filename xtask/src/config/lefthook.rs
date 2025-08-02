//! Lefthook configuration generation
//!
//! This module provides Rust structs for generating lefthook.yml configuration
//! from code rather than manually maintaining the YAML file.

use crate::config::ConfigFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lefthook hook command configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookCommand {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub glob: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_fixed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel: Option<bool>,
}

impl HookCommand {
    /// Create a simple command
    pub fn new(run: &str) -> Self {
        Self {
            run: Some(run.to_string()),
            glob: None,
            exclude: None,
            stage_fixed: None,
            parallel: None,
        }
    }

    /// Create a command with glob pattern
    pub fn with_glob(run: &str, glob: &str) -> Self {
        Self {
            run: Some(run.to_string()),
            glob: Some(glob.to_string()),
            exclude: None,
            stage_fixed: None,
            parallel: None,
        }
    }
}

/// Main Lefthook configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct LefthookConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_commit: Option<HashMap<String, HookCommand>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_push: Option<HashMap<String, HookCommand>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_msg: Option<HashMap<String, HookCommand>>,
}

impl ConfigFile for LefthookConfig {}

impl LefthookConfig {
    /// Generate default Lefthook configuration
    pub fn generate() -> Self {
        Self {
            pre_commit: Some(Self::default_pre_commit_hooks()),
            pre_push: Some(Self::default_pre_push_hooks()),
            commit_msg: Some(Self::default_commit_msg_hooks()),
        }
    }

    /// Default pre-commit hooks
    fn default_pre_commit_hooks() -> HashMap<String, HookCommand> {
        let mut hooks = HashMap::new();

        // Validate generated files
        hooks.insert(
            "validate-generated".to_string(),
            HookCommand::new("cargo run -p xtask -- validate-generated --staged-only --strict"),
        );

        // Contract validation with generated file validation
        hooks.insert(
            "contract-validate".to_string(),
            HookCommand::new("cargo run -p xtask -- contract-validate --validate-generated"),
        );

        // Validate file extensions
        hooks.insert(
            "validate-extensions".to_string(),
            HookCommand::new("cargo run -p xtask -- validate-extensions"),
        );

        // Check generated headers
        hooks.insert(
            "check-generated".to_string(),
            HookCommand::new("cargo run -p xtask -- validate-headers"),
        );

        hooks
    }

    /// Default pre-push hooks
    fn default_pre_push_hooks() -> HashMap<String, HookCommand> {
        let mut hooks = HashMap::new();

        // Comprehensive validation
        hooks.insert(
            "validate-all-generated".to_string(),
            HookCommand::new("cargo run -p xtask -- validate-generated --strict"),
        );

        // Comprehensive contract validation
        hooks.insert(
            "comprehensive-validate".to_string(),
            HookCommand::new(
                "cargo run -p xtask -- contract-validate --validate-generated --comprehensive",
            ),
        );

        // Validate all headers
        hooks.insert(
            "validate-headers".to_string(),
            HookCommand::new("cargo run -p xtask -- validate-headers --all"),
        );

        hooks
    }

    /// Default commit-msg hooks
    fn default_commit_msg_hooks() -> HashMap<String, HookCommand> {
        let mut hooks = HashMap::new();

        // Validate commit message format
        hooks.insert(
            "validate-commit-msg".to_string(),
            HookCommand::new("cargo run -p xtask -- validate-commit-msg"),
        );

        hooks
    }

    /// Add a custom pre-commit hook
    pub fn add_pre_commit_hook(&mut self, name: &str, command: HookCommand) {
        if let Some(ref mut pre_commit) = self.pre_commit {
            pre_commit.insert(name.to_string(), command);
        } else {
            let mut hooks = HashMap::new();
            hooks.insert(name.to_string(), command);
            self.pre_commit = Some(hooks);
        }
    }

    /// Add a custom pre-push hook
    pub fn add_pre_push_hook(&mut self, name: &str, command: HookCommand) {
        if let Some(ref mut pre_push) = self.pre_push {
            pre_push.insert(name.to_string(), command);
        } else {
            let mut hooks = HashMap::new();
            hooks.insert(name.to_string(), command);
            self.pre_push = Some(hooks);
        }
    }
}
