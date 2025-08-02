//! Hierarchical Contract Validation Module
//!
//! This module implements the bottom-up validation pipeline for nested contract scopes:
//! Char → Line → Chunk → File → Directory → Repository
//!
//! Each scope depends on the validation result of its child scopes, ensuring
//! that changes at any level properly cascade through the validation hierarchy.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Validation scope levels in hierarchical order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ValidationScope {
    /// Character-level validation scope
    Char,
    /// Line-level validation scope
    Line,
    /// Chunk-level validation scope (function/block)
    Chunk,
    /// File-level validation scope
    File,
    /// Directory-level validation scope
    Directory,
    /// Repository-level validation scope
    Repository,
}

impl ValidationScope {
    /// Get the parent scope for this scope
    pub fn parent(&self) -> Option<ValidationScope> {
        match self {
            ValidationScope::Char => None,
            ValidationScope::Line => Some(ValidationScope::Char),
            ValidationScope::Chunk => Some(ValidationScope::Line),
            ValidationScope::File => Some(ValidationScope::Chunk),
            ValidationScope::Directory => Some(ValidationScope::File),
            ValidationScope::Repository => Some(ValidationScope::Directory),
        }
    }

    /// Get the child scope for this scope
    pub fn child(&self) -> Option<ValidationScope> {
        match self {
            ValidationScope::Char => Some(ValidationScope::Line),
            ValidationScope::Line => Some(ValidationScope::Chunk),
            ValidationScope::Chunk => Some(ValidationScope::File),
            ValidationScope::File => Some(ValidationScope::Directory),
            ValidationScope::Directory => Some(ValidationScope::Repository),
            ValidationScope::Repository => None,
        }
    }

    /// Get all scopes in hierarchical order (bottom-up)
    pub fn hierarchy() -> Vec<ValidationScope> {
        vec![
            ValidationScope::Char,
            ValidationScope::Line,
            ValidationScope::Chunk,
            ValidationScope::File,
            ValidationScope::Directory,
            ValidationScope::Repository,
        ]
    }

    /// Get the contract type for this scope
    pub fn contract_type(&self) -> &'static str {
        match self {
            ValidationScope::Char => "char_contract",
            ValidationScope::Line => "line_contract",
            ValidationScope::Chunk => "blob_contract",
            ValidationScope::File => "tree_contract",
            ValidationScope::Directory => "tree_filename_chars_contract",
            ValidationScope::Repository => "repo_contract",
        }
    }
}

/// Content range for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRange {
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed)
    pub end_line: usize,
    /// Starting character position (0-indexed)
    pub start_char: Option<usize>,
    /// Ending character position (0-indexed)
    pub end_char: Option<usize>,
}

/// Git Notes validation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationNote {
    /// The validation scope level
    pub scope: String,
    /// The file path relative to repository root
    pub file: String,
    /// The range of content being validated
    pub range: Option<ContentRange>,
    /// SHA256 hash of the validated content
    pub hash: String,
    /// The parent scope that contains this scope
    pub parent_scope: Option<String>,
    /// SHA256 hash of the parent scope content
    pub parent_hash: Option<String>,
    /// Array of child scope hashes that contribute to this scope
    pub child_scopes: Vec<ChildScope>,
    /// Whether the content passed validation
    pub validated: bool,
    /// Array of validation errors if validation failed
    pub validation_errors: Vec<ValidationError>,
    /// Type of contract being validated
    pub contract_type: String,
    /// Name and version of the validation tool
    pub tool: String,
    /// ISO 8601 timestamp of when validation was performed
    pub timestamp: String,
    /// Git commit hash where this validation was performed
    pub commit_hash: Option<String>,
    /// Time taken for validation in milliseconds
    pub validation_duration_ms: u64,
    /// Additional metadata about the validation
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Child scope reference in validation hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildScope {
    /// The scope type
    pub scope: String,
    /// SHA256 hash of the child scope content
    pub hash: String,
}

