//! Git Filter Component
//! 
//! This module implements Git attributes as hook-like abstractions, providing
//! a structured way to handle file processing based on .gitattributes.

pub mod state;
pub mod actions;
pub mod filter;
pub mod error;

pub use state::{FileState, AttributeState};
pub use actions::{ActionResolver, HookAction};
pub use filter::{SafeAsciiFilter, FilterDriver};
pub use error::FilterError;

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        FileState, AttributeState, ActionResolver, HookAction,
        SafeAsciiFilter, FilterDriver, FilterError
    };
} 