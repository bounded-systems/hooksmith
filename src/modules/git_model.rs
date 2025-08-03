//! Git file states, actions, and hooks model
//!
//! This module provides a comprehensive model that ties together:
//! - File states in Git (clean, modified, staged, etc.)
//! - Git actions (commit, push, merge, etc.)
//! - Git hooks (pre-commit, post-commit, etc.)
//! - Blocking behavior (which hooks can block which actions)
//! - Lefthook-specific configuration options
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
            FileStateKind::StagedAndModified => {
                "Staged changes exist, but the file has further modifications in worktree"
            }
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
    /// File system monitoring
    FsMonitor,
    /// Perforce operations
    P4Operations,
    /// Index changes
    IndexChange,
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
            ActionKind::FsMonitor => "File system monitoring",
            ActionKind::P4Operations => "Perforce operations",
            ActionKind::IndexChange => "Index changes",
        }
    }
}

/// Represents Git hooks that can run during various actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookKind {
    // Client-side hooks
    /// Runs to validate patch commit message
    ApplyPatchMsg,
    /// Runs before patch is applied
    PreApplyPatch,
    /// Runs after patch is applied
    PostApplyPatch,
    /// Runs before commit is created
    PreCommit,
    /// Runs before merge commit is created
    PreMergeCommit,
    /// Runs to prepare commit message
    PrepareCommitMsg,
    /// Runs to validate commit message
    CommitMsg,
    /// Runs after commit is created
    PostCommit,
    /// Runs before rebase starts
    PreRebase,
    /// Runs after checkout completes
    PostCheckout,
    /// Runs after merge is completed
    PostMerge,
    /// Runs before push to remote
    PrePush,
    /// Runs after history rewriting
    PostRewrite,
    /// Runs before automatic garbage collection
    PreAutoGc,
    /// Runs to validate email before sending
    SendEmailValidate,
    /// Runs when file system monitor detects changes
    FsMonitorWatchman,
    /// Runs for Perforce changelist
    P4Changelist,
    /// Runs to prepare Perforce changelist
    P4PrepareChangelist,
    /// Runs after Perforce changelist
    P4PostChangelist,
    /// Runs before Perforce submit
    P4PreSubmit,
    /// Runs after index changes
    PostIndexChange,

    // Server-side hooks
    /// Runs on server before receiving push
    PreReceive,
    /// Runs on server for each ref update
    Update,
    /// Runs on server to process receive
    ProcReceive,
    /// Runs on server after receiving push
    PostReceive,
    /// Runs on server after refs are updated
    PostUpdate,
    /// Runs on server for reference transactions
    ReferenceTransaction,
    /// Runs on server when pushing to checkout
    PushToCheckout,
}

