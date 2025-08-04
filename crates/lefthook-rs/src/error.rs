//! Error handling for lefthook-rs
//!
//! This module provides custom error types and error handling utilities
//! for the lefthook-rs crate.

use thiserror::Error;

/// Custom error type for lefthook-rs operations
#[derive(Error, Debug)]
pub enum LefthookError {
    /// Binary not found error
    #[error("Lefthook binary not found: {0}")]
    BinaryNotFound(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// YAML serialization/deserialization error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Command execution error
    #[error("Command execution failed: {0}")]
    CommandExecution(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Download error
    #[error("Download error: {0}")]
    Download(String),

    /// Installation error
    #[error("Installation error: {0}")]
    Installation(String),

    /// Version error
    #[error("Version error: {0}")]
    Version(String),
}

/// Result type for lefthook-rs operations
pub type Result<T> = std::result::Result<T, LefthookError>;

impl From<anyhow::Error> for LefthookError {
    fn from(err: anyhow::Error) -> Self {
        LefthookError::Configuration(err.to_string())
    }
}

#[cfg(feature = "download")]
impl From<reqwest::Error> for LefthookError {
    fn from(err: reqwest::Error) -> Self {
        LefthookError::Download(err.to_string())
    }
}
