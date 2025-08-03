//! Configuration generation module
//!
//! This module provides Rust structs and generation logic for all configuration files
//! in the project, ensuring they are generated from code rather than manually maintained.

pub mod contract_state;
pub mod docs_manifest;
pub mod lefthook;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Main configuration generator
pub struct ConfigGenerator;

impl ConfigGenerator {
    /// Generate all configuration files from Rust structs
    pub fn generate_all() -> Result<()> {
        // Generate Lefthook configuration
        let lefthook_config = lefthook::LefthookConfig::generate();
        lefthook_config.save_to_file("lefthook.yml")?;

        // Generate contract state machine configuration
        let contract_config = contract_state::ContractStateMachine::default();
        contract_config.save_to_file("config/contract-state-machine.yml")?;

        // Generate docs manifest
        let docs_manifest = docs_manifest::DocsManifest::generate();
        docs_manifest.save_to_file("config/docs_manifest.yml")?;

        // Generate state transitions
        let state_transitions = contract_state::StateTransitions::default();
        state_transitions.save_to_file("config/state-transitions.yml")?;

        println!("✅ Generated all configuration files");
        Ok(())
    }

    /// Validate all configuration files
    pub fn validate_all() -> Result<()> {
        // Validate Lefthook configuration
        lefthook::LefthookConfig::load_from_file("lefthook.yml")?;

        // Validate contract state machine
        contract_state::ContractStateMachine::load_from_file("config/contract-state-machine.yml")?;

        // Validate docs manifest
        docs_manifest::DocsManifest::load_from_file("config/docs_manifest.yml")?;

        // Validate state transitions
        contract_state::StateTransitions::load_from_file("config/state-transitions.yml")?;

        println!("✅ All configuration files are valid");
        Ok(())
    }
}

/// Trait for configuration types that can be saved to files
pub trait ConfigFile: Serialize + for<'de> Deserialize<'de> {
    /// Save configuration to a file
    fn save_to_file(&self, path: &str) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        std::fs::write(path, yaml)?;
        println!("📝 Generated: {path}");
        Ok(())
    }

    /// Load configuration from a file
    fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}
