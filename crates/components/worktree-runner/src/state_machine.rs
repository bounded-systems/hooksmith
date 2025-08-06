use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, error, info, warn};

use crate::kube_crd::{
    WorktreeChangeRequest, WorktreeAction, WorktreeState, PrState,
};

/// State machine engine for worktree lifecycle management
pub struct WorktreeStateMachine {
    repo_path: PathBuf,
    worktree_base: PathBuf,
    github_token: Option<String>,
}

impl WorktreeStateMachine {
    /// Create a new state machine engine
    pub fn new(repo_path: PathBuf, worktree_base: PathBuf, github_token: Option<String>) -> Self {
        Self {
            repo_path,
            worktree_base,
            github_token,
        }
    }

    /// Scan all branches and create/update CRDs
    pub async fn scan_and_reconcile(&mut self) -> Result<Vec<WorktreeChangeRequest>> {
        info!("Starting worktree reconciliation scan");
        
        let mut crds = Vec::new();
        
        // Get all local branches
        let local_branches = self.get_local_branches().await?;
        debug!("Found {} local branches", local_branches.len());
        
        // Get all remote branches
        let remote_branches = self.get_remote_branches().await?;
        debug!("Found {} remote branches", remote_branches.len());
        
        // Get all worktrees
        let worktrees = self.get_worktrees().await?;
        debug!("Found {} worktrees", worktrees.len());
        
        // Get all PRs
        let prs = self.get_pull_requests().await?;
        debug!("Found {} pull requests", prs.len());
        
        // Create CRDs for all branches
        let all_branches = self.merge_branch_lists(&local_branches, &remote_branches);
        
        for branch in all_branches {
            let mut crd = WorktreeChangeRequest::create(&branch);
            
            // Populate domain states
            self.populate_local_domain(&mut crd, &local_branches).await;
            self.populate_remote_domain(&mut crd, &remote_branches).await;
            self.populate_worktree_domain(&mut crd, &worktrees).await;
            self.populate_pr_domain(&mut crd, &prs).await;
            
            // Determine current state
            self.determine_state(&mut crd).await;
            
            // Determine next action
            crd.spec.action = crd.determine_next_action();
            
            crds.push(crd);
        }
        
        info!("Created {} CRDs for reconciliation", crds.len());
        Ok(crds)
    }

