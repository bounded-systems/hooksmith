//! Hooksmith CLI Library
//! 
//! This library provides core functionality for building Rust binaries into Lefthook hooks with WASM components.

pub mod commands;
pub mod modules;

// Re-export main types (currently empty as all functionality is in main.rs)
// pub use commands::{};
// pub use modules::{};

/// Result type for CLI operations
pub type CliResult<T> = anyhow::Result<T>;

/// Configuration for CLI operations
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub hooks_dir: String,
    pub output_dir: String,
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
