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
pub mod filename_contract;
pub mod tree_filename_chars_contract;
pub mod unified_contracts;
pub mod ref_contracts;

pub use state::{FileState, AttributeState};
pub use actions::{ActionResolver, HookAction};
pub use filter::{SafeAsciiFilter, CharContractFilter, BlobContractFilter, CombinedContractFilter, FilterDriver};
pub use error::FilterError;
pub use contract::{CharacterContract, CharClass, CharAction, CharValidator, FileValidationResult};
pub use blob_contract::{BlobContract, BlobAction, BlobByteAudit, ByteClass, BlobValidator};
pub use line_contract::{BlobLineContract, LineAction, LineValidator};
pub use git_object_contract::{
    GitObjectContract, GitObjectType, GitObjectValidator
};
pub use tree_contract::{
    TreeMode, TreeObjectType, TreeEntryContract, TreeObjectContract, TreeValidator
};
pub use filename_contract::{
    FilenameContract, FilenameValidator
};
pub use tree_filename_chars_contract::{
    CharContract, TreeFilenameContractChars, TreeFilenameCharsValidator
};
pub use unified_contracts::{
    SHA1_RE, VALID_FILENAME_RE, VALID_CHAR_RE,
    BlobLineContract as UnifiedBlobLineContract, BlobContract as UnifiedBlobContract,
    TreeMode as UnifiedTreeMode, TreeEntryContract as UnifiedTreeEntryContract, TreeContract as UnifiedTreeContract,
    CommitContract, TagContract,
    GitObject, UnifiedValidator
};
pub use ref_contracts::{
    VALID_REF_NAME_RE, VALID_BRANCH_NAME_RE, VALID_TAG_NAME_RE, VALID_REMOTE_NAME_RE,
    RefType, RefStorageType,
    RefNameContract, RefTargetContract, RefMetaContract, RefContract,
    UpstreamInfo, ReflogInfo, ReflogEntry,
    RefValidator
};

/// Re-export common types for convenience
pub mod prelude {
    pub use super::{
        FileState, AttributeState, ActionResolver, HookAction,
        SafeAsciiFilter, CharContractFilter, BlobContractFilter, CombinedContractFilter, FilterDriver, FilterError,
        CharacterContract, CharClass, CharAction, CharValidator, FileValidationResult,
        BlobContract, BlobAction, BlobByteAudit, ByteClass, BlobValidator,
        BlobLineContract, LineAction, LineValidator,
        GitObjectContract, GitObjectType, GitObjectValidator,
        TreeMode, TreeObjectType, TreeEntryContract, TreeObjectContract, TreeValidator,
        FilenameContract, FilenameValidator,
        CharContract, TreeFilenameContractChars, TreeFilenameCharsValidator,
        SHA1_RE, VALID_FILENAME_RE, VALID_CHAR_RE,
        UnifiedBlobLineContract, UnifiedBlobContract,
        UnifiedTreeMode, UnifiedTreeEntryContract, UnifiedTreeContract,
        CommitContract, TagContract,
        GitObject, UnifiedValidator,
        VALID_REF_NAME_RE, VALID_BRANCH_NAME_RE, VALID_TAG_NAME_RE, VALID_REMOTE_NAME_RE,
        RefType, RefStorageType,
        RefNameContract, RefTargetContract, RefMetaContract, RefContract,
        UpstreamInfo, ReflogInfo, ReflogEntry,
        RefValidator
    };
} 
