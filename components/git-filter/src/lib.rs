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
pub mod line_contract;
pub mod git_object_contract;
pub mod tree_contract;

pub use state::{FileState, AttributeState};
pub use actions::{ActionResolver, HookAction};
pub use filter::{SafeAsciiFilter, CharContractFilter, BlobContractFilter, CombinedContractFilter, FilterDriver};
pub use error::FilterError;
pub use contract::{CharacterContract, CharClass, CharAction, CharValidator, FileValidationResult};
pub use blob_contract::{BlobContract, BlobAction, BlobByteAudit, ByteClass, BlobValidator};
pub use line_contract::{BlobLineContract, LineAction, LineValidator};
pub use git_object_contract::{
    GitObjectContract, BlobContract as GitBlobContract, BlobLineContract as GitBlobLineContract,
    BlobChunkContract, DiffLine, DiffLineType, LineAction as GitLineAction, GitObjectValidator
};
pub use tree_contract::{
    TreeMode, TreeObjectType, TreeEntryContract, TreeObjectContract, TreeValidator
};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        FileState, AttributeState, ActionResolver, HookAction,
        SafeAsciiFilter, CharContractFilter, BlobContractFilter, CombinedContractFilter, FilterDriver, FilterError,
        CharacterContract, CharClass, CharAction, CharValidator, FileValidationResult,
        BlobContract, BlobAction, BlobByteAudit, ByteClass, BlobValidator,
        BlobLineContract, LineAction, LineValidator,
        GitObjectContract, GitBlobContract, GitBlobLineContract, BlobChunkContract, DiffLine, DiffLineType, GitLineAction, GitObjectValidator,
        TreeMode, TreeObjectType, TreeEntryContract, TreeObjectContract, TreeValidator
    };
} 
