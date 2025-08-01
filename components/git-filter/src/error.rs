use thiserror::Error;

/// Errors that can occur during Git filter operations
#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Invalid character detected in file content")]
    InvalidCharacter,

    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    #[error("Filter driver error: {0}")]
    DriverError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git attributes error: {0}")]
    AttributesError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Unsupported attribute: {0}")]
    UnsupportedAttribute(String),
}
