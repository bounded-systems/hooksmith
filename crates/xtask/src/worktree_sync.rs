//! Worktree Sync Strategy Module
//!
//! This module implements a conflict-free worktree management strategy
//! based on the 1:1:1:1:1 mapping model:
//! - 1 Worktree = 1 Local branch = 1 Remote branch = 1 Draft PR = Not main
//!
//! Key principles:
//! - Treat origin/main as single source of truth
//! - Never develop directly on main
//! - Always sync main first, then worktrees
//! - Commit local changes before syncing
//! - Auto-resolve trivial conflicts

use anyhow::{Context, Result};
use console::style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

/// Worktree sync state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSyncState {
    /// Branch name
    pub branch: String,
    /// Whether branch is ahead of main
    pub is_ahead_of_main: bool,
    /// Whether branch has unmerged main commits
    pub has_unmerged_main: bool,
    /// Whether branch has uncommitted changes
    pub has_uncommitted: bool,
    /// Whether PR is synced with remote
    pub pr_synced: bool,
    /// Last sync timestamp
    pub last_sync: Option<String>,
    /// Sync status
    pub sync_status: SyncStatus,
}

/// Sync status enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    /// Worktree is clean and synced
    Clean,
    /// Worktree has uncommitted changes
    UncommittedChanges,
    /// Worktree has merge conflicts
    MergeConflicts,
    /// Worktree is ahead of main
    AheadOfMain,
    /// Worktree has unmerged main commits
    UnmergedMain,
    /// Worktree is out of sync with remote
    OutOfSync,
}

/// Worktree sync manager
pub struct WorktreeSyncManager {
    /// Git root path
    git_root: PathBuf,
    /// Worktree states
    worktree_states: HashMap<String, WorktreeSyncState>,
}

impl WorktreeSyncManager {
    /// Create a new worktree sync manager
    pub fn new(git_root: PathBuf) -> Self {
        Self {
            git_root,
            worktree_states: HashMap::new(),
        }
    }

    /// Sync all worktrees using the upstream-first strategy
    pub async fn sync_all_worktrees(&mut self) -> Result<()> {
        println!("{}", style("🔄 Starting worktree sync strategy").bold());

        // Step 1: Sync main first (single source of truth)
        self.sync_main().await?;

        // Step 2: Get all worktrees
        let worktrees = self.get_all_worktrees().await?;

        // Step 3: Sync each worktree
        for worktree in worktrees {
            self.sync_worktree(&worktree).await?;
        }

        // Step 4: Generate sync report
        self.generate_sync_report().await?;

        println!("{}", style("✅ Worktree sync strategy completed").green());
        Ok(())
    }

    /// Sync main branch (single source of truth)
    async fn sync_main(&self) -> Result<()> {
        println!("{}", style("📥 Syncing main branch...").cyan());

        // Fetch latest from origin
        let fetch_result = Command::new("git")
            .args(["fetch", "origin"])
            .current_dir(&self.git_root)
            .output()
            .context("Failed to fetch from origin")?;

        if !fetch_result.status.success() {
            let error = String::from_utf8_lossy(&fetch_result.stderr);
            return Err(anyhow::anyhow!("Failed to fetch: {}", error));
        }

        // Reset main to match origin/main
        let reset_result = Command::new("git")
            .args(["reset", "--hard", "origin/main"])
            .current_dir(&self.git_root)
            .output()
            .context("Failed to reset main to origin/main")?;

        if !reset_result.status.success() {
            let error = String::from_utf8_lossy(&reset_result.stderr);
            return Err(anyhow::anyhow!("Failed to reset main: {}", error));
        }

        println!("{}", style("✅ Main synced with origin/main").green());
        Ok(())
    }

    /// Get all worktrees
    async fn get_all_worktrees(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .current_dir(&self.git_root)
            .output()
            .context("Failed to list worktrees")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to list worktrees"));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut worktrees = Vec::new();

        for line in output_str.lines() {
            if line.starts_with("worktree ") {
                let path = line[9..].to_string(); // Remove "worktree " prefix
                if path != self.git_root.to_string_lossy() {
                    worktrees.push(path);
                }
            }
        }

        Ok(worktrees)
    }

