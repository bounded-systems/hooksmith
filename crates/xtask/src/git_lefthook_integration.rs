//! Git + Lefthook Integration for Event-Driven State Machine
//!
//! This module provides structured integration of Git and Lefthook outputs into
//! the event-driven state machine, with SARIF integration for contract validation.
//!
//! ## Features
//!
//! - **Structured Git Events**: Wraps git commit/push commands with JSONL events
//! - **Lefthook Integration**: Captures and normalizes Lefthook hook outputs
//! - **State Machine Integration**: Maps Git/Lefthook events to state transitions
//! - **SARIF Integration**: Emits contract violations as SARIF results
//! - **Event Blocking**: Supports dependency relationships between validation rules

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Instant;
use tokio::process::Command as TokioCommand;
use uuid::Uuid;

use crate::event_bus::{emit_event, HooksmithEvent};

/// Git workflow states for the state machine
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitWorkflowState {
    /// Initial state - no Git operations in progress
    IDLE,
    /// Git commit operation started
    COMMITTING,
    /// Lefthook hooks running after commit
    HOOK_RUNNING,
    /// Commit completed successfully
    COMMITTED,
    /// Git push operation started
    PUSHING,
    /// Push completed successfully
    PUSHED,
    /// Error state - operation failed
    ERROR,
}

/// Git workflow events that trigger state transitions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GitWorkflowEvent {
    /// Git commit started
    CommitStarted,
    /// Lefthook hook started
    HookStarted,
    /// Lefthook hook completed
    HookCompleted,
    /// Commit completed
    CommitCompleted,
    /// Git push started
    PushStarted,
    /// Git push completed
    PushCompleted,
    /// Operation failed
    OperationFailed,
}

/// Git commit metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommitMetadata {
    /// Commit hash
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Branch name
    pub branch: String,
    /// Files changed
    pub files: Vec<String>,
    /// Insertions count
    pub insertions: Option<u32>,
    /// Deletions count
    pub deletions: Option<u32>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Lefthook hook metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LefthookHookMetadata {
    /// Hook name (e.g., "pre-commit", "post-commit")
    pub name: String,
    /// Command executed
    pub command: String,
    /// Exit code
    pub exit_code: i32,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Output captured
    pub output: String,
    /// Files processed
    pub files_processed: Option<Vec<String>>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Contract validation result with SARIF integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Contract ID
    pub contract_id: String,
    /// File being validated
    pub file: String,
    /// Validation errors
    pub errors: Vec<ContractViolation>,
    /// Validation warnings
    pub warnings: Vec<ContractViolation>,
    /// SARIF result for external tooling
    pub sarif_result: Option<SarifResult>,
    /// Dependencies that block this validation
    pub blocked_by: Option<Vec<String>>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Contract violation with SARIF-compatible structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractViolation {
    /// Unique violation ID
    pub id: String,
    /// Rule ID
    pub rule_id: String,
    /// Violation message
    pub message: String,
    /// Severity level
    pub severity: ViolationSeverity,
    /// File path
    pub file: String,
    /// Line number (1-indexed)
    pub line: Option<u32>,
    /// Column number (1-indexed)
    pub column: Option<u32>,
    /// End line number
    pub end_line: Option<u32>,
    /// End column number
    pub end_column: Option<u32>,
    /// Additional details
    pub details: Option<Value>,
    /// Fingerprint for deduplication
    pub fingerprint: Option<String>,
    /// Dependencies that block this violation
    pub blocked_by: Option<Vec<String>>,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Information only
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical error
    Critical,
}

/// SARIF result structure for external tooling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifResult {
    /// Rule ID
    pub rule_id: String,
    /// Result level (error, warning, note, none)
    pub level: String,
    /// Result kind (fail, pass, review, etc.)
    pub kind: String,
    /// Message text
    pub message: String,
    /// Physical location
    pub location: Option<SarifLocation>,
    /// Related locations for dependency chains
    pub related_locations: Option<Vec<SarifLocation>>,
    /// Properties for custom metadata
    pub properties: Option<HashMap<String, Value>>,
    /// Partial fingerprint for deduplication
    pub partial_fingerprints: Option<HashMap<String, String>>,
}

