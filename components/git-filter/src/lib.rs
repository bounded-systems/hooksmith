//! Git Filter Component
//! 
//! This module implements Git attributes as hook-like abstractions, providing
//! a structured way to handle file processing based on .gitattributes.

pub mod state;
pub mod actions;
pub mod filter;
pub mod error;
pub mod contract;
pub mod blob_contract;

pub use state::{FileState, AttributeState};
pub use actions::{ActionResolver, HookAction};
pub use filter::{SafeAsciiFilter, CharContractFilter, BlobContractFilter, FilterDriver};
pub use error::FilterError;
pub use contract::{CharacterContract, CharClass, CharAction, CharValidator, FileValidationResult};
pub use blob_contract::{BlobContract, BlobAction, BlobByteAudit, ByteClass, BlobValidator};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        FileState, AttributeState, ActionResolver, HookAction,
        SafeAsciiFilter, CharContractFilter, BlobContractFilter, FilterDriver, FilterError,
        CharacterContract, CharClass, CharAction, CharValidator, FileValidationResult,
        BlobContract, BlobAction, BlobByteAudit, ByteClass, BlobValidator
    };
} 
