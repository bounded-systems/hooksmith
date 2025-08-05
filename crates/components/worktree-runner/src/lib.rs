//! Worktree Runner WASM Component with CRD Lifecycle Management
//!
//! This component provides WASM interface for managing Git worktrees using various tools.
//! It supports multiple worktree management tools and provides a unified interface.
//! 
//! The CRD system provides synchronized, self-healing worktree lifecycle management
//! across four domains: Local Branch, Remote Branch, Worktree, and Pull Request.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::PathBuf;

// CRD system modules
pub mod crd;
pub mod state_machine;
pub mod storage;
pub mod tools;

// Kubernetes-style CRD system
pub mod kube_crd;

/// Configuration for worktree tools
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    /// Preferred tool to use (wtp, wt, treekanga, git)
    pub preferred_tool: Option<String>,
    /// Base directory for worktrees
    pub worktree_base: Option<String>,
    /// Whether to run setup commands after creation
    pub run_setup: bool,
    /// Setup commands to run (e.g., ["npm install", "cargo build"])
    pub setup_commands: Vec<String>,
    /// Whether to copy environment files
    pub copy_env: bool,
    /// Environment files to copy (e.g., [".env", ".env.local"])
    pub env_files: Vec<String>,
}

/// Result of a worktree operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Output from the command
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Worktree path if created
    pub worktree_path: Option<String>,
    /// Branch name if created
    pub branch_name: Option<String>,
}

/// Available worktree tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorktreeTool {
    Wtp,
    Wt,
    Treekanga,
    Git,
}

impl WorktreeTool {
    /// Get the command name for this tool
    pub fn command_name(&self) -> &'static str {
        match self {
            WorktreeTool::Wtp => "wtp",
            WorktreeTool::Wt => "wt",
            WorktreeTool::Treekanga => "treekanga",
            WorktreeTool::Git => "git",
        }
    }

    /// Check if this tool is available on the system
    pub fn is_available(&self) -> bool {
        // In WASM environment, we can't check for external tools
        // For now, assume git is always available
        matches!(self, WorktreeTool::Git)
    }
}

/// Worktree runner component
pub struct WorktreeRunner {
    config: ToolConfig,
    state_machine: Option<state_machine::WorktreeStateMachine>,
    storage: Option<storage::WorktreeStorage>,
    enhanced_ops: Option<tools::EnhancedWorktreeOps>,
}