/// SARIF location structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifLocation {
    /// Physical location
    pub physical_location: SarifPhysicalLocation,
    /// Nesting level for dependency chains
    pub nesting_level: Option<u32>,
}

/// SARIF physical location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifPhysicalLocation {
    /// Artifact location
    pub artifact_location: SarifArtifactLocation,
    /// Region information
    pub region: Option<SarifRegion>,
}

/// SARIF artifact location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifArtifactLocation {
    /// File URI
    pub uri: String,
}

/// SARIF region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarifRegion {
    /// Start line (1-indexed)
    pub start_line: Option<u32>,
    /// Start column (1-indexed)
    pub start_column: Option<u32>,
    /// End line (1-indexed)
    pub end_line: Option<u32>,
    /// End column (1-indexed)
    pub end_column: Option<u32>,
}

/// Git + Lefthook integration manager
#[derive(Debug)]
pub struct GitLefthookIntegration {
    /// Current workflow state
    current_state: GitWorkflowState,
    /// Session ID for grouping related events
    session_id: String,
    /// Contract validation results
    validation_results: Vec<ContractValidationResult>,
    /// SARIF results for external tooling
    sarif_results: Vec<SarifResult>,
    /// Event blocking dependencies
    blocking_dependencies: HashMap<String, Vec<String>>,
}

impl GitLefthookIntegration {
    /// Create a new Git + Lefthook integration manager
    pub fn new() -> Self {
        Self {
            current_state: GitWorkflowState::IDLE,
            session_id: Uuid::new_v4().to_string(),
            validation_results: Vec::new(),
            sarif_results: Vec::new(),
            blocking_dependencies: HashMap::new(),
        }
    }

    /// Get current state
    pub fn current_state(&self) -> &GitWorkflowState {
        &self.current_state
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Transition to a new state
    pub fn transition(&mut self, event: GitWorkflowEvent, context: Value) -> Result<()> {
        let from_state = self.current_state.clone();
        let to_state = self.get_next_state(&from_state, &event)?;

        // Emit state transition event
        let transition_event = HooksmithEvent::new(
            "git-lefthook-integration".to_string(),
            "state_transition".to_string(),
            serde_json::json!({
                "from_state": from_state,
                "to_state": to_state,
                "event": event,
                "context": context,
                "session_id": self.session_id,
            }),
        )
        .with_state(format!("{to_state:?}"))
        .with_session_id(self.session_id.clone());

        emit_event(transition_event)?;

        self.current_state = to_state;
        Ok(())
    }

    /// Get next state based on current state and event
    fn get_next_state(
        &self,
        current: &GitWorkflowState,
        event: &GitWorkflowEvent,
    ) -> Result<GitWorkflowState> {
        match (current, event) {
            (GitWorkflowState::IDLE, GitWorkflowEvent::CommitStarted) => {
                Ok(GitWorkflowState::COMMITTING)
            }
            (GitWorkflowState::COMMITTING, GitWorkflowEvent::HookStarted) => {
                Ok(GitWorkflowState::HOOK_RUNNING)
            }
            (GitWorkflowState::HOOK_RUNNING, GitWorkflowEvent::HookCompleted) => {
                Ok(GitWorkflowState::COMMITTED)
            }
            (GitWorkflowState::COMMITTED, GitWorkflowEvent::PushStarted) => {
                Ok(GitWorkflowState::PUSHING)
            }
            (GitWorkflowState::PUSHING, GitWorkflowEvent::PushCompleted) => {
                Ok(GitWorkflowState::PUSHED)
            }
            (_, GitWorkflowEvent::OperationFailed) => Ok(GitWorkflowState::ERROR),
            _ => Err(anyhow::anyhow!(
                "Invalid state transition: {:?} -> {:?}",
                current,
                event
            )),
        }
    }

    /// Execute a Git commit with structured event emission
    pub async fn execute_git_commit(
        &mut self,
        message: &str,
        files: Option<Vec<String>>,
    ) -> Result<GitCommitMetadata> {
        let start_time = Instant::now();

        // Emit commit started event
        self.transition(
            GitWorkflowEvent::CommitStarted,
            serde_json::json!({
                "message": message,
                "files": files,
                "timestamp": Utc::now(),
            }),
        )?;

        // Execute git commit
        let mut cmd = TokioCommand::new("git");
        cmd.args(["commit", "-m", message]);

        if let Some(ref file_list) = files {
            for file in file_list {
                cmd.arg(file);
            }
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute git commit")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            self.transition(
                GitWorkflowEvent::OperationFailed,
                serde_json::json!({
                    "error": error_msg,
                    "exit_code": output.status.code(),
                }),
            )?;
            return Err(anyhow::anyhow!("Git commit failed: {}", error_msg));
        }

        // Parse commit hash from output
        let output_str = String::from_utf8_lossy(&output.stdout);
        let commit_hash = self.extract_commit_hash(&output_str)?;

        // Get commit metadata
        let metadata = self.get_commit_metadata(&commit_hash).await?;

        // Emit commit completed event
        self.transition(
            GitWorkflowEvent::CommitCompleted,
            serde_json::json!({
                "commit_hash": commit_hash,
                "metadata": metadata,
                "duration_ms": start_time.elapsed().as_millis(),
            }),
        )?;

        Ok(metadata)
    }

