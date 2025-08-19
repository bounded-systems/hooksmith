use anyhow::{bail, Result};
use std::collections::HashMap;

/// Git-native object types that map directly to git2::ObjectType and gix::ObjectKind
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitObjectType {
    /// File contents (git2::ObjectType::Blob, gix::ObjectKind::Blob)
    Blob,
    /// Directory structure (git2::ObjectType::Tree, gix::ObjectKind::Tree)
    Tree,
    /// Commit history (git2::ObjectType::Commit, gix::ObjectKind::Commit)
    Commit,
    /// Annotated tag (git2::ObjectType::Tag, gix::ObjectKind::Tag)
    Tag,
}

/// Git tree entry types (specific file modes)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitTreeEntryType {
    /// Regular file (100644) - Blob
    TreeFile,
    /// Executable file (100755) - Blob
    TreeExecutable,
    /// Symlink (120000) - Blob
    TreeSymlink,
    /// Directory (040000) - Tree
    TreeDirectory,
    /// Submodule (160000) - Commit (gitlink)
    TreeSubmodule,
}

/// Git namespaced metadata types (not objects but tracked by Git)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitMetadataType {
    /// References (heads, tags, etc.) - tracked in .git/refs/
    Ref,
    /// Notes (commit-attached metadata) - tracked in .git/refs/notes/
    Note,
    /// Attributes (file-based config) - tracked in working tree or .git/info
    Attr,
    /// Index (staging area) - tracked in .git/index
    Index,
    /// Stash (pseudo-refs for uncommitted work) - tracked in .git/refs/stash
    Stash,
    /// Worktree (linked working directories) - tracked in .git/worktrees/
    Worktree,
    /// Remote (remote repository configurations) - tracked in .git/config
    Remote,
    /// Branch (branch-specific configurations) - tracked in .git/config
    Branch,
    /// Head (current branch reference) - tracked in .git/HEAD
    Head,
    /// Reflog (reference history) - tracked in .git/logs/
    Reflog,
}

/// Git config section types (first-class concerns)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitConfigType {
    /// User configuration settings
    ConfigUser,
    /// Core configuration settings
    ConfigCore,
    /// Branch configuration settings
    ConfigBranch,
    /// Remote configuration settings
    ConfigRemote,
    /// Init configuration settings
    ConfigInit,
    /// Color configuration settings
    ConfigColor,
    /// Alias configuration settings
    ConfigAlias,
    /// Diff configuration settings
    ConfigDiff,
    /// Merge configuration settings
    ConfigMerge,
    /// GPG configuration settings
    ConfigGpg,
    /// Commit configuration settings
    ConfigCommit,
    /// Pull configuration settings
    ConfigPull,
    /// Push configuration settings
    ConfigPush,
    /// Rebase configuration settings
    ConfigRebase,
    /// Fetch configuration settings
    ConfigFetch,
    /// Status configuration settings
    ConfigStatus,
    /// Tar configuration settings
    ConfigTar,
    /// Rerere configuration settings
    ConfigRerere,
    /// Advice configuration settings
    ConfigAdvice,
    /// Interactive configuration settings
    ConfigInteractive,
    /// Submodule configuration settings
    ConfigSubmodule,
    /// Filter configuration settings
    ConfigFilter,
    /// Include configuration settings
    ConfigInclude,
    /// Credential configuration settings
    ConfigCredential,
    /// HTTP configuration settings
    ConfigHttp,
    /// URL configuration settings
    ConfigUrl,
    /// Safe configuration settings
    ConfigSafe,
    /// Notes configuration settings
    ConfigNotes,
    /// Garbage collection configuration settings
    ConfigGc,
    /// Maintenance configuration settings
    ConfigMaintenance,
    /// Pager configuration settings
    ConfigPager,
    /// Worktree configuration settings
    ConfigWorktree,
}

