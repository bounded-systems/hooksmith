//! Git file states, actions, and hooks model
//!
//! This module provides a comprehensive model that ties together:
//! - File states in Git (clean, modified, staged, etc.)
//! - Git actions (commit, push, merge, etc.)
//! - Git hooks (pre-commit, post-commit, etc.)
//! - Blocking behavior (which hooks can block which actions)
//!
//! This allows contracts to query whether actions are allowed for specific file states
//! and whether they can be blocked by hooks.

use std::collections::HashMap;

/// Represents the different states a file can be in within Git
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileStateKind {
    /// Same in HEAD, index, and worktree
    Clean,
    /// Different in worktree but not staged
    ModifiedUnstaged,
    /// Added/changed in index but not committed yet
    Staged,
    /// Staged changes exist, but the file has further modifications in worktree
    StagedAndModified,
    /// Exists in index, not in HEAD
    Added,
    /// Deleted from index, still in HEAD
    DeletedStaged,
    /// Exists in worktree, not in index or HEAD
    Untracked,
    /// Matches .gitignore
    Ignored,
}

impl FileStateKind {
    /// Get a human-readable description of the file state
    pub fn description(&self) -> &'static str {
        match self {
            FileStateKind::Clean => "Same in HEAD, index, and worktree",
            FileStateKind::ModifiedUnstaged => "Different in worktree but not staged",
            FileStateKind::Staged => "Added/changed in index but not committed yet",
            FileStateKind::StagedAndModified => "Staged changes exist, but the file has further modifications in worktree",
            FileStateKind::Added => "Exists in index, not in HEAD",
            FileStateKind::DeletedStaged => "Deleted from index, still in HEAD",
            FileStateKind::Untracked => "Exists in worktree, not in index or HEAD",
            FileStateKind::Ignored => "Matches .gitignore",
        }
    }
}

/// Represents Git actions that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionKind {
    /// Commit staged changes to history
    Commit,
    /// Combine branches
    Merge,
    /// Rewrite commit history
    Rebase,
    /// Upload commits to remote
    Push,
    /// Server-side: receive push from client
    ReceivePush,
    /// Switch branches or commits
    Checkout,
    /// Apply patches
    ApplyPatch,
    /// Garbage collection
    GarbageCollect,
    /// Send email patches
    EmailSend,
}

impl ActionKind {
    /// Get a human-readable description of the action
    pub fn description(&self) -> &'static str {
        match self {
            ActionKind::Commit => "Commit staged changes to history",
            ActionKind::Merge => "Combine branches",
            ActionKind::Rebase => "Rewrite commit history",
            ActionKind::Push => "Upload commits to remote",
            ActionKind::ReceivePush => "Server-side: receive push from client",
            ActionKind::Checkout => "Switch branches or commits",
            ActionKind::ApplyPatch => "Apply patches",
            ActionKind::GarbageCollect => "Garbage collection",
            ActionKind::EmailSend => "Send email patches",
        }
    }
}

/// Represents Git hooks that can run during various actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookKind {
    // Client-side hooks
    /// Runs before commit is created
    PreCommit,
    /// Runs to prepare commit message
    PrepareCommitMsg,
    /// Runs to validate commit message
    CommitMsg,
    /// Runs after commit is created
    PostCommit,
    /// Runs before merge commit is created
    PreMergeCommit,
    /// Runs after merge is completed
    PostMerge,
    /// Runs before rebase starts
    PreRebase,
    /// Runs after history rewriting
    PostRewrite,
    /// Runs before push to remote
    PrePush,
    /// Runs on server before receiving push
    PreReceive,
    /// Runs on server for each ref update
    Update,
    /// Runs on server after receiving push
    PostReceive,
    /// Runs after checkout completes
    PostCheckout,
    /// Runs to validate patch commit message
    ApplyPatchMsg,
    /// Runs before patch is applied
    PreApplyPatch,
    /// Runs after patch is applied
    PostApplyPatch,
    /// Runs before automatic garbage collection
    PreAutoGc,
    /// Runs to validate email before sending
    SendEmailValidate,
}

