use crate::modules::git_native::{
    GitObjectType, GitTreeEntryType, GitMetadataType, GitConfigType, GitAttributeType
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
    PreReBase,
    PostReBase,
    PreCommitMsg,
    CommitMsg,
    PostCommitMsg,
    PostRebase,
    PreRebaseAuto,
    PostRebaseAuto,
    PreRebaseAutoStash,
    PostRebaseAutoStash,
    PreRebaseAutoStashClean,
    PostRebaseAutoStashClean,
    PreRebaseAutoStashRestore,
    PostRebaseAutoStashRestore,
    PreRebaseAutoStashRestoreClean,
    PostRebaseAutoStashRestoreClean,
    PreRebaseAutoStashRestoreRestore,
    PostRebaseAutoStashRestoreRestore,
    PreRebaseAutoStashRestoreRestoreClean,
    PostRebaseAutoStashRestoreRestoreClean,
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
}

/// Snapshot of a concern's current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcernSnapshot {
    /// The concern symbol
    pub concern: ConcernSymbol,
    /// Snapshot data as JSON
    pub data: serde_json::Value,
    /// Timestamp of snapshot
    pub timestamp: String,
    /// Hash of the snapshot data
    pub hash: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Set of observed concern snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedConcernSet {
    /// Map of concern symbols to their snapshots
    pub snapshots: HashMap<ConcernSymbol, ConcernSnapshot>,
    /// Hook event that triggered this observation
    pub hook_event: HookEvent,
    /// Timestamp of the observation
    pub timestamp: String,
}

/// Contract specification for a concern
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

/// Individual rule within a contract
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

/// Types of contract rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    /// JSON Schema validation
    JsonSchema,
    /// Pattern matching (regex)
    Pattern,
    /// File size limits
    FileSize,
    /// File extension validation
    FileExtension,
    /// Custom validation function
    Custom,
}

/// Rule severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Expected snapshot based on contract rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedSnapshot {
    /// The concern symbol
    pub concern: ConcernSymbol,
    /// Expected data as JSON
    pub data: serde_json::Value,
    /// Contract that generated this expectation
    pub contract: String,
    /// Contract version
    pub contract_version: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Set of expected concern snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedConcernSet {
    /// Map of concern symbols to their expected snapshots
    pub snapshots: HashMap<ConcernSymbol, ExpectedSnapshot>,
    /// Hook event that triggered this expectation
    pub hook_event: HookEvent,
    /// Timestamp of the expectation
    pub timestamp: String,
}

/// Validation difference between observed and expected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDiff {
    /// Concern that has a difference
    pub concern: ConcernSymbol,
    /// Type of difference
    pub diff_type: DiffType,
    /// Description of the difference
    pub description: String,
    /// Observed value
    pub observed: Option<serde_json::Value>,
    /// Expected value
    pub expected: Option<serde_json::Value>,
    /// Severity of the difference
    pub severity: RuleSeverity,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of validation differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffType {
    /// Missing expected concern
    Missing,
    /// Unexpected observed concern
    Unexpected,
    /// Value mismatch
    Mismatch,
    /// Schema validation failure
    SchemaViolation,
    /// Rule violation
    RuleViolation,
}

/// Set of validation differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSet {
    /// List of differences
    pub diffs: Vec<ValidationDiff>,
    /// Whether validation passed
    pub is_valid: bool,
    /// Summary of validation
    pub summary: String,
}

/// Functional contract validation pipeline
pub struct FunctionalContractPipeline {
    /// Contract registry
    contracts: HashMap<ConcernSymbol, ContractSpec>,
    /// Git repository path
    repo_path: String,
}

impl FunctionalContractPipeline {
    /// Create a new functional contract pipeline
    pub fn new(repo_path: &str) -> Self {
        Self {
            contracts: HashMap::new(),
            repo_path: repo_path.to_string(),
        }
    }

    /// Register a contract specification
    pub fn register_contract(&mut self, contract: ContractSpec) -> Result<()> {
        self.contracts.insert(contract.concern.clone(), contract);
        Ok(())
    }