/// Git attribute behavior types (structured concerns)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitAttributeType {
    /// Line ending normalization (text, eol=lf, eol=crlf)
    AttrLineEndingNormalization,
    /// Diff strategy (diff, binary)
    AttrDiffStrategy,
    /// Merge strategy (merge=...)
    AttrMergeStrategy,
    /// Export control (export-ignore, export-subst)
    AttrExportControl,
    /// Filter driver (filter=...)
    AttrFilterDriver,
    /// External tool hints (linguist-language, linguist-vendored)
    AttrExternalToolHint,
    /// Locking hints (lockable for Git LFS)
    AttrLockingHint,
}

/// Git storage file types (.git/ structure)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitStorageType {
    /// Git HEAD pointer (current ref)
    HeadPointer,
    /// Git index/staging area
    IndexEntry,
    /// Git index file
    IndexFile,
    /// Git index stage
    IndexStage,
    /// Git branch references
    RefBranch,
    /// Git remote-tracking branch references
    RefRemoteBranch,
    /// Git tag references
    RefTag,
    /// Git packed references
    RefPacked,
    /// Git notes references
    NoteRef,
    /// Git repository-only attributes
    AttrRepoOnly,
    /// Git repository-only ignore patterns
    IgnoreRepoOnly,
    /// Git local configuration
    ConfigLocal,
    /// Git hook scripts
    HookScript,
    /// Git hook lifecycle
    HookLifecycle,
    /// Git reference logs
    RefLog,
    /// Git reference log entries
    RefLogEntry,
    /// Git worktree metadata
    WorktreeMeta,
    /// Git worktree lock
    WorktreeLock,
    /// Git rebase plan
    RebasePlan,
    /// Git rebase cache entry
    RRCacheEntry,
    /// Git merge state
    MergeState,
    /// Git merge head
    MergeHead,
    /// Git original head pointer
    OrigHeadPointer,
    /// Git commit message draft
    CommitMessageDraft,
    /// Git fetch head pointer
    FetchHeadPointer,
    /// Git repository description
    RepoDescription,
    /// Git filesystem monitor state
    FsMonitorState,
    /// Git shallow clone depth
    ShallowCloneDepth,
}

/// Git pattern types (ignore/attributes)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitPatternType {
    /// Git tree ignore patterns
    TreeIgnorePattern,
    /// Git global ignore patterns
    IgnoreGlobalPattern,
    /// Git attribute patterns
    AttrPattern,
    /// Git global attributes
    AttrGlobal,
}

/// Git remote and network types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitRemoteType {
    /// Git remote origin configuration
    RemoteOrigin,
    /// Git remote configuration
    RemoteConfig,
    /// Git push strategy configuration
    PushStrategyConfig,
    /// Git credential helper configuration
    CredentialHelperConfig,
    /// Git remote URL aliases
    RemoteURLAlias,
}

/// Git state management types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GitStateType {
    /// Git stash entries
    StashEntry,
    /// Git stash references
    StashRef,
    /// Git stash metadata
    StashMeta,
    /// Git rebase steps
    RebaseStep,
    /// Git merge conflict markers
    MergeConflictMarker,
    /// Git bisect state
    BisectState,
    /// Git bisect log
    BisectLog,
    /// Git tag objects
    TagObject,
    /// Git worktree index
    WorktreeIndex,
    /// Git worktree branch link
    WorktreeBranchLink,
    /// Git hook triggers
    HookTrigger,
    /// Git index conflicts
    IndexConflict,
    /// Git index modes
    IndexMode,
    /// Git global configuration
    ConfigGlobal,
    /// Git system configuration
    ConfigSystem,
}

/// Git-native validation context
pub struct GitNativeValidator {
    #[allow(dead_code)]
    object_counts: HashMap<GitObjectType, u32>,
    #[allow(dead_code)]
    metadata_counts: HashMap<GitMetadataType, u32>,
}

impl GitNativeValidator {
    /// Create a new Git-native validator instance
    pub fn new() -> Self {
        Self {
            object_counts: HashMap::new(),
            metadata_counts: HashMap::new(),
        }
    }