impl WorktreeRunner {
    /// Create a new worktree runner with default configuration
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
            state_machine: None,
            storage: None,
            enhanced_ops: None,
        }
    }

    /// Create a new worktree runner with custom configuration
    pub fn with_config(config: ToolConfig) -> Self {
        Self { 
            config,
            state_machine: None,
            storage: None,
            enhanced_ops: None,
        }
    }

    /// Initialize the CRD lifecycle management system
    pub async fn init_crd_system(&mut self, repo_path: PathBuf, worktree_base: PathBuf, storage_dir: PathBuf, github_token: Option<String>) -> Result<()> {
        let state_machine = state_machine::WorktreeStateMachine::new(
            repo_path,
            worktree_base,
            github_token,
        );
        
        let storage = storage::WorktreeStorage::new(storage_dir);
        storage.init().await?;
        
        // Initialize enhanced operations with preferred tool
        let preferred_tool = if self.config.preferred_tool.is_some() {
            Some(tools::WorktreeTool::Workbloom) // Default to workbloom if configured
        } else {
            None
        };
        let enhanced_ops = tools::EnhancedWorktreeOps::new(preferred_tool);
        
        self.state_machine = Some(state_machine);
        self.storage = Some(storage);
        self.enhanced_ops = Some(enhanced_ops);
        
        Ok(())
    }

    /// Run a complete reconciliation cycle
    pub async fn reconcile(&mut self) -> Result<Vec<kube_crd::WorktreeChangeRequest>> {
        let state_machine = self.state_machine.as_mut()
            .ok_or_else(|| anyhow::anyhow!("CRD system not initialized"))?;
        
        let storage = self.storage.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;
        
        // Scan and create CRDs
        let mut crds = state_machine.scan_and_reconcile().await?;
        
        // Load existing CRDs from storage
        let stored_crds = storage.load_all_crds().await?;
        
        // Merge with existing CRDs
        for (branch_name, stored_crd) in stored_crds {
            if let Some(existing_crd) = crds.iter_mut().find(|c| c.spec.branch == branch_name) {
                // Update with stored history and status
                // Note: Kubernetes CRD status is managed separately
                // We'll just update the spec for now
                existing_crd.spec = stored_crd.spec;
            } else {
                // Add stored CRD that wasn't found in current scan
                crds.push(stored_crd);
            }
        }
        
        // Execute actions
        state_machine.execute_actions(&mut crds).await?;
        
        // Save updated CRDs
        for crd in &crds {
            storage.save_crd(crd).await?;
        }
        
        Ok(crds)
    }

    /// Get status of all worktrees
    pub async fn get_status(&self) -> Result<Vec<kube_crd::WorktreeChangeRequest>> {
        let storage = self.storage.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;
        
        let crds = storage.load_all_crds().await?;
        Ok(crds.into_values().collect())
    }

    /// Get detailed status for a specific branch
    pub async fn get_branch_status(&self, branch_name: &str) -> Result<Option<kube_crd::WorktreeChangeRequest>> {
        let storage = self.storage.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;
        
        storage.load_crd(branch_name).await
    }

    /// Get storage reference for CLI operations
    pub fn get_storage(&self) -> Result<&storage::WorktreeStorage> {
        self.storage.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))
    }

    /// Get enhanced operations for tool integration
    pub fn get_enhanced_ops(&self) -> Result<&tools::EnhancedWorktreeOps> {
        self.enhanced_ops.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Enhanced operations not initialized"))
    }

    /// Get tool status
    pub fn get_tool_status(&self) -> Result<Vec<tools::ToolStatus>> {
        let enhanced_ops = self.get_enhanced_ops()?;
        Ok(enhanced_ops.tool_manager.get_tool_status())
    }

    /// Bulk pull all worktrees using integrated tools
    pub async fn bulk_pull_all(&self) -> Result<tools::ToolResult> {
        let enhanced_ops = self.get_enhanced_ops()?;
        enhanced_ops.bulk_pull_all().await
    }

    /// Prune stale worktrees using integrated tools
    pub async fn prune_worktrees(&self, force: bool) -> Result<tools::ToolResult> {
        let enhanced_ops = self.get_enhanced_ops()?;
        enhanced_ops.prune_worktrees(force).await
    }

    /// Create worktree with automatic setup using integrated tools
    pub async fn create_worktree_with_setup(&self, branch_name: &str) -> Result<tools::ToolResult> {
        let enhanced_ops = self.get_enhanced_ops()?;
        enhanced_ops.create_worktree_with_setup(branch_name, &[]).await
    }

    /// Switch context using devspace
    pub async fn switch_context(&self, context_name: &str) -> Result<tools::ToolResult> {
        let enhanced_ops = self.get_enhanced_ops()?;
        enhanced_ops.switch_context(context_name).await
    }

    /// List worktrees using devspace
    pub async fn list_devspace_worktrees(&self) -> Result<tools::ToolResult> {
        let enhanced_ops = self.get_enhanced_ops()?;
        enhanced_ops.list_worktrees().await
    }

    /// Get available worktree tools
    pub fn get_available_tools(&self) -> Vec<WorktreeTool> {
        let tools = vec![
            WorktreeTool::Wtp,
            WorktreeTool::Wt,
            WorktreeTool::Treekanga,
            WorktreeTool::Git,
        ];

        tools
            .into_iter()
            .filter(|tool| tool.is_available())
            .collect()
    }

    /// Create a new worktree
    pub async fn create_worktree(&self, branch_name: &str) -> Result<WorktreeResult> {
        self.create_worktree_internal(branch_name).await
    }

    /// List all worktrees
    pub async fn list_worktrees(&self) -> Result<WorktreeResult> {
        self.list_worktrees_internal().await
    }

    /// Switch to a worktree
    pub async fn switch_worktree(&self, worktree_name: &str) -> Result<WorktreeResult> {
        self.switch_worktree_internal(worktree_name).await
    }

    /// Remove a worktree
    pub async fn remove_worktree(
        &self,
        worktree_name: &str,
        with_branch: bool,
    ) -> Result<WorktreeResult> {
        self.remove_worktree_internal(worktree_name, with_branch)
            .await
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ToolConfig) {
        self.config = config;
    }
}

