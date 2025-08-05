use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

use crate::crd::{WorktreeChangeRequest, WorktreeAction};

/// Available worktree tools
#[derive(Debug, Clone)]
pub enum WorktreeTool {
    Workbloom,
    Gwtr,
    GitWorktreeCli,
    Devspace,
    Git,
}

impl WorktreeTool {
    /// Get the command name for this tool
    pub fn command_name(&self) -> &'static str {
        match self {
            WorktreeTool::Workbloom => "wb",
            WorktreeTool::Gwtr => "gwtr",
            WorktreeTool::GitWorktreeCli => "git-worktree-cli",
            WorktreeTool::Devspace => "devspace",
            WorktreeTool::Git => "git",
        }
    }

    /// Check if this tool is available on the system
    pub fn is_available(&self) -> bool {
        Command::new(self.command_name())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Get all available tools
    pub fn get_available_tools() -> Vec<Self> {
        vec![
            WorktreeTool::Workbloom,
            WorktreeTool::Gwtr,
            WorktreeTool::GitWorktreeCli,
            WorktreeTool::Devspace,
            WorktreeTool::Git,
        ]
        .into_iter()
        .filter(|tool| tool.is_available())
        .collect()
    }
}

/// Tool integration manager
pub struct ToolManager {
    preferred_tool: Option<WorktreeTool>,
    fallback_tool: WorktreeTool,
}

impl ToolManager {
    /// Create a new tool manager
    pub fn new(preferred_tool: Option<WorktreeTool>) -> Self {
        Self {
            preferred_tool,
            fallback_tool: WorktreeTool::Git,
        }
    }

    /// Get the best available tool for a specific operation
    pub fn get_best_tool(&self, operation: &ToolOperation) -> Result<WorktreeTool> {
        // Check if preferred tool is available and suitable
        if let Some(ref tool) = self.preferred_tool {
            if tool.is_available() && self.is_tool_suitable(tool, operation) {
                return Ok(tool.clone());
            }
        }

        // Find the best available tool for this operation
        let available_tools = WorktreeTool::get_available_tools();
        
        for tool in available_tools {
            if self.is_tool_suitable(&tool, operation) {
                return Ok(tool);
            }
        }

        // Fallback to git
        Ok(self.fallback_tool.clone())
    }

    /// Check if a tool is suitable for a specific operation
    fn is_tool_suitable(&self, tool: &WorktreeTool, operation: &ToolOperation) -> bool {
        match (tool, operation) {
            (WorktreeTool::Workbloom, ToolOperation::CreateWorktree) => true,
            (WorktreeTool::Workbloom, ToolOperation::SetupEnvironment) => true,
            (WorktreeTool::Gwtr, ToolOperation::BulkPull) => true,
            (WorktreeTool::Gwtr, ToolOperation::PruneWorktrees) => true,
            (WorktreeTool::GitWorktreeCli, ToolOperation::Status) => true,
            (WorktreeTool::Devspace, ToolOperation::CreateWorktree) => true,
            (WorktreeTool::Devspace, ToolOperation::SwitchContext) => true,
            (WorktreeTool::Devspace, ToolOperation::ListWorktrees) => true,
            (WorktreeTool::Git, _) => true, // Git can do everything
            _ => false,
        }
    }

    /// Execute a worktree operation using the best available tool
    pub async fn execute_operation(&self, operation: ToolOperation, args: &[&str]) -> Result<ToolResult> {
        let tool = self.get_best_tool(&operation)?;
        info!("Using {} for operation {:?}", tool.command_name(), operation);
        
        match tool {
            WorktreeTool::Workbloom => self.execute_with_workbloom(operation, args).await,
            WorktreeTool::Gwtr => self.execute_with_gwtr(operation, args).await,
            WorktreeTool::GitWorktreeCli => self.execute_with_git_worktree_cli(operation, args).await,
            WorktreeTool::Devspace => self.execute_with_devspace(operation, args).await,
            WorktreeTool::Git => self.execute_with_git(operation, args).await,
        }
    }