    /// Validate Git object types against canonical Git types
    pub fn validate_object_types(&self, object_counts: &HashMap<String, u32>) -> Result<()> {
        let mut errors = Vec::new();

        for (object_type, count) in object_counts {
            match object_type.as_str() {
                "blob" | "tree" | "commit" | "tag" => {
                    // These are valid Git object types
                }
                _ => {
                    errors.push(format!(
                        "Unknown Git object type '{}' found {} times",
                        object_type, count
                    ));
                }
            }
        }

        if !errors.is_empty() {
            bail!("Git object validation failed:\n{}", errors.join("\n"));
        }

        Ok(())
    }

    /// Map string object types to Git-native enums
    pub fn map_object_type(object_type: &str) -> Option<GitObjectType> {
        match object_type {
            "blob" => Some(GitObjectType::Blob),
            "tree" => Some(GitObjectType::Tree),
            "commit" => Some(GitObjectType::Commit),
            "tag" => Some(GitObjectType::Tag),
            _ => None,
        }
    }

    /// Map string tree entry types to Git-native enums
    pub fn map_tree_entry_type(tree_entry_type: &str) -> Option<GitTreeEntryType> {
        match tree_entry_type {
            "tree-file" => Some(GitTreeEntryType::TreeFile),
            "tree-executable" => Some(GitTreeEntryType::TreeExecutable),
            "tree-symlink" => Some(GitTreeEntryType::TreeSymlink),
            "tree-directory" => Some(GitTreeEntryType::TreeDirectory),
            "tree-submodule" => Some(GitTreeEntryType::TreeSubmodule),
            _ => None,
        }
    }

    /// Map string metadata types to Git-native enums
    pub fn map_metadata_type(metadata_type: &str) -> Option<GitMetadataType> {
        match metadata_type {
            "ref" => Some(GitMetadataType::Ref),
            "note" => Some(GitMetadataType::Note),
            "attr" => Some(GitMetadataType::Attr),
            "index" => Some(GitMetadataType::Index),
            "stash" => Some(GitMetadataType::Stash),
            "worktree" => Some(GitMetadataType::Worktree),
            "remote" => Some(GitMetadataType::Remote),
            "branch" => Some(GitMetadataType::Branch),
            "head" => Some(GitMetadataType::Head),
            "reflog" => Some(GitMetadataType::Reflog),
            _ => None,
        }
    }

    /// Map string config types to Git-native enums
    pub fn map_config_type(config_type: &str) -> Option<GitConfigType> {
        match config_type {
            "config-user" => Some(GitConfigType::ConfigUser),
            "config-core" => Some(GitConfigType::ConfigCore),
            "config-branch" => Some(GitConfigType::ConfigBranch),
            "config-remote" => Some(GitConfigType::ConfigRemote),
            "config-init" => Some(GitConfigType::ConfigInit),
            "config-color" => Some(GitConfigType::ConfigColor),
            "config-alias" => Some(GitConfigType::ConfigAlias),
            "config-diff" => Some(GitConfigType::ConfigDiff),
            "config-merge" => Some(GitConfigType::ConfigMerge),
            "config-gpg" => Some(GitConfigType::ConfigGpg),
            "config-commit" => Some(GitConfigType::ConfigCommit),
            "config-pull" => Some(GitConfigType::ConfigPull),
            "config-push" => Some(GitConfigType::ConfigPush),
            "config-rebase" => Some(GitConfigType::ConfigRebase),
            "config-fetch" => Some(GitConfigType::ConfigFetch),
            "config-status" => Some(GitConfigType::ConfigStatus),
            "config-tar" => Some(GitConfigType::ConfigTar),
            "config-rerere" => Some(GitConfigType::ConfigRerere),
            "config-advice" => Some(GitConfigType::ConfigAdvice),
            "config-interactive" => Some(GitConfigType::ConfigInteractive),
            "config-submodule" => Some(GitConfigType::ConfigSubmodule),
            "config-filter" => Some(GitConfigType::ConfigFilter),
            "config-include" => Some(GitConfigType::ConfigInclude),
            "config-credential" => Some(GitConfigType::ConfigCredential),
            "config-http" => Some(GitConfigType::ConfigHttp),
            "config-url" => Some(GitConfigType::ConfigUrl),
            "config-safe" => Some(GitConfigType::ConfigSafe),
            "config-notes" => Some(GitConfigType::ConfigNotes),
            "config-gc" => Some(GitConfigType::ConfigGc),
            "config-maintenance" => Some(GitConfigType::ConfigMaintenance),
            "config-pager" => Some(GitConfigType::ConfigPager),
            "config-worktree" => Some(GitConfigType::ConfigWorktree),
            _ => None,
        }
    }

