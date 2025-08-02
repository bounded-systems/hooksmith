//! CLI Core Component
//! 
//! This component provides core CLI operations for hooksmith development workflows.
//! It's designed to be used as a Wasm component for modular CLI functionality.

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Core CLI operations trait
pub trait CliOperations {
    /// Execute a CLI command
    fn execute(&self, command: &str, args: &[String]) -> Result<CliResult>;
    
    /// Validate configuration
    fn validate_config(&self, config: &CliConfig) -> Result<ValidationResult>;
    
    /// Get component information
    fn info(&self) -> ComponentInfo;
}

/// CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub aws_profile: String,
    pub target_env: String,
    pub project_root: String,
    pub dry_run: bool,
}

/// CLI operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
    pub data: Option<String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Component information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

/// Default implementation of CLI operations
pub struct DefaultCliOperations;

impl CliOperations for DefaultCliOperations {
    fn execute(&self, command: &str, args: &[String]) -> Result<CliResult> {
        // Placeholder implementation
        Ok(CliResult {
            success: true,
            message: format!("Executed {} with args {:?}", command, args),
            error: None,
            data: None,
        })
    }
    
    fn validate_config(&self, config: &CliConfig) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let warnings = Vec::new();
        
        if config.aws_profile.is_empty() {
            errors.push("AWS profile cannot be empty".to_string());
        }
        
        if config.target_env.is_empty() {
            errors.push("Target environment cannot be empty".to_string());
        }
        
        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    fn info(&self) -> ComponentInfo {
        ComponentInfo {
            name: "cli-core".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Core CLI operations component".to_string(),
            capabilities: vec![
                "command_execution".to_string(),
                "config_validation".to_string(),
                "result_handling".to_string(),
            ],
        }
    }
}

// Export the main operations
pub use DefaultCliOperations as CliCore; 