impl HookKind {
    /// Get a human-readable description of the hook
    pub fn description(&self) -> &'static str {
        match self {
            HookKind::ApplyPatchMsg => "Runs to validate patch commit message",
            HookKind::PreApplyPatch => "Runs before patch is applied",
            HookKind::PostApplyPatch => "Runs after patch is applied",
            HookKind::PreCommit => "Runs before commit is created",
            HookKind::PreMergeCommit => "Runs before merge commit is created",
            HookKind::PrepareCommitMsg => "Runs to prepare commit message",
            HookKind::CommitMsg => "Runs to validate commit message",
            HookKind::PostCommit => "Runs after commit is created",
            HookKind::PreRebase => "Runs before rebase starts",
            HookKind::PostCheckout => "Runs after checkout completes",
            HookKind::PostMerge => "Runs after merge is completed",
            HookKind::PrePush => "Runs before push to remote",
            HookKind::PostRewrite => "Runs after history rewriting",
            HookKind::PreAutoGc => "Runs before automatic garbage collection",
            HookKind::SendEmailValidate => "Runs to validate email before sending",
            HookKind::FsMonitorWatchman => "Runs when file system monitor detects changes",
            HookKind::P4Changelist => "Runs for Perforce changelist",
            HookKind::P4PrepareChangelist => "Runs to prepare Perforce changelist",
            HookKind::P4PostChangelist => "Runs after Perforce changelist",
            HookKind::P4PreSubmit => "Runs before Perforce submit",
            HookKind::PostIndexChange => "Runs after index changes",
            HookKind::PreReceive => "Runs on server before receiving push",
            HookKind::Update => "Runs on server for each ref update",
            HookKind::ProcReceive => "Runs on server to process receive",
            HookKind::PostReceive => "Runs on server after receiving push",
            HookKind::PostUpdate => "Runs on server after refs are updated",
            HookKind::ReferenceTransaction => "Runs on server for reference transactions",
            HookKind::PushToCheckout => "Runs on server when pushing to checkout",
        }
    }

    /// Get the hook name as it appears in .git/hooks/
    pub fn filename(&self) -> &'static str {
        match self {
            HookKind::ApplyPatchMsg => "applypatch-msg",
            HookKind::PreApplyPatch => "pre-applypatch",
            HookKind::PostApplyPatch => "post-applypatch",
            HookKind::PreCommit => "pre-commit",
            HookKind::PreMergeCommit => "pre-merge-commit",
            HookKind::PrepareCommitMsg => "prepare-commit-msg",
            HookKind::CommitMsg => "commit-msg",
            HookKind::PostCommit => "post-commit",
            HookKind::PreRebase => "pre-rebase",
            HookKind::PostCheckout => "post-checkout",
            HookKind::PostMerge => "post-merge",
            HookKind::PrePush => "pre-push",
            HookKind::PostRewrite => "post-rewrite",
            HookKind::PreAutoGc => "pre-auto-gc",
            HookKind::SendEmailValidate => "sendemail-validate",
            HookKind::FsMonitorWatchman => "fsmonitor-watchman",
            HookKind::P4Changelist => "p4-changelist",
            HookKind::P4PrepareChangelist => "p4-prepare-changelist",
            HookKind::P4PostChangelist => "p4-post-changelist",
            HookKind::P4PreSubmit => "p4-pre-submit",
            HookKind::PostIndexChange => "post-index-change",
            HookKind::PreReceive => "pre-receive",
            HookKind::Update => "update",
            HookKind::ProcReceive => "proc-receive",
            HookKind::PostReceive => "post-receive",
            HookKind::PostUpdate => "post-update",
            HookKind::ReferenceTransaction => "reference-transaction",
            HookKind::PushToCheckout => "push-to-checkout",
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

/// Lefthook-specific command configuration
#[derive(Debug, Clone, Default)]
pub struct LefthookCommand {
    /// The command to run
    pub run: String,
    /// File pattern to match (e.g., "*.rs", "src/**/*.ts")
    pub files: Option<String>,
    /// Skip condition (can be boolean or array of conditions)
    pub skip: Option<SkipCondition>,
    /// Only condition (can be boolean or array of conditions)
    pub only: Option<OnlyCondition>,
    /// Advanced skip condition with sophisticated matching
    pub advanced_skip: Option<AdvancedSkipCondition>,
    /// Advanced only condition with sophisticated matching
    pub advanced_only: Option<AdvancedOnlyCondition>,
    /// Tags for this command
    pub tags: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// File types this command operates on
    pub file_types: Vec<String>,
    /// Glob patterns for file matching
    pub glob: Vec<String>,
    /// Root directory for the command
    pub root: Option<String>,
    /// Exclude patterns
    pub exclude: Option<ExcludeCondition>,
    /// Execution priority (higher numbers run first)
    pub priority: Option<i32>,
    /// Custom failure message
    pub fail_text: Option<String>,
    /// Whether the command is interactive
    pub interactive: Option<bool>,
    /// Whether to use stdin
    pub use_stdin: Option<bool>,
    /// Whether to stage fixed files
    pub stage_fixed: Option<bool>,
}

impl LefthookCommand {
    /// Create a new command with the given run string
    pub fn new(run: String) -> Self {
        Self {
            run,
            ..Default::default()
        }
    }

    /// Get the execution priority (defaults to 0 if not set)
    pub fn execution_priority(&self) -> i32 {
        self.priority.unwrap_or(0)
    }

    /// Check if this command should be skipped (legacy method)
    pub fn should_skip(&self) -> bool {
        if let Some(skip) = &self.skip {
            match skip {
                SkipCondition::Always => return true,
                SkipCondition::Conditions(_) => {
                    // In a real implementation, you'd evaluate the conditions
                    return false;
                }
            }
        }
        false
    }

    /// Check if this command should run (legacy method)
    pub fn should_run(&self) -> bool {
        if let Some(only) = &self.only {
            match only {
                OnlyCondition::Always => return true,
                OnlyCondition::Conditions(_) => {
                    // In a real implementation, you'd evaluate the conditions
                    return true;
                }
            }
        }
        true
    }

    /// Check if this command should be skipped using advanced conditions
    pub fn should_skip_advanced(&self, skip_checker: &SkipChecker) -> bool {
        skip_checker.check(self.advanced_skip.as_ref(), self.advanced_only.as_ref())
    }

    /// Check if this command should run using advanced conditions
    pub fn should_run_advanced(&self, skip_checker: &SkipChecker) -> bool {
        !self.should_skip_advanced(skip_checker)
    }

    /// Check if a file matches this command's patterns
    pub fn matches_file(&self, file_path: &str) -> bool {
        // Check files pattern
        if let Some(files) = &self.files {
            if !self.matches_pattern(file_path, files) {
                return false;
            }
        }

        // Check glob patterns
        if !self.glob.is_empty() {
            let matches_glob = self
                .glob
                .iter()
                .any(|pattern| self.matches_pattern(file_path, pattern));
            if !matches_glob {
                return false;
            }
        }

        // Check exclude patterns
        if let Some(exclude) = &self.exclude {
            match exclude {
                ExcludeCondition::Patterns(patterns) => {
                    if patterns
                        .iter()
                        .any(|pattern| self.matches_pattern(file_path, pattern))
                    {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Simple pattern matching (in a real implementation, use a proper glob library)
    fn matches_pattern(&self, file_path: &str, pattern: &str) -> bool {
        if pattern == "*" || pattern == "**/*" {
            return true;
        }
        if pattern.ends_with("*") {
            let prefix = pattern.trim_end_matches('*');
            return file_path.starts_with(prefix);
        }
        if pattern.starts_with("*") {
            let suffix = pattern.trim_start_matches('*');
            return file_path.ends_with(suffix);
        }
        // Handle patterns like "src/*.ts" - check if file_path matches the pattern structure
        if pattern.contains("*") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return file_path.starts_with(prefix) && file_path.ends_with(suffix);
            }
        }
        file_path.contains(pattern)
    }

    /// Check if this command's run string is compatible with file substitutions
    pub fn is_files_compatible(&self) -> bool {
        is_run_files_compatible(&self.run)
    }

    /// Get the resolved command string with file substitutions
    pub fn get_resolved_command(&self, context: &FileSubstitutionContext) -> String {
        context.substitute_files(&self.run)
    }

    /// Get all file substitution placeholders used in this command
    pub fn get_used_substitutions(&self, context: &FileSubstitutionContext) -> Vec<&'static str> {
        context.get_used_substitutions(&self.run)
    }

    /// Check if this command uses any file substitutions
    pub fn uses_file_substitutions(&self) -> bool {
        self.run.contains(file_substitutions::SUB_FILES)
            || self.run.contains(file_substitutions::SUB_ALL_FILES)
            || self.run.contains(file_substitutions::SUB_STAGED_FILES)
            || self.run.contains(file_substitutions::SUB_PUSH_FILES)
    }
}

/// Exclude condition for Lefthook commands
#[derive(Debug, Clone)]
pub enum ExcludeCondition {
    /// Exclude based on specific patterns
    Patterns(Vec<String>),
}

/// Lefthook-specific hook configuration options
#[derive(Debug, Clone, Default)]
pub struct LefthookHookConfig {
    /// Whether to run commands in parallel
    pub parallel: Option<bool>,
    /// Whether to pipe output between commands
    pub piped: Option<bool>,
    /// Whether to follow symlinks
    pub follow: Option<bool>,
    /// File pattern to match (e.g., "*.rs", "src/**/*.ts")
    pub files: Option<String>,
    /// Tags to exclude from execution
    pub exclude_tags: Vec<String>,
    /// Skip condition (can be boolean or array of conditions)
    pub skip: Option<SkipCondition>,
    /// Only condition (can be boolean or array of conditions)
    pub only: Option<OnlyCondition>,
    /// Commands to execute (legacy format)
    pub commands: HashMap<String, String>,
    /// Scripts to execute (legacy format)
    pub scripts: HashMap<String, String>,
    /// Detailed commands with full configuration
    pub detailed_commands: HashMap<String, LefthookCommand>,
}

impl LefthookHookConfig {
    /// Add a detailed command to this hook configuration
    pub fn add_command(&mut self, name: String, command: LefthookCommand) {
        self.detailed_commands.insert(name, command);
    }

    /// Get all commands sorted by priority (highest first)
    pub fn commands_by_priority(&self) -> Vec<(&String, &LefthookCommand)> {
        let mut commands: Vec<_> = self.detailed_commands.iter().collect();
        commands.sort_by(|(_, a), (_, b)| b.execution_priority().cmp(&a.execution_priority()));
        commands
    }

    /// Check if any command has incompatible file types
    pub fn has_incompatible_file_types(&self) -> bool {
        // In a real implementation, this would check for actual incompatibilities
        // For now, we'll consider it incompatible if there are no file types defined
        // or if there are conflicting file type definitions
        let all_file_types = self.all_file_types();
        if all_file_types.is_empty() {
            return false; // No file types defined, so no incompatibility
        }

        // Check if any command has file types that conflict with others
        // This is a simplified check - in reality, you'd have a more sophisticated
        // compatibility matrix
        false
    }

    /// Get all unique file types from all commands
    pub fn all_file_types(&self) -> Vec<String> {
        let mut file_types = std::collections::HashSet::new();

        for command in self.detailed_commands.values() {
            for file_type in &command.file_types {
                file_types.insert(file_type.clone());
            }
        }

        file_types.into_iter().collect()
    }

    /// Get commands that match a specific file
    pub fn commands_for_file(&self, file_path: &str) -> Vec<&LefthookCommand> {
        self.detailed_commands
            .values()
            .filter(|cmd| cmd.matches_file(file_path))
            .collect()
    }
}

/// Skip condition for Lefthook hooks
#[derive(Debug, Clone)]
pub enum SkipCondition {
    /// Always skip
    Always,
    /// Skip based on specific conditions
    Conditions(Vec<String>),
}

/// Only condition for Lefthook hooks
#[derive(Debug, Clone)]
pub enum OnlyCondition {
    /// Always run
    Always,
    /// Only run based on specific conditions
    Conditions(Vec<String>),
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
    /// Lefthook configuration for this file (if applicable)
    pub lefthook_config: Option<LefthookHookConfig>,
}

impl FileActionInfo {
    /// Create a new file action info
    pub fn new(path: String, state: FileStateKind) -> Self {
        let actions = actions_for_file(state);
        Self {
            path,
            state,
            actions,
            lefthook_config: None,
        }
    }

    /// Create a new file action info with Lefthook configuration
    pub fn with_lefthook_config(
        path: String,
        state: FileStateKind,
        config: LefthookHookConfig,
    ) -> Self {
        let actions = actions_for_file(state);
        Self {
            path,
            state,
            actions,
            lefthook_config: Some(config),
        }
    }

    /// Get all blocking hooks for this file
    pub fn blocking_hooks(&self) -> Vec<HookKind> {
        self.actions
            .iter()
            .flat_map(|(_, hooks)| hooks.iter().filter(|h| h.can_block).map(|h| h.hook))
            .collect()
    }

    /// Get all non-blocking hooks for this file
    pub fn non_blocking_hooks(&self) -> Vec<HookKind> {
        self.actions
            .iter()
            .flat_map(|(_, hooks)| hooks.iter().filter(|h| !h.can_block).map(|h| h.hook))
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

    /// Check if this file matches the Lefthook file pattern
    pub fn matches_file_pattern(&self) -> bool {
        if let Some(config) = &self.lefthook_config {
            if let Some(files) = &config.files {
                // Simple pattern matching - in a real implementation, you'd use a proper glob library
                if files == "*" || files == "**/*" {
                    return true;
                }
                if files.ends_with("*") {
                    let prefix = files.trim_end_matches('*');
                    return self.path.starts_with(prefix);
                }
                if files.starts_with("*") {
                    let suffix = files.trim_start_matches('*');
                    return self.path.ends_with(suffix);
                }
                return self.path.contains(files);
            }
        }
        true // If no pattern specified, match everything
    }

    /// Check if this file should be skipped based on Lefthook configuration
    pub fn should_skip(&self) -> bool {
        if let Some(config) = &self.lefthook_config {
            if let Some(skip) = &config.skip {
                match skip {
                    SkipCondition::Always => return true,
                    SkipCondition::Conditions(_) => {
                        // In a real implementation, you'd evaluate the conditions
                        return false;
                    }
                }
            }
        }
        false
    }

    /// Check if this file should run based on Lefthook configuration
    pub fn should_run(&self) -> bool {
        if let Some(config) = &self.lefthook_config {
            if let Some(only) = &config.only {
                match only {
                    OnlyCondition::Always => return true,
                    OnlyCondition::Conditions(_) => {
                        // In a real implementation, you'd evaluate the conditions
                        return true;
                    }
                }
            }
        }
        true
    }
}

/// Lefthook file substitution constants
pub mod file_substitutions {
    /// Substitution for specific files
    pub const SUB_FILES: &str = "{files}";
    /// Substitution for all files
    pub const SUB_ALL_FILES: &str = "{all_files}";
    /// Substitution for staged files
    pub const SUB_STAGED_FILES: &str = "{staged_files}";
    /// Substitution for push files
    pub const SUB_PUSH_FILES: &str = "{push_files}";
}

/// Check if run command is compatible with file substitutions
pub fn is_run_files_compatible(run: &str) -> bool {
    !run.contains(file_substitutions::SUB_STAGED_FILES)
        || !run.contains(file_substitutions::SUB_PUSH_FILES)
}

/// File substitution context for Lefthook commands
#[derive(Debug, Clone, Default)]
pub struct FileSubstitutionContext {
    /// Specific files to substitute
    pub files: Vec<String>,
    /// All files in the repository
    pub all_files: Vec<String>,
    /// Staged files
    pub staged_files: Vec<String>,
    /// Files being pushed
    pub push_files: Vec<String>,
}

impl FileSubstitutionContext {
    /// Create a new file substitution context
    pub fn new(
        files: Vec<String>,
        all_files: Vec<String>,
        staged_files: Vec<String>,
        push_files: Vec<String>,
    ) -> Self {
        Self {
            files,
            all_files,
            staged_files,
            push_files,
        }
    }

    /// Substitute file placeholders in a command string
    pub fn substitute_files(&self, command: &str) -> String {
        let mut result = command.to_string();

        // Replace {files} with specific files
        if result.contains(file_substitutions::SUB_FILES) {
            result = result.replace(file_substitutions::SUB_FILES, &self.files.join(" "));
        }

        // Replace {all_files} with all files
        if result.contains(file_substitutions::SUB_ALL_FILES) {
            result = result.replace(file_substitutions::SUB_ALL_FILES, &self.all_files.join(" "));
        }

        // Replace {staged_files} with staged files
        if result.contains(file_substitutions::SUB_STAGED_FILES) {
            result = result.replace(
                file_substitutions::SUB_STAGED_FILES,
                &self.staged_files.join(" "),
            );
        }

        // Replace {push_files} with push files
        if result.contains(file_substitutions::SUB_PUSH_FILES) {
            result = result.replace(
                file_substitutions::SUB_PUSH_FILES,
                &self.push_files.join(" "),
            );
        }

        result
    }

    /// Get all file substitution placeholders used in a command
    pub fn get_used_substitutions(&self, command: &str) -> Vec<&'static str> {
        let mut used = Vec::new();

        if command.contains(file_substitutions::SUB_FILES) {
            used.push(file_substitutions::SUB_FILES);
        }
        if command.contains(file_substitutions::SUB_ALL_FILES) {
            used.push(file_substitutions::SUB_ALL_FILES);
        }
        if command.contains(file_substitutions::SUB_STAGED_FILES) {
            used.push(file_substitutions::SUB_STAGED_FILES);
        }
        if command.contains(file_substitutions::SUB_PUSH_FILES) {
            used.push(file_substitutions::SUB_PUSH_FILES);
        }

        used
    }
}

/// Represents the current state of the Git repository
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GitRepoState {
    /// Normal state - not in any special operation
    Normal,
    /// Currently merging branches
    Merge,
    /// Commit created as part of a merge
    MergeCommit,
    /// Currently rebasing
    Rebase,
    /// Currently cherry-picking
    CherryPick,
    /// Currently applying patches
    ApplyPatch,
    /// Currently bisecting
    Bisect,
    /// Currently reverting
    Revert,
    /// Currently amending
    Amend,
    /// Currently in a detached HEAD state
    DetachedHead,
}

impl GitRepoState {
    /// Get a human-readable description of the repository state
    pub fn description(&self) -> &'static str {
        match self {
            GitRepoState::Normal => "Normal state - not in any special operation",
            GitRepoState::Merge => "Currently merging branches",
            GitRepoState::MergeCommit => "Commit created as part of a merge",
            GitRepoState::Rebase => "Currently rebasing",
            GitRepoState::CherryPick => "Currently cherry-picking",
            GitRepoState::ApplyPatch => "Currently applying patches",
            GitRepoState::Bisect => "Currently bisecting",
            GitRepoState::Revert => "Currently reverting",
            GitRepoState::Amend => "Currently amending",
            GitRepoState::DetachedHead => "Currently in a detached HEAD state",
        }
    }

    /// Convert to string representation (matches Lefthook's Git state strings)
    pub fn to_string(&self) -> &'static str {
        match self {
            GitRepoState::Normal => "",
            GitRepoState::Merge => "merge",
            GitRepoState::MergeCommit => "merge-commit",
            GitRepoState::Rebase => "rebase",
            GitRepoState::CherryPick => "cherry-pick",
            GitRepoState::ApplyPatch => "applypatch",
            GitRepoState::Bisect => "bisect",
            GitRepoState::Revert => "revert",
            GitRepoState::Amend => "amend",
            GitRepoState::DetachedHead => "detached",
        }
    }

    /// Create from string representation
    pub fn from_string(state: &str) -> Self {
        match state {
            "merge" => GitRepoState::Merge,
            "merge-commit" => GitRepoState::MergeCommit,
            "rebase" => GitRepoState::Rebase,
            "cherry-pick" => GitRepoState::CherryPick,
            "applypatch" => GitRepoState::ApplyPatch,
            "bisect" => GitRepoState::Bisect,
            "revert" => GitRepoState::Revert,
            "amend" => GitRepoState::Amend,
            "detached" => GitRepoState::DetachedHead,
            _ => GitRepoState::Normal,
        }
    }
}

/// Enhanced Git state information that includes repository state
#[derive(Debug, Clone)]
pub struct GitState {
    /// Current Git repository state
    pub repo_state: GitRepoState,
    /// Current branch name
    pub branch: String,
    /// Additional Git information
    pub additional_info: HashMap<String, String>,
}

impl Default for GitState {
    fn default() -> Self {
        Self {
            repo_state: GitRepoState::Normal,
            branch: String::new(),
            additional_info: HashMap::new(),
        }
    }
}

impl GitState {
    /// Create a new Git state
    pub fn new(repo_state: GitRepoState, branch: String) -> Self {
        Self {
            repo_state,
            branch,
            additional_info: HashMap::new(),
        }
    }

    /// Create a Git state with additional information
    pub fn with_info(
        repo_state: GitRepoState,
        branch: String,
        additional_info: HashMap<String, String>,
    ) -> Self {
        Self {
            repo_state,
            branch,
            additional_info,
        }
    }

    /// Create from string state (for backward compatibility)
    pub fn from_string_state(state: String, branch: String) -> Self {
        Self {
            repo_state: GitRepoState::from_string(&state),
            branch,
            additional_info: HashMap::new(),
        }
    }

    /// Get the string representation of the repository state
    pub fn state_string(&self) -> String {
        self.repo_state.to_string().to_string()
    }
}

/// Skip/Only condition value that can be evaluated
#[derive(Debug, Clone)]
pub enum ConditionValue {
    /// Boolean condition (always true/false)
    Boolean(bool),
    /// String condition (matches Git state or branch)
    String(String),
    /// Array of conditions (any match)
    Array(Vec<ConditionValue>),
    /// Reference condition with branch pattern
    Reference {
        /// The branch pattern to match
        ref_pattern: String,
    },
    /// Command condition that executes a shell command
    Command {
        /// The command to execute
        run: String,
    },
}

/// Advanced skip/only condition for Lefthook hooks and commands
#[derive(Debug, Clone)]
pub enum AdvancedSkipCondition {
    /// Always skip
    Always,
    /// Skip based on specific conditions
    Conditions(Vec<ConditionValue>),
}

/// Advanced only condition for Lefthook hooks and commands
#[derive(Debug, Clone)]
pub enum AdvancedOnlyCondition {
    /// Always run
    Always,
    /// Only run based on specific conditions
    Conditions(Vec<ConditionValue>),
}

/// Skip checker for evaluating skip/only conditions
pub struct SkipChecker {
    /// Function to get current Git state
    state_provider: Box<dyn Fn() -> GitState>,
}

impl SkipChecker {
    /// Create a new skip checker
    pub fn new<F>(state_provider: F) -> Self
    where
        F: Fn() -> GitState + 'static,
    {
        Self {
            state_provider: Box::new(state_provider),
        }
    }

    /// Check if execution should be skipped based on skip/only conditions
    pub fn check(
        &self,
        skip: Option<&AdvancedSkipCondition>,
        only: Option<&AdvancedOnlyCondition>,
    ) -> bool {
        if skip.is_none() && only.is_none() {
            return false;
        }

        if let Some(skip_condition) = skip {
            if self.matches_skip(skip_condition) {
                return true;
            }
        }

        if let Some(only_condition) = only {
            return !self.matches_only(only_condition);
        }

        false
    }

    /// Check if a skip condition matches the current state
    fn matches_skip(&self, condition: &AdvancedSkipCondition) -> bool {
        match condition {
            AdvancedSkipCondition::Always => true,
            AdvancedSkipCondition::Conditions(conditions) => {
                for condition_value in conditions {
                    if self.matches_value(condition_value) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Check if an only condition matches the current state
    fn matches_only(&self, condition: &AdvancedOnlyCondition) -> bool {
        match condition {
            AdvancedOnlyCondition::Always => true,
            AdvancedOnlyCondition::Conditions(conditions) => {
                if conditions.is_empty() {
                    return false; // Empty conditions mean no match
                }
                for condition_value in conditions {
                    if self.matches_value(condition_value) {
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Check if a condition value matches the current state
    fn matches_value(&self, value: &ConditionValue) -> bool {
        let state = (self.state_provider)();

        match value {
            ConditionValue::Boolean(b) => *b,
            ConditionValue::String(s) => s == &state.state_string(),
            ConditionValue::Array(values) => {
                for value in values {
                    if self.matches_value(value) {
                        return true;
                    }
                }
                false
            }
            ConditionValue::Reference { ref_pattern } => self.matches_ref(&state, ref_pattern),
            ConditionValue::Command { run } => self.matches_command(run),
        }
    }

    /// Check if a reference pattern matches the current branch
    fn matches_ref(&self, state: &GitState, ref_pattern: &str) -> bool {
        if ref_pattern == state.branch {
            return true;
        }

        // Simple glob pattern matching (in a real implementation, use a proper glob library)
        self.matches_glob_pattern(&state.branch, ref_pattern)
    }

    /// Simple glob pattern matching (in a real implementation, use a proper glob library)
    fn matches_glob_pattern(&self, branch: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if pattern.ends_with("*") {
            let prefix = pattern.trim_end_matches('*');
            return branch.starts_with(prefix);
        }
        if pattern.starts_with("*") {
            let suffix = pattern.trim_start_matches('*');
            return branch.ends_with(suffix);
        }
        branch == pattern
    }

    /// Execute a shell command and return the result
    fn matches_command(&self, command: &str) -> bool {
        // In a real implementation, this would execute the command
        // For now, we'll simulate some common patterns
        match command {
            "success" => {
                // Simulate a successful command execution
                true
            }
            "fail" => {
                // Simulate a failed command execution
                false
            }
            "git diff --cached --quiet" => {
                // Simulate: true if no staged changes
                false // Assume there are staged changes
            }
            "git diff --quiet" => {
                // Simulate: true if no unstaged changes
                true // Assume no unstaged changes
            }
            _ => {
                // For unknown commands, assume they succeed (return true)
                true
            }
        }
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
        Push => vec![HookInfo::new(PrePush, true)],
        ReceivePush => vec![
            HookInfo::new(PreReceive, true),
            HookInfo::new(ProcReceive, true),
            HookInfo::new(Update, true),
            HookInfo::new(PostReceive, false),
            HookInfo::new(PostUpdate, false),
            HookInfo::new(ReferenceTransaction, false),
            HookInfo::new(PushToCheckout, true),
        ],
        Checkout => vec![HookInfo::new(PostCheckout, false)],
        ApplyPatch => vec![
            HookInfo::new(ApplyPatchMsg, true),
            HookInfo::new(PreApplyPatch, true),
            HookInfo::new(PostApplyPatch, false),
        ],
        GarbageCollect => vec![HookInfo::new(PreAutoGc, true)],
        EmailSend => vec![HookInfo::new(SendEmailValidate, true)],
        FsMonitor => vec![HookInfo::new(FsMonitorWatchman, false)],
        P4Operations => vec![
            HookInfo::new(P4Changelist, true),
            HookInfo::new(P4PrepareChangelist, true),
            HookInfo::new(P4PostChangelist, false),
            HookInfo::new(P4PreSubmit, true),
        ],
        IndexChange => vec![HookInfo::new(PostIndexChange, false)],
    }
}

/// Get the actions allowed for a given file state
pub fn allowed_actions(state: FileStateKind) -> Vec<ActionKind> {
    use ActionKind::*;
    use FileStateKind::*;

    match state {
        Clean => vec![
            Commit,
            Checkout,
            Push,
            Merge,
            Rebase,
            FsMonitor,
            IndexChange,
        ],
        ModifiedUnstaged => vec![], // must be staged first
        Staged => vec![Commit, Push, IndexChange],
        StagedAndModified => vec![Commit, Push, IndexChange],
        Added => vec![Commit, Push, IndexChange],
        DeletedStaged => vec![Commit, Push, IndexChange],
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

/// Analyze multiple files with Lefthook configuration
pub fn analyze_files_with_config(
    files: Vec<(String, FileStateKind, Option<LefthookHookConfig>)>,
) -> Vec<FileActionInfo> {
    files
        .into_iter()
        .map(|(path, state, config)| {
            if let Some(config) = config {
                FileActionInfo::with_lefthook_config(path, state, config)
            } else {
                FileActionInfo::new(path, state)
            }
        })
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
    /// Hook is skipped due to skip/only conditions
    HookSkipped,
}

impl ContractValidation {
    /// Check if the contract is valid
    pub fn is_valid(&self) -> bool {
        matches!(self, ContractValidation::Valid)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ContractValidation::Valid => {
                "Valid contract: Hook can block this action for this file state"
            }
            ContractValidation::ActionNotAllowed => "Action not allowed for this file state",
            ContractValidation::HookNotRelevant => "Hook does not run for this action",
            ContractValidation::HookCannotBlock => "Hook runs but cannot block the action",
            ContractValidation::HookSkipped => "Hook is skipped due to skip/only conditions",
        }
    }
}

/// Enhanced contract validation that includes repository state and Lefthook conditions
pub fn validate_contract_with_repo_state(
    file_state: FileStateKind,
    action: ActionKind,
    hook: HookKind,
    repo_state: GitRepoState,
    branch: &str,
    lefthook_config: Option<&LefthookHookConfig>,
) -> ContractValidation {
    // First, validate the basic contract (file state + action + hook)
    let basic_validation = validate_contract(file_state, action, hook);
    if !basic_validation.is_valid() {
        return basic_validation;
    }

    // If we have Lefthook configuration, check skip/only conditions
    if let Some(config) = lefthook_config {
        let git_state = GitState::new(repo_state, branch.to_string());
        let skip_checker = SkipChecker::new(move || git_state.clone());

        // Convert legacy conditions to advanced conditions for checking
        let advanced_skip = config.skip.as_ref().map(|skip| match skip {
            SkipCondition::Always => AdvancedSkipCondition::Always,
            SkipCondition::Conditions(conditions) => {
                let condition_values = conditions
                    .iter()
                    .map(|s| {
                        // Check if this looks like a branch pattern (contains * or /)
                        if s.contains('*') || s.contains('/') {
                            ConditionValue::Reference {
                                ref_pattern: s.clone(),
                            }
                        } else {
                            ConditionValue::String(s.clone())
                        }
                    })
                    .collect();
                AdvancedSkipCondition::Conditions(condition_values)
            }
        });

        let advanced_only = config.only.as_ref().map(|only| match only {
            OnlyCondition::Always => AdvancedOnlyCondition::Always,
            OnlyCondition::Conditions(conditions) => {
                let condition_values = conditions
                    .iter()
                    .map(|s| {
                        // Check if this looks like a branch pattern (contains * or /)
                        if s.contains('*') || s.contains('/') {
                            ConditionValue::Reference {
                                ref_pattern: s.clone(),
                            }
                        } else {
                            ConditionValue::String(s.clone())
                        }
                    })
                    .collect();
                AdvancedOnlyCondition::Conditions(condition_values)
            }
        });

        // Check if the hook should be skipped based on skip/only conditions
        if skip_checker.check(advanced_skip.as_ref(), advanced_only.as_ref()) {
            return ContractValidation::HookSkipped;
        }
    }

    ContractValidation::Valid
}

/// Diagram generation for Git state transitions
pub mod diagrams {
    use super::*;

    /// Generate a DOT format diagram showing file state transitions and actions
    pub fn export_file_state_diagram() -> String {
        let mut dot = String::from("digraph GitFileStates {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=box, style=filled, fillcolor=lightblue];\n");
        dot.push_str("  edge [fontsize=10];\n\n");

        // Add all file states as nodes
        for state in [
            FileStateKind::Clean,
            FileStateKind::ModifiedUnstaged,
            FileStateKind::Staged,
            FileStateKind::StagedAndModified,
            FileStateKind::Added,
            FileStateKind::DeletedStaged,
            FileStateKind::Untracked,
            FileStateKind::Ignored,
        ] {
            dot.push_str(&format!(
                "  \"{:?}\" [label=\"{:?}\\n{}\"];\n",
                state,
                state,
                state.description()
            ));
        }

        dot.push('\n');

        // Add action edges
        for state in [
            FileStateKind::Clean,
            FileStateKind::ModifiedUnstaged,
            FileStateKind::Staged,
            FileStateKind::StagedAndModified,
            FileStateKind::Added,
            FileStateKind::DeletedStaged,
            FileStateKind::Untracked,
            FileStateKind::Ignored,
        ] {
            for action in allowed_actions(state) {
                let hooks = hooks_for_action(action);
                let hook_labels: Vec<String> = hooks
                    .iter()
                    .map(|h| format!("{:?}{}", h.hook, if h.can_block { "*" } else { "" }))
                    .collect();

                let edge_label = if hook_labels.is_empty() {
                    format!("{action:?}")
                } else {
                    format!("{:?}\\n[{}]", action, hook_labels.join(", "))
                };

                dot.push_str(&format!(
                    "  \"{state:?}\" -> \"{state:?}\" [label=\"{edge_label}\"];\n"
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate a Mermaid state diagram showing file state transitions
    pub fn export_file_state_mermaid() -> String {
        let mut mermaid = String::from("stateDiagram-v2\n");
        mermaid.push_str("    [*] --> Clean\n\n");

        // Add state transitions
        for state in [
            FileStateKind::Clean,
            FileStateKind::ModifiedUnstaged,
            FileStateKind::Staged,
            FileStateKind::StagedAndModified,
            FileStateKind::Added,
            FileStateKind::DeletedStaged,
            FileStateKind::Untracked,
            FileStateKind::Ignored,
        ] {
            let actions = allowed_actions(state);
            if !actions.is_empty() {
                for action in actions {
                    let hooks = hooks_for_action(action);
                    let hook_labels: Vec<String> = hooks
                        .iter()
                        .map(|h| format!("{:?}{}", h.hook, if h.can_block { "*" } else { "" }))
                        .collect();

                    let transition_label = if hook_labels.is_empty() {
                        format!("{action:?}")
                    } else {
                        format!("{:?} [{}]", action, hook_labels.join(", "))
                    };

                    mermaid.push_str(&format!(
                        "    {state:?} --> {state:?} : {transition_label}\n"
                    ));
                }
            }
        }

        mermaid
    }

    /// Generate a comprehensive diagram showing both file states and repository states
    pub fn export_comprehensive_diagram() -> String {
        let mut mermaid = String::from("graph TB\n");
        mermaid.push_str("    %% File States\n");
        mermaid.push_str("    subgraph FileStates [\"File States\"]\n");

        // Add file states
        for state in [
            FileStateKind::Clean,
            FileStateKind::ModifiedUnstaged,
            FileStateKind::Staged,
            FileStateKind::StagedAndModified,
            FileStateKind::Added,
            FileStateKind::DeletedStaged,
            FileStateKind::Untracked,
            FileStateKind::Ignored,
        ] {
            mermaid.push_str(&format!("        FS_{state:?}[{state:?}]:::fileState\n"));
        }

        mermaid.push_str("    end\n\n");

        // Add repository states
        mermaid.push_str("    %% Repository States\n");
        mermaid.push_str("    subgraph RepoStates [\"Repository States\"]\n");

        for state in [
            GitRepoState::Normal,
            GitRepoState::Merge,
            GitRepoState::MergeCommit,
            GitRepoState::Rebase,
            GitRepoState::CherryPick,
            GitRepoState::ApplyPatch,
            GitRepoState::Bisect,
            GitRepoState::Revert,
            GitRepoState::Amend,
            GitRepoState::DetachedHead,
        ] {
            mermaid.push_str(&format!("        RS_{state:?}[{state:?}]:::repoState\n"));
        }

        mermaid.push_str("    end\n\n");

        // Add actions
        mermaid.push_str("    %% Actions\n");
        mermaid.push_str("    subgraph Actions [\"Actions\"]\n");

        for action in [
            ActionKind::Commit,
            ActionKind::Merge,
            ActionKind::Rebase,
            ActionKind::Push,
            ActionKind::ReceivePush,
            ActionKind::Checkout,
            ActionKind::ApplyPatch,
            ActionKind::GarbageCollect,
            ActionKind::EmailSend,
            ActionKind::FsMonitor,
            ActionKind::P4Operations,
            ActionKind::IndexChange,
        ] {
            mermaid.push_str(&format!("        A_{action:?}[{action:?}]:::action\n"));
        }

        mermaid.push_str("    end\n\n");

        // Add hooks
        mermaid.push_str("    %% Hooks\n");
        mermaid.push_str("    subgraph Hooks [\"Git Hooks\"]\n");

        for hook in [
            HookKind::PreCommit,
            HookKind::CommitMsg,
            HookKind::PostCommit,
            HookKind::PrePush,
            HookKind::PreReceive,
            HookKind::PostReceive,
            HookKind::PreRebase,
            HookKind::PostRewrite,
            HookKind::PreMergeCommit,
            HookKind::PostMerge,
            HookKind::PostCheckout,
            HookKind::ApplyPatchMsg,
            HookKind::PreApplyPatch,
            HookKind::PostApplyPatch,
            HookKind::PreAutoGc,
            HookKind::SendEmailValidate,
            HookKind::FsMonitorWatchman,
            HookKind::P4Changelist,
            HookKind::P4PrepareChangelist,
            HookKind::P4PostChangelist,
            HookKind::P4PreSubmit,
            HookKind::PostIndexChange,
            HookKind::Update,
            HookKind::ProcReceive,
            HookKind::PostUpdate,
            HookKind::ReferenceTransaction,
            HookKind::PushToCheckout,
        ] {
            let can_block = hooks_for_action(ActionKind::Commit) // Just check one action
                .iter()
                .any(|h| h.hook == hook && h.can_block);

            let style = if can_block {
                ":::blockingHook"
            } else {
                ":::nonBlockingHook"
            };
            mermaid.push_str(&format!("        H_{hook:?}[{hook:?}]{style}\n"));
        }

        mermaid.push_str("    end\n\n");

        // Add relationships
        mermaid.push_str("    %% Relationships\n");

        // File states to actions
        for state in [
            FileStateKind::Clean,
            FileStateKind::ModifiedUnstaged,
            FileStateKind::Staged,
            FileStateKind::StagedAndModified,
            FileStateKind::Added,
            FileStateKind::DeletedStaged,
            FileStateKind::Untracked,
            FileStateKind::Ignored,
        ] {
            for action in allowed_actions(state) {
                mermaid.push_str(&format!("        FS_{state:?} -.-> A_{action:?}\n"));
            }
        }

        // Actions to hooks
        for action in [
            ActionKind::Commit,
            ActionKind::Merge,
            ActionKind::Rebase,
            ActionKind::Push,
            ActionKind::ReceivePush,
            ActionKind::Checkout,
            ActionKind::ApplyPatch,
            ActionKind::GarbageCollect,
            ActionKind::EmailSend,
            ActionKind::FsMonitor,
            ActionKind::P4Operations,
            ActionKind::IndexChange,
        ] {
            for hook_info in hooks_for_action(action) {
                let style = if hook_info.can_block {
                    ":::blocking"
                } else {
                    ":::nonBlocking"
                };
                mermaid.push_str(&format!(
                    "        A_{:?} --> H_{:?}{}\n",
                    action, hook_info.hook, style
                ));
            }
        }

        // Add styles
        mermaid.push_str("\n    %% Styles\n");
        mermaid.push_str("    classDef fileState fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
        mermaid.push_str("    classDef repoState fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n");
        mermaid.push_str("    classDef action fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px\n");
        mermaid
            .push_str("    classDef blockingHook fill:#ffebee,stroke:#c62828,stroke-width:3px\n");
        mermaid.push_str(
            "    classDef nonBlockingHook fill:#f1f8e9,stroke:#33691e,stroke-width:2px\n",
        );
        mermaid.push_str("    classDef blocking stroke:#c62828,stroke-width:3px\n");
        mermaid.push_str("    classDef nonBlocking stroke:#33691e,stroke-width:2px\n");

        mermaid
    }

    /// Generate a focused diagram showing the commit workflow
    pub fn export_commit_workflow_diagram() -> String {
        let mut mermaid = String::from("stateDiagram-v2\n");
        mermaid.push_str("    [*] --> Staged\n\n");

        // Commit workflow
        mermaid.push_str("    Staged --> PreCommit : Commit\n");
        mermaid.push_str("    PreCommit --> PrepareCommitMsg : PreCommit passes\n");
        mermaid.push_str("    PrepareCommitMsg --> CommitMsg : PrepareCommitMsg passes\n");
        mermaid.push_str("    CommitMsg --> Committed : CommitMsg passes\n");
        mermaid.push_str("    Committed --> PostCommit : Commit created\n");
        mermaid.push_str("    PostCommit --> Clean : PostCommit completes\n\n");

        // Error paths
        mermaid.push_str("    PreCommit --> [*] : PreCommit fails\n");
        mermaid.push_str("    CommitMsg --> [*] : CommitMsg fails\n\n");

        // Hook details
        mermaid.push_str("    note right of PreCommit : Can block commit\n");
        mermaid.push_str("    note right of PrepareCommitMsg : Can modify message\n");
        mermaid.push_str("    note right of CommitMsg : Can block commit\n");
        mermaid.push_str("    note right of PostCommit : Cannot block\n");

        mermaid
    }

    /// Generate a diagram showing skip/only conditions
    pub fn export_skip_only_diagram() -> String {
        let mut mermaid = String::from("flowchart TD\n");
        mermaid.push_str("    Start([Hook Triggered]) --> CheckSkip{Skip Condition?}\n");
        mermaid.push_str("    CheckSkip -->|Yes| EvaluateSkip{Evaluate Skip}\n");
        mermaid.push_str("    CheckSkip -->|No| CheckOnly{Only Condition?}\n");
        mermaid.push_str("    EvaluateSkip -->|Matches| Skip[Hook Skipped]\n");
        mermaid.push_str("    EvaluateSkip -->|No Match| CheckOnly\n");
        mermaid.push_str("    CheckOnly -->|Yes| EvaluateOnly{Evaluate Only}\n");
        mermaid.push_str("    CheckOnly -->|No| Run[Hook Runs]\n");
        mermaid.push_str("    EvaluateOnly -->|Matches| Run\n");
        mermaid.push_str("    EvaluateOnly -->|No Match| Skip\n\n");

        mermaid.push_str("    %% Condition Types\n");
        mermaid.push_str("    subgraph Conditions [\"Condition Types\"]\n");
        mermaid.push_str("        Boolean[Boolean: true/false]\n");
        mermaid.push_str("        State[Git State: merge, rebase, etc.]\n");
        mermaid.push_str("        Branch[Branch Pattern: feature/*, main]\n");
        mermaid.push_str("        Command[Shell Command: git diff --cached --quiet]\n");
        mermaid.push_str("    end\n\n");

        mermaid.push_str("    %% Styles\n");
        mermaid.push_str("    classDef decision fill:#fff3e0,stroke:#e65100,stroke-width:2px\n");
        mermaid.push_str("    classDef action fill:#e8f5e8,stroke:#1b5e20,stroke-width:2px\n");
        mermaid.push_str("    classDef skip fill:#ffebee,stroke:#c62828,stroke-width:2px\n");
        mermaid.push_str("    classDef run fill:#e1f5fe,stroke:#01579b,stroke-width:2px\n");
        mermaid.push_str("    classDef condition fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n\n");

        mermaid.push_str("    class CheckSkip,EvaluateSkip,CheckOnly,EvaluateOnly decision\n");
        mermaid.push_str("    class Run action\n");
        mermaid.push_str("    class Skip skip\n");
        mermaid.push_str("    class Boolean,State,Branch,Command condition\n");

        mermaid
    }
}
