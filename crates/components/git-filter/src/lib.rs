//! Git Filter Component
//!
//! This module implements Git attributes as hook-like abstractions, providing
//! a structured way to handle file processing based on .gitattributes.

pub mod actions;
pub mod blob_contract;
pub mod contract;
pub mod error;
pub mod filename_contract;
pub mod filter;
pub mod git_object_contract;
pub mod line_contract;
pub mod ref_contracts;
pub mod state;
pub mod tree_contract;
pub mod tree_filename_chars_contract;
pub mod unified_contracts;

pub use actions::{ActionResolver, HookAction};
pub use blob_contract::{BlobAction, BlobByteAudit, BlobContract, BlobValidator, ByteClass};
pub use contract::{CharAction, CharClass, CharValidator, CharacterContract, FileValidationResult};
pub use error::FilterError;
pub use filename_contract::{FilenameContract, FilenameValidator};
pub use filter::{
    BlobContractFilter, CharContractFilter, CombinedContractFilter, FilterDriver, SafeAsciiFilter,
};
pub use git_object_contract::{GitObjectContract, GitObjectType, GitObjectValidator};
pub use line_contract::{BlobLineContract, LineAction, LineValidator};
pub use ref_contracts::{
    RefContract, RefMetaContract, RefNameContract, RefStorageType, RefTargetContract, RefType,
    RefValidator, ReflogEntry, ReflogInfo, UpstreamInfo, VALID_BRANCH_NAME_RE, VALID_REF_NAME_RE,
    VALID_REMOTE_NAME_RE, VALID_TAG_NAME_RE,
};
pub use state::{AttributeState, FileState};
pub use tree_contract::{
    TreeEntryContract, TreeMode, TreeObjectContract, TreeObjectType, TreeValidator,
};
pub use tree_filename_chars_contract::{
    CharContract, TreeFilenameCharsValidator, TreeFilenameContractChars,
};
pub use unified_contracts::{
    BlobContract as UnifiedBlobContract, BlobLineContract as UnifiedBlobLineContract,
    CommitContract, GitObject, TagContract, TreeContract as UnifiedTreeContract,
    TreeEntryContract as UnifiedTreeEntryContract, TreeMode as UnifiedTreeMode, UnifiedValidator,
    SHA1_RE, VALID_CHAR_RE, VALID_FILENAME_RE,
};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        ActionResolver, AttributeState, BlobAction, BlobByteAudit, BlobContract,
        BlobContractFilter, BlobLineContract, BlobValidator, ByteClass, CharAction, CharClass,
        CharContract, CharContractFilter, CharValidator, CharacterContract, CombinedContractFilter,
        CommitContract, FileState, FileValidationResult, FilenameContract, FilenameValidator,
        FilterDriver, FilterError, GitObject, GitObjectContract, GitObjectType, GitObjectValidator,
        HookAction, LineAction, LineValidator, RefContract, RefMetaContract, RefNameContract,
        RefStorageType, RefTargetContract, RefType, RefValidator, ReflogEntry, ReflogInfo,
        SafeAsciiFilter, TagContract, TreeEntryContract, TreeFilenameCharsValidator,
        TreeFilenameContractChars, TreeMode, TreeObjectContract, TreeObjectType, TreeValidator,
        UnifiedBlobContract, UnifiedBlobLineContract, UnifiedTreeContract,
        UnifiedTreeEntryContract, UnifiedTreeMode, UnifiedValidator, UpstreamInfo, SHA1_RE,
        VALID_BRANCH_NAME_RE, VALID_CHAR_RE, VALID_FILENAME_RE, VALID_REF_NAME_RE,
        VALID_REMOTE_NAME_RE, VALID_TAG_NAME_RE,
    };
}
