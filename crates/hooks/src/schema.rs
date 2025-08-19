use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Enumeration of all standard Git hooks and their expected invocation context
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitHook {
    // Patch management hooks
    ApplyPatchMsg,  // arg[0]: commit msg file
    PreApplyPatch,  // no args
    PostApplyPatch, // no args

    // Commit lifecycle hooks
    PreCommit,        // no args
    PrepareCommitMsg, // args: commit msg file, source, [SHA]
    CommitMsg,        // arg[0]: commit msg file
    PostCommit,       // no args

    // Merge and rebase hooks
    PreMergeCommit, // no args
    PreRebase,      // args: upstream, [branch]
    PostRebase,     // args: command (amend|rebase)
    PostRewrite,    // arg[0]: command (amend|rebase)

    // Branch management hooks
    PostCheckout, // args: old HEAD, new HEAD, flag
    PostMerge,    // arg[0]: squash?

    // Push and receive hooks
    PrePush,        // args: remote, url — stdin: ref lines
    PreReceive,     // stdin: ref lines
    Update,         // args: ref name, old SHA, new SHA
    PostReceive,    // stdin: ref lines
    PostUpdate,     // args: ref names...
    PushToCheckout, // arg[0]: commit SHA

    // Specialized hooks
    SendEmailValidate,    // args: file, header
    FSMonitorWatchman,    // args: version, token — stdout: JSON
    ReferenceTransaction, // args: prepared|committed|aborted
    PostIndexChange,      // args: working_dir_updated, skip_worktree_updated
}