/// Validation error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error message
    pub message: String,
    /// Error severity level
    pub severity: String,
    /// Line number where error occurred
    pub line: Option<usize>,
    /// Character position where error occurred
    pub char: Option<usize>,
}

/// Change detection result
#[derive(Debug, Clone)]
pub struct ChangeScope {
    /// The file that was changed
    pub file: PathBuf,
    /// The scope level of the change
    pub scope: ValidationScope,
    /// The range of content that was changed
    pub range: Option<ContentRange>,
    /// The content of the changed file
    pub content: String,
}

/// Hierarchical validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The scope that was validated
    pub scope: ValidationScope,
    /// The file that was validated
    pub file: PathBuf,
    /// The range of content that was validated
    pub range: Option<ContentRange>,
    /// SHA256 hash of the validated content
    pub hash: String,
    /// Whether the validation passed
    pub validated: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// Time taken for validation in milliseconds
    pub duration_ms: u64,
    /// Results from child scope validations
    pub child_results: Vec<ValidationResult>,
}

/// Hierarchical Contract Validator
pub struct HierarchicalValidator {
    repo_path: PathBuf,
    tool_version: String,
}

impl HierarchicalValidator {
    /// Create a new hierarchical validator
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path,
            tool_version: "xtask-contract-validate 0.1.0".to_string(),
        }
    }

    /// Detect changes in the repository and determine the smallest affected scope
    pub async fn detect_changes(&self, commit_range: Option<&str>) -> Result<Vec<ChangeScope>> {
        let mut changes = Vec::new();

        // Get the diff range
        let diff_range = commit_range.unwrap_or("HEAD~1..HEAD");

        // Get file changes
        let output = Command::new("git")
            .args(["diff", "--name-only", diff_range])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get git diff")?;

        if !output.status.success() {
            anyhow::bail!(
                "Git diff failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let files: Vec<PathBuf> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| self.repo_path.join(line))
            .collect();

        // For each changed file, detect the smallest scope of changes
        for file in files {
            if let Some(change_scopes) = self.detect_file_changes(&file, diff_range).await? {
                changes.extend(change_scopes);
            }
        }

        Ok(changes)
    }

    /// Detect changes in a specific file
    async fn detect_file_changes(
        &self,
        file: &Path,
        diff_range: &str,
    ) -> Result<Option<Vec<ChangeScope>>> {
        if !file.exists() {
            return Ok(None);
        }

        // Get word-level diff to detect character changes
        let word_diff = Command::new("git")
            .args([
                "diff",
                "--word-diff=porcelain",
                diff_range,
                "--",
                file.to_str().unwrap(),
            ])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get word diff")?;

        if !word_diff.status.success() {
            return Ok(None);
        }

        let mut changes = Vec::new();
        let content = fs::read_to_string(file).await?;

        // Parse word diff to find character-level changes
        let word_diff_str = String::from_utf8_lossy(&word_diff.stdout);
        for line in word_diff_str.lines() {
            if line.starts_with("@@") {
                // Parse hunk header
                if let Some(range) = self.parse_hunk_header(line) {
                    // For now, we'll treat any change as affecting the entire line
                    // In a more sophisticated implementation, we'd parse the actual word changes
                    changes.push(ChangeScope {
                        file: file.to_path_buf(),
                        scope: ValidationScope::Line,
                        range: Some(range),
                        content: content.clone(),
                    });
                }
            }
        }

        // If no specific changes detected, treat as file-level change
        if changes.is_empty() {
            changes.push(ChangeScope {
                file: file.to_path_buf(),
                scope: ValidationScope::File,
                range: None,
                content,
            });
        }

        Ok(Some(changes))
    }

    /// Parse git hunk header to extract line range
    fn parse_hunk_header(&self, header: &str) -> Option<ContentRange> {
        // Example: @@ -42,1 +42,1 @@
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 3 {
            return None;
        }

        let new_range = parts[2];
        if !new_range.starts_with('+') {
            return None;
        }

        let range_str = &new_range[1..];
        if let Some(comma_pos) = range_str.find(',') {
            let start_line: usize = range_str[..comma_pos].parse().ok()?;
            let count: usize = range_str[comma_pos + 1..].parse().ok()?;
            Some(ContentRange {
                start_line,
                end_line: start_line + count - 1,
                start_char: None,
                end_char: None,
            })
        } else {
            let start_line: usize = range_str.parse().ok()?;
            Some(ContentRange {
                start_line,
                end_line: start_line,
                start_char: None,
                end_char: None,
            })
        }
    }

    /// Run hierarchical validation starting from the smallest affected scope
    pub async fn validate_hierarchically(
        &self,
        changes: Vec<ChangeScope>,
    ) -> Result<Vec<ValidationResult>> {
        let mut results = Vec::new();
        let mut validated_scopes = HashSet::new();

        for change in changes {
            let mut current_scope = change.scope;
            let mut current_result = None;

            // Validate bottom-up through the hierarchy
            while let Some(scope) = self.get_next_scope(current_scope) {
                let scope_key = format!(
                    "{}:{}:{}",
                    change.file.display(),
                    scope as u8,
                    self.get_content_hash(&change.content, &change.range)
                );

                if validated_scopes.contains(&scope_key) {
                    break;
                }

                let start_time = std::time::Instant::now();
                let validation_result = self
                    .validate_scope(scope, &change.file, &change.range, &change.content)
                    .await?;
                let duration = start_time.elapsed();

                let result = ValidationResult {
                    scope,
                    file: change.file.clone(),
                    range: change.range.clone(),
                    hash: self.get_content_hash(&change.content, &change.range),
                    validated: validation_result.validated,
                    errors: validation_result.validation_errors.clone(),
                    duration_ms: duration.as_millis() as u64,
                    child_results: current_result.map(|r| vec![r]).unwrap_or_default(),
                };

                // Store validation note in Git
                self.store_validation_note(&result, &validation_result)
                    .await?;

                validated_scopes.insert(scope_key);
                current_result = Some(result.clone());

                // If validation failed, stop the hierarchy
                if !result.validated {
                    break;
                }

                current_scope = scope;
            }

            if let Some(result) = current_result {
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Get the next scope in the hierarchy
    fn get_next_scope(&self, current: ValidationScope) -> Option<ValidationScope> {
        match current {
            ValidationScope::Char => Some(ValidationScope::Line),
            ValidationScope::Line => Some(ValidationScope::Chunk),
            ValidationScope::Chunk => Some(ValidationScope::File),
            ValidationScope::File => Some(ValidationScope::Directory),
            ValidationScope::Directory => Some(ValidationScope::Repository),
            ValidationScope::Repository => None,
        }
    }

    /// Validate a specific scope
    async fn validate_scope(
        &self,
        scope: ValidationScope,
        file: &Path,
        range: &Option<ContentRange>,
        content: &str,
    ) -> Result<ValidationNote> {
        let start_time = std::time::Instant::now();

        // Extract content for the specific scope
        let scope_content = self.extract_scope_content(content, range, scope).await?;
        let hash = self.get_content_hash(&scope_content, range);

        // Run the appropriate contract validator
        let (validated, errors) = self.run_contract_validator(scope, &scope_content).await?;

        let duration = start_time.elapsed();

        // Get commit hash
        let commit_hash = self.get_current_commit_hash().await?;

        Ok(ValidationNote {
            scope: format!("{:?}", scope).to_lowercase(),
            file: file.to_string_lossy().to_string(),
            range: range.clone(),
            hash,
            parent_scope: scope.parent().map(|s| format!("{:?}", s).to_lowercase()),
            parent_hash: None,        // Will be set by caller
            child_scopes: Vec::new(), // Will be set by caller
            validated,
            validation_errors: errors,
            contract_type: scope.contract_type().to_string(),
            tool: self.tool_version.clone(),
            timestamp: Utc::now().to_rfc3339(),
            commit_hash: Some(commit_hash),
            validation_duration_ms: duration.as_millis() as u64,
            metadata: HashMap::new(),
        })
    }

    /// Extract content for a specific scope
    async fn extract_scope_content(
        &self,
        content: &str,
        range: &Option<ContentRange>,
        scope: ValidationScope,
    ) -> Result<String> {
        match scope {
            ValidationScope::Char => {
                if let Some(range) = range {
                    if let (Some(start), Some(end)) = (range.start_char, range.end_char) {
                        let lines: Vec<&str> = content.lines().collect();
                        if range.start_line <= lines.len() {
                            let line = lines[range.start_line - 1];
                            if start < line.len() && end <= line.len() {
                                return Ok(line[start..end].to_string());
                            }
                        }
                    }
                }
                Ok(content.to_string())
            }
            ValidationScope::Line => {
                if let Some(range) = range {
                    let lines: Vec<&str> = content.lines().collect();
                    if range.start_line <= lines.len() && range.end_line <= lines.len() {
                        let start_idx = range.start_line - 1;
                        let end_idx = range.end_line;
                        return Ok(lines[start_idx..end_idx].join("\n"));
                    }
                }
                Ok(content.to_string())
            }
            ValidationScope::Chunk => {
                // For chunk scope, we'll use the entire content for now
                // In a more sophisticated implementation, we'd parse function/block boundaries
                Ok(content.to_string())
            }
            ValidationScope::File => Ok(content.to_string()),
            ValidationScope::Directory => {
                // For directory scope, we'd aggregate all files in the directory
                // For now, we'll use the file content
                Ok(content.to_string())
            }
            ValidationScope::Repository => {
                // For repository scope, we'd aggregate all files
                // For now, we'll use the file content
                Ok(content.to_string())
            }
        }
    }

    /// Run the appropriate contract validator for a scope
    async fn run_contract_validator(
        &self,
        scope: ValidationScope,
        content: &str,
    ) -> Result<(bool, Vec<ValidationError>)> {
        // This is a placeholder implementation
        // In a real implementation, you would:
        // 1. Load the appropriate contract validator
        // 2. Run the validation
        // 3. Return the results

        match scope {
            ValidationScope::Char => {
                // Run char_contract validation
                self.validate_char_contract(content).await
            }
            ValidationScope::Line => {
                // Run line_contract validation
                self.validate_line_contract(content).await
            }
            ValidationScope::Chunk => {
                // Run blob_contract validation
                self.validate_blob_contract(content).await
            }
            ValidationScope::File => {
                // Run tree_contract validation
                self.validate_tree_contract(content).await
            }
            ValidationScope::Directory => {
                // Run tree_filename_chars_contract validation
                self.validate_tree_filename_chars_contract(content).await
            }
            ValidationScope::Repository => {
                // Run repo_contract validation
                self.validate_repo_contract(content).await
            }
        }
    }

    /// Placeholder contract validators
    async fn validate_char_contract(&self, _content: &str) -> Result<(bool, Vec<ValidationError>)> {
        // Placeholder: always pass
        Ok((true, Vec::new()))
    }

    async fn validate_line_contract(&self, _content: &str) -> Result<(bool, Vec<ValidationError>)> {
        // Placeholder: always pass
        Ok((true, Vec::new()))
    }

    async fn validate_blob_contract(&self, _content: &str) -> Result<(bool, Vec<ValidationError>)> {
        // Placeholder: always pass
        Ok((true, Vec::new()))
    }

    async fn validate_tree_contract(&self, _content: &str) -> Result<(bool, Vec<ValidationError>)> {
        // Placeholder: always pass
        Ok((true, Vec::new()))
    }

    async fn validate_tree_filename_chars_contract(
        &self,
        _content: &str,
    ) -> Result<(bool, Vec<ValidationError>)> {
        // Placeholder: always pass
        Ok((true, Vec::new()))
    }

    async fn validate_repo_contract(&self, _content: &str) -> Result<(bool, Vec<ValidationError>)> {
        // Placeholder: always pass
        Ok((true, Vec::new()))
    }

    /// Generate SHA256 hash of content
    fn get_content_hash(&self, content: &str, range: &Option<ContentRange>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());

        if let Some(range) = range {
            hasher.update(
                format!(
                    "{}:{}:{}:{}",
                    range.start_line,
                    range.end_line,
                    range.start_char.unwrap_or(0),
                    range.end_char.unwrap_or(0)
                )
                .as_bytes(),
            );
        }

        format!("sha256:{:x}", hasher.finalize())
    }

    /// Get current commit hash
    async fn get_current_commit_hash(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get commit hash")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get commit hash: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Store validation note in Git notes
    async fn store_validation_note(
        &self,
        result: &ValidationResult,
        note: &ValidationNote,
    ) -> Result<()> {
        let note_json = serde_json::to_string_pretty(note)?;

        // Create a unique note reference
        let _note_ref = format!("refs/notes/contract-validation/{}", result.hash);

        // Store the note
        let output = Command::new("git")
            .args([
                "notes",
                "--ref=contract-validation",
                "add",
                "-m",
                &note_json,
            ])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to store validation note")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to store validation note: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Verify validation chain integrity
    pub async fn verify_validation_chain(&self, commit_hash: &str) -> Result<bool> {
        // Get all validation notes for the commit
        let notes = self.get_validation_notes(commit_hash).await?;

        // Check that all parent-child relationships are consistent
        for note in &notes {
            if let Some(parent_hash) = &note.parent_hash {
                // Find the parent note
                let parent_note = notes.iter().find(|n| n.hash == *parent_hash);
                if let Some(parent) = parent_note {
                    // Verify that this note is listed as a child of the parent
                    let is_child = parent
                        .child_scopes
                        .iter()
                        .any(|child| child.hash == note.hash);
                    if !is_child {
                        eprintln!("Validation chain integrity error: note {} not found as child of parent {}", 
                            note.hash, parent_hash);
                        return Ok(false);
                    }
                } else {
                    eprintln!(
                        "Validation chain integrity error: parent note {} not found",
                        parent_hash
                    );
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Get validation notes for a commit
    pub async fn get_validation_notes(&self, commit_hash: &str) -> Result<Vec<ValidationNote>> {
        let output = Command::new("git")
            .args(["notes", "--ref=contract-validation", "show", commit_hash])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get validation notes")?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let notes_text = String::from_utf8_lossy(&output.stdout);
        let mut notes = Vec::new();

        // Parse the notes (this is a simplified parser)
        for line in notes_text.lines() {
            if line.trim().starts_with('{') {
                if let Ok(note) = serde_json::from_str::<ValidationNote>(line) {
                    notes.push(note);
                }
            }
        }

        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_validation_scope_hierarchy() {
        let hierarchy = ValidationScope::hierarchy();
        assert_eq!(hierarchy.len(), 6);
        assert_eq!(hierarchy[0], ValidationScope::Char);
        assert_eq!(hierarchy[5], ValidationScope::Repository);
    }

    #[tokio::test]
    async fn test_validation_scope_parent_child() {
        assert_eq!(ValidationScope::Char.parent(), None);
        assert_eq!(ValidationScope::Line.parent(), Some(ValidationScope::Char));
        assert_eq!(ValidationScope::Repository.child(), None);
        assert_eq!(
            ValidationScope::Directory.child(),
            Some(ValidationScope::Repository)
        );
    }

    #[tokio::test]
    async fn test_content_hash_generation() {
        let temp_dir = TempDir::new().unwrap();
        let validator = HierarchicalValidator::new(temp_dir.path().to_path_buf());

        let content = "test content";
        let hash = validator.get_content_hash(content, &None);
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), 71); // "sha256:" + 64 hex chars
    }
}
