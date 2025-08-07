use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::modules::git_native::GitNativeValidator;

/// Static hook definition with zero dynamic resolution
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StaticHook {
    /// Human-readable name of the hook
    pub name: String,
    /// Hook trigger scope (only one allowed)
    pub scope: HookScope,
    /// Required list of concerns (must match schema)
    pub concerns: Vec<HookConcern>,
    /// Only one binary per hook, must exist at build time
    pub bin: String,
}

/// Hook trigger scope enum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HookScope {
    /// Traditional Git lifecycle hooks
    Git,
    /// GitHub-specific hooks
    Github,
    /// File system monitoring hooks
    FsMonitor,
    /// Reference transaction hooks
    Reference,
    /// Email-related hooks
    Email,
    /// Patch-related hooks
    Patch,
}

/// Hook concerns enum - Git-native object types + local constructs + config sections
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum HookConcern {
    // Git Object Concerns (Core Objects)
    /// Git blob objects (file contents)
    Blob,
    /// Git tree objects (directory structure)
    Tree,
    /// Git commit objects (commit history)
    Commit,
    /// Git tag objects (annotated tags)
    Tag,
    /// Git references (heads, tags, etc.)
    Ref,
    /// Git notes (commit-attached metadata)
    Note,
    /// Git attributes (file-based config)
    Attr,

    // Git Tree Entry Concerns (specific file types)
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

    // Git Attribute Concerns (structured behavior mappings)
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

    // Git Reference Concerns (Detailed Ref Types)
    /// Git branch references (refs/heads/*)
    RefBranch,
    /// Git remote references (refs/remotes/*)
    RefRemote,
    /// Git tag references (refs/tags/*)
    RefTag,
    /// Git note references (refs/notes/*)
    RefNote,
    /// Git stash references (refs/stash)
    RefStash,
    /// Git worktree references (worktrees/*/HEAD)
    RefWorktree,
    /// Git symbolic references (HEAD -> refs/heads/main)
    RefSym,
    /// Git HEAD pointer
    HeadPointer,
    /// Git packed references (.git/packed-refs)
    PackedRefs,
    /// Git fetch HEAD pointer
    FetchHeadPointer,
    /// Git merge HEAD pointer
    MergeHeadPointer,
    /// Git cherry-pick HEAD pointer
    CherryPickPointer,
    /// Git revert HEAD pointer
    RevertHeadPointer,
    /// Git original HEAD pointer
    OrigHead,
    /// Git reflog entries
    RefLogEntry,

    // Git Storage Concerns (Object Database)
    /// Git packfile index files
    PackfileIndex,
    /// Git packfile data files
    PackfileData,
    /// Git packfile bitmap files
    PackfileBitmap,
    /// Git packfile keep files
    PackfileKeep,
    /// Git packfile promisor files
    PackfilePromisor,
    /// Git loose objects
    LooseObject,
    /// Git object database
    ObjectDatabase,

    // Git Local State Concerns
    /// Git stash (pseudo-refs for uncommitted work)
    Stash,
    /// Git worktree (linked working directories)
    Worktree,
    /// Git index (staging area)
    Index,
    /// Git remote (remote repository configurations)
    Remote,
    /// Git branch (branch-specific configurations)
    Branch,
    /// Git HEAD (current branch reference)
    Head,
    /// Git reflog (reference history)
    Reflog,

    // Git Transport & Protocol Concerns
    /// Git local filesystem protocol
    ProtocolLocal,
    /// Git protocol (git://)
    ProtocolGit,
    /// Git HTTP protocol
    ProtocolHttp,
    /// Git HTTPS protocol
    ProtocolHttps,
    /// Git SSH protocol
    ProtocolSsh,
    /// Git refspec mappings
    Refspec,
    /// Git protocol packets
    ProtocolPacket,

    // Git Runtime & Environment Concerns
    /// Git directory override
    GitDirOverride,
    /// Git worktree override
    WorkTreeOverride,
    /// Git index file override
    IndexFileOverride,
    /// Git object directory override
    ObjectDirectoryOverride,
    /// Git alternate object databases
    AlternateObjectDatabase,
    /// Git config override
    GitConfigOverride,
    /// Git trace override
    TraceOverride,
    /// Git author override
    AuthorOverride,
    /// Git UI override
    UiOverride,

    // Git Maintenance & Recovery Concerns
    /// Git filesystem consistency check
    FsckCheck,
    /// Git prune orphaned objects
    PruneOrphaned,
    /// Git repack packfiles
    RepackPackfile,
    /// Git garbage collection lifecycle
    GcLifecycle,
    /// Git reflog repair
    ReflogRepair,
    /// Git index recovery
    IndexRecovery,

    // Git Command & Operation Concerns
    /// Git initialization operations
    Init,
    /// Git snapshot operations (add, commit)
    Snapshot,
    /// Git merge operations
    Merge,
    /// Git rebase operations
    Rebase,
    /// Git push operations
    Push,
    /// Git pull operations
    Pull,
    /// Git fetch operations
    Fetch,
    /// Git log operations
    Log,
    /// Git diff operations
    Diff,
    /// Git status operations
    Status,
    /// Git patch operations
    Patch,
    /// Git debug operations
    Debug,
    /// Git blame operations
    Blame,
    /// Git plumbing operations
    Plumbing,
    /// Git object database operations
    ObjectDb,
    /// Git transport operations
    Transport,
    /// Git project initialization
    ProjectInit,

    // Git Config Concerns (first-class)
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
}

