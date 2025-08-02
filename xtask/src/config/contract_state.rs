//! Contract state machine configuration generation
//!
//! This module provides Rust structs for generating contract state machine
//! configuration files from code rather than manually maintaining YAML files.

use crate::config::ConfigFile;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Contract state definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractState {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_state: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

impl ContractState {
    /// Create a new contract state
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            initial: None,
            final_state: None,
            metadata: None,
        }
    }

    /// Mark as initial state
    pub fn initial(mut self) -> Self {
        self.initial = Some(true);
        self
    }

    /// Mark as final state
    pub fn final_state(mut self) -> Self {
        self.final_state = Some(true);
        self
    }
}

/// State transition definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub trigger: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

impl StateTransition {
    /// Create a new state transition
    pub fn new(from: &str, to: &str, trigger: &str) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            trigger: trigger.to_string(),
            condition: None,
            metadata: None,
        }
    }

    /// Add a condition to the transition
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        self
    }
}

/// Validation rule definition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub rule_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_yaml::Value>,
}

impl ValidationRule {
    /// Create a new validation rule
    pub fn new(name: &str, description: &str, rule_type: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            rule_type: rule_type.to_string(),
            parameters: None,
        }
    }
}

/// Main contract state machine configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractStateMachine {
    pub states: Vec<ContractState>,
    pub transitions: Vec<StateTransition>,
    pub validation_rules: Vec<ValidationRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

impl ConfigFile for ContractStateMachine {}

impl Default for ContractStateMachine {
    fn default() -> Self {
        Self {
            states: vec![
                ContractState::new("UNTRACKED", "File is not tracked by Git").initial(),
                ContractState::new("MODIFIED", "File has been modified").final_state(),
                ContractState::new("STAGED", "File is staged for commit").final_state(),
                ContractState::new("COMMITTED", "File is committed to repository").final_state(),
                ContractState::new("IGNORED", "File is ignored by Git"),
            ],
            transitions: vec![
                StateTransition::new("UNTRACKED", "MODIFIED", "file_created"),
                StateTransition::new("MODIFIED", "STAGED", "git_add"),
                StateTransition::new("STAGED", "COMMITTED", "git_commit"),
                StateTransition::new("COMMITTED", "MODIFIED", "file_modified"),
                StateTransition::new("MODIFIED", "IGNORED", "git_ignore"),
            ],
            validation_rules: vec![
                ValidationRule::new(
                    "file_extension_check",
                    "Validate file extensions are allowed",
                    "extension_validation",
                ),
                ValidationRule::new(
                    "generated_file_check",
                    "Ensure generated files are not manually modified",
                    "generated_file_validation",
                ),
            ],
            metadata: None,
        }
    }
}

/// State transitions configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct StateTransitions {
    pub transitions: Vec<StateTransition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_yaml::Value>,
}

impl ConfigFile for StateTransitions {}

impl Default for StateTransitions {
    fn default() -> Self {
        Self {
            transitions: vec![
                StateTransition::new("UNTRACKED", "MODIFIED", "file_created")
                    .with_condition("file_exists"),
                StateTransition::new("MODIFIED", "STAGED", "git_add")
                    .with_condition("file_not_ignored"),
                StateTransition::new("STAGED", "COMMITTED", "git_commit")
                    .with_condition("staged_files_exist"),
                StateTransition::new("COMMITTED", "MODIFIED", "file_modified")
                    .with_condition("file_changed"),
                StateTransition::new("MODIFIED", "IGNORED", "git_ignore")
                    .with_condition("in_gitignore"),
            ],
            metadata: None,
        }
    }
}
