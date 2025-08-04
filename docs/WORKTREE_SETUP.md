# Worktree Management Setup

This document describes how to set up and use the worktree management system in Hooksmith.

## Overview

The worktree management system provides a unified interface for creating, managing, and switching between Git worktrees. It supports multiple worktree management tools with automatic fallback and integrates seamlessly with the existing Hooksmith architecture.

## Available Tools

The system supports multiple worktree management tools in order of preference:

1. **wtp** - Git worktree management with hooks and automation (primary)
2. **gwtr** - Simple Git worktree manager
3. **workbloom** - Git worktree management with automatic file copying
4. **git** - Native Git worktree commands (fallback)

## Quick Setup

### 1. Install Tools

```bash
# Install all recommended worktree management tools
cargo xtask worktree setup --install-tools
```

### 2. Create Configuration Files

```bash
# Create configuration files for worktree management
cargo xtask worktree setup --config
```

This creates:
- `.wtp.yml` - Configuration for worktree tools with hooks
- `.worktree-config.jsonc` - Hooksmith-specific worktree configuration

### 3. Setup Git Aliases (Optional)

```bash
# Setup Git aliases for worktree commands
cargo xtask worktree setup --aliases
```

## Configuration

### .wtp.yml

The `.wtp.yml` file configures worktree management with declarative settings:

```yaml
# .wtp.yml
wtp_version: 1

defaults:
  base_path: ../
  template: '{repo}-{branch}'
  post_create:
    - echo "Setting up worktree: {worktree_path}"
    - cd {worktree_path} && cargo build
    - cd {worktree_path} && cargo xtask gen-all --validate
    - cd {worktree_path} && spin build || true

# Existing worktrees - mapped to current layout
branches:
  feature/spin-integration:
    path: ../hooksmith-spin
    post_create:
      - cd {worktree_path} && spin up
      - cd {worktree_path} && cargo xtask bootstrap --validate

  feature/spin-integration-v2:
    path: ../hooksmith-spin-integration
    post_create:
      - cd {worktree_path} && spin build
      - cd {worktree_path} && cargo xtask gen-config

# Common branch patterns for future worktrees
patterns:
  feature/*:
    template: '{repo}-{branch}'
    post_create:
      - cd {worktree_path} && cargo build
      - cd {worktree_path} && cargo xtask gen-all
```

### .worktree-config.jsonc

The `.worktree-config.jsonc` file provides Hooksmith-specific configuration:

```jsonc
{
  "preferred_tool": "wtp",
  "worktree_base": "../",
  "worktree_template": "{repo}-{branch}",
  "run_setup": true,
  "setup_commands": [
    "cargo build",
    "cargo xtask gen-all --validate",
    "spin build || true"
  ],
  "existing_worktrees": {
    "feature/spin-integration": "../hooksmith-spin",
    "feature/spin-integration-v2": "../hooksmith-spin-integration"
  },
  "branch_patterns": {
    "feature/*": {
      "template": "{repo}-{branch}",
      "setup": ["cargo build", "cargo xtask gen-all"]
    }
  }
}
```

## Usage

### List Worktrees

```bash
# List all worktrees
cargo xtask worktree list

# List with detailed information
cargo xtask worktree list --detailed

# Output in JSON format
cargo xtask worktree list --format json
```

### Create Worktrees

```bash
# Create a new worktree
cargo xtask worktree create --branch feature/new-feature

# Create and switch to the new worktree
cargo xtask worktree create --branch feature/new-feature --switch

# Create with setup commands
cargo xtask worktree create --branch feature/new-feature --setup
```

### Switch Between Worktrees

```bash
# Switch to an existing worktree
cargo xtask worktree switch --worktree feature/spin-integration
```

### Remove Worktrees

```bash
# Remove a worktree
cargo xtask worktree remove --worktree feature/test

# Remove worktree and branch
cargo xtask worktree remove --worktree feature/test --with-branch

# Force removal without confirmation
cargo xtask worktree remove --worktree feature/test --force
```

### Status and Information

```bash
# Show worktree tool status
cargo xtask worktree status

# Show detailed tool information
cargo xtask worktree status --detailed
```

## Integration Points

### Lefthook Integration

The worktree management system can be integrated with Lefthook for automated workflows:

```yaml
# lefthook.yml
pre-commit:
  commands:
    worktree-check:
      glob: "*.{rs,toml,yml}"
      run: cargo xtask worktree status
```

### WASM Components

The system integrates with the existing `worktree-runner` WASM component:

```rust
// Use the worktree-runner component for advanced operations
let worktree_runner = WorktreeRunner::new();
let result = worktree_runner.list_worktrees().await?;
```

### CI/CD Integration

The worktree management system can be used in CI/CD pipelines:

```yaml
# GitHub Actions
- name: Setup worktrees
  run: cargo xtask worktree setup --all

- name: List worktrees
  run: cargo xtask worktree list --format json
```

## Workflow Examples

### Feature Development

```bash
# 1. Create a new feature branch worktree
cargo xtask worktree create --branch feature/new-feature --switch

# 2. The system automatically runs setup commands:
#    - cargo build
#    - cargo xtask gen-all --validate
#    - spin build (if available)

# 3. Switch between worktrees as needed
cargo xtask worktree switch --worktree feature/spin-integration

# 4. Remove worktree when done
cargo xtask worktree remove --worktree feature/new-feature --with-branch
```

### Multi-Branch Development

```bash
# List all worktrees to see current state
cargo xtask worktree list --detailed

# Create worktrees for different features
cargo xtask worktree create --branch feature/ui-improvements
cargo xtask worktree create --branch bugfix/critical-fix
cargo xtask worktree create --branch hotfix/security-patch

# Switch between them as needed
cargo xtask worktree switch --worktree feature/ui-improvements
```

## Troubleshooting

### Tool Not Found

If a worktree management tool is not found:

```bash
# Check available tools
cargo xtask worktree status

# Install missing tools
cargo xtask worktree setup --install-tools
```

### Configuration Issues

If configuration files are missing or incorrect:

```bash
# Recreate configuration files
cargo xtask worktree setup --config
```

### Permission Issues

If you encounter permission issues with Git aliases:

```bash
# Check existing Git aliases
git config --global --list | grep alias

# Setup aliases manually if needed
git config --global alias.wt worktree
git config --global alias.wtl "worktree list"
```

## Best Practices

1. **Use Descriptive Branch Names**: Use clear, descriptive branch names that indicate the purpose (e.g., `feature/user-authentication`, `bugfix/login-error`)

2. **Regular Cleanup**: Periodically remove worktrees that are no longer needed to keep your workspace organized

3. **Consistent Setup**: Use the `--setup` flag when creating worktrees to ensure consistent environment setup

4. **Tool Preference**: Configure your preferred tool in `.worktree-config.jsonc` for consistent behavior

5. **Integration**: Leverage the integration points with Lefthook and WASM components for automated workflows

## Related Documentation

- [Hooksmith Architecture](../ARCHITECTURE.md)
- [Xtask Commands](../CLI_HELP.md)
- [Lefthook Integration](../GIT_LEFTHOOK_INTEGRATION.md)
- [WASM Components](../COMPONENT_RUNNER_GUIDE.md) 
