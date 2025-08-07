use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hook event that triggers the validation pipeline
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    PreCommit,
    PrePush,
    PreReceive,
    PostReceive,
    Update,
    PostUpdate,
    PreAutoGc,
    PostMerge,
    PreRebase,
    PostCheckout,
    PostCommit,
    PreApplyPatch,
    PostApplyPatch,
    PostRebase,
    PreCommitMsg,
    CommitMsg,
    PostCommitMsg,
}

/// Concern symbol that identifies a specific Git concern
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcernSymbol {
    // Git Objects
    Blob,
    Tree,
    Commit,
    Tag,
    
    // Tree Entries
    TreeFile,
    TreeExecutable,
    TreeSymlink,
    TreeDirectory,
    TreeSubmodule,
    
    // Metadata
    Ref,
    Note,
    Attr,
    Index,
    Stash,
    Worktree,
    Remote,
    Branch,
    Head,
    Reflog,
    
    // Config Sections
    ConfigUser,
    ConfigCore,
    ConfigBranch,
    ConfigRemote,
    ConfigInit,
    ConfigColor,
    ConfigAlias,
    ConfigDiff,
    ConfigMerge,
    ConfigGpg,
    ConfigCommit,
    ConfigPull,
    ConfigPush,
    ConfigRebase,
    ConfigFetch,
    ConfigStatus,
    ConfigTar,
    ConfigRerere,
    ConfigAdvice,
    ConfigInteractive,
    ConfigSubmodule,
    ConfigFilter,
    ConfigInclude,
    ConfigCredential,
    ConfigHttp,
    ConfigUrl,
    ConfigSafe,
    ConfigNotes,
    ConfigGc,
    ConfigMaintenance,
    ConfigPager,
    ConfigWorktree,
    
    // Attributes
    AttrLineEndingNormalization,
    AttrDiffStrategy,
    AttrMergeStrategy,
    AttrExportControl,
    AttrFilterDriver,
    AttrExternalToolHint,
    AttrLockingHint,
    
    // Ignore Patterns
    IgnorePatternTree,
    IgnorePatternBlob,
}

/// Contract symbol that identifies a specific validation rule
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContractSymbol(pub String);

impl ContractSymbol {
    /// Create a new contract symbol
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
    
    /// Get the contract name
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<String> for ContractSymbol {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl From<&str> for ContractSymbol {
    fn from(name: &str) -> Self {
        Self(name.to_string())
    }
}

/// Rule type for contract validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleType {
    /// Custom validation function
    Custom,
    /// Pattern matching (regex)
    Pattern,
    /// JSON Schema validation
    JsonSchema,
    /// File size limits
    FileSize,
    /// File extension validation
    FileExtension,
}

/// Rule severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleSeverity {
    /// Information only
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical error
    Critical,
}

/// Contract rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: Option<String>,
    /// Rule type
    pub rule_type: RuleType,
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Whether rule is required
    pub required: bool,
    /// Rule severity
    pub severity: RuleSeverity,
}

/// Contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractSpec {
    /// Contract name
    pub name: String,
    /// Contract version
    pub version: String,
    /// Concern this contract applies to
    pub concern: ConcernSymbol,
    /// Contract rules
    pub rules: Vec<ContractRule>,
    /// Contract metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ConcernSymbol {
    /// Get the name of the concern
    pub fn name(&self) -> &'static str {
        match self {
            ConcernSymbol::Blob => "Blob",
            ConcernSymbol::Tree => "Tree",
            ConcernSymbol::Commit => "Commit",
            ConcernSymbol::Tag => "Tag",
            ConcernSymbol::TreeFile => "TreeFile",
            ConcernSymbol::TreeExecutable => "TreeExecutable",
            ConcernSymbol::TreeSymlink => "TreeSymlink",
            ConcernSymbol::TreeDirectory => "TreeDirectory",
            ConcernSymbol::TreeSubmodule => "TreeSubmodule",
            ConcernSymbol::Ref => "Ref",
            ConcernSymbol::Note => "Note",
            ConcernSymbol::Attr => "Attr",
            ConcernSymbol::Index => "Index",
            ConcernSymbol::Stash => "Stash",
            ConcernSymbol::Worktree => "Worktree",
            ConcernSymbol::Remote => "Remote",
            ConcernSymbol::Branch => "Branch",
            ConcernSymbol::Head => "Head",
            ConcernSymbol::Reflog => "Reflog",
            ConcernSymbol::ConfigUser => "ConfigUser",
            ConcernSymbol::ConfigCore => "ConfigCore",
            ConcernSymbol::ConfigBranch => "ConfigBranch",
            ConcernSymbol::ConfigRemote => "ConfigRemote",
            ConcernSymbol::ConfigInit => "ConfigInit",
            ConcernSymbol::ConfigColor => "ConfigColor",
            ConcernSymbol::ConfigAlias => "ConfigAlias",
            ConcernSymbol::ConfigDiff => "ConfigDiff",
            ConcernSymbol::ConfigMerge => "ConfigMerge",
            ConcernSymbol::ConfigGpg => "ConfigGpg",
            ConcernSymbol::ConfigCommit => "ConfigCommit",
            ConcernSymbol::ConfigPull => "ConfigPull",
            ConcernSymbol::ConfigPush => "ConfigPush",
            ConcernSymbol::ConfigRebase => "ConfigRebase",
            ConcernSymbol::ConfigFetch => "ConfigFetch",
            ConcernSymbol::ConfigStatus => "ConfigStatus",
            ConcernSymbol::ConfigTar => "ConfigTar",
            ConcernSymbol::ConfigRerere => "ConfigRerere",
            ConcernSymbol::ConfigAdvice => "ConfigAdvice",
            ConcernSymbol::ConfigInteractive => "ConfigInteractive",
            ConcernSymbol::ConfigSubmodule => "ConfigSubmodule",
            ConcernSymbol::ConfigFilter => "ConfigFilter",
            ConcernSymbol::ConfigInclude => "ConfigInclude",
            ConcernSymbol::ConfigCredential => "ConfigCredential",
            ConcernSymbol::ConfigHttp => "ConfigHttp",
            ConcernSymbol::ConfigUrl => "ConfigUrl",
            ConcernSymbol::ConfigSafe => "ConfigSafe",
            ConcernSymbol::ConfigNotes => "ConfigNotes",
            ConcernSymbol::ConfigGc => "ConfigGc",
            ConcernSymbol::ConfigMaintenance => "ConfigMaintenance",
            ConcernSymbol::ConfigPager => "ConfigPager",
            ConcernSymbol::ConfigWorktree => "ConfigWorktree",
            ConcernSymbol::AttrLineEndingNormalization => "AttrLineEndingNormalization",
            ConcernSymbol::AttrDiffStrategy => "AttrDiffStrategy",
            ConcernSymbol::AttrMergeStrategy => "AttrMergeStrategy",
            ConcernSymbol::AttrExportControl => "AttrExportControl",
            ConcernSymbol::AttrFilterDriver => "AttrFilterDriver",
            ConcernSymbol::AttrExternalToolHint => "AttrExternalToolHint",
            ConcernSymbol::AttrLockingHint => "AttrLockingHint",
            ConcernSymbol::IgnorePatternTree => "IgnorePatternTree",
            ConcernSymbol::IgnorePatternBlob => "IgnorePatternBlob",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_symbol_creation() {
        let symbol = ContractSymbol::new("must-exist");
        assert_eq!(symbol.name(), "must-exist");
    }

    #[test]
    fn test_contract_symbol_from_string() {
        let symbol: ContractSymbol = "must-be-non-executable".into();
        assert_eq!(symbol.name(), "must-be-non-executable");
    }

    #[test]
    fn test_concern_symbol_name() {
        assert_eq!(ConcernSymbol::TreeFile.name(), "TreeFile");
        assert_eq!(ConcernSymbol::Ref.name(), "Ref");
        assert_eq!(ConcernSymbol::AttrLineEndingNormalization.name(), "AttrLineEndingNormalization");
    }
}