    /// Execute actions for all CRDs that need attention
    pub async fn execute_actions(&mut self, crds: &mut [WorktreeChangeRequest]) -> Result<()> {
        info!("Executing actions for {} CRDs", crds.len());
        
        // Sort by priority (lower number = higher priority)
        crds.sort_by(|a, b| a.spec.priority.cmp(&b.spec.priority));
        
        for crd in crds.iter_mut() {
            let action = crd.spec.action.clone();
            if let Some(action) = action {
                info!("Executing action {:?} for branch {}", action, crd.spec.branch);
                
                // Note: Kubernetes CRD status is managed separately
                // For now, we'll just execute the action and log the result
                let result = self.execute_action(crd, &action).await;
                
                match result {
                    Ok(success) => {
                        if success {
                            info!("Action completed successfully for branch {}", crd.spec.branch);
                        } else {
                            error!("Action failed for branch {}", crd.spec.branch);
                        }
                    }
                    Err(e) => {
                        error!("Action failed for branch {}: {}", crd.spec.branch, e);
                        crd.spec.retry_count += 1;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Execute a specific action for a CRD
    async fn execute_action(&mut self, crd: &mut WorktreeChangeRequest, action: &WorktreeAction) -> Result<bool> {
        match action {
            WorktreeAction::CreateBranch => {
                self.create_branch(&crd.spec.branch).await
            }
            WorktreeAction::CreateWorktree => {
                self.create_worktree(&crd.spec.branch).await
            }
            WorktreeAction::PushBranch => {
                self.push_branch(&crd.spec.branch).await
            }
            WorktreeAction::CreatePr => {
                self.create_pull_request(&crd.spec.branch).await
            }
            WorktreeAction::MergePr => {
                self.merge_pull_request(crd).await
            }
            WorktreeAction::ResolveConflicts => {
                self.resolve_conflicts(&crd.spec.branch).await
            }
            WorktreeAction::RebaseMain => {
                self.rebase_main(&crd.spec.branch).await
            }
            WorktreeAction::CleanupWorktree => {
                self.cleanup_worktree(&crd.spec.branch).await
            }
            WorktreeAction::RemoveBranch => {
                self.remove_branch(&crd.spec.branch).await
            }
            WorktreeAction::ResetMain => {
                self.reset_main().await
            }
            // New enhanced actions
            WorktreeAction::ExtractFeature => {
                self.extract_feature(&crd.spec.branch).await
            }
            WorktreeAction::RecoverDirtyMain => {
                self.recover_dirty_main().await
            }
            WorktreeAction::SquashCommits => {
                self.squash_commits(&crd.spec.branch).await
            }
            WorktreeAction::UpdateBranch => {
                self.update_branch(&crd.spec.branch).await
            }
            WorktreeAction::SyncRemote => {
                self.sync_remote(&crd.spec.branch).await
            }
            WorktreeAction::MarkStale => {
                self.mark_stale(&crd.spec.branch).await
            }
            WorktreeAction::ForcePush => {
                self.force_push(&crd.spec.branch).await
            }
            WorktreeAction::AbortRebase => {
                self.abort_rebase(&crd.spec.branch).await
            }
            WorktreeAction::StashChanges => {
                self.stash_changes(&crd.spec.branch).await
            }
            WorktreeAction::ApplyStash => {
                self.apply_stash(&crd.spec.branch).await
            }
            WorktreeAction::CreateBackup => {
                self.create_backup(&crd.spec.branch).await
            }
            WorktreeAction::RestoreBackup => {
                self.restore_backup(&crd.spec.branch).await
            }
            WorktreeAction::ValidateBranch => {
                self.validate_branch(&crd.spec.branch).await
            }
            WorktreeAction::RunTests => {
                self.run_tests(&crd.spec.branch).await
            }
            WorktreeAction::DeployPreview => {
                self.deploy_preview(&crd.spec.branch).await
            }
        }
    }

    /// Get all local branches
    async fn get_local_branches(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["branch", "--format=%(refname:short)"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get local branches")?;
        
        let branches = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .filter(|s| s != "main" && s != "master")
            .collect();
        
        Ok(branches)
    }

    /// Get all remote branches
    async fn get_remote_branches(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["branch", "-r", "--format=%(refname:short)"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get remote branches")?;
        
        let branches = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .filter(|s| s.starts_with("origin/") && !s.contains("main") && !s.contains("master"))
            .map(|s| s.trim_start_matches("origin/").to_string())
            .collect();
        
        Ok(branches)
    }

    /// Get all worktrees
    async fn get_worktrees(&self) -> Result<HashMap<String, PathBuf>> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get worktrees")?;
        
        let mut worktrees = HashMap::new();
        let output_str = String::from_utf8(output.stdout)?;
        let lines: Vec<&str> = output_str.lines().collect();
        
        let mut i = 0;
        while i < lines.len() {
            if lines[i].starts_with("worktree ") {
                let path = lines[i].trim_start_matches("worktree ");
                if i + 1 < lines.len() && lines[i + 1].starts_with("branch ") {
                    let branch = lines[i + 1].trim_start_matches("branch ");
                    worktrees.insert(branch.to_string(), PathBuf::from(path));
                }
            }
            i += 1;
        }
        
        Ok(worktrees)
    }

    /// Get all pull requests
    async fn get_pull_requests(&self) -> Result<HashMap<String, (i32, PrState)>> {
        if self.github_token.is_none() {
            warn!("No GitHub token provided, skipping PR detection");
            return Ok(HashMap::new());
        }
        
        let output = Command::new("gh")
            .args(["pr", "list", "--json", "number,headRefName,state"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get pull requests")?;
        
        // Parse JSON output from gh CLI
        let prs_json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let mut prs = HashMap::new();
        
        if let Some(prs_array) = prs_json.as_array() {
            for pr in prs_array {
                if let (Some(number), Some(head_ref), Some(state)) = (
                    pr["number"].as_i64(),
                    pr["headRefName"].as_str(),
                    pr["state"].as_str(),
                ) {
                    let pr_state = match state {
                        "OPEN" => PrState::Open,
                        "CLOSED" => PrState::Closed,
                        "MERGED" => PrState::Merged,
                        _ => PrState::Open,
                    };
                    prs.insert(head_ref.to_string(), (number as i32, pr_state));
                }
            }
        }
        
        Ok(prs)
    }

    /// Merge local and remote branch lists
    fn merge_branch_lists(&self, local: &[String], remote: &[String]) -> Vec<String> {
        let mut all_branches = std::collections::HashSet::new();
        all_branches.extend(local.iter().cloned());
        all_branches.extend(remote.iter().cloned());
        all_branches.into_iter().collect()
    }

    /// Populate local domain information
    async fn populate_local_domain(&mut self, crd: &mut WorktreeChangeRequest, local_branches: &[String]) {
        let branch = &crd.spec.branch;
        let exists = local_branches.contains(branch);
        
        crd.spec.domains.local.exists = exists;
        
        if exists {
            // Get current branch
            if let Ok(output) = Command::new("git")
                .args(["branch", "--show-current"])
                .current_dir(&self.repo_path)
                .output()
            {
                let current_str = String::from_utf8_lossy(&output.stdout);
                let current = current_str.trim();
                crd.spec.domains.local.current = current == branch;
            }
            
            // Get commit info
            if let Ok(output) = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&self.repo_path)
                .output()
            {
                let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                crd.spec.domains.local.last_commit = Some(commit);
            }
            
            // Get ahead/behind info
            if let Ok(output) = Command::new("git")
                .args(["rev-list", "--count", "main..HEAD"])
                .current_dir(&self.repo_path)
                .output()
            {
                if let Ok(ahead) = String::from_utf8_lossy(&output.stdout).trim().parse::<i32>() {
                    crd.spec.domains.local.ahead = ahead;
                }
            }
            
            if let Ok(output) = Command::new("git")
                .args(["rev-list", "--count", "HEAD..main"])
                .current_dir(&self.repo_path)
                .output()
            {
                if let Ok(behind) = String::from_utf8_lossy(&output.stdout).trim().parse::<i32>() {
                    crd.spec.domains.local.behind = behind;
                }
            }
        }
    }

    /// Populate remote domain information
    async fn populate_remote_domain(&mut self, crd: &mut WorktreeChangeRequest, remote_branches: &[String]) {
        let branch = &crd.spec.branch;
        let exists = remote_branches.contains(branch);
        
        crd.spec.domains.remote.exists = exists;
        
        if exists {
            crd.spec.domains.remote.upstream = Some(format!("origin/{}", branch));
            
            // Get remote commit
            if let Ok(output) = Command::new("git")
                .args(["rev-parse", &format!("origin/{}", branch)])
                .current_dir(&self.repo_path)
                .output()
            {
                let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
                crd.spec.domains.remote.last_commit = Some(commit);
            }
        }
    }

    /// Populate worktree domain information
    async fn populate_worktree_domain(&mut self, crd: &mut WorktreeChangeRequest, worktrees: &HashMap<String, PathBuf>) {
        let branch = &crd.spec.branch;
        let exists = worktrees.contains_key(branch);
        
        crd.spec.domains.worktree.exists = exists;
        
        if exists {
            let worktree_path = worktrees.get(branch).unwrap();
            crd.spec.domains.worktree.path = Some(worktree_path.to_string_lossy().to_string());
            
            // Check if worktree is dirty
            if let Ok(output) = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(worktree_path)
                .output()
            {
                crd.spec.domains.worktree.dirty = !output.stdout.is_empty();
            }
            
            // Check for conflicts
            if let Ok(output) = Command::new("git")
                .args(["status"])
                .current_dir(worktree_path)
                .output()
            {
                let status = String::from_utf8_lossy(&output.stdout);
                crd.spec.domains.worktree.conflicted = status.contains("You have unmerged paths") || 
                                                       status.contains("All conflicts fixed");
                crd.spec.domains.worktree.rebase_in_progress = status.contains("rebase in progress");
            }
        }
    }

    /// Populate PR domain information
    async fn populate_pr_domain(&mut self, crd: &mut WorktreeChangeRequest, prs: &HashMap<String, (i32, PrState)>) {
        let branch = &crd.spec.branch;
        let exists = prs.contains_key(branch);
        
        crd.spec.domains.pr.exists = exists;
        
        if exists {
            let (number, state) = prs.get(branch).unwrap();
            crd.spec.domains.pr.number = Some(*number);
            crd.spec.domains.pr.state = Some(state.clone());
            crd.spec.domains.pr.url = Some(format!("https://github.com/owner/repo/pull/{}", number));
        }
    }

    /// Determine the current state based on domain information
    async fn determine_state(&mut self, crd: &mut WorktreeChangeRequest) {
        let domains = &crd.spec.domains;
        
        // Check for merged state first
        if let Some(PrState::Merged) = domains.pr.state {
            crd.spec.state = WorktreeState::Merged;
            return;
        }
        
        // Check for dirty main recovery
        if domains.local.current && domains.local.dirty && crd.spec.branch == "main" {
            crd.spec.state = WorktreeState::DirtyMainRecovery;
            return;
        }
        
        // Check for orphaned worktree
        if domains.worktree.exists && !domains.local.exists {
            crd.spec.state = WorktreeState::OrphanedWorktree;
            return;
        }
        
        // Check for stale branch
        if domains.local.stale {
            crd.spec.state = WorktreeState::StaleBranch;
            return;
        }
        
        // Check for PR created state
        if domains.pr.exists {
            if let Some(PrState::Approved) = domains.pr.state {
                crd.spec.state = WorktreeState::Approved;
            } else if let Some(PrState::Blocked) = domains.pr.state {
                crd.spec.state = WorktreeState::Blocked;
            } else {
                crd.spec.state = WorktreeState::PrCreated;
            }
            return;
        }
        
        // Check for conflicts
        if domains.worktree.conflicted || domains.worktree.rebase_in_progress {
            crd.spec.state = WorktreeState::Conflicted;
            return;
        }
        
        // Check for needs rebase
        if domains.local.behind > 0 && domains.local.ahead > 0 {
            crd.spec.state = WorktreeState::NeedsRebase;
            return;
        }
        
        // Check for needs squash (too many commits)
        if domains.local.ahead > 10 {
            crd.spec.state = WorktreeState::NeedsSquash;
            return;
        }
        
        // Check for developing state (dirty worktree)
        if domains.worktree.dirty {
            crd.spec.state = WorktreeState::Developing;
            return;
        }
        
        // Check for ready for review
        if domains.local.exists && domains.local.ahead > 0 && domains.local.behind == 0 && !domains.pr.exists {
            crd.spec.state = WorktreeState::ReadyForReview;
            return;
        }
        
        // Check for ready state (clean, ahead of main)
        if domains.local.exists && domains.local.ahead > 0 && domains.local.behind == 0 {
            crd.spec.state = WorktreeState::Ready;
            return;
        }
        
        // Check for resolving state (behind main)
        if domains.local.behind > 0 {
            crd.spec.state = WorktreeState::Resolving;
            return;
        }
        
        // Default to created state
        crd.spec.state = WorktreeState::Created;
    }

    // Action implementations
    async fn create_branch(&self, branch: &str) -> Result<bool> {
        info!("Creating branch: {}", branch);
        
        let output = Command::new("git")
            .args(["checkout", "-b", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create branch")?;
        
        Ok(output.status.success())
    }

    async fn create_worktree(&self, branch: &str) -> Result<bool> {
        info!("Creating worktree for branch: {}", branch);
        
        let worktree_path = self.worktree_base.join(branch);
        
        let output = Command::new("git")
            .args(["worktree", "add", worktree_path.to_str().unwrap(), branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create worktree")?;
        
        Ok(output.status.success())
    }

    async fn push_branch(&self, branch: &str) -> Result<bool> {
        info!("Pushing branch: {}", branch);
        
        let output = Command::new("git")
            .args(["push", "origin", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to push branch")?;
        
        Ok(output.status.success())
    }

    async fn create_pull_request(&self, branch: &str) -> Result<bool> {
        info!("Creating PR for branch: {}", branch);
        
        if self.github_token.is_none() {
            warn!("No GitHub token provided, skipping PR creation");
            return Ok(false);
        }
        
        let output = Command::new("gh")
            .args(["pr", "create", "--title", &format!("{}", branch), "--body", "Auto-generated PR", "--draft"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create PR")?;
        
        Ok(output.status.success())
    }

    async fn merge_pull_request(&self, crd: &WorktreeChangeRequest) -> Result<bool> {
        info!("Merging PR for branch: {}", crd.spec.branch);
        
        if let Some(number) = crd.spec.domains.pr.number {
            let output = Command::new("gh")
                .args(["pr", "merge", &number.to_string(), "--squash"])
                .current_dir(&self.repo_path)
                .output()
                .context("Failed to merge PR")?;
            
            Ok(output.status.success())
        } else {
            Ok(false)
        }
    }

    async fn resolve_conflicts(&self, branch: &str) -> Result<bool> {
        info!("Resolving conflicts for branch: {}", branch);
        
        // This is a placeholder - actual conflict resolution would be more complex
        // In a real implementation, you might want to abort the rebase and let the user handle it
        let output = Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to abort rebase")?;
        
        Ok(output.status.success())
    }

    async fn rebase_main(&self, branch: &str) -> Result<bool> {
        info!("Rebasing branch {} onto main", branch);
        
        let output = Command::new("git")
            .args(["rebase", "main"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to rebase main")?;
        
        Ok(output.status.success())
    }

    async fn cleanup_worktree(&self, branch: &str) -> Result<bool> {
        info!("Cleaning up worktree for branch: {}", branch);
        
        let worktree_path = self.worktree_base.join(branch);
        
        if worktree_path.exists() {
            let output = Command::new("git")
                .args(["worktree", "remove", worktree_path.to_str().unwrap()])
                .current_dir(&self.repo_path)
                .output()
                .context("Failed to remove worktree")?;
            
            Ok(output.status.success())
        } else {
            Ok(true) // Already cleaned up
        }
    }

    async fn remove_branch(&self, branch: &str) -> Result<bool> {
        info!("Removing branch: {}", branch);
        
        let output = Command::new("git")
            .args(["branch", "-D", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to remove branch")?;
        
        Ok(output.status.success())
    }

    async fn reset_main(&self) -> Result<bool> {
        info!("Resetting main to match origin/main");
        
        let output = Command::new("git")
            .args(["reset", "--hard", "origin/main"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to reset main")?;
        
        Ok(output.status.success())
    }

    // Enhanced action implementations
    async fn extract_feature(&self, branch: &str) -> Result<bool> {
        info!("Extracting feature from branch: {}", branch);
        
        // Create a new branch for the extracted feature
        let feature_branch = format!("feature/extracted-{}", branch);
        
        let output = Command::new("git")
            .args(["checkout", "-b", &feature_branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create feature branch")?;
        
        Ok(output.status.success())
    }

    async fn recover_dirty_main(&self) -> Result<bool> {
        info!("Recovering dirty main branch");
        
        // Stash changes first
        let stash_output = Command::new("git")
            .args(["stash", "push", "-m", "Dirty main recovery"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to stash changes")?;
        
        if !stash_output.status.success() {
            return Ok(false);
        }
        
        // Reset main to origin/main
        let reset_output = Command::new("git")
            .args(["reset", "--hard", "origin/main"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to reset main")?;
        
        Ok(reset_output.status.success())
    }

    async fn squash_commits(&self, branch: &str) -> Result<bool> {
        info!("Squashing commits for branch: {}", branch);
        
        // Interactive rebase to squash commits
        let output = Command::new("git")
            .args(["rebase", "-i", "main"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to squash commits")?;
        
        Ok(output.status.success())
    }

    async fn update_branch(&self, branch: &str) -> Result<bool> {
        info!("Updating branch: {}", branch);
        
        // Fetch latest changes
        let fetch_output = Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to fetch")?;
        
        if !fetch_output.status.success() {
            return Ok(false);
        }
        
        // Rebase onto origin/main
        let rebase_output = Command::new("git")
            .args(["rebase", "origin/main"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to rebase")?;
        
        Ok(rebase_output.status.success())
    }

    async fn sync_remote(&self, branch: &str) -> Result<bool> {
        info!("Syncing remote for branch: {}", branch);
        
        // Push to remote
        let push_output = Command::new("git")
            .args(["push", "origin", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to push")?;
        
        Ok(push_output.status.success())
    }

    async fn mark_stale(&self, branch: &str) -> Result<bool> {
        info!("Marking branch as stale: {}", branch);
        
        // Add stale label to PR if it exists
        if let Some(pr_number) = self.get_pr_number(branch).await? {
            let output = Command::new("gh")
                .args(["pr", "edit", &pr_number.to_string(), "--add-label", "stale"])
                .current_dir(&self.repo_path)
                .output()
                .context("Failed to add stale label")?;
            
            Ok(output.status.success())
        } else {
            Ok(true) // No PR to mark
        }
    }

    async fn force_push(&self, branch: &str) -> Result<bool> {
        info!("Force pushing branch: {}", branch);
        
        let output = Command::new("git")
            .args(["push", "--force-with-lease", "origin", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to force push")?;
        
        Ok(output.status.success())
    }

    async fn abort_rebase(&self, branch: &str) -> Result<bool> {
        info!("Aborting rebase for branch: {}", branch);
        
        let output = Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to abort rebase")?;
        
        Ok(output.status.success())
    }

    async fn stash_changes(&self, branch: &str) -> Result<bool> {
        info!("Stashing changes for branch: {}", branch);
        
        let output = Command::new("git")
            .args(["stash", "push", "-m", &format!("Auto-stash for {}", branch)])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to stash changes")?;
        
        Ok(output.status.success())
    }

    async fn apply_stash(&self, branch: &str) -> Result<bool> {
        info!("Applying stash for branch: {}", branch);
        
        let output = Command::new("git")
            .args(["stash", "pop"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to apply stash")?;
        
        Ok(output.status.success())
    }

    async fn create_backup(&self, branch: &str) -> Result<bool> {
        info!("Creating backup for branch: {}", branch);
        
        let backup_branch = format!("backup/{}", branch);
        
        let output = Command::new("git")
            .args(["branch", &backup_branch, branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to create backup branch")?;
        
        Ok(output.status.success())
    }

    async fn restore_backup(&self, branch: &str) -> Result<bool> {
        info!("Restoring backup for branch: {}", branch);
        
        let backup_branch = format!("backup/{}", branch);
        
        let output = Command::new("git")
            .args(["reset", "--hard", &backup_branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to restore backup")?;
        
        Ok(output.status.success())
    }

    async fn validate_branch(&self, branch: &str) -> Result<bool> {
        info!("Validating branch: {}", branch);
        
        // Run basic validation checks
        let output = Command::new("git")
            .args(["log", "--oneline", "-n", "1", branch])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to validate branch")?;
        
        Ok(output.status.success())
    }

    async fn run_tests(&self, branch: &str) -> Result<bool> {
        info!("Running tests for branch: {}", branch);
        
        // This would typically run cargo test or other test commands
        let output = Command::new("cargo")
            .args(["test"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to run tests")?;
        
        Ok(output.status.success())
    }

    async fn deploy_preview(&self, branch: &str) -> Result<bool> {
        info!("Deploying preview for branch: {}", branch);
        
        // This would typically trigger a deployment pipeline
        // For now, just return success
        Ok(true)
    }

    /// Helper method to get PR number for a branch
    async fn get_pr_number(&self, branch: &str) -> Result<Option<i32>> {
        if self.github_token.is_none() {
            return Ok(None);
        }
        
        let output = Command::new("gh")
            .args(["pr", "list", "--head", branch, "--json", "number"])
            .current_dir(&self.repo_path)
            .output()
            .context("Failed to get PR number")?;
        
        if let Ok(prs_json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            if let Some(prs_array) = prs_json.as_array() {
                if let Some(first_pr) = prs_array.first() {
                    if let Some(number) = first_pr["number"].as_i64() {
                        return Ok(Some(number as i32));
                    }
                }
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_state_machine_creation() {
        let temp_dir = tempdir().unwrap();
        let worktree_base = temp_dir.path().join("worktrees");
        
        let sm = WorktreeStateMachine::new(
            temp_dir.path().to_path_buf(),
            worktree_base,
            None,
        );
        
        assert_eq!(sm.repo_path, temp_dir.path());
    }
} 