    /// Execute Lefthook hooks with structured event emission
    pub async fn execute_lefthook_hooks(
        &mut self,
        hook_name: &str,
        quiet: bool,
    ) -> Result<LefthookHookMetadata> {
        let start_time = Instant::now();

        // Emit hook started event
        self.transition(
            GitWorkflowEvent::HookStarted,
            serde_json::json!({
                "hook_name": hook_name,
                "timestamp": Utc::now(),
            }),
        )?;

        // Configure Lefthook output
        let mut env = std::collections::HashMap::new();
        if quiet {
            env.insert("LEFTHOOK_QUIET".to_string(), "true".to_string());
            env.insert("LEFTHOOK_OUTPUT".to_string(), "false".to_string());
        } else {
            // Use minimal output for easier parsing
            env.insert("LEFTHOOK_OUTPUT".to_string(), "summary,failure".to_string());
        }

        // Execute Lefthook hook
        let mut cmd = TokioCommand::new("lefthook");
        cmd.args(["run", hook_name]).envs(&env);

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context(format!("Failed to execute lefthook run {hook_name}"))?;

        let duration_ms = start_time.elapsed().as_millis();
        let output_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        let metadata = LefthookHookMetadata {
            name: hook_name.to_string(),
            command: format!("lefthook run {hook_name}"),
            exit_code: output.status.code().unwrap_or(-1),
            duration_ms: duration_ms as u64,
            output: if !output_str.is_empty() {
                output_str.to_string()
            } else {
                stderr_str.to_string()
            },
            files_processed: self.extract_processed_files(&output_str),
            timestamp: Utc::now(),
        };

        // Emit hook completed event
        self.transition(
            GitWorkflowEvent::HookCompleted,
            serde_json::json!({
                "hook_metadata": metadata,
                "success": output.status.success(),
            }),
        )?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Lefthook hook {} failed with exit code {}",
                hook_name,
                output.status.code().unwrap_or(-1)
            ));
        }

        Ok(metadata)
    }

    /// Execute Git push with structured event emission
    pub async fn execute_git_push(
        &mut self,
        remote: &str,
        branch: &str,
        force: bool,
    ) -> Result<()> {
        let start_time = Instant::now();

        // Emit push started event
        self.transition(
            GitWorkflowEvent::PushStarted,
            serde_json::json!({
                "remote": remote,
                "branch": branch,
                "force": force,
                "timestamp": Utc::now(),
            }),
        )?;

        // Execute git push
        let mut cmd = TokioCommand::new("git");
        cmd.args(["push", remote, branch]);

        if force {
            cmd.arg("--force");
        }

        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute git push")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            self.transition(
                GitWorkflowEvent::OperationFailed,
                serde_json::json!({
                    "error": error_msg,
                    "exit_code": output.status.code(),
                }),
            )?;
            return Err(anyhow::anyhow!("Git push failed: {}", error_msg));
        }

        // Emit push completed event
        self.transition(
            GitWorkflowEvent::PushCompleted,
            serde_json::json!({
                "remote": remote,
                "branch": branch,
                "force": force,
                "duration_ms": start_time.elapsed().as_millis(),
                "output": String::from_utf8_lossy(&output.stdout),
            }),
        )?;

        Ok(())
    }

    /// Add contract validation result with SARIF integration
    pub fn add_validation_result(&mut self, result: ContractValidationResult) -> Result<()> {
        // Check if validation is blocked by dependencies
        if let Some(ref blocked_by) = result.blocked_by {
            for blocking_id in blocked_by {
                if !self.is_validation_passed(blocking_id) {
                    // Emit blocked validation event
                    let blocked_event = HooksmithEvent::new(
                        "contract-validation".to_string(),
                        "validation_blocked".to_string(),
                        serde_json::json!({
                            "contract_id": result.contract_id,
                            "blocked_by": blocked_by,
                            "file": result.file,
                        }),
                    )
                    .with_session_id(self.session_id.clone());

                    emit_event(blocked_event)?;
                    return Ok(());
                }
            }
        }

        // Add to validation results
        self.validation_results.push(result.clone());

        // Convert to SARIF if validation failed
        if !result.is_valid {
            if let Some(sarif_result) = result.sarif_result.clone() {
                self.sarif_results.push(sarif_result);
            }
        }

        // Emit validation event
        let validation_event = HooksmithEvent::new(
            "contract-validation".to_string(),
            if result.is_valid {
                "validation_passed".to_string()
            } else {
                "validation_failed".to_string()
            },
            serde_json::json!({
                "contract_id": result.contract_id,
                "file": result.file,
                "is_valid": result.is_valid,
                "error_count": result.errors.len(),
                "warning_count": result.warnings.len(),
                "sarif_result": result.sarif_result,
            }),
        )
        .with_session_id(self.session_id.clone());

        emit_event(validation_event)?;

        Ok(())
    }

    /// Check if a validation has passed
    fn is_validation_passed(&self, contract_id: &str) -> bool {
        self.validation_results
            .iter()
            .find(|r| r.contract_id == contract_id)
            .map(|r| r.is_valid)
            .unwrap_or(false)
    }

    /// Generate SARIF document from collected results
    pub fn generate_sarif_document(&self) -> Result<String> {
        let sarif_doc = serde_json::json!({
            "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
            "version": "2.1.0",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "hooksmith",
                        "version": env!("CARGO_PKG_VERSION"),
                        "rules": self.generate_sarif_rules()
                    }
                },
                "results": self.sarif_results,
                "invocations": [{
                    "executionSuccessful": true,
                    "toolExecutionNotifications": []
                }]
            }]
        });

        serde_json::to_string_pretty(&sarif_doc).context("Failed to serialize SARIF document")
    }

    /// Generate SARIF rules from validation results
    fn generate_sarif_rules(&self) -> Vec<Value> {
        let mut rules = Vec::new();
        let mut rule_ids = std::collections::HashSet::new();

        for result in &self.validation_results {
            for error in &result.errors {
                if !rule_ids.contains(&error.rule_id) {
                    rule_ids.insert(error.rule_id.clone());
                    rules.push(serde_json::json!({
                        "id": error.rule_id,
                        "name": error.rule_id,
                        "shortDescription": {
                            "text": error.message
                        },
                        "defaultConfiguration": {
                            "level": match error.severity {
                                ViolationSeverity::Info => "note",
                                ViolationSeverity::Warning => "warning",
                                ViolationSeverity::Error => "error",
                                ViolationSeverity::Critical => "error",
                            }
                        }
                    }));
                }
            }
        }

        rules
    }

    /// Extract commit hash from git commit output
    fn extract_commit_hash(&self, output: &str) -> Result<String> {
        // Look for patterns like "[main abc1234] commit message"
        for line in output.lines() {
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    let content = &line[start + 1..end];
                    if let Some(hash_start) = content.rfind(' ') {
                        let hash = &content[hash_start + 1..];
                        if hash.len() >= 7 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                            return Ok(hash.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: try to get the latest commit hash
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .context("Failed to get commit hash")?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !hash.is_empty() {
                return Ok(hash);
            }
        }

        Err(anyhow::anyhow!("Could not extract commit hash from output"))
    }

    /// Get commit metadata
    async fn get_commit_metadata(&self, commit_hash: &str) -> Result<GitCommitMetadata> {
        // Get commit details
        let output = TokioCommand::new("git")
            .args(["show", "--format=format:%H%n%s%n%b", "--stat", commit_hash])
            .output()
            .await
            .context("Failed to get commit details")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get commit details"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = output_str.lines().collect();

        if lines.len() < 2 {
            return Err(anyhow::anyhow!("Invalid commit output format"));
        }

        let hash = lines[0].to_string();
        let message = lines[1].to_string();

        // Get current branch
        let branch_output = TokioCommand::new("git")
            .args(["branch", "--show-current"])
            .output()
            .await
            .context("Failed to get current branch")?;

        let branch = if branch_output.status.success() {
            String::from_utf8_lossy(&branch_output.stdout)
                .trim()
                .to_string()
        } else {
            "unknown".to_string()
        };

        // Parse files and statistics
        let mut files = Vec::new();
        let mut insertions = None;
        let mut deletions = None;

        for line in lines.iter().skip(2) {
            if line.contains("|") && line.contains("+++") {
                // Parse file statistics
                if let Some(file_part) = line.split("|").next() {
                    let file = file_part.trim();
                    if !file.is_empty() && file != "0 files changed" {
                        files.push(file.to_string());
                    }
                }

                // Parse insertions/deletions
                if let Some(stats_part) = line.split("|").nth(1) {
                    let stats = stats_part.trim();
                    if let Some(plus_minus) = stats.split_whitespace().next() {
                        if let Some(plus) = plus_minus.split('+').nth(1) {
                            if let Some(plus_num) = plus.split('-').next() {
                                insertions = plus_num.parse::<u32>().ok();
                            }
                        }
                        if let Some(minus) = plus_minus.split('-').nth(1) {
                            deletions = minus.parse::<u32>().ok();
                        }
                    }
                }
            }
        }

        Ok(GitCommitMetadata {
            hash,
            message,
            branch,
            files,
            insertions,
            deletions,
            timestamp: Utc::now(),
        })
    }

    /// Extract processed files from Lefthook output
    fn extract_processed_files(&self, output: &str) -> Option<Vec<String>> {
        let mut files = Vec::new();

        for line in output.lines() {
            // Look for file patterns in Lefthook output
            if line.contains(".rs") || line.contains(".toml") || line.contains(".yml") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.contains('.') && !part.contains("cargo") && !part.contains("lefthook") {
                        files.push(part.to_string());
                    }
                }
            }
        }

        if files.is_empty() {
            None
        } else {
            Some(files)
        }
    }

    /// Add blocking dependency between validations
    pub fn add_blocking_dependency(&mut self, dependent: &str, blocking: &str) {
        self.blocking_dependencies
            .entry(dependent.to_string())
            .or_default()
            .push(blocking.to_string());
    }

    /// Get all validation results
    pub fn validation_results(&self) -> &[ContractValidationResult] {
        &self.validation_results
    }

    /// Get all SARIF results
    pub fn sarif_results(&self) -> &[SarifResult] {
        &self.sarif_results
    }

    /// Clear all results
    pub fn clear_results(&mut self) {
        self.validation_results.clear();
        self.sarif_results.clear();
    }
}

