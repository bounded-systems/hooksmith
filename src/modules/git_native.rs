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

/// Git-native validation context
pub struct GitNativeValidator {
    object_counts: HashMap<GitObjectType, u32>,
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
                    errors.push(format!("Unknown Git object type '{}' found {} times", object_type, count));
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

    /// Get canonical Git object type names
    pub fn canonical_object_types() -> Vec<&'static str> {
        vec!["blob", "tree", "commit", "tag"]
    }

    /// Get canonical Git tree entry type names
    pub fn canonical_tree_entry_types() -> Vec<&'static str> {
        vec!["tree-file", "tree-executable", "tree-symlink", "tree-directory", "tree-submodule"]
    }

    /// Get canonical Git metadata type names
    pub fn canonical_metadata_types() -> Vec<&'static str> {
        vec!["ref", "note", "attr", "index", "stash", "worktree", "remote", "branch", "head", "reflog"]
    }

    /// Get canonical Git config type names
    pub fn canonical_config_types() -> Vec<&'static str> {
        vec![
            "config-user", "config-core", "config-branch", "config-remote", "config-init",
            "config-color", "config-alias", "config-diff", "config-merge", "config-gpg",
            "config-commit", "config-pull", "config-push", "config-rebase", "config-fetch",
            "config-status", "config-tar", "config-rerere", "config-advice", "config-interactive",
            "config-submodule", "config-filter", "config-include", "config-credential",
            "config-http", "config-url", "config-safe", "config-notes", "config-gc",
            "config-maintenance", "config-pager", "config-worktree"
        ]
    }

    /// Validate that all concerns are Git-native
    pub fn validate_concerns(concerns: &[String]) -> Result<()> {
        let canonical_objects = Self::canonical_object_types();
        let canonical_tree_entries = Self::canonical_tree_entry_types();
        let canonical_metadata = Self::canonical_metadata_types();
        let canonical_configs = Self::canonical_config_types();
        let all_canonical: Vec<&str> = canonical_objects.iter()
            .chain(canonical_tree_entries.iter())
            .chain(canonical_metadata.iter())
            .chain(canonical_configs.iter())
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
        assert_eq!(GitNativeValidator::map_object_type("blob"), Some(GitObjectType::Blob));
        assert_eq!(GitNativeValidator::map_object_type("tree"), Some(GitObjectType::Tree));
        assert_eq!(GitNativeValidator::map_object_type("commit"), Some(GitObjectType::Commit));
        assert_eq!(GitNativeValidator::map_object_type("tag"), Some(GitObjectType::Tag));
        assert_eq!(GitNativeValidator::map_object_type("invalid"), None);
    }

    #[test]
    fn test_map_tree_entry_type() {
        assert_eq!(GitNativeValidator::map_tree_entry_type("tree-file"), Some(GitTreeEntryType::TreeFile));
        assert_eq!(GitNativeValidator::map_tree_entry_type("tree-executable"), Some(GitTreeEntryType::TreeExecutable));
        assert_eq!(GitNativeValidator::map_tree_entry_type("tree-symlink"), Some(GitTreeEntryType::TreeSymlink));
        assert_eq!(GitNativeValidator::map_tree_entry_type("tree-directory"), Some(GitTreeEntryType::TreeDirectory));
        assert_eq!(GitNativeValidator::map_tree_entry_type("tree-submodule"), Some(GitTreeEntryType::TreeSubmodule));
        assert_eq!(GitNativeValidator::map_tree_entry_type("invalid"), None);
    }

    #[test]
    fn test_map_metadata_type() {
        assert_eq!(GitNativeValidator::map_metadata_type("ref"), Some(GitMetadataType::Ref));
        assert_eq!(GitNativeValidator::map_metadata_type("note"), Some(GitMetadataType::Note));
        assert_eq!(GitNativeValidator::map_metadata_type("attr"), Some(GitMetadataType::Attr));
        assert_eq!(GitNativeValidator::map_metadata_type("index"), Some(GitMetadataType::Index));
        assert_eq!(GitNativeValidator::map_metadata_type("stash"), Some(GitMetadataType::Stash));
        assert_eq!(GitNativeValidator::map_metadata_type("worktree"), Some(GitMetadataType::Worktree));
        assert_eq!(GitNativeValidator::map_metadata_type("remote"), Some(GitMetadataType::Remote));
        assert_eq!(GitNativeValidator::map_metadata_type("branch"), Some(GitMetadataType::Branch));
        assert_eq!(GitNativeValidator::map_metadata_type("head"), Some(GitMetadataType::Head));
        assert_eq!(GitNativeValidator::map_metadata_type("reflog"), Some(GitMetadataType::Reflog));
        assert_eq!(GitNativeValidator::map_metadata_type("invalid"), None);
    }

    #[test]
    fn test_map_config_type() {
        assert_eq!(GitNativeValidator::map_config_type("config-user"), Some(GitConfigType::ConfigUser));
        assert_eq!(GitNativeValidator::map_config_type("config-core"), Some(GitConfigType::ConfigCore));
        assert_eq!(GitNativeValidator::map_config_type("config-branch"), Some(GitConfigType::ConfigBranch));
        assert_eq!(GitNativeValidator::map_config_type("config-remote"), Some(GitConfigType::ConfigRemote));
        assert_eq!(GitNativeValidator::map_config_type("config-commit"), Some(GitConfigType::ConfigCommit));
        assert_eq!(GitNativeValidator::map_config_type("config-push"), Some(GitConfigType::ConfigPush));
        assert_eq!(GitNativeValidator::map_config_type("config-worktree"), Some(GitConfigType::ConfigWorktree));
        assert_eq!(GitNativeValidator::map_config_type("invalid"), None);
    }

    #[test]
    fn test_validate_concerns() {
        let valid_concerns = vec!["blob".to_string(), "tree".to_string(), "ref".to_string()];
        assert!(GitNativeValidator::validate_concerns(&valid_concerns).is_ok());
        
        let invalid_concerns = vec!["blob".to_string(), "contract-violation".to_string()];
        assert!(GitNativeValidator::validate_concerns(&invalid_concerns).is_err());
    }
}