    /// Map string attribute types to Git-native enums
    pub fn map_attribute_type(attribute_type: &str) -> Option<GitAttributeType> {
        match attribute_type {
            "attr-line-ending-normalization" => Some(GitAttributeType::AttrLineEndingNormalization),
            "attr-diff-strategy" => Some(GitAttributeType::AttrDiffStrategy),
            "attr-merge-strategy" => Some(GitAttributeType::AttrMergeStrategy),
            "attr-export-control" => Some(GitAttributeType::AttrExportControl),
            "attr-filter-driver" => Some(GitAttributeType::AttrFilterDriver),
            "attr-external-tool-hint" => Some(GitAttributeType::AttrExternalToolHint),
            "attr-locking-hint" => Some(GitAttributeType::AttrLockingHint),
            _ => None,
        }
    }

    /// Map string storage types to Git-native enums
    pub fn map_storage_type(storage_type: &str) -> Option<GitStorageType> {
        match storage_type {
            "head-pointer" => Some(GitStorageType::HeadPointer),
            "index-entry" => Some(GitStorageType::IndexEntry),
            "index-file" => Some(GitStorageType::IndexFile),
            "index-stage" => Some(GitStorageType::IndexStage),
            "ref-branch" => Some(GitStorageType::RefBranch),
            "ref-remote-branch" => Some(GitStorageType::RefRemoteBranch),
            "ref-tag" => Some(GitStorageType::RefTag),
            "ref-packed" => Some(GitStorageType::RefPacked),
            "note-ref" => Some(GitStorageType::NoteRef),
            "attr-repo-only" => Some(GitStorageType::AttrRepoOnly),
            "ignore-repo-only" => Some(GitStorageType::IgnoreRepoOnly),
            "config-local" => Some(GitStorageType::ConfigLocal),
            "hook-script" => Some(GitStorageType::HookScript),
            "hook-lifecycle" => Some(GitStorageType::HookLifecycle),
            "ref-log" => Some(GitStorageType::RefLog),
            "ref-log-entry" => Some(GitStorageType::RefLogEntry),
            "worktree-meta" => Some(GitStorageType::WorktreeMeta),
            "worktree-lock" => Some(GitStorageType::WorktreeLock),
            "rebase-plan" => Some(GitStorageType::RebasePlan),
            "rr-cache-entry" => Some(GitStorageType::RRCacheEntry),
            "merge-state" => Some(GitStorageType::MergeState),
            "merge-head" => Some(GitStorageType::MergeHead),
            "orig-head-pointer" => Some(GitStorageType::OrigHeadPointer),
            "commit-message-draft" => Some(GitStorageType::CommitMessageDraft),
            "fetch-head-pointer" => Some(GitStorageType::FetchHeadPointer),
            "repo-description" => Some(GitStorageType::RepoDescription),
            "fs-monitor-state" => Some(GitStorageType::FsMonitorState),
            "shallow-clone-depth" => Some(GitStorageType::ShallowCloneDepth),
            _ => None,
        }
    }