impl HookKind {
    /// Get a human-readable description of the hook
    pub fn description(&self) -> &'static str {
        match self {
            HookKind::PreCommit => "Runs before commit is created",
            HookKind::PrepareCommitMsg => "Runs to prepare commit message",
            HookKind::CommitMsg => "Runs to validate commit message",
            HookKind::PostCommit => "Runs after commit is created",
            HookKind::PreMergeCommit => "Runs before merge commit is created",
            HookKind::PostMerge => "Runs after merge is completed",
            HookKind::PreRebase => "Runs before rebase starts",
            HookKind::PostRewrite => "Runs after history rewriting",
            HookKind::PrePush => "Runs before push to remote",
            HookKind::PreReceive => "Runs on server before receiving push",
            HookKind::Update => "Runs on server for each ref update",
            HookKind::PostReceive => "Runs on server after receiving push",
            HookKind::PostCheckout => "Runs after checkout completes",
            HookKind::ApplyPatchMsg => "Runs to validate patch commit message",
            HookKind::PreApplyPatch => "Runs before patch is applied",
            HookKind::PostApplyPatch => "Runs after patch is applied",
            HookKind::PreAutoGc => "Runs before automatic garbage collection",
            HookKind::SendEmailValidate => "Runs to validate email before sending",
        }
    }

    /// Get the hook name as it appears in .git/hooks/
    pub fn filename(&self) -> &'static str {
        match self {
            HookKind::PreCommit => "pre-commit",
            HookKind::PrepareCommitMsg => "prepare-commit-msg",
            HookKind::CommitMsg => "commit-msg",
            HookKind::PostCommit => "post-commit",
            HookKind::PrePush => "pre-push",
            HookKind::PreMergeCommit => "pre-merge-commit",
            HookKind::PostMerge => "post-merge",
            HookKind::PreRebase => "pre-rebase",
            HookKind::PostRewrite => "post-rewrite",
            HookKind::PostCheckout => "post-checkout",
            HookKind::ApplyPatchMsg => "applypatch-msg",
            HookKind::PreApplyPatch => "pre-applypatch",
            HookKind::PostApplyPatch => "post-applypatch",
            HookKind::PreAutoGc => "pre-auto-gc",
            HookKind::SendEmailValidate => "sendemail-validate",
            HookKind::PreReceive => "pre-receive",
            HookKind::Update => "update",
            HookKind::PostReceive => "post-receive",
        }
    }
}

/// Information about a hook including whether it can block the action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HookInfo {
    /// The hook that runs
    pub hook: HookKind,
    /// Whether this hook can block the action
    pub can_block: bool,
}

impl HookInfo {
    /// Create a new hook info
    pub fn new(hook: HookKind, can_block: bool) -> Self {
        Self { hook, can_block }
    }
}

/// Information about a file including its state and available actions
#[derive(Debug, Clone)]
pub struct FileActionInfo {
    /// The file path
    pub path: String,
    /// The current state of the file
    pub state: FileStateKind,
    /// Available actions and their associated hooks
    pub actions: Vec<(ActionKind, Vec<HookInfo>)>,
}

impl FileActionInfo {
    /// Create a new file action info
    pub fn new(path: String, state: FileStateKind) -> Self {
        let actions = actions_for_file(state);
        Self { path, state, actions }
    }

    /// Get all blocking hooks for this file
    pub fn blocking_hooks(&self) -> Vec<HookKind> {
        self.actions
            .iter()
            .flat_map(|(_, hooks)| {
                hooks.iter()
                    .filter(|h| h.can_block)
                    .map(|h| h.hook)
            })
            .collect()
    }

    /// Get all non-blocking hooks for this file
    pub fn non_blocking_hooks(&self) -> Vec<HookKind> {
        self.actions
            .iter()
            .flat_map(|(_, hooks)| {
                hooks.iter()
                    .filter(|h| !h.can_block)
                    .map(|h| h.hook)
            })
            .collect()
    }

    /// Check if any action can be blocked
    pub fn can_be_blocked(&self) -> bool {
        !self.blocking_hooks().is_empty()
    }

    /// Get actions that can be blocked
    pub fn blockable_actions(&self) -> Vec<ActionKind> {
        self.actions
            .iter()
            .filter(|(_, hooks)| hooks.iter().any(|h| h.can_block))
            .map(|(action, _)| *action)
            .collect()
    }
}

