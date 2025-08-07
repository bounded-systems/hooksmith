//! Git proxy validation module
//!
//! This module handles validation rules and checks for Git operations,
//! including commit message validation, file size checks, and security validations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Commit, ObjectType, Repository, Tree};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::{
    ClientInfo, GitProtocol, GitProxyConfig, ValidationOperationType, ValidationRequest,
    ValidationResult,
};

/// Validation rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    /// Commit message pattern validation
    CommitMessagePattern(String),
    /// File size limit validation
    FileSizeLimit(u64),
    /// File type validation
    FileTypePattern(String),
    /// Branch name validation
    BranchNamePattern(String),
    /// Force push validation
    ForcePushAllowed(bool),
    /// Protected branch validation
    ProtectedBranch(String),
    /// Custom validation rule
    Custom(String, String),
}

/// Validation result with details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDetails {
    /// Rule that was checked
    pub rule: String,
    /// Whether the rule passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Warning message if applicable
    pub warning: Option<String>,
}

/// Validation engine
pub struct ValidationEngine {
    config: GitProxyConfig,
    rules: Vec<ValidationRule>,
    repository: Option<Repository>,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new(config: GitProxyConfig) -> Self {
        let mut rules = Vec::new();

        // Add default rules from config
        for pattern in &config.validation.required_patterns {
            rules.push(ValidationRule::CommitMessagePattern(pattern.clone()));
        }

        if let Some(max_size) = config.validation.max_file_size {
            rules.push(ValidationRule::FileSizeLimit(max_size));
        }

        for pattern in &config.validation.blocked_patterns {
            rules.push(ValidationRule::FileTypePattern(pattern.clone()));
        }

        // Add protected branch rules
        rules.push(ValidationRule::ProtectedBranch("main".to_string()));
        rules.push(ValidationRule::ProtectedBranch("master".to_string()));

        // Add force push rules
        rules.push(ValidationRule::ForcePushAllowed(false));

        Self {
            config,
            rules,
            repository: None,
        }
    }

    /// Set repository for validation
    pub fn set_repository(&mut self, repository: Repository) {
        self.repository = Some(repository);
    }

    /// Validate an operation
    pub async fn validate_operation(&self, request: ValidationRequest) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut details = Vec::new();