impl GitHook {
    /// Get the hook name as a string
    pub fn name(&self) -> &'static str {
        match self {
            GitHook::ApplyPatchMsg => "applypatch-msg",
            GitHook::PreApplyPatch => "pre-applypatch",
            GitHook::PostApplyPatch => "post-applypatch",
            GitHook::PreCommit => "pre-commit",
            GitHook::PrepareCommitMsg => "prepare-commit-msg",
            GitHook::CommitMsg => "commit-msg",
            GitHook::PostCommit => "post-commit",
            GitHook::PreMergeCommit => "pre-merge-commit",
            GitHook::PreRebase => "pre-rebase",
            GitHook::PostRebase => "post-rebase",
            GitHook::PostRewrite => "post-rewrite",
            GitHook::PostCheckout => "post-checkout",
            GitHook::PostMerge => "post-merge",
            GitHook::PrePush => "pre-push",
            GitHook::PreReceive => "pre-receive",
            GitHook::Update => "update",
            GitHook::PostReceive => "post-receive",
            GitHook::PostUpdate => "post-update",
            GitHook::PushToCheckout => "push-to-checkout",
            GitHook::SendEmailValidate => "sendemail-validate",
            GitHook::FSMonitorWatchman => "fsmonitor-watchman",
            GitHook::ReferenceTransaction => "reference-transaction",
            GitHook::PostIndexChange => "post-index-change",
        }
    }

    /// Get expected number of arguments (returns None for variable args)
    pub fn expected_args(&self) -> Option<usize> {
        match self {
            GitHook::ApplyPatchMsg => Some(1),
            GitHook::PreApplyPatch => Some(0),
            GitHook::PostApplyPatch => Some(0),
            GitHook::PreCommit => Some(0),
            GitHook::PrepareCommitMsg => None, // 1-3 args (variable)
            GitHook::CommitMsg => Some(1),
            GitHook::PostCommit => Some(0),
            GitHook::PreMergeCommit => Some(0),
            GitHook::PreRebase => None, // 1-2 args (variable)
            GitHook::PostRebase => Some(1),
            GitHook::PostRewrite => Some(1),
            GitHook::PostCheckout => Some(3),
            GitHook::PostMerge => Some(1),
            GitHook::PrePush => Some(2),
            GitHook::PreReceive => Some(0),
            GitHook::Update => Some(3),
            GitHook::PostReceive => Some(0),
            GitHook::PostUpdate => None, // variable args
            GitHook::PushToCheckout => Some(1),
            GitHook::SendEmailValidate => Some(2),
            GitHook::FSMonitorWatchman => Some(2),
            GitHook::ReferenceTransaction => Some(1),
            GitHook::PostIndexChange => Some(2),
        }
    }

    /// Get minimum expected arguments
    pub fn min_args(&self) -> usize {
        match self {
            GitHook::PrepareCommitMsg => 1,
            GitHook::PreRebase => 1,
            GitHook::PostUpdate => 0,
            _ => self.expected_args().unwrap_or(0),
        }
    }

    /// Get maximum expected arguments
    pub fn max_args(&self) -> Option<usize> {
        match self {
            GitHook::PrepareCommitMsg => Some(3),
            GitHook::PreRebase => Some(2),
            GitHook::PostUpdate => None, // unlimited
            _ => self.expected_args(),
        }
    }

    /// Check if hook expects stdin
    pub fn expects_stdin(&self) -> bool {
        matches!(
            self,
            GitHook::PrePush | GitHook::PreReceive | GitHook::PostReceive
        )
    }

    /// Check if hook produces stdout
    pub fn produces_stdout(&self) -> bool {
        matches!(self, GitHook::FSMonitorWatchman)
    }

    /// Get working directory context
    pub fn working_directory(&self) -> WorkingDirectory {
        match self {
            // Server-side hooks always use $GIT_DIR
            GitHook::PreReceive
            | GitHook::Update
            | GitHook::PostReceive
            | GitHook::PostUpdate
            | GitHook::PushToCheckout => WorkingDirectory::GitDir,

            // Client-side hooks use repository root
            _ => WorkingDirectory::RepositoryRoot,
        }
    }

    /// Get important environment variables for this hook
    pub fn important_env_vars(&self) -> Vec<&'static str> {
        match self {
            GitHook::PreCommit | GitHook::PrepareCommitMsg | GitHook::CommitMsg => {
                vec!["GIT_EDITOR"]
            }
            GitHook::PrePush => {
                vec![
                    "GIT_PUSH_OPTION_COUNT",
                    "GIT_PUSH_OPTION_0",
                    "GIT_PUSH_OPTION_1",
                ]
            }
            GitHook::SendEmailValidate => {
                vec!["GIT_SENDEMAIL_FILE_COUNTER", "GIT_SENDEMAIL_FILE_TOTAL"]
            }
            GitHook::FSMonitorWatchman => {
                vec!["GIT_WORK_TREE"]
            }
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkingDirectory {
    RepositoryRoot,
    GitDir,
}

/// Hook invocation context with validation
#[derive(Debug)]
pub struct HookContext {
    pub hook: GitHook,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub stdin_data: Option<String>,
}

impl HookContext {
    /// Create a new hook context from command line arguments
    pub fn from_args(args: Vec<String>) -> Result<Self, HookError> {
        if args.is_empty() {
            return Err(HookError::NoHookName);
        }

        let hook_name = &args[0];
        let hook_args = args[1..].to_vec();

        let hook = GitHook::from_name(hook_name)?;

        // Validate argument count
        let min_args = hook.min_args();
        let max_args = hook.max_args();
        let actual = hook_args.len();

        if actual < min_args {
            return Err(HookError::InvalidArgCount {
                hook: hook_name.clone(),
                expected: min_args,
                actual,
            });
        }

        if let Some(max) = max_args {
            if actual > max {
                return Err(HookError::InvalidArgCount {
                    hook: hook_name.clone(),
                    expected: max,
                    actual,
                });
            }
        }

        // Get environment variables
        let env = std::env::vars().collect();

        Ok(HookContext {
            hook,
            args: hook_args,
            env,
            stdin_data: None,
        })
    }

    /// Validate the hook context
    pub fn validate(&self) -> Result<(), HookError> {
        // Check required environment variables
        for env_var in self.hook.important_env_vars() {
            if !self.env.contains_key(env_var) {
                return Err(HookError::MissingEnvVar {
                    hook: self.hook.name().to_string(),
                    env_var: env_var.to_string(),
                });
            }
        }

        // Validate working directory
        let current_dir =
            std::env::current_dir().map_err(|e| HookError::WorkingDirectoryError(e.to_string()))?;

        match self.hook.working_directory() {
            WorkingDirectory::RepositoryRoot => {
                // Should be in repository root
                if !current_dir.join(".git").exists() {
                    return Err(HookError::InvalidWorkingDirectory {
                        hook: self.hook.name().to_string(),
                        expected: "repository root".to_string(),
                        actual: current_dir.to_string_lossy().to_string(),
                    });
                }
            }
            WorkingDirectory::GitDir => {
                // Should be in .git directory
                if !current_dir.join("HEAD").exists() {
                    return Err(HookError::InvalidWorkingDirectory {
                        hook: self.hook.name().to_string(),
                        expected: ".git directory".to_string(),
                        actual: current_dir.to_string_lossy().to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Get a specific argument by index
    pub fn arg(&self, index: usize) -> Option<&str> {
        self.args.get(index).map(|s| s.as_str())
    }

    /// Get environment variable
    pub fn env_var(&self, name: &str) -> Option<&str> {
        self.env.get(name).map(|s| s.as_str())
    }
}

impl GitHook {
    /// Create a hook from its name
    pub fn from_name(name: &str) -> Result<Self, HookError> {
        match name {
            "applypatch-msg" => Ok(GitHook::ApplyPatchMsg),
            "pre-applypatch" => Ok(GitHook::PreApplyPatch),
            "post-applypatch" => Ok(GitHook::PostApplyPatch),
            "pre-commit" => Ok(GitHook::PreCommit),
            "prepare-commit-msg" => Ok(GitHook::PrepareCommitMsg),
            "commit-msg" => Ok(GitHook::CommitMsg),
            "post-commit" => Ok(GitHook::PostCommit),
            "pre-merge-commit" => Ok(GitHook::PreMergeCommit),
            "pre-rebase" => Ok(GitHook::PreRebase),
            "post-rebase" => Ok(GitHook::PostRebase),
            "post-rewrite" => Ok(GitHook::PostRewrite),
            "post-checkout" => Ok(GitHook::PostCheckout),
            "post-merge" => Ok(GitHook::PostMerge),
            "pre-push" => Ok(GitHook::PrePush),
            "pre-receive" => Ok(GitHook::PreReceive),
            "update" => Ok(GitHook::Update),
            "post-receive" => Ok(GitHook::PostReceive),
            "post-update" => Ok(GitHook::PostUpdate),
            "push-to-checkout" => Ok(GitHook::PushToCheckout),
            "sendemail-validate" => Ok(GitHook::SendEmailValidate),
            "fsmonitor-watchman" => Ok(GitHook::FSMonitorWatchman),
            "reference-transaction" => Ok(GitHook::ReferenceTransaction),
            "post-index-change" => Ok(GitHook::PostIndexChange),
            _ => Err(HookError::UnknownHook(name.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("No hook name provided")]
    NoHookName,

    #[error("Unknown hook: {0}")]
    UnknownHook(String),

    #[error("Invalid argument count for {hook}: expected {expected}, got {actual}")]
    InvalidArgCount {
        hook: String,
        expected: usize,
        actual: usize,
    },

    #[error("Missing environment variable for {hook}: {env_var}")]
    MissingEnvVar { hook: String, env_var: String },

    #[error("Invalid working directory for {hook}: expected {expected}, got {actual}")]
    InvalidWorkingDirectory {
        hook: String,
        expected: String,
        actual: String,
    },

    #[error("Working directory error: {0}")]
    WorkingDirectoryError(String),
}

/// Hook manifest for configuration
/// Hook metadata for testing and validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    pub name: String,
    pub phase: HookPhase,
    pub scope: HookScope,
    pub objects: Vec<GitObject>,
    pub validation_capabilities: Vec<ValidationCapability>,
}

/// Hook execution phase
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookPhase {
    PreValidation,
    PostAction,
}

/// Hook execution scope
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HookScope {
    Client,
    Server,
}

/// Git objects that hooks can operate on
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitObject {
    Commit,
    Tree,
    Blob,
    Tag,
    Index,
    Ref,
    Remote,
    Head,
}

/// Hook validation capabilities
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationCapability {
    BasicValidation,
    RefValidation,
    ContentValidation,
    PerformanceValidation,
}

impl HookContext {
    /// Get metadata for this hook context
    pub fn metadata(&self) -> HookMetadata {
        let phase = match self.hook {
            GitHook::PreCommit | GitHook::PrePush | GitHook::PreReceive | GitHook::Update => {
                HookPhase::PreValidation
            }
            _ => HookPhase::PostAction,
        };

        let scope = match self.hook {
            GitHook::PreReceive
            | GitHook::Update
            | GitHook::PostReceive
            | GitHook::PostUpdate
            | GitHook::PushToCheckout => HookScope::Server,
            _ => HookScope::Client,
        };

        let objects = match self.hook {
            GitHook::PreCommit | GitHook::CommitMsg | GitHook::PostCommit => {
                vec![GitObject::Commit, GitObject::Index]
            }
            GitHook::PrePush | GitHook::PostRewrite => vec![GitObject::Ref, GitObject::Remote],
            GitHook::PostCheckout | GitHook::PostMerge => vec![GitObject::Head, GitObject::Tree],
            _ => vec![GitObject::Commit, GitObject::Tree],
        };

        let validation_capabilities = match self.hook {
            GitHook::PreCommit | GitHook::CommitMsg | GitHook::PostCommit => {
                vec![ValidationCapability::BasicValidation]
            }
            GitHook::PrePush | GitHook::PreReceive => vec![ValidationCapability::RefValidation],
            _ => vec![ValidationCapability::BasicValidation],
        };

        HookMetadata {
            name: self.hook.name().to_string(),
            phase,
            scope,
            objects,
            validation_capabilities,
        }
    }
}

/// Hook manifest for configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct HookManifest {
    pub hooks: Vec<HookDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HookDefinition {
    pub name: String,
    pub args: Vec<String>,
    pub stdin: bool,
    pub stdin_format: Option<String>,
    pub stdout_format: Option<String>,
    pub env: Vec<String>,
    pub description: String,
}

impl Default for HookManifest {
    fn default() -> Self {
        Self {
            hooks: vec![
                HookDefinition {
                    name: "pre-commit".to_string(),
                    args: vec![],
                    stdin: false,
                    stdin_format: None,
                    stdout_format: None,
                    env: vec![],
                    description: "Run before commit creation".to_string(),
                },
                HookDefinition {
                    name: "commit-msg".to_string(),
                    args: vec!["path_to_commit_msg_file".to_string()],
                    stdin: false,
                    stdin_format: None,
                    stdout_format: None,
                    env: vec![],
                    description: "Validate commit message".to_string(),
                },
                HookDefinition {
                    name: "pre-push".to_string(),
                    args: vec!["remote".to_string(), "remote_url".to_string()],
                    stdin: true,
                    stdin_format: Some("ref lines: <src> <src_sha> <dst> <dst_sha>".to_string()),
                    stdout_format: None,
                    env: vec!["GIT_PUSH_OPTION_COUNT".to_string()],
                    description: "Validate before push".to_string(),
                },
                HookDefinition {
                    name: "pre-receive".to_string(),
                    args: vec![],
                    stdin: true,
                    stdin_format: Some("ref lines: <src> <src_sha> <dst> <dst_sha>".to_string()),
                    stdout_format: None,
                    env: vec![],
                    description: "Server-side pre-receive validation".to_string(),
                },
                HookDefinition {
                    name: "update".to_string(),
                    args: vec![
                        "ref_name".to_string(),
                        "old_sha".to_string(),
                        "new_sha".to_string(),
                    ],
                    stdin: false,
                    stdin_format: None,
                    stdout_format: None,
                    env: vec![],
                    description: "Server-side ref update validation".to_string(),
                },
                HookDefinition {
                    name: "fsmonitor-watchman".to_string(),
                    args: vec!["version".to_string(), "token".to_string()],
                    stdin: false,
                    stdin_format: None,
                    stdout_format: Some("json".to_string()),
                    env: vec!["GIT_WORK_TREE".to_string()],
                    description: "File system monitoring".to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_names() {
        assert_eq!(GitHook::PreCommit.name(), "pre-commit");
        assert_eq!(GitHook::CommitMsg.name(), "commit-msg");
        assert_eq!(GitHook::PrePush.name(), "pre-push");
    }

    #[test]
    fn test_hook_from_name() {
        assert!(matches!(
            GitHook::from_name("pre-commit"),
            Ok(GitHook::PreCommit)
        ));
        assert!(matches!(
            GitHook::from_name("commit-msg"),
            Ok(GitHook::CommitMsg)
        ));
        assert!(matches!(
            GitHook::from_name("unknown"),
            Err(HookError::UnknownHook(_))
        ));
    }

    #[test]
    fn test_expected_args() {
        assert_eq!(GitHook::PreCommit.expected_args(), Some(0));
        assert_eq!(GitHook::CommitMsg.expected_args(), Some(1));
        assert_eq!(GitHook::PrePush.expected_args(), Some(2));
        assert_eq!(GitHook::PrepareCommitMsg.expected_args(), None); // variable args
    }

    #[test]
    fn test_expects_stdin() {
        assert!(!GitHook::PreCommit.expects_stdin());
        assert!(GitHook::PrePush.expects_stdin());
        assert!(GitHook::PreReceive.expects_stdin());
    }

    #[test]
    fn test_working_directory() {
        assert_eq!(
            GitHook::PreCommit.working_directory(),
            WorkingDirectory::RepositoryRoot
        );
        assert_eq!(
            GitHook::PreReceive.working_directory(),
            WorkingDirectory::GitDir
        );
    }
}