    /// Map string pattern types to Git-native enums
    pub fn map_pattern_type(pattern_type: &str) -> Option<GitPatternType> {
        match pattern_type {
            "tree-ignore-pattern" => Some(GitPatternType::TreeIgnorePattern),
            "ignore-global-pattern" => Some(GitPatternType::IgnoreGlobalPattern),
            "attr-pattern" => Some(GitPatternType::AttrPattern),
            "attr-global" => Some(GitPatternType::AttrGlobal),
            _ => None,
        }
    }

    /// Map string remote types to Git-native enums
    pub fn map_remote_type(remote_type: &str) -> Option<GitRemoteType> {
        match remote_type {
            "remote-origin" => Some(GitRemoteType::RemoteOrigin),
            "remote-config" => Some(GitRemoteType::RemoteConfig),
            "push-strategy-config" => Some(GitRemoteType::PushStrategyConfig),
            "credential-helper-config" => Some(GitRemoteType::CredentialHelperConfig),
            "remote-url-alias" => Some(GitRemoteType::RemoteURLAlias),
            _ => None,
        }
    }

    /// Map string state types to Git-native enums
    pub fn map_state_type(state_type: &str) -> Option<GitStateType> {
        match state_type {
            "stash-entry" => Some(GitStateType::StashEntry),
            "stash-ref" => Some(GitStateType::StashRef),
            "stash-meta" => Some(GitStateType::StashMeta),
            "rebase-step" => Some(GitStateType::RebaseStep),
            "merge-conflict-marker" => Some(GitStateType::MergeConflictMarker),
            "bisect-state" => Some(GitStateType::BisectState),
            "bisect-log" => Some(GitStateType::BisectLog),
            "tag-object" => Some(GitStateType::TagObject),
            "worktree-index" => Some(GitStateType::WorktreeIndex),
            "worktree-branch-link" => Some(GitStateType::WorktreeBranchLink),
            "hook-trigger" => Some(GitStateType::HookTrigger),
            "index-conflict" => Some(GitStateType::IndexConflict),
            "index-mode" => Some(GitStateType::IndexMode),
            "config-global" => Some(GitStateType::ConfigGlobal),
            "config-system" => Some(GitStateType::ConfigSystem),
            _ => None,
        }
    }

