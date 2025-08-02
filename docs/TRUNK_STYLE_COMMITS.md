# Trunk-Style Empty Commit Messages

This document describes the implementation of Trunk-style empty commit message support in Hooksmith, which allows developers to create commits with empty messages while still maintaining conventional commit validation for non-empty messages.

## Overview

Trunk (the Rust web framework) allows empty commit messages by default, which can be useful for quick commits during development. This implementation provides the same behavior while integrating with Hooksmith's existing commit validation system.

## Features

### ✅ Empty Message Support
- **Allow empty commit messages** by default (Trunk-style)
- **No validation errors** for empty messages
- **Clear feedback** when empty messages are accepted

### ✅ Conventional Commit Validation
- **Validate non-empty messages** with conventional commit format
- **Comprehensive error messages** with examples
- **Support for all conventional commit types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`, `revert`

### ✅ Git Integration
- **Lefthook integration** via commit-msg hooks
- **Git aliases** for convenient Trunk-style commits
- **Post-commit reminders** for empty commits

## Implementation Components

### 1. Xtask Command: `validate-commit-msg`

The core validation logic is implemented as an xtask command:

```bash
cargo run -p xtask -- validate-commit-msg --file "$1"
```

**Features:**
- Reads commit message from file (defaults to `$1` from lefthook)
- Allows empty messages when `--allow-empty=true` (default)
- Validates conventional commit format for non-empty messages
- Provides detailed error messages with examples

**Usage:**
```bash
# Validate with default settings (allows empty, validates conventional)
cargo run -p xtask -- validate-commit-msg --file .git/COMMIT_EDITMSG

# Disable empty message support
cargo run -p xtask -- validate-commit-msg --allow-empty=false

# Disable conventional commit validation
cargo run -p xtask -- validate-commit-msg --validate-conventional=false
```

### 2. Lefthook Configuration

The commit-msg hook is configured in `lefthook-example.yml`:

```yaml
commit-msg:
  commands:
    conventional:
      run: cargo run -p xtask -- validate-commit-msg --file "$1"
      description: "Validate commit message (Trunk-style: allows empty messages)"
```

**Post-commit reminder hook:**
```yaml
post-commit:
  commands:
    empty-commit-reminder:
      run: |
        if [ -z "$(git log -1 --pretty=%B)" ]; then
          echo "✅ Empty commit message accepted (Trunk-style)."
          echo "💡 Use 'git commit --amend' if you want to add details later."
        fi
      description: "Remind about empty commit messages (Trunk-style)"
```

### 3. Git Aliases

Convenient git aliases for Trunk-style commits:

```bash
# Set up aliases
cargo run -p xtask -- setup-git-aliases

# Available aliases:
git cm [options]  # Commit with Trunk-style empty message support
git cc [options]  # Regular commit (requires message)
git ce [options]  # Quick empty commit (Trunk-style)
```

### 4. Git Commit Wrapper Script

The `scripts/git-trunk-commit.sh` script provides a wrapper around `git commit`:

```bash
#!/bin/bash
# Git commit wrapper with Trunk-style empty message support

# Execute git commit with --allow-empty-message flag
git commit --allow-empty-message "$@"

# Check if the commit message is empty and show reminder
if [ -z "$(git log -1 --pretty=%B)" ]; then
    echo "✅ Empty commit message accepted (Trunk-style)"
    echo "💡 Use 'git commit --amend' if you want to add details later"
fi
```

## Usage Examples

### Basic Usage

```bash
# 1. Set up git aliases (one-time setup)
cargo run -p xtask -- setup-git-aliases

# 2. Use Trunk-style commits
git cm                    # Commit with empty message (Trunk-style)
git cm -m 'feat: add feature'  # Commit with conventional message
git ce                    # Quick empty commit

# 3. Regular git commit still works
git commit -m 'fix: bug fix'   # Standard commit with message
```

### Workflow Examples

**Quick Development Commits:**
```bash
# Make changes
git add .
git cm  # Empty commit for quick save
```

**Feature Development:**
```bash
# Start feature
git checkout -b feature/new-feature

# Quick commits during development
git add .
git cm  # Empty commit

# More changes
git add .
git cm  # Another empty commit

# Final commit with proper message
git cm -m 'feat: implement new feature with comprehensive tests'
```

**Bug Fixes:**
```bash
# Quick fix
git add .
git cm  # Empty commit

