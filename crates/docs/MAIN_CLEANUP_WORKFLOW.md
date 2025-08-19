# Main Cleanup Workflow

## Overview

The Main Cleanup Workflow is an automated system that ensures your local `main` branch is never ahead of `origin/main`. This prevents confusion and maintains a clean, collaborative development environment.

## Why This Matters

- **Clean Main Branch**: Main should always match `origin/main` for easy collaboration
- **Organized Work**: All changes are automatically moved to dedicated worktrees
- **No Lost Work**: Changes are safely preserved in worktrees with proper commits
- **Automated Safety**: Prevents accidental commits to main that could cause confusion

## How It Works

### 1. Detection
The workflow automatically detects when:
- Main is ahead of `origin/main` (has local commits)
- There are uncommitted changes on main
- You're working on the main branch

### 2. Automatic Cleanup
When changes are detected:
1. Creates a timestamped worktree (e.g., `fix/main-cleanup-20250804-143022`)
2. Moves all local commits and changes to the new worktree
3. Commits the changes in the worktree
4. Resets main to match `origin/main`
5. Provides clear feedback about what was done

### 3. Worktree Organization
- All worktrees follow the naming pattern: `fix/main-cleanup-YYYYMMDD-HHMMSS`
- Each worktree has a corresponding branch with the same name
- Changes are properly committed and tracked

## Usage

### Manual Cleanup

Run the cleanup script manually:

```bash
# Using the shell script (recommended)
./scripts/ensure-clean-main.sh

# Or using cargo directly
cargo run --manifest-path scripts/ensure-clean-main/Cargo.toml
```

### Automatic Cleanup

The workflow runs automatically via Git hooks:

- **Pre-commit Hook**: Automatically runs before commits on main
- **Manual Trigger**: Run the script anytime you want to ensure main is clean

## Workflow Examples

### Scenario 1: Main is Ahead with Commits

```bash
# Current state: main is ahead by 2 commits
$ git log --oneline origin/main..HEAD
f10b52c8 feat: Add workspace dependencies
a1b2c3d4 feat: Update documentation

# Run cleanup
$ ./scripts/ensure-clean-main.sh

# Result: Changes moved to worktree, main is clean
✅ Main cleanup completed!
📁 Changes moved to worktree: fix/main-cleanup-20250804-143022
🌿 Branch: fix/main-cleanup-20250804-143022
```

### Scenario 2: Uncommitted Changes

```bash
# Current state: uncommitted changes on main
$ git status
 M Cargo.toml
?? new-file.txt

# Run cleanup
$ ./scripts/ensure-clean-main.sh

# Result: Changes committed in worktree, main is clean
✅ Main cleanup completed!
📁 Changes moved to worktree: fix/main-cleanup-20250804-143022
🌿 Branch: fix/main-cleanup-20250804-143022
```

### Scenario 3: Main is Already Clean

```bash
# Current state: main matches origin/main
$ git status
On branch main
Your branch is up to date with 'origin/main'.

# Run cleanup
$ ./scripts/ensure-clean-main.sh

# Result: No action needed
✅ Main is clean and up to date with origin/main
```

## File Structure

```
scripts/
├── ensure-clean-main/
│   ├── Cargo.toml          # Rust script dependencies
│   └── src/
│       └── main.rs         # Main cleanup logic
├── ensure-clean-main.sh    # Shell script wrapper
└── ensure-clean-main.rs    # Legacy script file

hooks/
└── pre-commit-main-cleanup # Git pre-commit hook

docs/
└── MAIN_CLEANUP_WORKFLOW.md # This documentation
```

## Configuration

### Git Hook Setup

The pre-commit hook is automatically installed when you run:

```bash
# Install the hook (if not already installed)
chmod +x hooks/pre-commit-main-cleanup
```

### Workspace Integration

The cleanup script is integrated into the workspace:

```toml
[workspace]
members = [
    ".",
    "crates/components/cli-core",
    "crates/xtask",
    "scripts/ensure-clean-main",  # Cleanup script
]
```

## Best Practices

### 1. Regular Cleanup
- Run the cleanup script regularly to keep main clean
- Use the pre-commit hook for automatic protection
- Check worktree status after cleanup

### 2. Worktree Management
- Review worktrees created by the cleanup process
- Merge or delete worktrees when work is complete
- Use descriptive commit messages in worktrees

### 3. Collaboration
- Always pull latest changes before starting work
- Use worktrees for feature development
- Keep main clean for easy collaboration

## Troubleshooting

### Common Issues

**Issue**: Script fails to create worktree
```
Solution: Ensure you have proper permissions and the xtask tool is working
```

**Issue**: Main still shows as ahead after cleanup
```
Solution: Check if there are any uncommitted changes or stashed changes
```

**Issue**: Worktree directory not found
```
Solution: The worktree creation may have failed. Check the xtask worktree list
```

### Debug Mode

For debugging, you can run the script with verbose output:

```bash
# Run with cargo for more detailed output
cargo run --manifest-path scripts/ensure-clean-main/Cargo.toml
```

### Manual Recovery

If the automated cleanup fails:

1. Check current status: `git status`
2. Check worktrees: `cargo xtask worktree list`
3. Manually create worktree if needed
4. Reset main: `git reset --hard origin/main`

## Integration with Other Workflows

### Worktree Management
- Works seamlessly with the existing worktree management system
- Creates properly configured worktrees with metadata
- Integrates with Workbloom for enhanced tracking

### Git Operations
- Compatible with all standard Git operations
- Preserves commit history in worktrees
- Maintains proper branch tracking

### CI/CD Integration
- Can be integrated into CI/CD pipelines
- Ensures clean main branch for deployments
- Provides audit trail of changes

## Future Enhancements

### Planned Features
- [ ] Integration with Lefthook for enhanced Git hooks
- [ ] Automatic worktree cleanup for old worktrees
- [ ] Webhook notifications for cleanup events
- [ ] Integration with issue tracking systems

### Configuration Options
- [ ] Configurable worktree naming patterns
- [ ] Custom commit message templates
- [ ] Selective file inclusion/exclusion
- [ ] Integration with external tools

## Contributing

To contribute to the main cleanup workflow:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

### Development

To develop the cleanup script:

```bash
# Build the script
cargo build --manifest-path scripts/ensure-clean-main/Cargo.toml

# Run tests
cargo test --manifest-path scripts/ensure-clean-main/Cargo.toml

# Run with debug output
RUST_LOG=debug cargo run --manifest-path scripts/ensure-clean-main/Cargo.toml
```

## Support

For issues or questions about the main cleanup workflow:

1. Check this documentation
2. Review the troubleshooting section
3. Check existing worktrees and their status
4. Open an issue in the repository

---

**Note**: This workflow is designed to be safe and non-destructive. All changes are preserved in worktrees, and you can always recover your work if needed. 