    /// Get canonical Git object type names
    pub fn canonical_object_types() -> Vec<&'static str> {
        vec!["blob", "tree", "commit", "tag"]
    }

    /// Get canonical Git tree entry type names
    pub fn canonical_tree_entry_types() -> Vec<&'static str> {
        vec![
            "tree-file",
            "tree-executable",
            "tree-symlink",
            "tree-directory",
            "tree-submodule",
        ]
    }

    /// Get canonical Git metadata type names
    pub fn canonical_metadata_types() -> Vec<&'static str> {
        vec![
            "ref", "note", "attr", "index", "stash", "worktree", "remote", "branch", "head",
            "reflog",
        ]
    }

    /// Get canonical Git config type names
    pub fn canonical_config_types() -> Vec<&'static str> {
        vec![
            "config-user",
            "config-core",
            "config-branch",
            "config-remote",
            "config-init",
            "config-color",
            "config-alias",
            "config-diff",
            "config-merge",
            "config-gpg",
            "config-commit",
            "config-pull",
            "config-push",
            "config-rebase",
            "config-fetch",
            "config-status",
            "config-tar",
            "config-rerere",
            "config-advice",
            "config-interactive",
            "config-submodule",
            "config-filter",
            "config-include",
            "config-credential",
            "config-http",
            "config-url",
            "config-safe",
            "config-notes",
            "config-gc",
            "config-maintenance",
            "config-pager",
            "config-worktree",
        ]
    }

    /// Get canonical Git attribute type names
    pub fn canonical_attribute_types() -> Vec<&'static str> {
        vec![
            "attr-line-ending-normalization",
            "attr-diff-strategy",
            "attr-merge-strategy",
            "attr-export-control",
            "attr-filter-driver",
            "attr-external-tool-hint",
            "attr-locking-hint",
        ]
    }

    /// Get canonical Git storage type names
    pub fn canonical_storage_types() -> Vec<&'static str> {
        vec![
            "head-pointer",
            "index-entry",
            "index-file",
            "index-stage",
            "ref-branch",
            "ref-remote-branch",
            "ref-tag",
            "ref-packed",
            "note-ref",
            "attr-repo-only",
            "ignore-repo-only",
            "config-local",
            "hook-script",
            "hook-lifecycle",
            "ref-log",
            "ref-log-entry",
            "worktree-meta",
            "worktree-lock",
            "rebase-plan",
            "rr-cache-entry",
            "merge-state",
            "merge-head",
            "orig-head-pointer",
            "commit-message-draft",
            "fetch-head-pointer",
            "repo-description",
            "fs-monitor-state",
            "shallow-clone-depth",
        ]
    }

    /// Get canonical Git pattern type names
    pub fn canonical_pattern_types() -> Vec<&'static str> {
        vec![
            "tree-ignore-pattern",
            "ignore-global-pattern",
            "attr-pattern",
            "attr-global",
        ]
    }

    /// Get canonical Git remote type names
    pub fn canonical_remote_types() -> Vec<&'static str> {
        vec![
            "remote-origin",
            "remote-config",
            "push-strategy-config",
            "credential-helper-config",
            "remote-url-alias",
        ]
    }

    /// Get canonical Git state type names
    pub fn canonical_state_types() -> Vec<&'static str> {
        vec![
            "stash-entry",
            "stash-ref",
            "stash-meta",
            "rebase-step",
            "merge-conflict-marker",
            "bisect-state",
            "bisect-log",
            "tag-object",
            "worktree-index",
            "worktree-branch-link",
            "hook-trigger",
            "index-conflict",
            "index-mode",
            "config-global",
            "config-system",
        ]
    }

    /// Validate that all concerns are Git-native
    pub fn validate_concerns(concerns: &[String]) -> Result<()> {
        let canonical_objects = Self::canonical_object_types();
        let canonical_tree_entries = Self::canonical_tree_entry_types();
        let canonical_metadata = Self::canonical_metadata_types();
        let canonical_configs = Self::canonical_config_types();
        let canonical_attributes = Self::canonical_attribute_types();
        let canonical_storage = Self::canonical_storage_types();
        let canonical_patterns = Self::canonical_pattern_types();
        let canonical_remotes = Self::canonical_remote_types();
        let canonical_states = Self::canonical_state_types();
        let all_canonical: Vec<&str> = canonical_objects
            .iter()
            .chain(canonical_tree_entries.iter())
            .chain(canonical_metadata.iter())
            .chain(canonical_configs.iter())
            .chain(canonical_attributes.iter())
            .chain(canonical_storage.iter())
            .chain(canonical_patterns.iter())
            .chain(canonical_remotes.iter())
            .chain(canonical_states.iter())
            .cloned()
            .collect();

        let mut errors = Vec::new();

        for concern in concerns {
            if !all_canonical.contains(&concern.as_str()) {
                errors.push(format!("Non-Git-native concern '{}' not allowed", concern));
            }
        }

        if !errors.is_empty() {
            bail!("Hook concerns validation failed:\n{}", errors.join("\n"));
        }

        Ok(())
    }
}

