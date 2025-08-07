use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use git2::{Commit, ObjectType, Oid, Repository, Tree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::{
    ClientInfo, GitProtocol, GitProxyConfig, GitProxyEvent, ValidationOperationType,
    ValidationRequest, ValidationResult,
};

/// Server-side Git hook types
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum ServerHookType {
    /// Pre-receive hook - called before refs are updated
    PreReceive,
    /// Post-receive hook - called after refs are updated
    PostReceive,
    /// Update hook - called for each ref update
    Update,
    /// Post-update hook - called after all refs are updated
    PostUpdate,
    /// Pre-commit hook - called before commit is created
    PreCommit,
    /// Post-commit hook - called after commit is created
    PostCommit,
    /// Prepare-commit-msg hook - called before editor starts
    PrepareCommitMsg,
    /// Commit-msg hook - called after commit message is prepared
    CommitMsg,
    /// Pre-push hook - called before push
    PrePush,
    /// Post-merge hook - called after merge
    PostMerge,
    /// Post-checkout hook - called after checkout
    PostCheckout,
    /// Pre-rebase hook - called before rebase
    PreRebase,
    /// Post-rewrite hook - called after commit rewriting
    PostRewrite,
    /// Reference transaction hook - called during ref transactions
    ReferenceTransaction,
    /// Push-to-checkout hook - called when pushing to checked out branch
    PushToCheckout,
    /// Pre-auto-gc hook - called before auto garbage collection
    PreAutoGc,
    /// Sendemail-validate hook - called before sending email
    SendemailValidate,
    /// Fsmonitor-watchman hook - called for file system monitoring
    FsmonitorWatchman,
    /// P4 hooks - called during Perforce operations
    P4Changelist,
    P4PrepareChangelist,
    P4PostChangelist,
    P4PreSubmit,
    /// Post-index-change hook - called when index is written
    PostIndexChange,
}

/// Hook execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// Hook type
    pub hook_type: ServerHookType,
    /// Repository path
    pub repo_path: PathBuf,
    /// Working directory
    pub work_dir: Option<PathBuf>,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Command line arguments
    pub args: Vec<String>,
    /// Standard input data
    pub stdin_data: Option<String>,
    /// Client information
    pub client_info: Option<ClientInfo>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Success status
    pub success: bool,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: Option<String>,
    /// Standard error
    pub stderr: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Server-side hook handler
pub struct ServerHookHandler {
    config: GitProxyConfig,
    repository: Option<Repository>,
    hooks_enabled: HashMap<ServerHookType, bool>,
}

impl ServerHookHandler {
    /// Create a new server hook handler
    pub fn new(config: GitProxyConfig) -> Self {
        let mut hooks_enabled = HashMap::new();

        // Enable common hooks by default
        hooks_enabled.insert(ServerHookType::PreReceive, true);
        hooks_enabled.insert(ServerHookType::PostReceive, true);
        hooks_enabled.insert(ServerHookType::Update, true);
        hooks_enabled.insert(ServerHookType::PreCommit, true);
        hooks_enabled.insert(ServerHookType::PostCommit, true);
        hooks_enabled.insert(ServerHookType::PrePush, true);

        Self {
            config,
            repository: None,
            hooks_enabled,
        }
    }

    /// Set repository for hook execution
    pub fn set_repository(&mut self, repository: Repository) {
        self.repository = Some(repository);
    }

    /// Enable or disable a specific hook
    pub fn set_hook_enabled(&mut self, hook_type: ServerHookType, enabled: bool) {
        self.hooks_enabled.insert(hook_type, enabled);
    }

    /// Execute a server-side hook
    pub async fn execute_hook(&self, context: HookContext) -> Result<HookResult> {
        let start_time = std::time::Instant::now();

        if !self.hooks_enabled.get(&context.hook_type).unwrap_or(&false) {
            return Ok(HookResult {
                success: true,
                exit_code: 0,
                stdout: None,
                stderr: None,
                error: None,
                duration_ms: 0,
                timestamp: Utc::now(),
            });
        }

        info!("Executing server hook: {:?}", context.hook_type);

        let result = match context.hook_type {
            ServerHookType::PreReceive => self.handle_pre_receive(&context).await,
            ServerHookType::PostReceive => self.handle_post_receive(&context).await,
            ServerHookType::Update => self.handle_update(&context).await,
            ServerHookType::PostUpdate => self.handle_post_update(&context).await,
            ServerHookType::PreCommit => self.handle_pre_commit(&context).await,
            ServerHookType::PostCommit => self.handle_post_commit(&context).await,
            ServerHookType::PrepareCommitMsg => self.handle_prepare_commit_msg(&context).await,
            ServerHookType::CommitMsg => self.handle_commit_msg(&context).await,
            ServerHookType::PrePush => self.handle_pre_push(&context).await,
            ServerHookType::PostMerge => self.handle_post_merge(&context).await,
            ServerHookType::PostCheckout => self.handle_post_checkout(&context).await,
            ServerHookType::PreRebase => self.handle_pre_rebase(&context).await,
            ServerHookType::PostRewrite => self.handle_post_rewrite(&context).await,
            ServerHookType::ReferenceTransaction => {
                self.handle_reference_transaction(&context).await
            }
            ServerHookType::PushToCheckout => self.handle_push_to_checkout(&context).await,
            ServerHookType::PreAutoGc => self.handle_pre_auto_gc(&context).await,
            ServerHookType::SendemailValidate => self.handle_sendemail_validate(&context).await,
            ServerHookType::FsmonitorWatchman => self.handle_fsmonitor_watchman(&context).await,
            ServerHookType::P4Changelist => self.handle_p4_changelist(&context).await,
            ServerHookType::P4PrepareChangelist => {
                self.handle_p4_prepare_changelist(&context).await
            }
            ServerHookType::P4PostChangelist => self.handle_p4_post_changelist(&context).await,
            ServerHookType::P4PreSubmit => self.handle_p4_pre_submit(&context).await,
            ServerHookType::PostIndexChange => self.handle_post_index_change(&context).await,
        };

        let duration_ms = start_time.elapsed().as_millis() as u64;

        match result {
            Ok(output) => Ok(HookResult {
                success: true,
                exit_code: 0,
                stdout: Some(output),
                stderr: None,
                error: None,
                duration_ms,
                timestamp: Utc::now(),
            }),
            Err(e) => {
                error!("Hook execution failed: {}", e);
                Ok(HookResult {
                    success: false,
                    exit_code: 1,
                    stdout: None,
                    stderr: Some(e.to_string()),
                    error: Some(e.to_string()),
                    duration_ms,
                    timestamp: Utc::now(),
                })
            }
        }
    }

    /// Handle pre-receive hook
    async fn handle_pre_receive(&self, context: &HookContext) -> Result<String> {
        debug!("Executing pre-receive hook");

        // Parse ref updates from stdin
        let ref_updates = self.parse_ref_updates(&context.stdin_data)?;

        // Validate each ref update
        for update in &ref_updates {
            self.validate_ref_update(update).await?;
        }

        Ok("Pre-receive hook completed successfully".to_string())
    }

    /// Handle post-receive hook
    async fn handle_post_receive(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-receive hook");

        // Parse ref updates from stdin
        let ref_updates = self.parse_ref_updates(&context.stdin_data)?;

        // Log successful updates
        for update in &ref_updates {
            info!("Ref updated: {} -> {}", update.old_oid, update.new_oid);
        }

        Ok("Post-receive hook completed successfully".to_string())
    }

    /// Handle update hook
    async fn handle_update(&self, context: &HookContext) -> Result<String> {
        debug!("Executing update hook");

        if context.args.len() < 3 {
            return Err(anyhow::anyhow!(
                "Update hook requires 3 arguments: ref, old-oid, new-oid"
            ));
        }

        let ref_name = &context.args[0];
        let old_oid = &context.args[1];
        let new_oid = &context.args[2];

        // Validate the specific ref update
        let update = RefUpdate {
            ref_name: ref_name.clone(),
            old_oid: old_oid.clone(),
            new_oid: new_oid.clone(),
        };

        self.validate_ref_update(&update).await?;

        Ok("Update hook completed successfully".to_string())
    }

    /// Handle post-update hook
    async fn handle_post_update(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-update hook");

        // Log updated refs
        for ref_name in &context.args {
            info!("Ref updated: {}", ref_name);
        }

        Ok("Post-update hook completed successfully".to_string())
    }

    /// Handle pre-commit hook
    async fn handle_pre_commit(&self, _context: &HookContext) -> Result<String> {
        debug!("Executing pre-commit hook");

        // Validate working tree
        self.validate_working_tree().await?;

        Ok("Pre-commit hook completed successfully".to_string())
    }

    /// Handle post-commit hook
    async fn handle_post_commit(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-commit hook");

        if let Some(commit_id) = context.args.get(0) {
            info!("Commit created: {}", commit_id);
        }

        Ok("Post-commit hook completed successfully".to_string())
    }

    /// Handle prepare-commit-msg hook
    async fn handle_prepare_commit_msg(&self, context: &HookContext) -> Result<String> {
        debug!("Executing prepare-commit-msg hook");

        if let Some(msg_file) = context.args.get(0) {
            // Could modify the commit message file here
            debug!("Preparing commit message file: {}", msg_file);
        }

        Ok("Prepare-commit-msg hook completed successfully".to_string())
    }

    /// Handle commit-msg hook
    async fn handle_commit_msg(&self, context: &HookContext) -> Result<String> {
        debug!("Executing commit-msg hook");

        if let Some(msg_file) = context.args.get(0) {
            // Could validate or modify the commit message here
            debug!("Validating commit message file: {}", msg_file);
        }

        Ok("Commit-msg hook completed successfully".to_string())
    }

    /// Handle pre-push hook
    async fn handle_pre_push(&self, context: &HookContext) -> Result<String> {
        debug!("Executing pre-push hook");

        // Parse push information from stdin
        let push_info = self.parse_push_info(&context.stdin_data)?;

        // Validate push
        for info in &push_info {
            self.validate_push(info).await?;
        }

        Ok("Pre-push hook completed successfully".to_string())
    }

    /// Handle post-merge hook
    async fn handle_post_merge(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-merge hook");

        if let Some(squash_flag) = context.args.get(0) {
            info!("Merge completed, squash: {}", squash_flag);
        }

        Ok("Post-merge hook completed successfully".to_string())
    }

    /// Handle post-checkout hook
    async fn handle_post_checkout(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-checkout hook");

        if context.args.len() >= 3 {
            let prev_head = &context.args[0];
            let new_head = &context.args[1];
            let branch_flag = &context.args[2];

            info!(
                "Checkout completed: {} -> {} (branch: {})",
                prev_head, new_head, branch_flag
            );
        }

        Ok("Post-checkout hook completed successfully".to_string())
    }

    /// Handle pre-rebase hook
    async fn handle_pre_rebase(&self, context: &HookContext) -> Result<String> {
        debug!("Executing pre-rebase hook");

        if context.args.len() >= 1 {
            let upstream = &context.args[0];
            info!("Preparing to rebase onto: {}", upstream);
        }

        Ok("Pre-rebase hook completed successfully".to_string())
    }

    /// Handle post-rewrite hook
    async fn handle_post_rewrite(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-rewrite hook");

        if let Some(command) = context.args.get(0) {
            info!("Post-rewrite hook for command: {}", command);
        }

        Ok("Post-rewrite hook completed successfully".to_string())
    }

    /// Handle reference transaction hook
    async fn handle_reference_transaction(&self, context: &HookContext) -> Result<String> {
        debug!("Executing reference transaction hook");

        if let Some(state) = context.args.get(0) {
            info!("Reference transaction state: {}", state);
        }

        Ok("Reference transaction hook completed successfully".to_string())
    }

    /// Handle push-to-checkout hook
    async fn handle_push_to_checkout(&self, context: &HookContext) -> Result<String> {
        debug!("Executing push-to-checkout hook");

        if let Some(commit_id) = context.args.get(0) {
            info!("Push to checkout for commit: {}", commit_id);
        }

        Ok("Push-to-checkout hook completed successfully".to_string())
    }

    /// Handle pre-auto-gc hook
    async fn handle_pre_auto_gc(&self, _context: &HookContext) -> Result<String> {
        debug!("Executing pre-auto-gc hook");

        Ok("Pre-auto-gc hook completed successfully".to_string())
    }

    /// Handle sendemail-validate hook
    async fn handle_sendemail_validate(&self, context: &HookContext) -> Result<String> {
        debug!("Executing sendemail-validate hook");

        if context.args.len() >= 2 {
            let email_file = &context.args[0];
            let headers_file = &context.args[1];

            debug!(
                "Validating email: {} with headers: {}",
                email_file, headers_file
            );
        }

        Ok("Sendemail-validate hook completed successfully".to_string())
    }

    /// Handle fsmonitor-watchman hook
    async fn handle_fsmonitor_watchman(&self, context: &HookContext) -> Result<String> {
        debug!("Executing fsmonitor-watchman hook");

        if context.args.len() >= 2 {
            let version = &context.args[0];
            let token = &context.args[1];

            debug!("Fsmonitor version: {}, token: {}", version, token);
        }

        Ok("Fsmonitor-watchman hook completed successfully".to_string())
    }

    /// Handle p4-changelist hook
    async fn handle_p4_changelist(&self, context: &HookContext) -> Result<String> {
        debug!("Executing p4-changelist hook");

        if let Some(changelist_file) = context.args.get(0) {
            debug!("P4 changelist file: {}", changelist_file);
        }

        Ok("P4-changelist hook completed successfully".to_string())
    }

    /// Handle p4-prepare-changelist hook
    async fn handle_p4_prepare_changelist(&self, context: &HookContext) -> Result<String> {
        debug!("Executing p4-prepare-changelist hook");

        if let Some(changelist_file) = context.args.get(0) {
            debug!("P4 prepare changelist file: {}", changelist_file);
        }

        Ok("P4-prepare-changelist hook completed successfully".to_string())
    }

    /// Handle p4-post-changelist hook
    async fn handle_p4_post_changelist(&self, _context: &HookContext) -> Result<String> {
        debug!("Executing p4-post-changelist hook");

        Ok("P4-post-changelist hook completed successfully".to_string())
    }

    /// Handle p4-pre-submit hook
    async fn handle_p4_pre_submit(&self, _context: &HookContext) -> Result<String> {
        debug!("Executing p4-pre-submit hook");

        Ok("P4-pre-submit hook completed successfully".to_string())
    }

    /// Handle post-index-change hook
    async fn handle_post_index_change(&self, context: &HookContext) -> Result<String> {
        debug!("Executing post-index-change hook");

        if context.args.len() >= 2 {
            let worktree_updated = &context.args[0];
            let index_updated = &context.args[1];

            debug!(
                "Index change - worktree: {}, index: {}",
                worktree_updated, index_updated
            );
        }

        Ok("Post-index-change hook completed successfully".to_string())
    }

    /// Parse ref updates from stdin
    fn parse_ref_updates(&self, stdin_data: &Option<String>) -> Result<Vec<RefUpdate>> {
        let mut updates = Vec::new();

        if let Some(data) = stdin_data {
            for line in data.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    updates.push(RefUpdate {
                        old_oid: parts[0].to_string(),
                        new_oid: parts[1].to_string(),
                        ref_name: parts[2].to_string(),
                    });
                }
            }
        }

        Ok(updates)
    }

    /// Parse push information from stdin
    fn parse_push_info(&self, stdin_data: &Option<String>) -> Result<Vec<PushInfo>> {
        let mut push_info = Vec::new();

        if let Some(data) = stdin_data {
            for line in data.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    push_info.push(PushInfo {
                        local_ref: parts[0].to_string(),
                        local_oid: parts[1].to_string(),
                        remote_ref: parts[2].to_string(),
                        remote_oid: parts[3].to_string(),
                    });
                }
            }
        }

        Ok(push_info)
    }

    /// Validate a ref update
    async fn validate_ref_update(&self, update: &RefUpdate) -> Result<()> {
        debug!(
            "Validating ref update: {} -> {}",
            update.old_oid, update.new_oid
        );

        // Check for force push
        if update.old_oid != "0000000000000000000000000000000000000000" {
            // This is not a new ref, check if it's a force push
            if let Some(repo) = &self.repository {
                if let (Ok(old_commit), Ok(new_commit)) = (
                    repo.find_commit(Oid::from_str(&update.old_oid)?),
                    repo.find_commit(Oid::from_str(&update.new_oid)?),
                ) {
                    if !repo.graph_descendant_of(new_commit.id(), old_commit.id())? {
                        warn!("Force push detected for ref: {}", update.ref_name);
                    }
                }
            }
        }

        // Check protected branches
        if self.is_protected_branch(&update.ref_name) {
            info!("Protected branch update: {}", update.ref_name);
        }

        Ok(())
    }

    /// Validate a push
    async fn validate_push(&self, push_info: &PushInfo) -> Result<()> {
        debug!(
            "Validating push: {} -> {}",
            push_info.local_ref, push_info.remote_ref
        );

        // Check for deletion
        if push_info.local_oid == "0000000000000000000000000000000000000000" {
            warn!("Branch deletion detected: {}", push_info.remote_ref);
        }

        // Check protected branches
        if self.is_protected_branch(&push_info.remote_ref) {
            info!("Push to protected branch: {}", push_info.remote_ref);
        }

        Ok(())
    }

    /// Validate working tree
    async fn validate_working_tree(&self) -> Result<()> {
        debug!("Validating working tree");

        // Could add working tree validation here
        // For example, check for uncommitted changes, file size limits, etc.

        Ok(())
    }

    /// Check if a ref is a protected branch
    fn is_protected_branch(&self, ref_name: &str) -> bool {
        let protected_branches = vec!["main", "master", "develop", "production"];

        for protected in &protected_branches {
            if ref_name.ends_with(protected) {
                return true;
            }
        }

        false
    }
}

