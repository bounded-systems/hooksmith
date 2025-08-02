# Git File States, Actions, and Hooks Model for Contracts

This document explains how to use the comprehensive Git model in Hooksmith for contract validation and analysis.

## Overview

The Git model provides a complete matrix that ties together:
- **File States**: What state a file is in (clean, modified, staged, etc.)
- **Git Actions**: What action is being performed (commit, push, merge, etc.)
- **Git Hooks**: Which hooks run during the action
- **Blocking Behavior**: Whether hooks can block the action

This allows contracts to query whether actions are allowed for specific file states and whether they can be blocked by hooks.

## Core Components

### File States (`FileStateKind`)

Files can be in one of these states:

| State | Description |
|-------|-------------|
| `Clean` | Same in HEAD, index, and worktree |
| `ModifiedUnstaged` | Different in worktree but not staged |
| `Staged` | Added/changed in index but not committed yet |
| `StagedAndModified` | Staged changes exist, but file has further modifications |
| `Added` | Exists in index, not in HEAD |
| `DeletedStaged` | Deleted from index, still in HEAD |
| `Untracked` | Exists in worktree, not in index or HEAD |
| `Ignored` | Matches .gitignore |

### Git Actions (`ActionKind`)

Actions that can be performed:

| Action | Description |
|--------|-------------|
| `Commit` | Commit staged changes to history |
| `Merge` | Combine branches |
| `Rebase` | Rewrite commit history |
| `Push` | Upload commits to remote |
| `ReceivePush` | Server-side: receive push from client |
| `Checkout` | Switch branches or commits |
| `ApplyPatch` | Apply patches |
| `GarbageCollect` | Garbage collection |
| `EmailSend` | Send email patches |

### Git Hooks (`HookKind`)

Hooks that can run during actions:

#### Client-side Hooks
- `PreCommit` - Runs before commit is created
- `PrepareCommitMsg` - Runs to prepare commit message
- `CommitMsg` - Runs to validate commit message
- `PostCommit` - Runs after commit is created
- `PrePush` - Runs before push to remote
- `PreMergeCommit` - Runs before merge commit is created
- `PostMerge` - Runs after merge is completed
- `PreRebase` - Runs before rebase starts
- `PostRewrite` - Runs after history rewriting
- `PostCheckout` - Runs after checkout completes
- `ApplyPatchMsg` - Runs to validate patch commit message
- `PreApplyPatch` - Runs before patch is applied
- `PostApplyPatch` - Runs after patch is applied
- `PreAutoGc` - Runs before automatic garbage collection
- `SendEmailValidate` - Runs to validate email before sending

#### Server-side Hooks
- `PreReceive` - Runs on server before receiving push
- `Update` - Runs on server for each ref update
- `PostReceive` - Runs on server after receiving push

### Hook Information (`HookInfo`)

Each hook includes information about whether it can block:

```rust
pub struct HookInfo {
    pub hook: HookKind,
    pub can_block: bool,
}
```

### File Action Information (`FileActionInfo`)

Complete information about a file including its state and available actions:

```rust
pub struct FileActionInfo {
    pub path: String,
    pub state: FileStateKind,
    pub actions: Vec<(ActionKind, Vec<HookInfo>)>,
}
```

## Blocking Behavior

### Hooks That CAN Block
- `PreCommit`
- `CommitMsg`
- `PreMergeCommit`
- `PreRebase`
- `PrePush`
- `PreReceive`
- `Update`
- `ApplyPatchMsg`
- `PreApplyPatch`
- `PreAutoGc`
- `SendEmailValidate`

### Hooks That CANNOT Block
- `PrepareCommitMsg`
- `PostCommit`
- `PostMerge`
- `PostRewrite`
- `PostCheckout`
- `PostApplyPatch`
- `PostReceive`

## Usage Examples

### Basic Usage