    /// Step 1: Identify concerns based on hook event
    /// Stateless: Input is git event, output is concern list
    pub fn identify_concerns(&self, hook: &HookEvent) -> Vec<ConcernSymbol> {
        match hook {
            HookEvent::PreCommit => vec![
                ConcernSymbol::Index,
                ConcernSymbol::AttrLineEndingNormalization,
                ConcernSymbol::AttrDiffStrategy,
            ],
            HookEvent::PrePush => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
                ConcernSymbol::Remote,
                ConcernSymbol::TreeExecutable,
                ConcernSymbol::AttrLineEndingNormalization,
            ],
            HookEvent::PreReceive => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
                ConcernSymbol::Remote,
            ],
            HookEvent::PostReceive => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
                ConcernSymbol::Remote,
            ],
            HookEvent::Update => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
            ],
            HookEvent::PostUpdate => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
            ],
            HookEvent::PreAutoGc => vec![
                ConcernSymbol::ConfigGc,
            ],
            HookEvent::PostMerge => vec![
                ConcernSymbol::Commit,
                ConcernSymbol::Tree,
                ConcernSymbol::Ref,
            ],
            HookEvent::PreRebase => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
                ConcernSymbol::ConfigRebase,
            ],
            HookEvent::PostCheckout => vec![
                ConcernSymbol::Head,
                ConcernSymbol::Ref,
                ConcernSymbol::Index,
            ],
            HookEvent::PostCommit => vec![
                ConcernSymbol::Commit,
                ConcernSymbol::Ref,
                ConcernSymbol::Head,
            ],
            HookEvent::PreApplyPatch => vec![
                ConcernSymbol::Index,
                ConcernSymbol::AttrDiffStrategy,
            ],
            HookEvent::PostApplyPatch => vec![
                ConcernSymbol::Index,
                ConcernSymbol::Commit,
            ],
            HookEvent::PreReBase => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
                ConcernSymbol::ConfigRebase,
            ],
            HookEvent::PostReBase => vec![
                ConcernSymbol::Ref,
                ConcernSymbol::Branch,
                ConcernSymbol::Commit,
            ],
            HookEvent::PreCommitMsg => vec![
                ConcernSymbol::ConfigCommit,
                ConcernSymbol::ConfigUser,
            ],
            HookEvent::CommitMsg => vec![
                ConcernSymbol::ConfigCommit,
                ConcernSymbol::ConfigUser,
            ],
            HookEvent::PostCommitMsg => vec![
                ConcernSymbol::Commit,
                ConcernSymbol::Ref,
            ],
            _ => vec![], // Default empty for unimplemented hooks
        }
    }

    /// Step 2: Archive concerns (snapshot current state)
    /// Stateless: Each concern resolved to observed snapshot (pure FS/git read)
    pub fn archive_concerns(&self, concerns: &[ConcernSymbol]) -> Result<ObservedConcernSet> {
        let mut snapshots = HashMap::new();
        
        for concern in concerns {
            let snapshot = self.snapshot_concern(concern)?;
            snapshots.insert(concern.clone(), snapshot);
        }
        
        Ok(ObservedConcernSet {
            snapshots,
            hook_event: HookEvent::PreCommit, // This would be passed in
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Step 3: Map concerns to contracts
    /// Stateless: Map concern symbols to static contracts (pure lookup)
    pub fn map_contracts(&self, concerns: &[ConcernSymbol]) -> Vec<ContractSpec> {
        concerns
            .iter()
            .filter_map(|concern| self.contracts.get(concern).cloned())
            .collect()
    }

    /// Step 4: Specify expectations based on contracts
    /// Stateless: Build expected data for verifier (no side effects)
    pub fn specify_expectations(&self, contracts: &[ContractSpec]) -> Result<ExpectedConcernSet> {
        let mut snapshots = HashMap::new();
        
        for contract in contracts {
            let expected = self.generate_expected_snapshot(contract)?;
            snapshots.insert(contract.concern.clone(), expected);
        }
        
        Ok(ExpectedConcernSet {
            snapshots,
            hook_event: HookEvent::PreCommit, // This would be passed in
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    /// Step 5: Verify observed vs expected
    /// Stateless: Diff observed vs expected
    pub fn verify(
        &self,
        observed: &ObservedConcernSet,
        expected: &ExpectedConcernSet,
    ) -> Result<DiffSet> {
        let mut diffs = Vec::new();
        
        // Check for missing expected concerns
        for (concern, expected_snapshot) in &expected.snapshots {
            if !observed.snapshots.contains_key(concern) {
                diffs.push(ValidationDiff {
                    concern: concern.clone(),
                    diff_type: DiffType::Missing,
                    description: format!("Missing expected concern: {}", concern.name()),
                    observed: None,
                    expected: Some(expected_snapshot.data.clone()),
                    severity: RuleSeverity::Error,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Check for unexpected observed concerns
        for (concern, observed_snapshot) in &observed.snapshots {
            if !expected.snapshots.contains_key(concern) {
                diffs.push(ValidationDiff {
                    concern: concern.clone(),
                    diff_type: DiffType::Unexpected,
                    description: format!("Unexpected observed concern: {}", concern.name()),
                    observed: Some(observed_snapshot.data.clone()),
                    expected: None,
                    severity: RuleSeverity::Warning,
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Check for mismatches in common concerns
        for (concern, observed_snapshot) in &observed.snapshots {
            if let Some(expected_snapshot) = expected.snapshots.get(concern) {
                let concern_diffs = self.compare_snapshots(concern, observed_snapshot, expected_snapshot)?;
                diffs.extend(concern_diffs);
            }
        }
        
        let is_valid = diffs.iter().all(|diff| {
            matches!(diff.severity, RuleSeverity::Info | RuleSeverity::Warning)
        });
        
        let summary = if is_valid {
            "Validation passed".to_string()
        } else {
            format!("Validation failed with {} differences", diffs.len())
        };
        
        Ok(DiffSet {
            diffs,
            is_valid,
            summary,
        })
    }

    /// Complete pipeline execution
    pub fn execute_pipeline(&self, hook: &HookEvent) -> Result<DiffSet> {
        // Step 1: Identify concerns
        let concerns = self.identify_concerns(hook);
        
        // Step 2: Archive concerns
        let observed = self.archive_concerns(&concerns)?;
        
        // Step 3: Map contracts
        let contracts = self.map_contracts(&concerns);
        
        // Step 4: Specify expectations
        let expected = self.specify_expectations(&contracts)?;
        
        // Step 5: Verify
        self.verify(&observed, &expected)
    }

    /// Snapshot a specific concern
    fn snapshot_concern(&self, concern: &ConcernSymbol) -> Result<ConcernSnapshot> {
        let data = match concern {
            ConcernSymbol::Index => self.snapshot_index()?,
            ConcernSymbol::AttrLineEndingNormalization => self.snapshot_attr_line_ending()?,
            ConcernSymbol::AttrDiffStrategy => self.snapshot_attr_diff_strategy()?,
            ConcernSymbol::Ref => self.snapshot_refs()?,
            ConcernSymbol::Branch => self.snapshot_branches()?,
            ConcernSymbol::Remote => self.snapshot_remotes()?,
            ConcernSymbol::TreeExecutable => self.snapshot_executable_files()?,
            ConcernSymbol::Commit => self.snapshot_commits()?,
            ConcernSymbol::Tree => self.snapshot_trees()?,
            ConcernSymbol::Head => self.snapshot_head()?,
            ConcernSymbol::ConfigCommit => self.snapshot_config_commit()?,
            ConcernSymbol::ConfigUser => self.snapshot_config_user()?,
            ConcernSymbol::ConfigGc => self.snapshot_config_gc()?,
            ConcernSymbol::ConfigRebase => self.snapshot_config_rebase()?,
            _ => serde_json::json!({ "error": "Unimplemented concern snapshot" }),
        };
        
        let hash = self.compute_hash(&data);
        
        Ok(ConcernSnapshot {
            concern: concern.clone(),
            data,
            timestamp: chrono::Utc::now().to_rfc3339(),
            hash,
            metadata: HashMap::new(),
        })
    }

    /// Generate expected snapshot from contract
    fn generate_expected_snapshot(&self, contract: &ContractSpec) -> Result<ExpectedSnapshot> {
        let data = self.apply_contract_rules(contract)?;
        
        Ok(ExpectedSnapshot {
            concern: contract.concern.clone(),
            data,
            contract: contract.name.clone(),
            contract_version: contract.version.clone(),
            metadata: HashMap::new(),
        })
    }

    /// Compare observed and expected snapshots
    fn compare_snapshots(
        &self,
        concern: &ConcernSymbol,
        observed: &ConcernSnapshot,
        expected: &ExpectedSnapshot,
    ) -> Result<Vec<ValidationDiff>> {
        let mut diffs = Vec::new();
        
        // Simple JSON comparison for now
        if observed.data != expected.data {
            diffs.push(ValidationDiff {
                concern: concern.clone(),
                diff_type: DiffType::Mismatch,
                description: format!("Data mismatch for concern: {}", concern.name()),
                observed: Some(observed.data.clone()),
                expected: Some(expected.data.clone()),
                severity: RuleSeverity::Error,
                metadata: HashMap::new(),
            });
        }
        
        Ok(diffs)
    }

    /// Apply contract rules to generate expected data
    fn apply_contract_rules(&self, contract: &ContractSpec) -> Result<serde_json::Value> {
        // For now, return a basic structure based on the concern
        match &contract.concern {
            ConcernSymbol::Index => Ok(serde_json::json!({
                "staged_files": [],
                "unstaged_files": [],
                "untracked_files": []
            })),
            ConcernSymbol::AttrLineEndingNormalization => Ok(serde_json::json!({
                "text_files": [],
                "binary_files": [],
                "line_ending_rules": []
            })),
            ConcernSymbol::AttrDiffStrategy => Ok(serde_json::json!({
                "diff_rules": [],
                "binary_files": []
            })),
            _ => Ok(serde_json::json!({ "expected": "default" })),
        }
    }

    /// Compute hash of data
    fn compute_hash(&self, data: &serde_json::Value) -> String {
        use sha2::{Digest, Sha256};
        let json_string = serde_json::to_string(data).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_string.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    // Snapshot methods for different concerns
    fn snapshot_index(&self) -> Result<serde_json::Value> {
        // This would use git2 or gix to read the index
        Ok(serde_json::json!({
            "staged_files": [],
            "unstaged_files": [],
            "untracked_files": []
        }))
    }

    fn snapshot_attr_line_ending(&self) -> Result<serde_json::Value> {
        // This would read .gitattributes and analyze files
        Ok(serde_json::json!({
            "text_files": [],
            "binary_files": [],
            "line_ending_rules": []
        }))
    }

    fn snapshot_attr_diff_strategy(&self) -> Result<serde_json::Value> {
        // This would read .gitattributes for diff rules
        Ok(serde_json::json!({
            "diff_rules": [],
            "binary_files": []
        }))
    }

    fn snapshot_refs(&self) -> Result<serde_json::Value> {
        // This would read .git/refs
        Ok(serde_json::json!({
            "heads": [],
            "tags": [],
            "remotes": []
        }))
    }

    fn snapshot_branches(&self) -> Result<serde_json::Value> {
        // This would read branch information
        Ok(serde_json::json!({
            "current": "main",
            "branches": []
        }))
    }

    fn snapshot_remotes(&self) -> Result<serde_json::Value> {
        // This would read remote configuration
        Ok(serde_json::json!({
            "remotes": []
        }))
    }

    fn snapshot_executable_files(&self) -> Result<serde_json::Value> {
        // This would scan for executable files
        Ok(serde_json::json!({
            "executable_files": []
        }))
    }

    fn snapshot_commits(&self) -> Result<serde_json::Value> {
        // This would read commit information
        Ok(serde_json::json!({
            "commits": []
        }))
    }

    fn snapshot_trees(&self) -> Result<serde_json::Value> {
        // This would read tree information
        Ok(serde_json::json!({
            "trees": []
        }))
    }

    fn snapshot_head(&self) -> Result<serde_json::Value> {
        // This would read HEAD
        Ok(serde_json::json!({
            "head": "refs/heads/main"
        }))
    }

    fn snapshot_config_commit(&self) -> Result<serde_json::Value> {
        // This would read commit config
        Ok(serde_json::json!({
            "commit_config": {}
        }))
    }

    fn snapshot_config_user(&self) -> Result<serde_json::Value> {
        // This would read user config
        Ok(serde_json::json!({
            "user_config": {}
        }))
    }

    fn snapshot_config_gc(&self) -> Result<serde_json::Value> {
        // This would read gc config
        Ok(serde_json::json!({
            "gc_config": {}
        }))
    }

    fn snapshot_config_rebase(&self) -> Result<serde_json::Value> {
        // This would read rebase config
        Ok(serde_json::json!({
            "rebase_config": {}
        }))
    }
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identify_concerns() {
        let pipeline = FunctionalContractPipeline::new("/tmp/test");
        let concerns = pipeline.identify_concerns(&HookEvent::PreCommit);
        assert!(!concerns.is_empty());
        assert!(concerns.contains(&ConcernSymbol::Index));
    }

    #[test]
    fn test_archive_concerns() {
        let pipeline = FunctionalContractPipeline::new("/tmp/test");
        let concerns = vec![ConcernSymbol::Index, ConcernSymbol::AttrLineEndingNormalization];
        let observed = pipeline.archive_concerns(&concerns).unwrap();
        assert_eq!(observed.snapshots.len(), 2);
    }

    #[test]
    fn test_map_contracts() {
        let mut pipeline = FunctionalContractPipeline::new("/tmp/test");
        
        let contract = ContractSpec {
            name: "test".to_string(),
            version: "1.0".to_string(),
            concern: ConcernSymbol::Index,
            rules: vec![],
            metadata: HashMap::new(),
        };
        
        pipeline.register_contract(contract).unwrap();
        
        let concerns = vec![ConcernSymbol::Index];
        let contracts = pipeline.map_contracts(&concerns);
        assert_eq!(contracts.len(), 1);
    }

    #[test]
    fn test_verify() {
        let pipeline = FunctionalContractPipeline::new("/tmp/test");
        
        let observed = ObservedConcernSet {
            snapshots: HashMap::new(),
            hook_event: HookEvent::PreCommit,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let expected = ExpectedConcernSet {
            snapshots: HashMap::new(),
            hook_event: HookEvent::PreCommit,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        let diff = pipeline.verify(&observed, &expected).unwrap();
        assert!(diff.is_valid);
    }

    #[test]
    fn test_execute_pipeline() {
        let pipeline = FunctionalContractPipeline::new("/tmp/test");
        let diff = pipeline.execute_pipeline(&HookEvent::PreCommit).unwrap();
        assert!(diff.is_valid);
    }
}