impl Default for WorktreeRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl WorktreeRunner {
    /// Internal method to create a worktree
    async fn create_worktree_internal(&self, branch_name: &str) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;

        match tool {
            WorktreeTool::Wtp => self.create_with_wtp(branch_name).await,
            WorktreeTool::Wt => self.create_with_wt(branch_name).await,
            WorktreeTool::Treekanga => self.create_with_treekanga(branch_name).await,
            WorktreeTool::Git => self.create_with_git(branch_name).await,
        }
    }

    /// Internal method to list worktrees
    async fn list_worktrees_internal(&self) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;

        match tool {
            WorktreeTool::Wtp => self.list_with_wtp().await,
            WorktreeTool::Wt => self.list_with_wt().await,
            WorktreeTool::Treekanga => self.list_with_treekanga().await,
            WorktreeTool::Git => self.list_with_git().await,
        }
    }

    /// Internal method to switch worktree
    async fn switch_worktree_internal(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;

        match tool {
            WorktreeTool::Wtp => self.switch_with_wtp(worktree_name).await,
            WorktreeTool::Wt => self.switch_with_wt(worktree_name).await,
            WorktreeTool::Treekanga => self.switch_with_treekanga(worktree_name).await,
            WorktreeTool::Git => self.switch_with_git(worktree_name).await,
        }
    }

    /// Internal method to remove worktree
    async fn remove_worktree_internal(
        &self,
        worktree_name: &str,
        with_branch: bool,
    ) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;

        match tool {
            WorktreeTool::Wtp => self.remove_with_wtp(worktree_name, with_branch).await,
            WorktreeTool::Wt => self.remove_with_wt(worktree_name).await,
            WorktreeTool::Treekanga => self.remove_with_treekanga(worktree_name).await,
            WorktreeTool::Git => self.remove_with_git(worktree_name, with_branch).await,
        }
    }

    /// Select the best available tool
    async fn select_best_tool(&self) -> Result<WorktreeTool> {
        // Check if preferred tool is available
        if let Some(ref preferred) = self.config.preferred_tool {
            match preferred.as_str() {
                "wtp" if WorktreeTool::Wtp.is_available() => return Ok(WorktreeTool::Wtp),
                "wt" if WorktreeTool::Wt.is_available() => return Ok(WorktreeTool::Wt),
                "treekanga" if WorktreeTool::Treekanga.is_available() => {
                    return Ok(WorktreeTool::Treekanga)
                }
                "git" if WorktreeTool::Git.is_available() => return Ok(WorktreeTool::Git),
                _ => {}
            }
        }

        // Fall back to best available tool
        if WorktreeTool::Wtp.is_available() {
            Ok(WorktreeTool::Wtp)
        } else if WorktreeTool::Wt.is_available() {
            Ok(WorktreeTool::Wt)
        } else if WorktreeTool::Treekanga.is_available() {
            Ok(WorktreeTool::Treekanga)
        } else if WorktreeTool::Git.is_available() {
            Ok(WorktreeTool::Git)
        } else {
            Err(anyhow::anyhow!("No worktree management tool available"))
        }
    }

    /// Create worktree using wtp
    async fn create_with_wtp(&self, branch_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("wtp")
            .arg("create")
            .arg(branch_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None, // Would need to parse output
            branch_name: Some(branch_name.to_string()),
        })
    }

    /// Create worktree using wt
    async fn create_with_wt(&self, branch_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("wt").arg("create").arg(branch_name).output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: Some(branch_name.to_string()),
        })
    }

    /// Create worktree using treekanga
    async fn create_with_treekanga(&self, branch_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("treekanga")
            .arg("create")
            .arg(branch_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: Some(branch_name.to_string()),
        })
    }

    /// Create worktree using git
    async fn create_with_git(&self, branch_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg(format!("../{branch_name}"))
            .arg(branch_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: Some(format!("../{branch_name}")),
            branch_name: Some(branch_name.to_string()),
        })
    }

    /// List worktrees using wtp
    async fn list_with_wtp(&self) -> Result<WorktreeResult> {
        let output = Command::new("wtp").arg("list").output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// List worktrees using wt
    async fn list_with_wt(&self) -> Result<WorktreeResult> {
        let output = Command::new("wt").arg("list").output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// List worktrees using treekanga
    async fn list_with_treekanga(&self) -> Result<WorktreeResult> {
        let output = Command::new("treekanga").arg("list").output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// List worktrees using git
    async fn list_with_git(&self) -> Result<WorktreeResult> {
        let output = Command::new("git").arg("worktree").arg("list").output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Switch worktree using wtp
    async fn switch_with_wtp(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("wtp")
            .arg("switch")
            .arg(worktree_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Switch worktree using wt
    async fn switch_with_wt(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("wt")
            .arg("switch")
            .arg(worktree_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Switch worktree using treekanga
    async fn switch_with_treekanga(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("treekanga")
            .arg("switch")
            .arg(worktree_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Switch worktree using git
    async fn switch_with_git(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg(worktree_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: Some(worktree_name.to_string()),
            branch_name: None,
        })
    }

    /// Remove worktree using wtp
    async fn remove_with_wtp(
        &self,
        worktree_name: &str,
        with_branch: bool,
    ) -> Result<WorktreeResult> {
        let mut cmd = Command::new("wtp");
        cmd.arg("remove").arg(worktree_name);

        if with_branch {
            cmd.arg("--with-branch");
        }

        let output = cmd.output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Remove worktree using wt
    async fn remove_with_wt(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("wt")
            .arg("remove")
            .arg(worktree_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Remove worktree using treekanga
    async fn remove_with_treekanga(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let output = Command::new("treekanga")
            .arg("remove")
            .arg(worktree_name)
            .output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }

    /// Remove worktree using git
    async fn remove_with_git(
        &self,
        worktree_name: &str,
        with_branch: bool,
    ) -> Result<WorktreeResult> {
        let mut cmd = Command::new("git");
        cmd.arg("worktree").arg("remove").arg(worktree_name);

        if with_branch {
            cmd.arg("--force");
        }

        let output = cmd.output()?;

        Ok(WorktreeResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            worktree_path: None,
            branch_name: None,
        })
    }
}

/// Initialize panic hook for better error reporting
pub fn init_panic_hook() {
    // In a real WASM environment, this would set up panic handling
    // For now, we'll leave it empty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worktree_runner_creation() {
        let runner = WorktreeRunner::new();
        assert!(!runner.get_available_tools().is_empty());
    }

    #[tokio::test]
    async fn test_tool_availability() {
        let tools = vec![
            WorktreeTool::Wtp,
            WorktreeTool::Wt,
            WorktreeTool::Treekanga,
            WorktreeTool::Git,
        ];

        for tool in tools {
            let available = tool.is_available();
            // At least git should be available
            if matches!(tool, WorktreeTool::Git) {
                assert!(available);
            }
        }
    }
}