impl Default for GitLefthookIntegration {
    fn default() -> Self {
        Self::new()
    }
}

// CLI Command Functions

/// Execute a complete Git workflow (commit + hooks + push)
pub async fn run_workflow_command(
    message: String,
    files: Option<Vec<String>>,
    hook: String,
    remote: String,
    branch: Option<String>,
    force: bool,
    quiet: bool,
    sarif_output: Option<String>,
) -> Result<()> {
    println!("🚀 Executing complete Git workflow");
    println!("   Message: {message}");
    println!("   Hook: {hook}");
    println!("   Remote: {remote}");
    println!("   Branch: {branch:?}");
    println!("   Force: {force}");
    println!("   Quiet: {quiet}");

    let mut integration = GitLefthookIntegration::new();

    // Execute Git commit
    let commit_metadata = integration.execute_git_commit(&message, files).await?;
    println!("✅ Commit completed: {}", commit_metadata.hash);

    // Execute Lefthook hooks
    let hook_metadata = integration.execute_lefthook_hooks(&hook, quiet).await?;
    println!(
        "✅ Hook completed: {} (exit code: {})",
        hook_metadata.name, hook_metadata.exit_code
    );

    // Execute Git push
    let branch_name = branch.unwrap_or_else(|| "main".to_string());
    integration
        .execute_git_push(&remote, &branch_name, force)
        .await?;
    println!("✅ Push completed to {remote}/{branch_name}");

    // Generate SARIF output if requested
    if let Some(output_path) = sarif_output {
        let sarif_document = integration.generate_sarif_document()?;
        std::fs::write(&output_path, &sarif_document)
            .context(format!("Failed to write SARIF file: {output_path}"))?;
        println!("📄 SARIF results written to: {output_path}");
    }

    println!("🎉 Complete workflow executed successfully!");
    Ok(())
}