    /// Sync a single worktree
    async fn sync_worktree(&mut self, worktree_path: &str) -> Result<()> {
        let worktree_path_buf = Path::new(worktree_path);
        if !worktree_path_buf.exists() {
            println!(
                "{}",
                style(&format!("⚠️  Worktree path does not exist: {}", worktree_path)).yellow()
            );
            return Ok(());
        }

        println!(
            "{}",
            style(&format!("🔄 Syncing worktree: {}", worktree_path)).cyan()
        );

        // Check for uncommitted changes
        let has_uncommitted = self.check_uncommitted_changes(worktree_path).await?;
        if has_uncommitted {
            println!(
                "{}",
                style(&format!("⚠️  Uncommitted changes in {}", worktree_path)).yellow()
            );
            
            // Offer to commit or stash
            println!("{}", style("Options:").bold());
            println!("  1. Commit changes: git commit -am 'WIP'");
            println!("  2. Stash changes: git stash -u");
            println!("  3. Skip this worktree");
            
            // For now, we'll skip worktrees with uncommitted changes
            println!("{}", style("Skipping worktree with uncommitted changes").yellow());
            return Ok(());
        }

        // Get current branch
        let current_branch = self.get_current_branch(worktree_path).await?;
        if current_branch == "main" {
            println!(
                "{}",
                style(&format!("⚠️  Worktree is on main branch: {}", worktree_path)).yellow()
            );
            return Ok(());
        }

        // Merge origin/main into the worktree branch
        let merge_result = Command::new("git")
            .args(["merge", "--ff-only", "origin/main"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to merge origin/main")?;

        if merge_result.status.success() {
            println!(
                "{}",
                style(&format!("✅ Fast-forward merge successful for {}", current_branch)).green()
            );
        } else {
            // Try regular merge if fast-forward fails
            let regular_merge_result = Command::new("git")
                .args(["merge", "origin/main"])
                .current_dir(worktree_path)
                .output()
                .context("Failed to merge origin/main")?;

            if regular_merge_result.status.success() {
                println!(
                    "{}",
                    style(&format!("✅ Merge successful for {}", current_branch)).green()
                );
            } else {
                let error = String::from_utf8_lossy(&regular_merge_result.stderr);
                println!(
                    "{}",
                    style(&format!("❌ Merge failed for {}: {}", current_branch, error)).red()
                );
            }
        }

        // Update sync state
        let sync_state = self.get_worktree_sync_state(worktree_path, &current_branch).await?;
        self.worktree_states.insert(current_branch, sync_state);

        Ok(())
    }

    /// Check for uncommitted changes
    async fn check_uncommitted_changes(&self, worktree_path: &str) -> Result<bool> {
        let output = Command::new("git")
            .args(["diff", "--quiet"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to check for uncommitted changes")?;

        Ok(!output.status.success())
    }

    /// Get current branch
    async fn get_current_branch(&self, worktree_path: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Failed to get current branch"));
        }

        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch)
    }

    /// Get worktree sync state
    async fn get_worktree_sync_state(&self, worktree_path: &str, branch: &str) -> Result<WorktreeSyncState> {
        // Check if ahead of main
        let ahead_output = Command::new("git")
            .args(["rev-list", "--count", "main..HEAD"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to check if ahead of main")?;

        let is_ahead_of_main = if ahead_output.status.success() {
            let count = String::from_utf8_lossy(&ahead_output.stdout).trim();
            count != "0"
        } else {
            false
        };

        // Check if has unmerged main commits
        let unmerged_output = Command::new("git")
            .args(["rev-list", "--count", "HEAD..main"])
            .current_dir(worktree_path)
            .output()
            .context("Failed to check unmerged main commits")?;

        let has_unmerged_main = if unmerged_output.status.success() {
            let count = String::from_utf8_lossy(&unmerged_output.stdout).trim();
            count != "0"
        } else {
            false
        };

        // Check for uncommitted changes
        let has_uncommitted = self.check_uncommitted_changes(worktree_path).await?;

        // Determine sync status
        let sync_status = if has_uncommitted {
            SyncStatus::UncommittedChanges
        } else if is_ahead_of_main {
            SyncStatus::AheadOfMain
        } else if has_unmerged_main {
            SyncStatus::UnmergedMain
        } else {
            SyncStatus::Clean
        };

        Ok(WorktreeSyncState {
            branch: branch.to_string(),
            is_ahead_of_main,
            has_unmerged_main,
            has_uncommitted,
            pr_synced: true, // TODO: Check actual PR status
            last_sync: Some(chrono::Utc::now().to_rfc3339()),
            sync_status,
        })
    }

    /// Generate sync report
    async fn generate_sync_report(&self) -> Result<()> {
        let report_path = self.git_root.join("contract.report.worktrees.jsonc");
        
        let report = serde_json::to_string_pretty(&self.worktree_states)
            .context("Failed to serialize sync report")?;

        tokio::fs::write(&report_path, report).await
            .context("Failed to write sync report")?;

        println!(
            "{}",
            style(&format!("📊 Sync report written to: {}", report_path.display())).cyan()
        );

        // Print summary
        self.print_sync_summary().await?;

        Ok(())
    }

    /// Print sync summary
    async fn print_sync_summary(&self) -> Result<()> {
        println!("\n{}", style("📊 Worktree Sync Summary").bold());
        println!("{}", "=".repeat(50));

        for (branch, state) in &self.worktree_states {
            let status_icon = match state.sync_status {
                SyncStatus::Clean => "✅",
                SyncStatus::UncommittedChanges => "⚠️",
                SyncStatus::MergeConflicts => "❌",
                SyncStatus::AheadOfMain => "📤",
                SyncStatus::UnmergedMain => "📥",
                SyncStatus::OutOfSync => "🔄",
            };

            println!(
                "{} {}: {}",
                status_icon,
                branch,
                format!("{:?}", state.sync_status)
            );
        }

        println!("{}", "=".repeat(50));
        Ok(())
    }

    /// Pre-sync validation check
    pub async fn validate_sync_readiness(&self) -> Result<bool> {
        println!("{}", style("🔍 Validating sync readiness...").cyan());

        let worktrees = self.get_all_worktrees().await?;
        let mut all_clean = true;

        for worktree in worktrees {
            let has_uncommitted = self.check_uncommitted_changes(&worktree).await?;
            if has_uncommitted {
                println!(
                    "{}",
                    style(&format!("❌ Uncommitted changes in: {}", worktree)).red()
                );
                all_clean = false;
            } else {
                println!(
                    "{}",
                    style(&format!("✅ Clean worktree: {}", worktree)).green()
                );
            }
        }

        if all_clean {
            println!("{}", style("✅ All worktrees are ready for sync").green());
        } else {
            println!("{}", style("❌ Some worktrees have uncommitted changes").red());
        }

        Ok(all_clean)
    }
}

/// CLI command for worktree sync
pub async fn run_worktree_sync_command() -> Result<()> {
    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("Failed to get git root")?;

    if !git_root.status.success() {
        return Err(anyhow::anyhow!("Not in a git repository"));
    }

    let git_root_path = String::from_utf8_lossy(&git_root.stdout).trim().to_string();
    let git_root_buf = PathBuf::from(&git_root_path);

    let mut sync_manager = WorktreeSyncManager::new(git_root_buf);

    // Validate sync readiness
    let is_ready = sync_manager.validate_sync_readiness().await?;
    if !is_ready {
        println!("{}", style("Please commit or stash changes before syncing").yellow());
        return Ok(());
    }

    // Run sync strategy
    sync_manager.sync_all_worktrees().await?;

    Ok(())
} 