```rust
use hooksmith::modules::git_model::*;

// Get allowed actions for a file state
let staged_actions = allowed_actions(FileStateKind::Staged);
assert_eq!(staged_actions, vec![ActionKind::Commit, ActionKind::Push]);

// Get hooks for an action
let commit_hooks = hooks_for_action(ActionKind::Commit);
// Returns: [HookInfo { hook: PreCommit, can_block: true }, ...]

// Check if a hook can block an action
let can_block = can_block(ActionKind::Commit, HookKind::PreCommit);
assert!(can_block); // true
```

### Per-File Analysis

```rust
// Analyze a single file
let file_info = FileActionInfo::new("src/main.rs".to_string(), FileStateKind::Staged);

println!("File: {}", file_info.path);
println!("State: {:?}", file_info.state);
println!("Actions: {:?}", file_info.actions.iter().map(|(action, _)| action).collect::<Vec<_>>());
println!("Can be blocked: {}", file_info.can_be_blocked());
println!("Blocking hooks: {:?}", file_info.blocking_hooks());
```

### Multiple File Analysis

```rust
let files = vec![
    ("src/main.rs".to_string(), FileStateKind::Staged),
    ("src/lib.rs".to_string(), FileStateKind::ModifiedUnstaged),
    ("docs/README.md".to_string(), FileStateKind::Untracked),
];

let file_analyses = analyze_files(files);
for file_info in file_analyses {
    println!("File: {} - State: {:?} - Actions: {}", 
        file_info.path, 
        file_info.state,
        file_info.actions.len()
    );
}
```

### Contract Validation

```rust
// Validate a contract
let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PreCommit);
match result {
    ContractValidation::Valid => println!("✅ Valid contract"),
    ContractValidation::ActionNotAllowed => println!("❌ Action not allowed"),
    ContractValidation::HookNotRelevant => println!("❌ Hook not relevant"),
    ContractValidation::HookCannotBlock => println!("⚠️ Hook cannot block"),
}
```

## Action-Hook Relationships

### Commit Workflow
1. `PreCommit` → can block
2. `PrepareCommitMsg` → can modify commit message, cannot block
3. User edits commit message (if not using -m)
4. `CommitMsg` → can block based on commit message validation
5. Commit is written
6. `PostCommit` → runs after commit (cannot block)

### Push Workflow
1. Client runs `PrePush` → can block push locally
2. Server runs:
   - `PreReceive` → can block push for all refs
   - `Update` → runs per branch, can block per-ref update
   - If accepted, `PostReceive` → runs after refs updated (cannot block)

### Merge Workflow
1. `PreMergeCommit` → can block merge commit
2. `CommitMsg` → can block based on commit message
3. Merge commit is written
4. `PostMerge` → runs after merge (cannot block)

## File State Action Matrix

| File State | Allowed Actions | Hooks That Run | Blockable? |
|------------|----------------|----------------|------------|
| Clean | Commit, Checkout, Push, Merge, Rebase | Depends on action | Varies |
| Modified (unstaged) | None (must be staged first) | No hooks | N/A |
| Staged | Commit, Push | pre-commit, prepare-commit-msg, commit-msg, post-commit, pre-push | ✅ |
| Staged + Modified | Commit, Push | Same as staged | ✅ |
| Added | Commit, Push | Same as staged | ✅ |
| Deleted (staged) | Commit, Push | Same as staged | ✅ |
| Untracked | None (must be added first) | No hooks | N/A |
| Ignored | None | No hooks | N/A |

## Contract System Integration

This model enables contracts to:

1. **Validate Action Permissions**: Check if an action is allowed for a given file state
2. **Determine Hook Execution**: Know which hooks will run for an action
3. **Assess Blocking Potential**: Understand whether hooks can block actions
4. **Analyze Workflows**: Understand the complete flow of actions and hooks

### Example Contract Rules

```rust
// Rule: Only allow commits on staged files
fn rule_commit_only_on_staged(state: FileStateKind, action: ActionKind) -> bool {
    if action == ActionKind::Commit {
        state == FileStateKind::Staged || state == FileStateKind::StagedAndModified
    } else {
        true
    }
}

// Rule: Ensure pre-commit hooks can block commits
fn rule_precommit_can_block(state: FileStateKind, action: ActionKind) -> bool {
    if action == ActionKind::Commit {
        can_block(ActionKind::Commit, HookKind::PreCommit)
    } else {
        true
    }
}

// Rule: Validate hook execution order
fn rule_hook_execution_order(action: ActionKind) -> Vec<HookKind> {
    hooks_for_action(action).iter().map(|h| h.hook).collect()
}
```

