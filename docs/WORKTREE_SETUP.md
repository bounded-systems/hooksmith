# Worktree Management Setup

This document describes how to set up and use the worktree management system in Hooksmith.

## Overview

The worktree management system provides a unified interface for creating, managing, and switching between Git worktrees. It supports multiple worktree management tools with automatic fallback and integrates seamlessly with the existing Hooksmith architecture.

## Available Tools

The system supports multiple worktree management tools in order of preference:

1. **gwtr** - Rust-based Git worktree manager with configuration layers (primary)
2. **workbloom** - Rust-based CLI with automatic file copying and port allocation
3. **wtp** - Git worktree management with hooks and automation
4. **git** - Native Git worktree commands (fallback)

### Tool Comparison

| Feature | gwtr | workbloom | wtp | git |
|---------|------|-----------|-----|-----|
| Language | Rust | Rust | Go | Native |
| Configuration | TOML layers | Line-based | YAML | Git config |
| File copying | ❌ | ✅ Automatic | ✅ Hooks | ❌ |
| Port allocation | ❌ | ✅ Automatic | ❌ | ❌ |
| Interactive cleanup | ❌ | ✅ Smart | ✅ Yes | ❌ |
| Shell auto-cd | ❌ | ✅ Built-in | ✅ Via alias | ❌ |
| Setup automation | ✅ Hooks | ✅ Built-in | ✅ Hooks | ❌ |

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
- `.gwtr.toml` - Configuration for gwtr with TOML schema
- `.workbloom` - Workbloom configuration for file copying
- `.wtp.yml` - Worktree tool configuration with hooks
- `.worktree-config.jsonc` - Hooksmith-specific worktree configuration

### 3. Setup Git Aliases (Optional)

```bash
# Setup Git aliases for worktree commands
cargo xtask worktree setup --aliases
```

## Configuration

### .gwtr.toml

The `.gwtr.toml` file configures gwtr with comprehensive TOML schema:

```toml
# .gwtr.toml
[general]
worktree_storage = "../"
default_branch_pattern = "feature/* => {repo}-{branch}"
ignore_glob = ["target/", ".cache/", "node_modules/", "dist/"]

[storage]
lru_limit = 15
use_named_subdirs = true
auto_cleanup = true

[named_worktrees]
"feature/spin-integration" = "hooksmith-spin"
"feature/spin-integration-v2" = "hooksmith-spin-integration"

[hooks]
post_create = [
    "echo 'Setting up worktree: {worktree_path}'",
    "cd {worktree_path} && cargo build",
    "cd {worktree_path} && cargo xtask gen-all --validate",
    "cd {worktree_path} && spin build || true"
]

[patterns]
[patterns.feature]
glob = "feature/*"
template = "{repo}-{branch}"
hooks = [
    "cd {worktree_path} && cargo build",
    "cd {worktree_path} && cargo xtask gen-all"
]
```

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

### .wtp.yml

The `.wtp.yml` file configures worktree tools with hooks:

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

branches:
  feature/spin-integration:
    path: ../hooksmith-spin
    post_create:
      - cd {worktree_path} && spin up
      - cd {worktree_path} && cargo xtask bootstrap --validate

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
  "preferred_tool": "workbloom",
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
# Create a new worktree (uses best available tool)
cargo xtask worktree create --branch feature/new-feature

# Create and switch to the new worktree
cargo xtask worktree create --branch feature/new-feature --switch

# Create with specific tool
cargo xtask worktree create --branch feature/new-feature --tool workbloom

# Create with setup commands
cargo xtask worktree create --branch feature/new-feature --setup
```

### Switch Between Worktrees

```bash
# Switch to an existing worktree
cargo xtask worktree switch --worktree feature/spin-integration

# Switch with specific tool
cargo xtask worktree switch --worktree feature/spin-integration --tool workbloom
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

## Tool-Specific Features

### Workbloom Features

Workbloom provides unique features that enhance the worktree experience:

#### Automatic File Copying
- Copies configuration files automatically to new worktrees
- Configurable via `.workbloom` file
- Includes environment files, tool configs, and development settings