    /// Execute operation using workbloom
    async fn execute_with_workbloom(&self, operation: ToolOperation, args: &[&str]) -> Result<ToolResult> {
        let mut cmd = Command::new("wb");
        
        match operation {
            ToolOperation::CreateWorktree => {
                cmd.args(&["add", args[0]]);
                if args.len() > 1 {
                    cmd.args(&args[1..]);
                }
            }
            ToolOperation::SetupEnvironment => {
                cmd.args(&["setup", args[0]]);
            }
            ToolOperation::CleanupWorktree => {
                cmd.args(&["remove", args[0]]);
            }
            ToolOperation::Status => {
                cmd.args(&["status"]);
            }
            _ => {
                warn!("Workbloom doesn't support operation {:?}, falling back to git", operation);
                return self.execute_with_git(operation, args).await;
            }
        }

        let output = cmd.output().context("Failed to execute workbloom command")?;
        
        Ok(ToolResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            tool_used: "workbloom".to_string(),
        })
    }

    /// Execute operation using gwtr
    async fn execute_with_gwtr(&self, operation: ToolOperation, args: &[&str]) -> Result<ToolResult> {
        let mut cmd = Command::new("gwtr");
        
        match operation {
            ToolOperation::CreateWorktree => {
                cmd.args(&["add", args[0]]);
            }
            ToolOperation::BulkPull => {
                cmd.args(&["pull", "--all"]);
            }
            ToolOperation::PruneWorktrees => {
                cmd.args(&["prune"]);
                if args.contains(&"--force") {
                    cmd.arg("--force");
                }
            }
            ToolOperation::Status => {
                cmd.args(&["status"]);
            }
            _ => {
                warn!("Gwtr doesn't support operation {:?}, falling back to git", operation);
                return self.execute_with_git(operation, args).await;
            }
        }

        let output = cmd.output().context("Failed to execute gwtr command")?;
        
        Ok(ToolResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            tool_used: "gwtr".to_string(),
        })
    }

    /// Execute operation using git-worktree-cli
    async fn execute_with_git_worktree_cli(&self, operation: ToolOperation, args: &[&str]) -> Result<ToolResult> {
        let mut cmd = Command::new("git-worktree-cli");
        
        match operation {
            ToolOperation::Status => {
                cmd.args(&["status"]);
            }
            ToolOperation::CreateWorktree => {
                cmd.args(&["add", args[0]]);
            }
            _ => {
                warn!("Git-worktree-cli doesn't support operation {:?}, falling back to git", operation);
                return self.execute_with_git(operation, args).await;
            }
        }

        let output = cmd.output().context("Failed to execute git-worktree-cli command")?;
        
        Ok(ToolResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            tool_used: "git-worktree-cli".to_string(),
        })
    }

    /// Execute operation using devspace
    async fn execute_with_devspace(&self, operation: ToolOperation, args: &[&str]) -> Result<ToolResult> {
        let mut cmd = Command::new("devspace");
        
        match operation {
            ToolOperation::CreateWorktree => {
                cmd.args(&["add", args[0]]);
            }
            ToolOperation::SwitchContext => {
                cmd.args(&["switch", args[0]]);
            }
            ToolOperation::ListWorktrees => {
                cmd.args(&["list"]);
            }
            ToolOperation::Status => {
                cmd.args(&["status"]);
            }
            _ => {
                warn!("Devspace doesn't support operation {:?}, falling back to git", operation);
                return self.execute_with_git(operation, args).await;
            }
        }

        let output = cmd.output().context("Failed to execute devspace command")?;
        
        Ok(ToolResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            tool_used: "devspace".to_string(),
        })
    }

    /// Execute operation using git (fallback)
    async fn execute_with_git(&self, operation: ToolOperation, args: &[&str]) -> Result<ToolResult> {
        let mut cmd = Command::new("git");
        
        match operation {
            ToolOperation::CreateWorktree => {
                cmd.args(&["worktree", "add", args[0], args[1]]);
            }
            ToolOperation::CleanupWorktree => {
                cmd.args(&["worktree", "remove", args[0]]);
            }
            ToolOperation::CreateBranch => {
                cmd.args(&["checkout", "-b", args[0]]);
            }
            ToolOperation::PushBranch => {
                cmd.args(&["push", "origin", args[0]]);
            }
            ToolOperation::PullBranch => {
                cmd.args(&["pull", "origin", args[0]]);
            }
            ToolOperation::Status => {
                cmd.args(&["worktree", "list"]);
            }
            _ => {
                return Err(anyhow::anyhow!("Git doesn't support operation {:?}", operation));
            }
        }

        let output = cmd.output().context("Failed to execute git command")?;
        
        Ok(ToolResult {
            success: output.status.success(),
            output: String::from_utf8_lossy(&output.stdout).to_string(),
            error: if output.status.success() {
                None
            } else {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            },
            tool_used: "git".to_string(),
        })
    }

    /// Get status of all available tools
    pub fn get_tool_status(&self) -> Vec<ToolStatus> {
        let mut status = Vec::new();
        
        for tool in WorktreeTool::get_available_tools() {
            let available = tool.is_available();
            let preferred = self.preferred_tool.as_ref().map_or(false, |pt| pt.command_name() == tool.command_name());
            
            status.push(ToolStatus {
                name: tool.command_name().to_string(),
                available,
                preferred,
                version: self.get_tool_version(&tool),
            });
        }
        
        status
    }

    /// Get version of a tool
    fn get_tool_version(&self, tool: &WorktreeTool) -> Option<String> {
        let output = Command::new(tool.command_name())
            .arg("--version")
            .output()
            .ok()?;
        
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }
}

