//! Git Filter Component
//!
//! This module implements Git attributes as hook-like abstractions, providing
//! a structured way to handle file processing based on .gitattributes.
//!
//! This component implements the WIT interface for git filtering operations.

#[cfg(feature = "host")]
wit_bindgen::generate!({
    path: "wit/git-filter.wit",
    world: "git-filter-world",
});

#[cfg(not(feature = "host"))]
wit_bindgen::generate!({
    path: "wit/git-filter.wit",
    world: "git-filter-world",
});

// Export the generated bindings
pub use git_filter_world::*;

// Re-export the WIT interface for use in other crates
pub mod wit {
    pub use super::git_filter_world::*;
}

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

/// Git Filter Component Implementation
struct GitFilterComponent;

impl git_filter::GitFilter for GitFilterComponent {
    fn validate_blob(
        blob_content: String,
        config: git_filter::FilterConfig,
    ) -> Result<git_filter::FilterResult, String> {
        // Implementation using existing filter logic
        let filter_driver = FilterDriver::new();
        
        match filter_driver.validate_blob(&blob_content, &config.params) {
            Ok(result) => Ok(git_filter::FilterResult {
                success: result.is_valid(),
                content: Some(blob_content),
                error: None,
                details: vec!["Validation completed".to_string()],
            }),
            Err(e) => Ok(git_filter::FilterResult {
                success: false,
                content: None,
                error: Some(e.to_string()),
                details: vec![],
            }),
        }
    }

    fn filter_object(
        object_content: String,
        config: git_filter::FilterConfig,
    ) -> Result<git_filter::FilterResult, String> {
        // Implementation using existing filter logic
        let filter_driver = FilterDriver::new();
        
        match filter_driver.filter_content(&object_content, &config.params) {
            Ok(filtered_content) => Ok(git_filter::FilterResult {
                success: true,
                content: Some(filtered_content),
                error: None,
                details: vec!["Object filtered successfully".to_string()],
            }),
            Err(e) => Ok(git_filter::FilterResult {
                success: false,
                content: None,
                error: Some(e.to_string()),
                details: vec![],
            }),
        }
    }

    fn check_contract(content: String, contract_name: String) -> Result<bool, String> {
        // Implementation using existing contract validation
        let validator = UnifiedValidator::new();
        
        match validator.validate_contract(&content, &contract_name) {
            Ok(is_valid) => Ok(is_valid),
            Err(e) => Err(e.to_string()),
        }
    }

    fn transform_content(content: String, contract_name: String) -> Result<String, String> {
        // Implementation using existing transformation logic
        let filter_driver = FilterDriver::new();
        
        match filter_driver.transform_content(&content, &contract_name) {
            Ok(transformed) => Ok(transformed),
            Err(e) => Err(e.to_string()),
        }
    }
}

export_git_filter!(GitFilterComponent);