/// Execute Git commit with structured events
pub async fn run_commit_command(message: String, files: Option<Vec<String>>) -> Result<()> {
    println!("📝 Executing Git commit");
    println!("   Message: {message}");
    println!("   Files: {files:?}");

    let mut integration = GitLefthookIntegration::new();
    let commit_metadata = integration.execute_git_commit(&message, files).await?;

    println!("✅ Commit completed successfully!");
    println!("   Hash: {}", commit_metadata.hash);
    println!("   Branch: {}", commit_metadata.branch);
    println!("   Files: {:?}", commit_metadata.files);
    if let Some(insertions) = commit_metadata.insertions {
        println!("   Insertions: {insertions}");
    }
    if let Some(deletions) = commit_metadata.deletions {
        println!("   Deletions: {deletions}");
    }

    Ok(())
}

/// Execute Lefthook hooks with structured events
pub async fn run_hooks_command(hook: String, quiet: bool) -> Result<()> {
    println!("🔧 Executing Lefthook hooks");
    println!("   Hook: {hook}");
    println!("   Quiet: {quiet}");

    let mut integration = GitLefthookIntegration::new();
    let hook_metadata = integration.execute_lefthook_hooks(&hook, quiet).await?;

    println!("✅ Hook executed successfully!");
    println!("   Name: {}", hook_metadata.name);
    println!("   Command: {}", hook_metadata.command);
    println!("   Exit code: {}", hook_metadata.exit_code);
    println!("   Duration: {}ms", hook_metadata.duration_ms);
    if let Some(files) = hook_metadata.files_processed {
        println!("   Files processed: {files:?}");
    }

    Ok(())
}