#### Port Allocation
- Automatically assigns unique ports based on branch names
- Consistent port assignment for same branch names
- Injects ports into `.env` files

#### Smart Cleanup
- Interactive cleanup with `workbloom cleanup`
- Pattern-based cleanup with `workbloom cleanup --pattern`
- Status reporting with `workbloom cleanup --status`

#### Shell Integration
- Automatic shell opening with `workbloom setup`
- Environment synchronization on worktree switching
- Built-in `cd` command for worktree navigation

### gwtr Features

gwtr provides configuration layers and advanced management:

#### Configuration Layers
- Project-specific override files
- User/global configuration support
- Environment variable support
- XDG configuration patterns

#### Named Worktrees
- Map long branch names to friendly directory names
- Pattern-based configuration
- Comprehensive hook system

### wtp Features

wtp provides hook-based automation:

#### Hook System
- Post-create hooks for setup automation
- Post-switch hooks for environment validation
- Pre-remove hooks for cleanup

#### YAML Configuration
- Declarative configuration in YAML format
- Branch-specific and pattern-based configuration
- Project-specific settings

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

### Feature Development with Workbloom

```bash
# 1. Create a new feature branch worktree with Workbloom
cargo xtask worktree create --branch feature/new-feature --tool workbloom --switch

# 2. Workbloom automatically:
#    - Creates the worktree
#    - Copies configuration files
#    - Allocates unique ports
#    - Opens shell in the new worktree
#    - Runs setup commands

# 3. Switch between worktrees with environment sync
cargo xtask worktree switch --worktree feature/spin-integration --tool workbloom

# 4. Remove worktree with smart cleanup
cargo xtask worktree remove --worktree feature/new-feature --tool workbloom
```

### Multi-Branch Development

```bash
# List all worktrees to see current state
cargo xtask worktree list --detailed

# Create worktrees for different features
cargo xtask worktree create --branch feature/ui-improvements --tool workbloom
cargo xtask worktree create --branch bugfix/critical-fix --tool gwtr
cargo xtask worktree create --branch hotfix/security-patch --tool wtp

# Switch between them as needed
cargo xtask worktree switch --worktree feature/ui-improvements
```

### Tool-Specific Workflows

#### Workbloom Workflow
```bash
# Create with automatic setup
workbloom setup feature/new-feature

# List with status
workbloom list

# Smart cleanup
workbloom cleanup --status
workbloom cleanup --pattern "feature/spin-old"
```

#### gwtr Workflow
```bash
# Create with configuration
gwtr add feature/new-feature

# List worktrees
gwtr list

# Switch to worktree
gwtr cd feature/new-feature
```

#### wtp Workflow
```bash
# Create with hooks
wtp add feature/new-feature

# List worktrees
wtp list

# Switch to worktree
wtp cd feature/new-feature
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
```

## Best Practices

1. **Use Descriptive Branch Names**: Use clear, descriptive branch names that indicate the purpose (e.g., `feature/user-authentication`, `bugfix/login-error`)

2. **Leverage Tool Strengths**: 
   - Use Workbloom for automatic file copying and port allocation
   - Use gwtr for configuration layers and named worktrees
   - Use wtp for hook-based automation

3. **Regular Cleanup**: Periodically remove worktrees that are no longer needed to keep your workspace organized

4. **Consistent Setup**: Use the `--setup` flag when creating worktrees to ensure consistent environment setup

5. **Tool Preference**: Configure your preferred tool in `.worktree-config.jsonc` for consistent behavior

6. **Integration**: Leverage the integration points with Lefthook and WASM components for automated workflows

7. **Configuration Management**: Keep configuration files in version control for team consistency

## Related Documentation

- [Hooksmith Architecture](../ARCHITECTURE.md)
- [Xtask Commands](../CLI_HELP.md)
- [Lefthook Integration](../GIT_LEFTHOOK_INTEGRATION.md)
- [WASM Components](../COMPONENT_RUNNER_GUIDE.md) 
