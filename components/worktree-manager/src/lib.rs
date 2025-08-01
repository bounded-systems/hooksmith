//! Worktree Manager Component
//! 
//! This component provides Git worktree management operations for pushd-web development workflows.

use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Worktree manager operations trait
pub trait WorktreeManager {
    /// Create a new worktree
    fn create_worktree(&self, name: &str, branch: &str) -> Result<WorktreeResult>;
    
    /// Remove a worktree
    fn remove_worktree(&self, name: &str) -> Result<WorktreeResult>;
    
    /// List all worktrees
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;
    
    /// Get worktree status
    fn get_worktree_status(&self, name: &str) -> Result<WorktreeStatus>;
}

/// Worktree configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    pub name: String,
    pub branch: String,
    pub path: Option<String>,
    pub force: bool,
}

/// Worktree operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
    pub path: Option<String>,
}

/// Worktree information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub head: String,
    pub is_current: bool,
}

/// Worktree status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeStatus {
    pub name: String,
    pub path: String,
    pub branch: String,
    pub head: String,
    pub is_current: bool,
    pub has_changes: bool,
    pub untracked_files: Vec<String>,
}

/// Default implementation of worktree manager
pub struct DefaultWorktreeManager;

impl WorktreeManager for DefaultWorktreeManager {
    fn create_worktree(&self, name: &str, branch: &str) -> Result<WorktreeResult> {
        // Placeholder implementation
        Ok(WorktreeResult {
            success: true,
            message: format!("Created worktree {} for branch {}", name, branch),
            error: None,
            path: Some(format!("./{}", name)),
        })
    }
    
    fn remove_worktree(&self, name: &str) -> Result<WorktreeResult> {
        // Placeholder implementation
        Ok(WorktreeResult {
            success: true,
            message: format!("Removed worktree {}", name),
            error: None,
            path: None,
        })
    }
    
    fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        // Placeholder implementation
        Ok(vec![])
    }
    
    fn get_worktree_status(&self, name: &str) -> Result<WorktreeStatus> {
        // Placeholder implementation
        Ok(WorktreeStatus {
            name: name.to_string(),
            path: format!("./{}", name),
            branch: "main".to_string(),
            head: "HEAD".to_string(),
            is_current: false,
            has_changes: false,
            untracked_files: vec![],
        })
    }
}

// Export the main operations
pub use DefaultWorktreeManager as WorktreeManagerComponent; 