/// Execute Git push with structured events
pub async fn run_push_command(remote: String, branch: Option<String>, force: bool) -> Result<()> {
    println!("🚀 Executing Git push");
    println!("   Remote: {remote}");
    println!("   Branch: {branch:?}");
    println!("   Force: {force}");

    let mut integration = GitLefthookIntegration::new();
    let branch_name = branch.unwrap_or_else(|| "main".to_string());
    integration
        .execute_git_push(&remote, &branch_name, force)
        .await?;

    println!("✅ Push completed successfully!");
    println!("   Remote: {remote}/{branch_name}");

    Ok(())
}

/// Add contract validation with SARIF integration
pub async fn run_validate_command(
    contract_id: String,
    file: String,
    rule_id: String,
    message: String,
    severity: ViolationSeverity,
    line: Option<u32>,
    column: Option<u32>,
    end_line: Option<u32>,
    end_column: Option<u32>,
    blocked_by: Option<String>,
) -> Result<()> {
    println!("🔍 Adding contract validation");
    println!("   Contract ID: {contract_id}");
    println!("   File: {file}");
    println!("   Rule ID: {rule_id}");
    println!("   Message: {message}");
    println!("   Severity: {severity:?}");

    let mut integration = GitLefthookIntegration::new();

    // Create contract violation
    let violation = ContractViolation {
        id: format!("{contract_id}-{rule_id}"),
        rule_id,
        message,
        severity,
        file: file.clone(),
        line,
        column,
        end_line,
        end_column,
        details: Some(serde_json::json!({
            "contract_id": contract_id,
            "validation_timestamp": chrono::Utc::now()
        })),
        fingerprint: Some(format!("{}-{}-{}", contract_id, file, line.unwrap_or(0))),
        blocked_by: blocked_by.map(|b| vec![b]),
    };

    // Create validation result
    let validation_result = ContractValidationResult {
        is_valid: false,
        contract_id,
        file,
        errors: vec![violation],
        warnings: vec![],
        sarif_result: None,
        blocked_by: None,
        timestamp: chrono::Utc::now(),
    };

    integration.add_validation_result(validation_result)?;
    println!("✅ Contract validation added successfully!");

    Ok(())
}

