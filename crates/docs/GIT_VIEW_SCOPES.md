# Git View Scopes - Comprehensive Reference

This document explains the different Git views/states that our dircheck validation system operates on, and which commands are appropriate for each validation scenario.

## 🎯 Git Views Overview

Git has three main views/states that commands can operate on:

1. **Index** - Staged files (what will be committed)
2. **Working Directory** - Unstaged changes and untracked files
3. **Commit Tree** - Files as stored in a specific commit (usually HEAD)

## 📊 Git View Matrix

| Command | View | Description | Use Case |
|---------|------|-------------|----------|
| `git ls-files` | Index | Shows all files currently tracked in the index | Validate committed/staged files |
| `git ls-files --others` | Working Directory | Shows untracked files | Validate new files |
| `git ls-tree HEAD` | Commit Tree | Lists files as stored in HEAD commit | Validate committed state |
| `git diff` | Working Directory | Compares working changes vs index | Validate unstaged changes |
| `git diff --cached` | Index | Compares staged changes vs last commit | Validate staged changes |
| `git diff HEAD` | Mixed | Compares working + staged vs last commit | Validate all uncommitted changes |
| `git status` | Mixed | Summarizes everything | Get comprehensive file status |
| `git show <commit>` | Commit Tree | Shows changes in a specific commit | Validate specific commit |
| `git diff-tree <commit>` | Commit Tree | Lists changed files in a commit | Validate commit changes (fast) |

## 🔍 DirCheck Implementation

Our dircheck system uses specific Git views for validation:

### Directory Structure Validation (`dircheck-tree`)
- **Command**: `git ls-tree -r --name-only HEAD`
- **View**: CommitTree
- **Purpose**: Validates directory structure as it exists in the HEAD commit
- **Scope**: Only committed files, not including uncommitted changes

### File Structure Validation (`dircheck-files`)
- **Command**: `git ls-files`
- **View**: Index
- **Purpose**: Validates file structure of tracked files in the index
- **Scope**: Staged files, not unstaged changes or untracked files

## 🛠️ Git Query Module

The `git_query` module provides structured access to Git commands with their scopes:

```rust
use dircheck_core::git_query::{GitQueryCommands, GitView};

// Get command for validating HEAD commit tree
let tree_cmd = GitQueryCommands::ls_tree_head();
println!("View: {:?}", tree_cmd.view); // CommitTree

// Get command for validating index files
let files_cmd = GitQueryCommands::ls_files_index();
println!("View: {:?}", files_cmd.view); // Index
```

## 📋 Available Git Commands

### Index-Only Commands
```rust
GitQueryCommands::ls_files_index()      // All tracked files
GitQueryCommands::diff_staged()         // Staged vs HEAD
```

### Working Directory Commands
```rust
GitQueryCommands::ls_files_working()    // Untracked files
GitQueryCommands::diff_working()        // Unstaged changes
```

### Commit Tree Commands
```rust
GitQueryCommands::ls_tree_head()        // HEAD commit files
GitQueryCommands::show_commit(commit)   // Specific commit
GitQueryCommands::diff_tree_commit(commit) // Fast commit diff
```

### Mixed View Commands
```rust
GitQueryCommands::diff_all()            // All uncommitted changes
GitQueryCommands::status_porcelain()    // Comprehensive status
```

## 🎯 Validation Scenarios

### Scenario 1: CI/CD Pipeline
- **Use**: `git ls-tree HEAD` + `git ls-files`
- **Reason**: Validate committed state and staged changes
- **Scope**: What will be deployed

### Scenario 2: Pre-commit Hook
- **Use**: `git diff --cached`
- **Reason**: Validate only staged changes
- **Scope**: What will be committed

### Scenario 3: Working Directory Check
- **Use**: `git status --porcelain`
- **Reason**: Validate all changes including untracked files
- **Scope**: Complete working state

### Scenario 4: Historical Validation
- **Use**: `git show <commit>` or `git diff-tree <commit>`
- **Reason**: Validate specific commit or branch
- **Scope**: Historical state

## 🔧 Command Execution

```rust
use dircheck_core::git_query::{execute_git_command, GitQueryCommands};

// Execute a Git command
let cmd = GitQueryCommands::ls_tree_head();
let paths = execute_git_command(&cmd)?;

// Validate the paths
let violations = validate_tree_commit(&paths, &rules);
```

## 📊 Output Examples

### Directory Structure (CommitTree View)
```
🔍 Validating directory structure using: Lists all files and folders in HEAD commit
📋 Git view: CommitTree
✅ All directory structure rules passed
```

### File Structure (Index View)
```
🔍 Validating file structure using: Lists all tracked files in the index
📋 Git view: Index
❌ Found 3 file structure violations:
  Rule: forbidden_root_extensions
  Path: temp.md
  Error: File with forbidden extension '.md' found in root
  Suggestion: Move 'temp.md' to appropriate subdirectory
```

## 🚀 Performance Considerations

### Fast Commands
- `git ls-tree` - Fast, reads commit tree directly
- `git diff-tree` - Fast, optimized for commit diffs
- `git ls-files` - Fast, reads index directly

### Slower Commands
- `git status` - Slower, checks working directory
- `git diff` - Slower, compares working vs index
- `git show` - Slower, full commit object parsing

## 🔄 Integration with Act

For CI testing with act, use the fastest commands:

```yaml
# .github/workflows/validate.yml
- name: Validate committed state
  run: cargo run -p dircheck-tree  # Uses git ls-tree (fast)

- name: Validate staged files
  run: cargo run -p dircheck-files  # Uses git ls-files (fast)
```

## 📝 Best Practices

1. **Choose the right view**: Use CommitTree for CI, Index for pre-commit
2. **Be explicit**: Always document which Git view you're validating
3. **Optimize for speed**: Use fast commands in CI pipelines
4. **Handle errors**: Git commands can fail (no repo, no commits, etc.)
5. **Clear output**: Show which view is being validated

## 🔮 Future Enhancements

1. **Multiple view validation**: Validate across different Git views
2. **Custom Git commands**: Allow project-specific Git command configuration
3. **View comparison**: Compare validation results across different views
4. **Performance profiling**: Measure and optimize Git command execution
5. **Error recovery**: Handle Git command failures gracefully
