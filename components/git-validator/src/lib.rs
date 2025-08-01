//! Git Validator Component
//! 
//! This component provides Git validation operations for pushd-web development workflows.

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Git validator operations trait
pub trait GitValidator {
    /// Validate commit message
    fn validate_commit_message(&self, message: &str) -> Result<ValidationResult>;
    
    /// Validate branch name
    fn validate_branch_name(&self, name: &str) -> Result<ValidationResult>;
    
    /// Validate file changes
    fn validate_file_changes(&self, files: &[String]) -> Result<ValidationResult>;
    
    /// Check for sensitive data
    fn check_sensitive_data(&self, content: &str) -> Result<SensitiveDataResult>;
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Sensitive data check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveDataResult {
    pub has_sensitive_data: bool,
    pub found_patterns: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Git validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitValidationConfig {
    pub commit_message_pattern: String,
    pub branch_name_pattern: String,
    pub forbidden_patterns: Vec<String>,
    pub sensitive_patterns: Vec<String>,
}

/// Default implementation of git validator
pub struct DefaultGitValidator {
    config: GitValidationConfig,
}

impl DefaultGitValidator {
    pub fn new() -> Self {
        Self {
            config: GitValidationConfig {
                commit_message_pattern: r"^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+".to_string(),
                branch_name_pattern: r"^[a-z0-9-]+(/[a-z0-9-]+)*$".to_string(),
                forbidden_patterns: vec![
                    r"TODO".to_string(),
                    r"FIXME".to_string(),
                    r"XXX".to_string(),
                ],
                sensitive_patterns: vec![
                    r"password\s*=".to_string(),
                    r"api_key\s*=".to_string(),
                    r"secret\s*=".to_string(),
                ],
            },
        }
    }
}

impl GitValidator for DefaultGitValidator {
    fn validate_commit_message(&self, message: &str) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check if message is empty
        if message.trim().is_empty() {
            errors.push("Commit message cannot be empty".to_string());
        }
        
        // Check if message follows conventional commits pattern
        if !message.lines().next().unwrap_or("").contains(':') {
            errors.push("Commit message should follow conventional commits format".to_string());
        }
        
        // Check for forbidden patterns
        for pattern in &self.config.forbidden_patterns {
            if message.contains(pattern) {
                warnings.push(format!("Commit message contains '{}'", pattern));
            }
        }
        
        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    fn validate_branch_name(&self, name: &str) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check if name is empty
        if name.trim().is_empty() {
            errors.push("Branch name cannot be empty".to_string());
        }
        
        // Check for forbidden patterns
        for pattern in &self.config.forbidden_patterns {
            if name.contains(pattern) {
                warnings.push(format!("Branch name contains '{}'", pattern));
            }
        }
        
        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    fn validate_file_changes(&self, files: &[String]) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Check for forbidden files
        let forbidden_files = ["main", "master", "develop"];
        for file in files {
            if forbidden_files.contains(&file.as_str()) {
                errors.push(format!("File '{}' is not allowed", file));
            }
        }
        
        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }
    
    fn check_sensitive_data(&self, content: &str) -> Result<SensitiveDataResult> {
        let mut found_patterns = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check for sensitive patterns
        for pattern in &self.config.sensitive_patterns {
            if content.contains(pattern) {
                found_patterns.push(pattern.clone());
                recommendations.push("Consider using environment variables or secure storage".to_string());
            }
        }
        
        Ok(SensitiveDataResult {
            has_sensitive_data: !found_patterns.is_empty(),
            found_patterns,
            recommendations,
        })
    }
}

// Export the main operations
pub use DefaultGitValidator as GitValidatorComponent; 