/// Generate SARIF document from validation results
pub async fn run_generate_sarif_command(output: String) -> Result<()> {
    println!("📄 Generating SARIF document");
    println!("   Output: {output}");

    let integration = GitLefthookIntegration::new();
    let sarif_document = integration.generate_sarif_document()?;

    std::fs::write(&output, &sarif_document)
        .context(format!("Failed to write SARIF file: {output}"))?;

    println!("✅ SARIF document generated successfully!");
    println!("   File: {output}");
    println!("   Size: {} bytes", sarif_document.len());

    Ok(())
}

/// Show current state and validation results
pub async fn run_status_command() -> Result<()> {
    println!("📊 Git + Lefthook Integration Status");
    println!("===================================");

    let integration = GitLefthookIntegration::new();

    println!("Session ID: {}", integration.session_id());
    println!("Current State: {:?}", integration.current_state());
    println!(
        "Validation Results: {}",
        integration.validation_results().len()
    );
    println!("SARIF Results: {}", integration.sarif_results().len());

    if !integration.validation_results().is_empty() {
        println!("\nValidation Results:");
        println!("-------------------");
        for result in integration.validation_results() {
            println!("  Contract: {}", result.contract_id);
            println!("  File: {}", result.file);
            println!("  Valid: {}", result.is_valid);
            println!("  Errors: {}", result.errors.len());
            println!("  Warnings: {}", result.warnings.len());
            if let Some(ref blocked_by) = result.blocked_by {
                println!("  Blocked by: {blocked_by:?}");
            }
            println!();
        }
    }

    if !integration.sarif_results().is_empty() {
        println!("SARIF Results:");
        println!("--------------");
        for result in integration.sarif_results() {
            println!("  Rule: {}", result.rule_id);
            println!("  Level: {}", result.level);
            println!("  Message: {}", result.message);
            println!();
        }
    }

    Ok(())
}

