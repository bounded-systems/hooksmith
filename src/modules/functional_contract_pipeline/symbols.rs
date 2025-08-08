use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hook event that triggers the validation pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    /// Pre-commit hook event
    PreCommit,
    /// Pre-push hook event
    PrePush,
    /// Pre-receive hook event
    PreReceive,
    /// Post-receive hook event
    PostReceive,
    /// Update hook event
    Update,
    /// Post-update hook event
    PostUpdate,
    /// Pre-auto-gc hook event
    PreAutoGc,
    /// Post-merge hook event
    PostMerge,
    /// Pre-rebase hook event
    PreRebase,
    /// Post-checkout hook event
    PostCheckout,
    /// Post-commit hook event
    PostCommit,
    /// Pre-apply-patch hook event
    PreApplyPatch,
    /// Post-apply-patch hook event
    PostApplyPatch,
    /// Post-rebase hook event
    PostRebase,
    /// Pre-commit-msg hook event
    PreCommitMsg,
    /// Commit-msg hook event
    CommitMsg,
    /// Post-commit-msg hook event
    PostCommitMsg,
}

/// Concern symbol that identifies a specific Git concern
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConcernSymbol {
    // Git Objects
    /// Git blob objects (file contents)
    Blob,
    /// Git tree objects (directory structure)
    Tree,
    /// Git commit objects (commit history)
    Commit,
    /// Git tag objects (annotated tags)
    Tag,

    // Tree Entries
    /// Tree entry: Regular file (100644)
    TreeFile,
    /// Tree entry: Executable file (100755)
    TreeExecutable,
    /// Tree entry: Symlink (120000)
    TreeSymlink,
    /// Tree entry: Directory (040000)
    TreeDirectory,
    /// Tree entry: Submodule (160000)
    TreeSubmodule,

    // Metadata
    /// Git references (heads, tags, etc.)
    Ref,
    /// Git notes (commit-attached metadata)
    Note,
    /// Git attributes (file-based config)
    Attr,
    /// Git index (staging area)
    Index,
    /// Git stash (pseudo-refs for uncommitted work)
    Stash,
    /// Git worktree (linked working directories)
    Worktree,
    /// Git remote (remote repository configurations)
    Remote,
    /// Git branch (branch-specific configurations)
    Branch,
    /// Git HEAD (current branch reference)
    Head,
    /// Git reflog (reference history)
    Reflog,

    // Config Sections
    /// Git config user settings
    ConfigUser,
    /// Git config core settings
    ConfigCore,
    /// Git config branch settings
    ConfigBranch,
    /// Git config remote settings
    ConfigRemote,
    /// Git config init settings
    ConfigInit,
    /// Git config color settings
    ConfigColor,
    /// Git config alias settings
    ConfigAlias,
    /// Git config diff settings
    ConfigDiff,
    /// Git config merge settings
    ConfigMerge,
    /// Git config GPG settings
    ConfigGpg,
    /// Git config commit settings
    ConfigCommit,
    /// Git config pull settings
    ConfigPull,
    /// Git config push settings
    ConfigPush,
    /// Git config rebase settings
    ConfigRebase,
    /// Git config fetch settings
    ConfigFetch,
    /// Git config status settings
    ConfigStatus,
    /// Git config tar settings
    ConfigTar,
    /// Git config rerere settings
    ConfigRerere,
    /// Git config advice settings
    ConfigAdvice,
    /// Git config interactive settings
    ConfigInteractive,
    /// Git config submodule settings
    ConfigSubmodule,
    /// Git config filter settings
    ConfigFilter,
    /// Git config include settings
    ConfigInclude,
    /// Git config credential settings
    ConfigCredential,
    /// Git config HTTP settings
    ConfigHttp,
    /// Git config URL settings
    ConfigUrl,
    /// Git config safe settings
    ConfigSafe,
    /// Git config notes settings
    ConfigNotes,
    /// Git config garbage collection settings
    ConfigGc,
    /// Git config maintenance settings
    ConfigMaintenance,
    /// Git config pager settings
    ConfigPager,
    /// Git config worktree settings
    ConfigWorktree,

    // Attributes
    /// Attribute: Line ending normalization (text, eol=lf, eol=crlf)
    AttrLineEndingNormalization,
    /// Attribute: Diff strategy (diff, binary)
    AttrDiffStrategy,
    /// Attribute: Merge strategy (merge=...)
    AttrMergeStrategy,
    /// Attribute: Export control (export-ignore, export-subst)
    AttrExportControl,
    /// Attribute: Filter driver (filter=...)
    AttrFilterDriver,
    /// Attribute: External tool hints (linguist-language, linguist-vendored)
    AttrExternalToolHint,
    /// Attribute: Locking hints (lockable for Git LFS)
    AttrLockingHint,

    // Ignore Patterns
    /// Git tree ignore patterns
    IgnorePatternTree,
    /// Git blob ignore patterns
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

impl std::fmt::Display for RuleSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuleSeverity::Info => write!(f, "info"),
            RuleSeverity::Warning => write!(f, "warning"),
            RuleSeverity::Error => write!(f, "error"),
            RuleSeverity::Critical => write!(f, "critical"),
        }
    }
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
        assert_eq!(
            ConcernSymbol::AttrLineEndingNormalization.name(),
            "AttrLineEndingNormalization"
        );
    }
}
