# Worktree Management Setup

This document describes how to set up and use the worktree management system in Hooksmith.

## Overview

The worktree management system provides a unified interface for creating, managing, and switching between Git worktrees using **Workbloom** as the primary tool. Workbloom offers superior features for automatic file copying, port allocation, smart cleanup, and shell integration.

## Available Tools

The system supports two worktree management tools in order of preference:

1. **workbloom** - Rust-based CLI with automatic file copying and port allocation (primary)
2. **git** - Native Git worktree commands (fallback)

### Why Workbloom?

Workbloom is the superior choice for Hooksmith because it provides:

| Feature | Workbloom ✅ | git ❌ |
|---------|-------------|--------|
| Worktree creation | ✅ yes | ✅ yes |
| Interactive shell after setup | ✅ built-in | ❌ |
| Auto .env / .envrc sync | ✅ yes | ❌ manual |
| Port assignment for local dev | ✅ automatic | ❌ |
| Cleanup of old worktrees | ✅ smart + batch | ❌ manual |
| Config file support | .workbloom | ad hoc |
| Component reuse / ecosystem | 🔄 better | limited |

## Quick Setup

### 1. Install Workbloom

```bash
# Install Workbloom (primary tool)
cargo install workbloom

# Or use the setup command to install automatically
cargo xtask worktree setup --install-tools
```

### 2. Create Configuration Files

```bash
# Create configuration files for worktree management
cargo xtask worktree setup --config
```

This creates:
- `.workbloom` - Workbloom configuration for file copying and port allocation
- `.worktree-config.jsonc` - Hooksmith-specific worktree configuration

### 3. Setup Git Aliases (Optional)

```bash
# Setup Git aliases for worktree commands
cargo xtask worktree setup --aliases
```

## Configuration

### .workbloom

The `.workbloom` file configures automatic file copying and port allocation:

```bash
# .workbloom
# Environment files
.env
.envrc
.env.local
.env.example

# Configuration files
hooksmith.toml
.worktree-config.jsonc
.worktree-config.json

# Development configuration
.vscode/settings.json
.vscode/launch.json
.vscode/extensions.json

# Tool configuration
.claude/settings.json
.config/my-settings.json

# Spin configuration
spin.toml
spin.toml.example

# Cargo configuration
.cargo/config.toml
.cargo/config

# Git configuration
.gitignore
.gitattributes

# Documentation
README.md
docs/

# Scripts and utilities
scripts/
tools/

# Secrets and local config (if they exist)
secrets/
local/
```

### .worktree-config.jsonc

The `.worktree-config.jsonc` file provides Hooksmith-specific configuration:

```jsonc
{
  "worktree_base": "../",
  "worktree_template": "{repo}-{branch}",
  "run_setup": true,
  "setup_commands": [
    "cargo build",
    "cargo xtask gen-all --validate",
    "spin build || true"
  ],
  "copy_env": true,
  "env_files": [
    ".env.example",
    ".env",
    ".envrc",
    "hooksmith.toml",
    ".worktree-config.jsonc"
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
# Create a new worktree with Workbloom (automatic file copying and port allocation)
cargo xtask worktree create --branch feature/new-feature

# Create and switch to the new worktree
cargo xtask worktree create --branch feature/new-feature --switch

# Create with setup commands
cargo xtask worktree create --branch feature/new-feature --setup
```

### Switch Between Worktrees

```bash
# Switch to an existing worktree with environment synchronization
cargo xtask worktree switch --worktree feature/spin-integration
```

### Remove Worktrees

```bash
# Remove a worktree with smart cleanup
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

## Workbloom Features

Workbloom provides unique features that enhance the worktree experience:

### Automatic File Copying
- Copies configuration files automatically to new worktrees
- Configurable via `.workbloom` file
- Includes environment files, tool configs, and development settings

### Port Allocation
- Automatically assigns unique ports based on branch names
- Consistent port assignment for same branch names
- Injects ports into `.env` files

### Smart Cleanup
- Interactive cleanup with `workbloom cleanup`
- Pattern-based cleanup with `workbloom cleanup --pattern`
- Status reporting with `workbloom cleanup --status`

### Shell Integration
- Automatic shell opening with `workbloom setup`
- Environment synchronization on worktree switching
- Built-in `cd` command for worktree navigation

## Workflow Examples

### Feature Development with Workbloom

```bash
# 1. Create a new feature branch worktree with Workbloom
cargo xtask worktree create --branch feature/new-feature --switch

# 2. Workbloom automatically:
#    - Creates the worktree
#    - Copies configuration files
#    - Allocates unique ports
#    - Opens shell in the new worktree
#    - Runs setup commands

# 3. Switch between worktrees with environment sync
cargo xtask worktree switch --worktree feature/spin-integration

# 4. Remove worktree with smart cleanup
cargo xtask worktree remove --worktree feature/new-feature
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

### Direct Workbloom Commands

You can also use Workbloom directly for advanced features:

```bash
# Create with automatic setup
workbloom setup feature/new-feature

# List with status
workbloom list

# Smart cleanup
workbloom cleanup --status
workbloom cleanup --pattern "feature/spin-old"

# Switch to worktree
workbloom cd feature/spin-integration
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

## Troubleshooting

### Tool Not Found

If Workbloom is not found:

```bash
# Check available tools
cargo xtask worktree status

# Install Workbloom
cargo install workbloom
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

### Workbloom-Specific Issues

If Workbloom encounters issues:

```bash
# Check Workbloom configuration
cat .workbloom

# Verify file copying
workbloom setup feature/test --no-shell
ls -la ../hooksmith-test/

# Check port allocation
workbloom list

# Smart cleanup
workbloom cleanup --status
```

## Best Practices

1. **Use Descriptive Branch Names**: Use clear, descriptive branch names that indicate the purpose (e.g., `feature/user-authentication`, `bugfix/login-error`)

2. **Leverage Workbloom's Features**: 
   - Use automatic file copying for consistent environments
   - Take advantage of port allocation for local development
   - Use smart cleanup for maintenance

3. **Regular Cleanup**: Periodically remove worktrees that are no longer needed to keep your workspace organized

4. **Consistent Setup**: Use the `--setup` flag when creating worktrees to ensure consistent environment setup

5. **Configuration Management**: Keep `.workbloom` configuration in version control for team consistency

6. **Integration**: Leverage the integration points with Lefthook and WASM components for automated workflows

7. **Direct Workbloom Usage**: For advanced features, use Workbloom commands directly when needed

## Related Documentation

- [Hooksmith Architecture](../ARCHITECTURE.md)
- [Xtask Commands](../CLI_HELP.md)
- [Lefthook Integration](../GIT_LEFTHOOK_INTEGRATION.md)
- [WASM Components](../COMPONENT_RUNNER_GUIDE.md) 