        // Validate based on operation type
        match request.operation_type {
            ValidationOperationType::PrePush => {
                self.validate_pre_push(&request, &mut errors, &mut warnings, &mut details)
                    .await?;
            }
            ValidationOperationType::PreReceive => {
                self.validate_pre_receive(&request, &mut errors, &mut warnings, &mut details)
                    .await?;
            }
            ValidationOperationType::PostReceive => {
                self.validate_post_receive(&request, &mut errors, &mut warnings, &mut details)
                    .await?;
            }
            ValidationOperationType::PreCommit => {
                self.validate_pre_commit(&request, &mut errors, &mut warnings, &mut details)
                    .await?;
            }
            ValidationOperationType::PostCommit => {
                self.validate_post_commit(&request, &mut errors, &mut warnings, &mut details)
                    .await?;
            }
        }

        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(ValidationResult {
            request_id: request.request_id,
            valid: errors.is_empty(),
            errors,
            warnings,
            duration_ms,
            timestamp: Utc::now(),
        })
    }

    /// Validate pre-push operation
    async fn validate_pre_push(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        // Validate commit messages
        if self.config.validation.enable_commit_validation {
            self.validate_commit_messages(request, errors, warnings, details)
                .await?;
        }

        // Validate file sizes
        if self.config.validation.enable_file_size_validation {
            self.validate_file_sizes(request, errors, warnings, details)
                .await?;
        }

        // Validate blocked file patterns
        self.validate_blocked_files(request, errors, warnings, details)
            .await?;

        // Validate protected branches
        self.validate_protected_branches(request, errors, warnings, details)
            .await?;

        // Validate force pushes
        self.validate_force_pushes(request, errors, warnings, details)
            .await?;

        Ok(())
    }

    /// Validate pre-receive operation
    async fn validate_pre_receive(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        // Similar to pre-push but for server-side validation
        self.validate_pre_push(request, errors, warnings, details)
            .await?;
        Ok(())
    }

    /// Validate post-receive operation
    async fn validate_post_receive(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        // Post-receive validations are typically for logging and auditing
        info!("Post-receive validation for request {}", request.request_id);
        Ok(())
    }

    /// Validate pre-commit operation
    async fn validate_pre_commit(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        // Pre-commit validations are typically for local development
        info!("Pre-commit validation for request {}", request.request_id);
        Ok(())
    }

    /// Validate post-commit operation
    async fn validate_post_commit(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        // Post-commit validations are typically for logging and auditing
        info!("Post-commit validation for request {}", request.request_id);
        Ok(())
    }

    /// Validate commit messages
    async fn validate_commit_messages(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        if let Some(repo) = &self.repository {
            for commit_hash in &request.commit_hashes {
                if let Ok(oid) = git2::Oid::from_str(commit_hash) {
                    if let Ok(commit) = repo.find_commit(oid) {
                        let message = commit.message().unwrap_or("");

                        // Check against required patterns
                        let mut pattern_matched = false;
                        for pattern in &self.config.validation.required_patterns {
                            if message.starts_with(pattern) {
                                pattern_matched = true;
                                break;
                            }
                        }

                        if !pattern_matched {
                            let error_msg = format!(
                                "Commit message '{}' does not match required patterns: {:?}",
                                message, self.config.validation.required_patterns
                            );
                            errors.push(error_msg.clone());

                            details.push(ValidationDetails {
                                rule: "CommitMessagePattern".to_string(),
                                passed: false,
                                error: Some(error_msg),
                                warning: None,
                            });
                        } else {
                            details.push(ValidationDetails {
                                rule: "CommitMessagePattern".to_string(),
                                passed: true,
                                error: None,
                                warning: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate file sizes
    async fn validate_file_sizes(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        if let Some(max_size) = self.config.validation.max_file_size {
            if let Some(repo) = &self.repository {
                for commit_hash in &request.commit_hashes {
                    if let Ok(oid) = git2::Oid::from_str(commit_hash) {
                        if let Ok(commit) = repo.find_commit(oid) {
                            if let Ok(tree) = commit.tree() {
                                self.check_tree_file_sizes(
                                    &tree, max_size, errors, warnings, details,
                                )
                                .await?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check file sizes in a tree
    async fn check_tree_file_sizes(
        &self,
        tree: &git2::Tree,
        max_size: u64,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        for entry in tree.iter() {
            if let Ok(object) = entry.to_object(self.repository.as_ref().unwrap()) {
                match object.kind() {
                    Some(ObjectType::Blob) => {
                        if let Ok(blob) = object.as_blob() {
                            let size = blob.size() as u64;
                            if size > max_size {
                                let error_msg = format!(
                                    "File '{}' is too large: {} bytes (max: {} bytes)",
                                    entry.name().unwrap_or("unknown"),
                                    size,
                                    max_size
                                );
                                errors.push(error_msg.clone());

                                details.push(ValidationDetails {
                                    rule: "FileSizeLimit".to_string(),
                                    passed: false,
                                    error: Some(error_msg),
                                    warning: None,
                                });
                            } else {
                                details.push(ValidationDetails {
                                    rule: "FileSizeLimit".to_string(),
                                    passed: true,
                                    error: None,
                                    warning: None,
                                });
                            }
                        }
                    }
                    Some(ObjectType::Tree) => {
                        if let Ok(subtree) = object.as_tree() {
                            self.check_tree_file_sizes(
                                &subtree, max_size, errors, warnings, details,
                            )
                            .await?;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Validate blocked file patterns
    async fn validate_blocked_files(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        if let Some(repo) = &self.repository {
            for commit_hash in &request.commit_hashes {
                if let Ok(oid) = git2::Oid::from_str(commit_hash) {
                    if let Ok(commit) = repo.find_commit(oid) {
                        if let Ok(tree) = commit.tree() {
                            self.check_tree_blocked_files(&tree, errors, warnings, details)
                                .await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check for blocked files in a tree
    async fn check_tree_blocked_files(
        &self,
        tree: &git2::Tree,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        for entry in tree.iter() {
            if let Ok(object) = entry.to_object(self.repository.as_ref().unwrap()) {
                match object.kind() {
                    Some(ObjectType::Blob) => {
                        let filename = entry.name().unwrap_or("unknown");

                        // Check against blocked patterns
                        for pattern in &self.config.validation.blocked_patterns {
                            if self.matches_pattern(filename, pattern) {
                                let error_msg = format!(
                                    "File '{}' matches blocked pattern '{}'",
                                    filename, pattern
                                );
                                errors.push(error_msg.clone());

                                details.push(ValidationDetails {
                                    rule: "FileTypePattern".to_string(),
                                    passed: false,
                                    error: Some(error_msg),
                                    warning: None,
                                });
                            }
                        }
                    }
                    Some(ObjectType::Tree) => {
                        if let Ok(subtree) = object.as_tree() {
                            self.check_tree_blocked_files(&subtree, errors, warnings, details)
                                .await?;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Validate protected branches
    async fn validate_protected_branches(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        let protected_branches = vec!["main", "master", "develop", "production"];

        for ref_name in &request.refs {
            for protected in &protected_branches {
                if ref_name.contains(protected) {
                    let error_msg = format!(
                        "Direct push to protected branch '{}' is not allowed",
                        ref_name
                    );
                    errors.push(error_msg.clone());

                    details.push(ValidationDetails {
                        rule: "ProtectedBranch".to_string(),
                        passed: false,
                        error: Some(error_msg),
                        warning: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate force pushes
    async fn validate_force_pushes(
        &self,
        request: &ValidationRequest,
        errors: &mut Vec<String>,
        warnings: &mut Vec<String>,
        details: &mut Vec<ValidationDetails>,
    ) -> Result<()> {
        // This would typically check if the push is a force push
        // For now, we'll add a warning for any push to protected branches
        let protected_branches = vec!["main", "master", "develop", "production"];

        for ref_name in &request.refs {
            for protected in &protected_branches {
                if ref_name.contains(protected) {
                    let warning_msg = format!("Push to protected branch '{}' detected", ref_name);
                    warnings.push(warning_msg.clone());

                    details.push(ValidationDetails {
                        rule: "ForcePushAllowed".to_string(),
                        passed: true,
                        error: None,
                        warning: Some(warning_msg),
                    });
                }
            }
        }

        Ok(())
    }

    /// Check if a filename matches a pattern
    fn matches_pattern(&self, filename: &str, pattern: &str) -> bool {
        // Simple glob pattern matching
        if pattern.starts_with("*.") {
            let extension = pattern[1..].to_string();
            filename.ends_with(&extension)
        } else if pattern.starts_with("*") {
            filename.contains(&pattern[1..])
        } else {
            filename == pattern
        }
    }

    /// Add a custom validation rule
    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.rules.push(rule);
    }

    /// Remove a validation rule
    pub fn remove_rule(&mut self, rule_type: &str) {
        self.rules.retain(|r| match r {
            ValidationRule::CommitMessagePattern(_) => rule_type != "CommitMessagePattern",
            ValidationRule::FileSizeLimit(_) => rule_type != "FileSizeLimit",
            ValidationRule::FileTypePattern(_) => rule_type != "FileTypePattern",
            ValidationRule::BranchNamePattern(_) => rule_type != "BranchNamePattern",
            ValidationRule::ForcePushAllowed(_) => rule_type != "ForcePushAllowed",
            ValidationRule::ProtectedBranch(_) => rule_type != "ProtectedBranch",
            ValidationRule::Custom(name, _) => name != rule_type,
        });
    }

    /// Get all validation rules
    pub fn get_rules(&self) -> &[ValidationRule] {
        &self.rules
    }
}

/// Validation rule builder
pub struct ValidationRuleBuilder {
    rules: Vec<ValidationRule>,
}

impl ValidationRuleBuilder {
    /// Create a new validation rule builder
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add commit message pattern rule
    pub fn commit_message_pattern(mut self, pattern: String) -> Self {
        self.rules
            .push(ValidationRule::CommitMessagePattern(pattern));
        self
    }

    /// Add file size limit rule
    pub fn file_size_limit(mut self, max_size: u64) -> Self {
        self.rules.push(ValidationRule::FileSizeLimit(max_size));
        self
    }

    /// Add file type pattern rule
    pub fn file_type_pattern(mut self, pattern: String) -> Self {
        self.rules.push(ValidationRule::FileTypePattern(pattern));
        self
    }

    /// Add branch name pattern rule
    pub fn branch_name_pattern(mut self, pattern: String) -> Self {
        self.rules.push(ValidationRule::BranchNamePattern(pattern));
        self
    }

    /// Add force push rule
    pub fn force_push_allowed(mut self, allowed: bool) -> Self {
        self.rules.push(ValidationRule::ForcePushAllowed(allowed));
        self
    }

    /// Add protected branch rule
    pub fn protected_branch(mut self, branch: String) -> Self {
        self.rules.push(ValidationRule::ProtectedBranch(branch));
        self
    }

    /// Add custom rule
    pub fn custom(mut self, name: String, value: String) -> Self {
        self.rules.push(ValidationRule::Custom(name, value));
        self
    }

    /// Build the validation engine
    pub fn build(self, config: GitProxyConfig) -> ValidationEngine {
        let mut engine = ValidationEngine::new(config);

        for rule in self.rules {
            engine.add_rule(rule);
        }

        engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_validation_rule_builder() {
        let engine = ValidationRuleBuilder::new()
            .commit_message_pattern("feat:".to_string())
            .file_size_limit(1024 * 1024)
            .protected_branch("main".to_string())
            .build(GitProxyConfig::default());

        assert!(!engine.get_rules().is_empty());
    }

    #[test]
    fn test_pattern_matching() {
        let engine = ValidationEngine::new(GitProxyConfig::default());

        assert!(engine.matches_pattern("test.key", "*.key"));
        assert!(engine.matches_pattern("id_rsa", "id_rsa"));
        assert!(!engine.matches_pattern("test.txt", "*.key"));
    }

    #[tokio::test]
    async fn test_validation_engine_creation() {
        let config = GitProxyConfig::default();
        let engine = ValidationEngine::new(config);

        assert!(!engine.get_rules().is_empty());
        assert!(engine.repository.is_none());
    }
}