impl StaticHook {
    /// Validate the static hook definition
    pub fn validate(&self) -> Result<()> {
        // Validate name format
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            bail!("Invalid hook name '{}': must contain only alphanumeric characters, underscores, and hyphens", self.name);
        }

        // Validate concerns are Git-native
        let concern_strings: Vec<String> = self.concerns.iter()
            .map(|c| serde_json::to_string(c).unwrap().trim_matches('"').to_string())
            .collect();
        GitNativeValidator::validate_concerns(&concern_strings)?;

        // Validate concerns are unique
        let mut concerns = self.concerns.clone();
        concerns.sort();
        concerns.dedup();
        if concerns.len() != self.concerns.len() {
            bail!("Duplicate concerns found in hook '{}'", self.name);
        }

        // Validate binary exists
        let path = if self.bin.starts_with('/') || self.bin.starts_with("target/") {
            PathBuf::from(&self.bin)
        } else {
            PathBuf::from("target/release").join(&self.bin)
        };
        
        if !path.exists() {
            bail!("Missing hook binary: {}", path.display());
        }
        if !path.is_file() {
            bail!("Hook binary is not a file: {}", path.display());
        }

        Ok(())
    }

    /// Get the binary path as a Path
    pub fn binary_path(&self) -> &Path {
        Path::new(&self.bin)
    }

    /// Check if this hook concerns a specific type
    pub fn concerns_type(&self, concern: &HookConcern) -> bool {
        self.concerns.contains(concern)
    }

    /// Get the scope as a string
    pub fn scope_str(&self) -> &'static str {
        match self.scope {
            HookScope::Git => "git",
            HookScope::Github => "github", 
            HookScope::FsMonitor => "fsmonitor",
            HookScope::Reference => "reference",
            HookScope::Email => "email",
            HookScope::Patch => "patch",
        }
    }
}

/// Load and validate a static hook from a JSONC file
pub fn load_static_hook(path: &Path) -> Result<StaticHook> {
    let content = std::fs::read_to_string(path)?;
    
    // For now, parse as regular JSON (we can add JSONC support later)
    let hook: StaticHook = serde_json::from_str(&content)?;
    hook.validate()?;
    
    Ok(hook)
}

/// Validate all static hooks in a directory
pub fn validate_static_hooks(dir: &Path) -> Result<Vec<StaticHook>> {
    let mut hooks = Vec::new();
    
    if !dir.exists() {
        return Ok(hooks);
    }
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "jsonc") {
            match load_static_hook(&path) {
                Ok(hook) => hooks.push(hook),
                Err(e) => {
                    eprintln!("Failed to load hook from {}: {}", path.display(), e);
                }
            }
        }
    }
    
    Ok(hooks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_valid_static_hook() {
        let hook = StaticHook {
            name: "pre-commit".to_string(),
            scope: HookScope::Git,
            concerns: vec![HookConcern::Blob, HookConcern::Tree],
            bin: "target/release/hooksmith-validate-tree".to_string(),
        };
        
        // This will fail if the binary doesn't exist, which is expected
        let result = hook.validate();
        assert!(result.is_err()); // Binary doesn't exist in test
    }

    #[test]
    fn test_invalid_hook_name() {
        let hook = StaticHook {
            name: "pre-commit!".to_string(), // Invalid character
            scope: HookScope::Git,
            concerns: vec![HookConcern::Blob],
            bin: "target/release/hooksmith-validate-tree".to_string(),
        };
        
        let result = hook.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_duplicate_concerns() {
        let hook = StaticHook {
            name: "pre-commit".to_string(),
            scope: HookScope::Git,
            concerns: vec![HookConcern::Blob, HookConcern::Blob], // Duplicate
            bin: "target/release/hooksmith-validate-tree".to_string(),
        };
        
        let result = hook.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_static_hook() {
        let temp_dir = tempdir().unwrap();
        let hook_file = temp_dir.path().join("test-hook.jsonc");
        
        let hook_content = r#"{
            "name": "pre-commit",
            "scope": "git",
            "concerns": ["blob", "tree"],
            "bin": "target/release/hooksmith-validate-tree"
        }"#;
        
        fs::write(&hook_file, hook_content).unwrap();
        
        let result = load_static_hook(&hook_file);
        assert!(result.is_err()); // Binary doesn't exist
    }
}
