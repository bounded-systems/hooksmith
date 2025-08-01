//! Pushd Worktree CLI Library
//! 
//! This library provides core functionality for Git worktree management and safety.

pub mod commands;
pub mod modules;

// Re-export main types
pub use commands::{worktree, hooks};
pub use modules::{worktree as worktree_modules, hooks as hook_modules, utils};

/// Result type for CLI operations
pub type CliResult<T> = anyhow::Result<T>;

/// Configuration for CLI operations
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub worktree_dir: String,
    pub hooks_dir: String,
    pub dry_run: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            worktree_dir: ".worktree".to_string(),
            hooks_dir: ".cli-helper/hooks".to_string(),
            dry_run: false,
        }
    }
} 