## Advanced Usage

### Custom Analysis

```rust
// Get all actions and their hooks for a file state
let actions_hooks = actions_for_file(FileStateKind::Staged);
for (action, hooks) in actions_hooks {
    println!("Action: {:?}", action);
    for hook_info in hooks {
        println!("  Hook: {} ({})", 
            hook_info.hook.filename(), 
            if hook_info.can_block { "can block" } else { "cannot block" }
        );
    }
}
```

### Hook Execution Analysis

```rust
// Get execution order for an action
let hooks = hooks_for_action(ActionKind::Commit);
for (i, hook_info) in hooks.iter().enumerate() {
    println!("Step {}: {} ({})", 
        i + 1, 
        hook_info.hook.filename(), 
        if hook_info.can_block { "can block" } else { "cannot block" }
    );
}
```

### Contract Validation Function

```rust
fn validate_git_contract(state: FileStateKind, action: ActionKind, hook: HookKind) {
    let result = validate_contract(state, action, hook);
    
    match result {
        ContractValidation::Valid => {
            println!("✅ Valid contract: Hook can block this action for this file state");
        }
        ContractValidation::ActionNotAllowed => {
            println!("❌ Action not allowed for this file state");
        }
        ContractValidation::HookNotRelevant => {
            println!("❌ Hook does not run for this action");
        }
        ContractValidation::HookCannotBlock => {
            println!("⚠️ Hook runs but cannot block the action");
        }
    }
}
```

## Integration with Hooksmith

This model integrates with Hooksmith's hook generation system to:

1. **Validate Hook Configurations**: Ensure hooks are configured for appropriate actions
2. **Generate Appropriate Hooks**: Create hooks that match the expected workflow
3. **Analyze Hook Dependencies**: Understand which hooks depend on which file states
4. **Optimize Hook Performance**: Place hooks in the most effective positions

### Example Integration

```rust
use hooksmith::modules::{git_model::*, lefthook::*};

fn generate_hooks_for_action(action: ActionKind) -> Vec<LefthookHook> {
    let hooks = hooks_for_action(action);
    
    hooks.into_iter().map(|hook_info| {
        LefthookHook {
            run: format!("echo 'Running {}'", hook_info.hook.filename()),
            files: Some("*.rs".to_string()),
            parallel: Some(false),
            env: None,
        }
    }).collect()
}

fn analyze_repository_files(files: Vec<(String, FileStateKind)>) -> Vec<FileActionInfo> {
    analyze_files(files)
}
```

## Benefits of the New Model

### 1. **Concise and Focused**
- Direct mappings without complex matrix structures
- Clear separation of concerns
- Easy to understand and extend

### 2. **Explicit Blocking Behavior**
- `HookInfo` struct makes blocking capability explicit
- No need to query separate data structures
- Clear and unambiguous

### 3. **Per-File Analysis**
- `FileActionInfo` provides complete analysis for each file
- Easy to show file state, actions, and hooks in one place
- Perfect for UI/CLI output

### 4. **Simple API**
- Function-based API instead of complex object methods
- Easy to compose and combine
- Clear function names and purposes

### 5. **Contract Validation**
- Built-in `validate_contract` function
- Clear validation results with descriptions
- Easy to integrate into contract systems

## Conclusion

The Git file states, actions, and hooks model provides a comprehensive foundation for:

- **Contract Validation**: Ensuring actions and hooks follow expected patterns
- **Workflow Analysis**: Understanding the complete Git workflow
- **Hook Generation**: Creating appropriate hooks for different scenarios
- **System Integration**: Building robust Git-based systems

This model enables contracts to make informed decisions about Git operations and ensures that hook systems behave predictably and correctly. 