# Later, add proper message
git commit --amend -m 'fix: resolve null pointer exception in parser'
```

## Configuration

### Lefthook Configuration

The commit validation is automatically configured when using the provided `lefthook-example.yml`:

```yaml
# This enables Trunk-style empty message support
commit-msg:
  commands:
    conventional:
      run: cargo run -p xtask -- validate-commit-msg --file "$1"
```

### Git Configuration

Git aliases are configured via the setup command:

```bash
cargo run -p xtask -- setup-git-aliases
```

This sets up:
- `git cm` → Trunk-style commit wrapper
- `git cc` → Regular commit
- `git ce` → Quick empty commit

## Validation Rules

### Empty Messages
- ✅ **Always allowed** (Trunk-style behavior)
- ✅ **No validation errors**
- ✅ **Clear feedback** when accepted

### Non-Empty Messages
Must follow conventional commit format:
```
<type>(<scope>): <description>
```

**Supported Types:**
- `feat` - New features
- `fix` - Bug fixes
- `docs` - Documentation changes
- `style` - Code style changes
- `refactor` - Code refactoring
- `test` - Adding or updating tests
- `chore` - Maintenance tasks
- `perf` - Performance improvements
- `ci` - CI/CD changes
- `build` - Build system changes
- `revert` - Reverting changes

**Examples:**
```bash
feat(cli): add new command
fix(wasm): correct parsing bug
docs: update README
chore(ci): update GitHub Actions
```

## Error Messages

When validation fails, users get comprehensive error messages:

```
Commit message must follow conventional commit format:

Format: <type>(<scope>): <description>

Types: feat, fix, docs, style, refactor, test, chore, perf, ci, build, revert

Examples:
• feat(cli): add new command
• fix(wasm): correct parsing bug
• docs: update README
• chore(ci): update GitHub Actions

Your message: invalid message

Note: Empty commit messages are allowed (Trunk-style).
```

## Integration with Existing Workflows

### CI/CD Integration
The commit validation works seamlessly with existing CI/CD pipelines:
- **Pre-commit hooks** validate commit messages
- **Post-commit hooks** provide reminders
- **No breaking changes** to existing workflows

### Team Workflows
- **New developers** can use Trunk-style commits immediately
- **Experienced developers** can continue using conventional commits
- **Gradual adoption** - teams can choose their preferred style

## Benefits

### For Developers
- **Faster development** with quick empty commits
- **No interruption** during rapid iteration
- **Flexible workflow** - empty or conventional messages
- **Clear feedback** on validation status

### For Teams
- **Consistent validation** for non-empty messages
- **No breaking changes** to existing workflows
- **Gradual adoption** possible
- **Clear documentation** and examples

### For Projects
- **Maintains code quality** with conventional commit validation
- **Supports rapid development** with empty message support
- **Integrates seamlessly** with existing tooling
- **Configurable** for different project needs

## Troubleshooting

### Common Issues

**"No commit message file path provided"**
- Ensure lefthook is properly configured
- Check that the commit-msg hook is installed

**"Not in a git repository"**
- Run setup commands from the project root
- Ensure you're in a git repository

**Alias conflicts**
- Use `--force` flag to overwrite existing aliases
- Check existing aliases with `git config --get-regexp alias`

### Debugging

**Test validation manually:**
```bash
# Create a test commit message file
echo "feat: test commit" > test-commit.txt

# Test validation
cargo run -p xtask -- validate-commit-msg --file test-commit.txt
```

**Check lefthook configuration:**
```bash
# Verify lefthook is installed
lefthook --version

# Check hook installation
lefthook install
```

## Future Enhancements

### Potential Improvements
- **Custom commit types** configuration
- **Project-specific** validation rules
- **Integration** with issue tracking systems
- **Analytics** on commit message patterns
- **Template system** for common commit messages

### Extensibility
The system is designed to be extensible:
- **Modular validation** logic
- **Configurable** rules and patterns
- **Plugin system** for custom validators
- **Integration points** for external tools

## Conclusion

The Trunk-style empty commit message implementation provides a flexible, developer-friendly approach to commit validation that:

1. **Supports rapid development** with empty message commits
2. **Maintains code quality** with conventional commit validation
3. **Integrates seamlessly** with existing workflows
4. **Provides clear feedback** and documentation
5. **Enables gradual adoption** by teams

This implementation strikes the right balance between developer productivity and code quality, making it easier for teams to maintain good commit practices while supporting fast-paced development workflows. 