impl Default for GitNativeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_object_type() {
        assert_eq!(
            GitNativeValidator::map_object_type("blob"),
            Some(GitObjectType::Blob)
        );
        assert_eq!(
            GitNativeValidator::map_object_type("tree"),
            Some(GitObjectType::Tree)
        );
        assert_eq!(
            GitNativeValidator::map_object_type("commit"),
            Some(GitObjectType::Commit)
        );
        assert_eq!(
            GitNativeValidator::map_object_type("tag"),
            Some(GitObjectType::Tag)
        );
        assert_eq!(GitNativeValidator::map_object_type("invalid"), None);
    }

    #[test]
    fn test_map_tree_entry_type() {
        assert_eq!(
            GitNativeValidator::map_tree_entry_type("tree-file"),
            Some(GitTreeEntryType::TreeFile)
        );
        assert_eq!(
            GitNativeValidator::map_tree_entry_type("tree-executable"),
            Some(GitTreeEntryType::TreeExecutable)
        );
        assert_eq!(
            GitNativeValidator::map_tree_entry_type("tree-symlink"),
            Some(GitTreeEntryType::TreeSymlink)
        );
        assert_eq!(
            GitNativeValidator::map_tree_entry_type("tree-directory"),
            Some(GitTreeEntryType::TreeDirectory)
        );
        assert_eq!(
            GitNativeValidator::map_tree_entry_type("tree-submodule"),
            Some(GitTreeEntryType::TreeSubmodule)
        );
        assert_eq!(GitNativeValidator::map_tree_entry_type("invalid"), None);
    }

    #[test]
    fn test_map_metadata_type() {
        assert_eq!(
            GitNativeValidator::map_metadata_type("ref"),
            Some(GitMetadataType::Ref)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("note"),
            Some(GitMetadataType::Note)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("attr"),
            Some(GitMetadataType::Attr)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("index"),
            Some(GitMetadataType::Index)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("stash"),
            Some(GitMetadataType::Stash)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("worktree"),
            Some(GitMetadataType::Worktree)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("remote"),
            Some(GitMetadataType::Remote)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("branch"),
            Some(GitMetadataType::Branch)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("head"),
            Some(GitMetadataType::Head)
        );
        assert_eq!(
            GitNativeValidator::map_metadata_type("reflog"),
            Some(GitMetadataType::Reflog)
        );
        assert_eq!(GitNativeValidator::map_metadata_type("invalid"), None);
    }

    #[test]
    fn test_map_config_type() {
        assert_eq!(
            GitNativeValidator::map_config_type("config-user"),
            Some(GitConfigType::ConfigUser)
        );
        assert_eq!(
            GitNativeValidator::map_config_type("config-core"),
            Some(GitConfigType::ConfigCore)
        );
        assert_eq!(
            GitNativeValidator::map_config_type("config-branch"),
            Some(GitConfigType::ConfigBranch)
        );
        assert_eq!(
            GitNativeValidator::map_config_type("config-remote"),
            Some(GitConfigType::ConfigRemote)
        );
        assert_eq!(
            GitNativeValidator::map_config_type("config-commit"),
            Some(GitConfigType::ConfigCommit)
        );
        assert_eq!(
            GitNativeValidator::map_config_type("config-push"),
            Some(GitConfigType::ConfigPush)
        );
        assert_eq!(
            GitNativeValidator::map_config_type("config-worktree"),
            Some(GitConfigType::ConfigWorktree)
        );
        assert_eq!(GitNativeValidator::map_config_type("invalid"), None);
    }

    #[test]
    fn test_map_attribute_type() {
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-line-ending-normalization"),
            Some(GitAttributeType::AttrLineEndingNormalization)
        );
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-diff-strategy"),
            Some(GitAttributeType::AttrDiffStrategy)
        );
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-merge-strategy"),
            Some(GitAttributeType::AttrMergeStrategy)
        );
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-export-control"),
            Some(GitAttributeType::AttrExportControl)
        );
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-filter-driver"),
            Some(GitAttributeType::AttrFilterDriver)
        );
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-external-tool-hint"),
            Some(GitAttributeType::AttrExternalToolHint)
        );
        assert_eq!(
            GitNativeValidator::map_attribute_type("attr-locking-hint"),
            Some(GitAttributeType::AttrLockingHint)
        );
        assert_eq!(GitNativeValidator::map_attribute_type("invalid"), None);
    }

    #[test]
    fn test_map_storage_type() {
        assert_eq!(
            GitNativeValidator::map_storage_type("head-pointer"),
            Some(GitStorageType::HeadPointer)
        );
        assert_eq!(
            GitNativeValidator::map_storage_type("index-entry"),
            Some(GitStorageType::IndexEntry)
        );
        assert_eq!(
            GitNativeValidator::map_storage_type("index-file"),
            Some(GitStorageType::IndexFile)
        );
        assert_eq!(
            GitNativeValidator::map_storage_type("ref-branch"),
            Some(GitStorageType::RefBranch)
        );
        assert_eq!(
            GitNativeValidator::map_storage_type("ref-remote-branch"),
            Some(GitStorageType::RefRemoteBranch)
        );
        assert_eq!(
            GitNativeValidator::map_storage_type("merge-state"),
            Some(GitStorageType::MergeState)
        );
        assert_eq!(GitNativeValidator::map_storage_type("invalid"), None);
    }

    #[test]
    fn test_map_pattern_type() {
        assert_eq!(
            GitNativeValidator::map_pattern_type("tree-ignore-pattern"),
            Some(GitPatternType::TreeIgnorePattern)
        );
        assert_eq!(
            GitNativeValidator::map_pattern_type("ignore-global-pattern"),
            Some(GitPatternType::IgnoreGlobalPattern)
        );
        assert_eq!(
            GitNativeValidator::map_pattern_type("attr-pattern"),
            Some(GitPatternType::AttrPattern)
        );
        assert_eq!(
            GitNativeValidator::map_pattern_type("attr-global"),
            Some(GitPatternType::AttrGlobal)
        );
        assert_eq!(GitNativeValidator::map_pattern_type("invalid"), None);
    }

    #[test]
    fn test_map_remote_type() {
        assert_eq!(
            GitNativeValidator::map_remote_type("remote-origin"),
            Some(GitRemoteType::RemoteOrigin)
        );
        assert_eq!(
            GitNativeValidator::map_remote_type("remote-config"),
            Some(GitRemoteType::RemoteConfig)
        );
        assert_eq!(
            GitNativeValidator::map_remote_type("push-strategy-config"),
            Some(GitRemoteType::PushStrategyConfig)
        );
        assert_eq!(
            GitNativeValidator::map_remote_type("credential-helper-config"),
            Some(GitRemoteType::CredentialHelperConfig)
        );
        assert_eq!(
            GitNativeValidator::map_remote_type("remote-url-alias"),
            Some(GitRemoteType::RemoteURLAlias)
        );
        assert_eq!(GitNativeValidator::map_remote_type("invalid"), None);
    }

    #[test]
    fn test_map_state_type() {
        assert_eq!(
            GitNativeValidator::map_state_type("stash-entry"),
            Some(GitStateType::StashEntry)
        );
        assert_eq!(
            GitNativeValidator::map_state_type("stash-ref"),
            Some(GitStateType::StashRef)
        );
        assert_eq!(
            GitNativeValidator::map_state_type("rebase-step"),
            Some(GitStateType::RebaseStep)
        );
        assert_eq!(
            GitNativeValidator::map_state_type("merge-conflict-marker"),
            Some(GitStateType::MergeConflictMarker)
        );
        assert_eq!(
            GitNativeValidator::map_state_type("bisect-state"),
            Some(GitStateType::BisectState)
        );
        assert_eq!(
            GitNativeValidator::map_state_type("tag-object"),
            Some(GitStateType::TagObject)
        );
        assert_eq!(
            GitNativeValidator::map_state_type("config-global"),
            Some(GitStateType::ConfigGlobal)
        );
        assert_eq!(GitNativeValidator::map_state_type("invalid"), None);
    }

    #[test]
    fn test_validate_concerns() {
        let valid_concerns = vec!["blob".to_string(), "tree".to_string(), "ref".to_string()];
        assert!(GitNativeValidator::validate_concerns(&valid_concerns).is_ok());

        let invalid_concerns = vec!["blob".to_string(), "contract-violation".to_string()];
        assert!(GitNativeValidator::validate_concerns(&invalid_concerns).is_err());
    }
}