/// Ref update information
#[derive(Debug, Clone)]
struct RefUpdate {
    old_oid: String,
    new_oid: String,
    ref_name: String,
}

/// Push information
#[derive(Debug, Clone)]
struct PushInfo {
    local_ref: String,
    local_oid: String,
    remote_ref: String,
    remote_oid: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_hook_handler_creation() {
        let config = GitProxyConfig::default();
        let handler = ServerHookHandler::new(config);

        assert!(handler
            .hooks_enabled
            .contains_key(&ServerHookType::PreReceive));
        assert!(handler
            .hooks_enabled
            .contains_key(&ServerHookType::PostReceive));
    }

    #[test]
    fn test_protected_branch_detection() {
        let config = GitProxyConfig::default();
        let handler = ServerHookHandler::new(config);

        assert!(handler.is_protected_branch("refs/heads/main"));
        assert!(handler.is_protected_branch("refs/heads/master"));
        assert!(!handler.is_protected_branch("refs/heads/feature"));
    }

    #[tokio::test]
    async fn test_hook_execution() {
        let config = GitProxyConfig::default();
        let handler = ServerHookHandler::new(config);

        let context = HookContext {
            hook_type: ServerHookType::PreReceive,
            repo_path: PathBuf::from("/tmp/test"),
            work_dir: None,
            env_vars: HashMap::new(),
            args: vec![],
            stdin_data: Some("old new refs/heads/main".to_string()),
            client_info: None,
            timestamp: Utc::now(),
        };

        let result = handler.execute_hook(context).await.unwrap();
        assert!(result.success);
    }
}