/// Helper function to create a contract violation with SARIF integration
pub fn create_contract_violation(
    rule_id: &str,
    message: &str,
    file: &str,
    severity: ViolationSeverity,
    line: Option<u32>,
    column: Option<u32>,
    end_line: Option<u32>,
    end_column: Option<u32>,
    details: Option<Value>,
    blocked_by: Option<Vec<String>>,
) -> ContractViolation {
    let id = Uuid::new_v4().to_string();
    let fingerprint = format!("{}-{}-{}", rule_id, file, line.unwrap_or(0));

    let sarif_result = SarifResult {
        rule_id: rule_id.to_string(),
        level: match severity {
            ViolationSeverity::Info => "note".to_string(),
            ViolationSeverity::Warning => "warning".to_string(),
            ViolationSeverity::Error => "error".to_string(),
            ViolationSeverity::Critical => "error".to_string(),
        },
        kind: "fail".to_string(),
        message: message.to_string(),
        location: Some(SarifLocation {
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation {
                    uri: file.to_string(),
                },
                region: Some(SarifRegion {
                    start_line: line,
                    start_column: column,
                    end_line,
                    end_column,
                }),
            },
            nesting_level: None,
        }),
        related_locations: None,
        properties: Some({
            let mut props = HashMap::new();
            if let Some(ref blocked) = blocked_by {
                props.insert(
                    "hooksmith.blockedBy".to_string(),
                    serde_json::to_value(blocked).unwrap(),
                );
            }
            props
        }),
        partial_fingerprints: Some({
            let mut fps = HashMap::new();
            fps.insert("hooksmith/v1".to_string(), fingerprint.clone());
            fps
        }),
    };

    ContractViolation {
        id,
        rule_id: rule_id.to_string(),
        message: message.to_string(),
        severity,
        file: file.to_string(),
        line,
        column,
        end_line,
        end_column,
        details,
        fingerprint: Some(fingerprint),
        blocked_by,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let mut integration = GitLefthookIntegration::new();

        // Test valid transitions
        assert!(integration
            .transition(GitWorkflowEvent::CommitStarted, Value::Null)
            .is_ok());
        assert_eq!(*integration.current_state(), GitWorkflowState::COMMITTING);

        assert!(integration
            .transition(GitWorkflowEvent::HookStarted, Value::Null)
            .is_ok());
        assert_eq!(*integration.current_state(), GitWorkflowState::HOOK_RUNNING);

        assert!(integration
            .transition(GitWorkflowEvent::HookCompleted, Value::Null)
            .is_ok());
        assert_eq!(*integration.current_state(), GitWorkflowState::COMMITTED);
    }

    #[test]
    fn test_invalid_state_transitions() {
        let mut integration = GitLefthookIntegration::new();

        // Test invalid transition
        assert!(integration
            .transition(GitWorkflowEvent::HookStarted, Value::Null)
            .is_err());
    }

    #[test]
    fn test_contract_violation_creation() {
        let violation = create_contract_violation(
            "line-length",
            "Line exceeds maximum length",
            "src/main.rs",
            ViolationSeverity::Error,
            Some(10),
            Some(5),
            Some(10),
            Some(120),
            None,
            None,
        );

        assert_eq!(violation.rule_id, "line-length");
        assert_eq!(violation.severity, ViolationSeverity::Error);
        assert_eq!(violation.file, "src/main.rs");
        assert_eq!(violation.line, Some(10));
        assert!(violation.sarif_result.is_some());
    }
}
