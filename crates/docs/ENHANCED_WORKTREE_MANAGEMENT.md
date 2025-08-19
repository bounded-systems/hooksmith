# Enhanced Worktree Management Workflow

## Overview

The enhanced worktree management workflow provides comprehensive tools to manage your Git worktrees from anywhere in the repository, ensuring a clean main branch and synchronized remote branches.

## Features

- **Universal Access**: Run from any worktree or the main repository
- **Main Cleanup**: Automatically move changes from main to worktrees
- **Batch Operations**: Commit and push changes across all worktrees
- **Smart Detection**: Automatically detect worktree locations and branches
- **Safe Operations**: Preserve your current location and return after operations

## Commands

### 1. Main Cleanup (`cleanup`)

Cleans the main branch by moving any local changes to a new worktree.

```bash
# From main repository
./scripts/ensure-clean-main.sh cleanup

# From any worktree
./scripts/ensure-clean-main.sh cleanup

# Direct Rust execution
cargo run --manifest-path scripts/ensure-clean-main/Cargo.toml cleanup
```

**What it does:**
1. Detects if you're in a worktree or main repository
2. Switches to main repository if needed
3. Checks for uncommitted changes or commits ahead of origin/main
4. Creates a timestamped worktree for the changes
5. Moves all changes to the new worktree
6. Resets main to match origin/main
7. Returns to your original location

### 2. Commit All Worktrees (`commit-all`)

Commits changes in all worktrees with descriptive messages.

```bash
./scripts/ensure-clean-main.sh commit-all
```

**What it does:**
1. Scans all worktrees in the repository
2. Checks each worktree for uncommitted changes
3. Commits changes with descriptive messages
4. Reports progress for each worktree

### 3. Push All Worktrees (`push-all`)

Pushes all worktree branches to the remote repository.

```bash
./scripts/ensure-clean-main.sh push-all
```

**What it does:**
1. Identifies all worktree branches
2. Pushes each branch to the remote repository
3. Reports success/failure for each push operation

### 4. Sync All Worktrees (`sync-all`)

Combines commit and push operations for all worktrees.

```bash
./scripts/ensure-clean-main.sh sync-all
```

**What it does:**
1. Commits all changes in worktrees (equivalent to `commit-all`)
2. Pushes all branches to remote (equivalent to `push-all`)
3. Provides comprehensive status report

### 5. Help (`help`)

Shows available commands and their descriptions.

```bash
./scripts/ensure-clean-main.sh help
```

## Usage Examples

### Daily Workflow

```bash
# 1. Start your day - ensure main is clean
./scripts/ensure-clean-main.sh cleanup

# 2. Work in your worktrees throughout the day
# ... make changes in various worktrees ...

# 3. End of day - commit and push all changes
./scripts/ensure-clean-main.sh sync-all
```

### From Any Worktree

```bash
# You can run any command from any worktree
cd worktree-feature/my-feature
./scripts/ensure-clean-main.sh cleanup    # Works from worktree
./scripts/ensure-clean-main.sh commit-all # Works from worktree
./scripts/ensure-clean-main.sh push-all   # Works from worktree
```

### Batch Operations

```bash
# Commit all worktrees at once
./scripts/ensure-clean-main.sh commit-all

# Push all branches to remote
./scripts/ensure-clean-main.sh push-all

# Complete sync (commit + push)
./scripts/ensure-clean-main.sh sync-all
```

## Worktree Structure

The workflow automatically detects and manages worktrees with this structure:

```
hooksmith/
├── .git/                    # Main repository
├── worktree-feature-1/      # Worktree for feature-1 branch
├── worktree-feature-2/      # Worktree for feature-2 branch
├── worktree-fix-bug-1/      # Worktree for fix-bug-1 branch
└── scripts/
    └── ensure-clean-main/   # Management scripts
```

## Safety Features

### Location Preservation
- The script remembers your current location
- Automatically returns to your original worktree after operations
- Works seamlessly from any worktree or the main repository

### Error Handling
- Comprehensive error checking for all Git operations
- Detailed error messages with context
- Graceful failure handling

### State Validation
- Validates worktree integrity before operations
- Checks for valid Git repositories
- Ensures proper branch tracking

## Configuration

### Workspace Integration
The script is integrated into the Cargo workspace:

```toml
[workspace]
members = [
    ".",
    "crates/components/cli-core",
    "crates/xtask",
    "scripts/ensure-clean-main",
]
```

### Dependencies
The script uses standard Rust dependencies:
- `anyhow` - Error handling
- `tokio` - Async runtime
- `chrono` - Timestamp generation
- `serde` - JSON serialization

## Best Practices

### 1. Regular Cleanup
Run cleanup regularly to maintain a clean main branch:
```bash
# Before starting new work
./scripts/ensure-clean-main.sh cleanup
```

### 2. Frequent Commits
Commit worktree changes frequently:
```bash
# After completing a feature
./scripts/ensure-clean-main.sh commit-all
```

### 3. Regular Syncing
Sync with remote to ensure backup and collaboration:
```bash
# End of day or before switching contexts
./scripts/ensure-clean-main.sh sync-all
```

### 4. Worktree Organization
- Use descriptive branch names
- Keep worktrees focused on single features/fixes
- Clean up completed worktrees regularly

## Troubleshooting

### Common Issues

**1. "Failed to get git root directory"**
- Ensure you're in a Git repository
- Check that `.git` directory exists

**2. "Worktree directory not found"**
- Verify worktree exists
- Check worktree naming conventions

**3. "Failed to push branch"**
- Ensure remote repository is accessible
- Check branch permissions
- Verify remote tracking is set up

### Debug Mode

For detailed debugging, you can run the Rust script directly:
```bash
cargo run --manifest-path scripts/ensure-clean-main/Cargo.toml help
```

## Integration with Existing Workflows

### Git Hooks
The workflow integrates with existing Git hooks:
- Pre-commit hooks can call cleanup
- Post-commit hooks can trigger sync operations

### CI/CD Integration
- Can be integrated into CI/CD pipelines
- Supports automated worktree management
- Provides structured output for automation

## Future Enhancements

### Planned Features
- Interactive mode for selective operations
- Worktree cleanup and removal
- Branch merging workflows
- Conflict resolution assistance
- Performance optimizations

### Extensibility
The modular design allows for easy extension:
- Custom commit message templates
- Integration with external tools
- Custom validation rules
- Workflow customization

## Conclusion

The enhanced worktree management workflow provides a comprehensive solution for maintaining clean Git repositories with multiple worktrees. It ensures that your main branch stays clean while providing efficient tools for managing work across multiple features and fixes.

By following the best practices and using the provided commands, you can maintain a well-organized repository that supports efficient collaboration and development workflows. 