/// Get the hooks that run for a given action
pub fn hooks_for_action(action: ActionKind) -> Vec<HookInfo> {
    use ActionKind::*;
    use HookKind::*;

    match action {
        Commit => vec![
            HookInfo::new(PreCommit, true),
            HookInfo::new(PrepareCommitMsg, false),
            HookInfo::new(CommitMsg, true),
            HookInfo::new(PostCommit, false),
        ],
        Merge => vec![
            HookInfo::new(PreMergeCommit, true),
            HookInfo::new(CommitMsg, true),
            HookInfo::new(PostMerge, false),
        ],
        Rebase => vec![
            HookInfo::new(PreRebase, true),
            HookInfo::new(PostRewrite, false),
        ],
        Push => vec![
            HookInfo::new(PrePush, true),
        ],
        ReceivePush => vec![
            HookInfo::new(PreReceive, true),
            HookInfo::new(Update, true),
            HookInfo::new(PostReceive, false),
        ],
        Checkout => vec![
            HookInfo::new(PostCheckout, false),
        ],
        ApplyPatch => vec![
            HookInfo::new(ApplyPatchMsg, true),
            HookInfo::new(PreApplyPatch, true),
            HookInfo::new(PostApplyPatch, false),
        ],
        GarbageCollect => vec![
            HookInfo::new(PreAutoGc, true),
        ],
        EmailSend => vec![
            HookInfo::new(SendEmailValidate, true),
        ],
    }
}

/// Get the actions allowed for a given file state
pub fn allowed_actions(state: FileStateKind) -> Vec<ActionKind> {
    use ActionKind::*;
    use FileStateKind::*;

    match state {
        Clean => vec![Commit, Checkout, Push, Merge, Rebase],
        ModifiedUnstaged => vec![], // must be staged first
        Staged => vec![Commit, Push],
        StagedAndModified => vec![Commit, Push],
        Added => vec![Commit, Push],
        DeletedStaged => vec![Commit, Push],
        Untracked => vec![], // must be added first
        Ignored => vec![],   // ignored files are not acted on
    }
}

/// Check if a specific hook can block a specific action
pub fn can_block(action: ActionKind, hook: HookKind) -> bool {
    hooks_for_action(action)
        .iter()
        .any(|h| h.hook == hook && h.can_block)
}

/// Get all actions and their hooks for a given file state
pub fn actions_for_file(state: FileStateKind) -> Vec<(ActionKind, Vec<HookInfo>)> {
    allowed_actions(state)
        .into_iter()
        .map(|action| (action, hooks_for_action(action)))
        .collect()
}

/// Analyze multiple files and their states
pub fn analyze_files(files: Vec<(String, FileStateKind)>) -> Vec<FileActionInfo> {
    files
        .into_iter()
        .map(|(path, state)| FileActionInfo::new(path, state))
        .collect()
}

/// Contract validation function
pub fn validate_contract(
    state: FileStateKind,
    action: ActionKind,
    hook: HookKind,
) -> ContractValidation {
    // Check if action is allowed for this state
    let allowed_actions = allowed_actions(state);
    if !allowed_actions.contains(&action) {
        return ContractValidation::ActionNotAllowed;
    }

    // Check if this hook runs for this action
    let hooks = hooks_for_action(action);
    let hook_info = hooks.iter().find(|h| h.hook == hook);
    
    match hook_info {
        None => ContractValidation::HookNotRelevant,
        Some(info) => {
            if info.can_block {
                ContractValidation::Valid
            } else {
                ContractValidation::HookCannotBlock
            }
        }
    }
}

/// Result of contract validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractValidation {
    /// Contract is valid - hook can block this action for this state
    Valid,
    /// Action is not allowed for this file state
    ActionNotAllowed,
    /// Hook does not run for this action
    HookNotRelevant,
    /// Hook runs but cannot block the action
    HookCannotBlock,
}

impl ContractValidation {
    /// Check if the contract is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, ContractValidation::Valid)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ContractValidation::Valid => "Valid contract: Hook can block this action for this file state",
            ContractValidation::ActionNotAllowed => "Action not allowed for this file state",
            ContractValidation::HookNotRelevant => "Hook does not run for this action",
            ContractValidation::HookCannotBlock => "Hook runs but cannot block the action",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_state_descriptions() {
        assert_eq!(FileStateKind::Clean.description(), "Same in HEAD, index, and worktree");
        assert_eq!(FileStateKind::ModifiedUnstaged.description(), "Different in worktree but not staged");
    }

    #[test]
    fn test_action_descriptions() {
        assert_eq!(ActionKind::Commit.description(), "Commit staged changes to history");
        assert_eq!(ActionKind::Push.description(), "Upload commits to remote");
    }

    #[test]
    fn test_hook_descriptions() {
        assert_eq!(HookKind::PreCommit.description(), "Runs before commit is created");
        assert_eq!(HookKind::PostCommit.description(), "Runs after commit is created");
    }

