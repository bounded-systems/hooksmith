#![deny(missing_docs)]

//! Hooksmith CLI Library
//! 
//! This library provides core functionality for building Rust binaries into Lefthook hooks with WASM components.

/// Command implementations for the CLI
pub mod commands;
/// Core modules for CLI functionality
pub mod modules;

// Re-export main types (currently empty as all functionality is in main.rs)
// pub use commands::{};
// pub use modules::{};

/// Result type for CLI operations
pub type CliResult<T> = anyhow::Result<T>;

/// Configuration for CLI operations
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Directory containing hook scripts
    pub hooks_dir: String,
    /// Output directory for built binaries
    pub output_dir: String,
    /// Whether to perform a dry run (no actual changes)
    pub dry_run: bool,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            hooks_dir: "hooks".to_string(),
            output_dir: "target/hooks".to_string(),
            dry_run: false,
        }
    }
} 