/// Available tool operations
#[derive(Debug, Clone)]
pub enum ToolOperation {
    CreateWorktree,
    CreateBranch,
    SetupEnvironment,
    CleanupWorktree,
    BulkPull,
    PruneWorktrees,
    PushBranch,
    PullBranch,
    Status,
    SwitchContext,
    ListWorktrees,
}

/// Result of a tool operation
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub tool_used: String,
}

/// Status of a tool
#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub name: String,
    pub available: bool,
    pub preferred: bool,
    pub version: Option<String>,
}

/// Enhanced worktree operations using integrated tools
pub struct EnhancedWorktreeOps {
    pub tool_manager: ToolManager,
}

impl EnhancedWorktreeOps {
    /// Create a new enhanced worktree operations manager
    pub fn new(preferred_tool: Option<WorktreeTool>) -> Self {
        Self {
            tool_manager: ToolManager::new(preferred_tool),
        }
    }

    /// Create a worktree with automatic setup
    pub async fn create_worktree_with_setup(&self, branch_name: &str, _setup_commands: &[&str]) -> Result<ToolResult> {
        // Create worktree
        let create_result = self.tool_manager.execute_operation(
            ToolOperation::CreateWorktree,
            &[branch_name],
        ).await?;

        if !create_result.success {
            return Ok(create_result);
        }

        // Setup environment if workbloom is available
        if let Ok(workbloom) = self.tool_manager.get_best_tool(&ToolOperation::SetupEnvironment) {
            if matches!(workbloom, WorktreeTool::Workbloom) {
                let setup_result = self.tool_manager.execute_operation(
                    ToolOperation::SetupEnvironment,
                    &[branch_name],
                ).await?;
                
                return Ok(setup_result);
            }
        }

        Ok(create_result)
    }

    /// Bulk pull all worktrees
    pub async fn bulk_pull_all(&self) -> Result<ToolResult> {
        self.tool_manager.execute_operation(
            ToolOperation::BulkPull,
            &[],
        ).await
    }

    /// Prune stale worktrees
    pub async fn prune_worktrees(&self, force: bool) -> Result<ToolResult> {
        let args = if force { vec!["--force"] } else { vec![] };
        self.tool_manager.execute_operation(
            ToolOperation::PruneWorktrees,
            &args.iter().map(|s| *s).collect::<Vec<_>>(),
        ).await
    }

    /// Get comprehensive status
    pub async fn get_status(&self) -> Result<ToolResult> {
        self.tool_manager.execute_operation(
            ToolOperation::Status,
            &[],
        ).await
    }

    /// Switch context using devspace
    pub async fn switch_context(&self, context_name: &str) -> Result<ToolResult> {
        self.tool_manager.execute_operation(
            ToolOperation::SwitchContext,
            &[context_name],
        ).await
    }

    /// List worktrees using devspace
    pub async fn list_worktrees(&self) -> Result<ToolResult> {
        self.tool_manager.execute_operation(
            ToolOperation::ListWorktrees,
            &[],
        ).await
    }

    /// Execute CRD action using enhanced tools
    pub async fn execute_crd_action(&self, crd: &WorktreeChangeRequest, action: &WorktreeAction) -> Result<ToolResult> {
        match action {
            WorktreeAction::CreateWorktree => {
                self.create_worktree_with_setup(&crd.spec.branch, &[]).await
            }
            WorktreeAction::CreateBranch => {
                self.tool_manager.execute_operation(
                    ToolOperation::CreateBranch,
                    &[&crd.spec.branch],
                ).await
            }
            WorktreeAction::PushBranch => {
                self.tool_manager.execute_operation(
                    ToolOperation::PushBranch,
                    &[&crd.spec.branch],
                ).await
            }
            WorktreeAction::CleanupWorktree => {
                self.tool_manager.execute_operation(
                    ToolOperation::CleanupWorktree,
                    &[&crd.spec.branch],
                ).await
            }
            _ => {
                // Fallback to git for other operations
                self.tool_manager.execute_operation(
                    ToolOperation::CreateWorktree, // Placeholder
                    &[&crd.spec.branch],
                ).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_availability() {
        let tools = WorktreeTool::get_available_tools();
        assert!(!tools.is_empty()); // At least git should be available
    }

    #[test]
    fn test_tool_manager_creation() {
        let manager = ToolManager::new(Some(WorktreeTool::Git));
        assert!(manager.get_best_tool(&ToolOperation::CreateWorktree).is_ok());
    }
} 