    #[test]
    fn test_hook_filenames() {
        assert_eq!(HookKind::PreCommit.filename(), "pre-commit");
        assert_eq!(HookKind::CommitMsg.filename(), "commit-msg");
        assert_eq!(HookKind::PrePush.filename(), "pre-push");
    }

    #[test]
    fn test_hooks_for_action() {
        let commit_hooks = hooks_for_action(ActionKind::Commit);
        assert_eq!(commit_hooks.len(), 4);
        
        let pre_commit = commit_hooks.iter().find(|h| h.hook == HookKind::PreCommit).unwrap();
        assert!(pre_commit.can_block);
        
        let post_commit = commit_hooks.iter().find(|h| h.hook == HookKind::PostCommit).unwrap();
        assert!(!post_commit.can_block);
    }

    #[test]
    fn test_allowed_actions() {
        let staged_actions = allowed_actions(FileStateKind::Staged);
        assert!(staged_actions.contains(&ActionKind::Commit));
        assert!(staged_actions.contains(&ActionKind::Push));
        assert!(!staged_actions.contains(&ActionKind::Checkout));

        let untracked_actions = allowed_actions(FileStateKind::Untracked);
        assert!(untracked_actions.is_empty());
    }

    #[test]
    fn test_can_block() {
        assert!(can_block(ActionKind::Commit, HookKind::PreCommit));
        assert!(can_block(ActionKind::Commit, HookKind::CommitMsg));
        assert!(!can_block(ActionKind::Commit, HookKind::PostCommit));
        assert!(!can_block(ActionKind::Checkout, HookKind::PostCheckout));
    }

    #[test]
    fn test_actions_for_file() {
        let staged_actions = actions_for_file(FileStateKind::Staged);
        assert_eq!(staged_actions.len(), 2); // Commit and Push
        
        let (commit_action, commit_hooks) = &staged_actions[0];
        assert_eq!(*commit_action, ActionKind::Commit);
        assert_eq!(commit_hooks.len(), 4); // PreCommit, PrepareCommitMsg, CommitMsg, PostCommit
    }

    #[test]
    fn test_file_action_info() {
        let file_info = FileActionInfo::new("src/main.rs".to_string(), FileStateKind::Staged);
        
        assert_eq!(file_info.path, "src/main.rs");
        assert_eq!(file_info.state, FileStateKind::Staged);
        assert_eq!(file_info.actions.len(), 2); // Commit and Push
        
        let blocking_hooks = file_info.blocking_hooks();
        assert!(blocking_hooks.contains(&HookKind::PreCommit));
        assert!(blocking_hooks.contains(&HookKind::CommitMsg));
        assert!(!blocking_hooks.contains(&HookKind::PostCommit));
        
        assert!(file_info.can_be_blocked());
    }

    #[test]
    fn test_analyze_files() {
        let files = vec![
            ("src/main.rs".to_string(), FileStateKind::Staged),
            ("src/lib.rs".to_string(), FileStateKind::ModifiedUnstaged),
            ("docs/README.md".to_string(), FileStateKind::Untracked),
        ];
        
        let analysis = analyze_files(files);
        assert_eq!(analysis.len(), 3);
        
        assert_eq!(analysis[0].actions.len(), 2); // Staged file has actions
        assert_eq!(analysis[1].actions.len(), 0); // ModifiedUnstaged has no actions
        assert_eq!(analysis[2].actions.len(), 0); // Untracked has no actions
    }

    #[test]
    fn test_validate_contract() {
        // Valid contract
        let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PreCommit);
        assert!(result.is_valid());
        
        // Action not allowed
        let result = validate_contract(FileStateKind::Untracked, ActionKind::Commit, HookKind::PreCommit);
        assert!(!result.is_valid());
        assert_eq!(result, ContractValidation::ActionNotAllowed);
        
        // Hook not relevant
        let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PrePush);
        assert!(!result.is_valid());
        assert_eq!(result, ContractValidation::HookNotRelevant);
        
        // Hook cannot block
        let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PostCommit);
        assert!(!result.is_valid());
        assert_eq!(result, ContractValidation::HookCannotBlock);
    }

    #[test]
    fn test_contract_validation_descriptions() {
        assert_eq!(ContractValidation::Valid.description(), "Valid contract: Hook can block this action for this file state");
        assert_eq!(ContractValidation::ActionNotAllowed.description(), "Action not allowed for this file state");
        assert_eq!(ContractValidation::HookNotRelevant.description(), "Hook does not run for this action");
        assert_eq!(ContractValidation::HookCannotBlock.description(), "Hook runs but cannot block the action");
    }
} 
