# Worktree Management Setup for Hooksmith

This document describes the worktree management setup for the Hooksmith project, including tool installation, configuration, and usage.

## Overview

Hooksmith uses a comprehensive worktree management system that integrates multiple tools and provides a unified interface through the `xtask` command-line tool. This setup enables efficient development workflows with isolated environments for different features and branches.

## Available Tools

### 1. wtp (Recommended)
- **Description**: Go-based CLI with hooks and automation
- **Features**: Post-create hooks, configuration files, templating
- **Installation**: `cargo install wtp`
- **Configuration**: `.wtp.yml`

### 2. wt (git-wt)
- **Description**: Minimalist CLI for short aliases
- **Features**: Simple commands, fast startup
- **Installation**: `cargo install git-wt`
- **Configuration**: Built-in defaults

### 3. Git Native
- **Description**: Native Git worktree commands
- **Features**: Always available, full Git integration
- **Installation**: Included with Git
- **Configuration**: Git configuration

## Quick Setup

### 1. Install Tools

```bash
# Install recommended tools
cargo xtask worktree setup --install-tools

# Or install manually
cargo install wtp
cargo install git-wt
```

### 2. Create Configuration

```bash
# Create all configuration files
cargo xtask worktree setup --config

# Or create manually
cargo xtask worktree setup --all
```

### 3. Setup Git Aliases

```bash
# Setup Git aliases for convenience
cargo xtask worktree setup --aliases
```

## Configuration Files

### .wtp.yml
Main configuration file for wtp tool:

```yaml
version: 1.0

defaults:
  base_dir: worktrees
  post_create:
    - type: copy
      from: .env.example
      to: .env
    - type: command
      command: cargo build
    - type: command
      command: cargo xtask gen-all

hooks:
  post_create:
    - name: setup-hooksmith
      commands:
        - cargo xtask bootstrap --validate
        - cargo xtask gen-config
        - cargo xtask gen-docs
```

### .worktree-config.jsonc
Hooksmith-specific configuration:

```jsonc
{
  "preferred_tool": "wtp",
  "worktree_base": "worktrees",
  "run_setup": true,
  "setup_commands": [
    "cargo build",
    "cargo xtask gen-all --validate"
  ],
  "git_aliases": {
    "wt": "worktree",
    "wtl": "worktree list",
    "wtc": "worktree create"
  }
}
```

## Usage

### Basic Commands

```bash
# List all worktrees
cargo xtask worktree list

# Create a new worktree
cargo xtask worktree create --branch feature/new-feature

# Switch to a worktree
cargo xtask worktree switch --worktree feature/new-feature

# Remove a worktree
cargo xtask worktree remove --worktree feature/new-feature --with-branch

# Show tool status
cargo xtask worktree status
```

### Advanced Usage

```bash
# Create worktree with custom base directory
cargo xtask worktree create --branch feature/test --base-dir ../worktrees

# Create and switch to worktree
cargo xtask worktree create --branch feature/test --switch

# List with detailed information
cargo xtask worktree list --detailed

# Output in JSON format
cargo xtask worktree list --format json
```

### Git Aliases

After setup, you can use convenient Git aliases:

```bash
# List worktrees
git wtl

# Create worktree
git wtc feature/new-feature

# Remove worktree
git wtr feature/new-feature

# Switch worktree
git wts feature/new-feature
```

## Integration with Hooksmith

### Lefthook Integration

The worktree system integrates with Lefthook for automated workflows:

```yaml
# lefthook.yml
pre-commit:
  worktree-check:
    run: cargo xtask worktree status --format json
    stage_fixed: all
```

### WASM Component Integration

The worktree-runner component provides WASM interface for worktree management:

```rust
use hooksmith::WorktreeOperation;

let operation = WorktreeOperation::Create {
    branch_name: "feature/test".to_string(),
    base_path: Some("worktrees".to_string()),
};

let result = orchestrator.manage_worktree(operation).await?;
```

### Xtask Integration

All worktree operations are available through xtask:

```bash
# Bootstrap with worktree setup
cargo xtask bootstrap --worktree-setup

# Development workflow with worktree management
cargo xtask dev-workflow --worktree feature/test
```

## Workflow Examples

### Feature Development

```bash
# 1. Create feature worktree
cargo xtask worktree create --branch feature/new-component --switch

# 2. Develop in isolated environment
# ... make changes ...

# 3. Run validation
cargo xtask check-all

# 4. Commit and push
git add .
git commit -m "feat: add new component"
git push origin feature/new-component

# 5. Clean up when done
cargo xtask worktree remove --worktree feature/new-component --with-branch
```

### Multi-Branch Development

```bash
# Create multiple worktrees for different features
cargo xtask worktree create --branch feature/ui-improvements
cargo xtask worktree create --branch feature/api-enhancements
cargo xtask worktree create --branch bugfix/critical-fix

# Switch between them
cargo xtask worktree switch --worktree feature/ui-improvements
# ... work on UI ...

cargo xtask worktree switch --worktree feature/api-enhancements
# ... work on API ...

cargo xtask worktree switch --worktree bugfix/critical-fix
# ... fix bug ...
```

### CI/CD Integration

```yaml
# .github/workflows/worktree-test.yml
name: Worktree Tests

on:
  pull_request:
    branches: [main]

jobs:
  test-worktrees:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup worktree environment
        run: |
          cargo xtask worktree setup --all
          cargo xtask worktree create --branch test-branch
      
      - name: Run tests in worktree
        run: |
          cargo xtask worktree switch --worktree test-branch
          cargo test
```

## Troubleshooting

### Common Issues

1. **Tool not found**
   ```bash
   # Check tool availability
   cargo xtask worktree status
   
   # Install missing tools
   cargo xtask worktree setup --install-tools
   ```

2. **Permission issues**
   ```bash
   # Check Git configuration
   git config --list | grep worktree
   
   # Reset Git aliases
   cargo xtask worktree setup --aliases
   ```

3. **Configuration conflicts**
   ```bash
   # Validate configuration
   cargo xtask worktree status --detailed
   
   # Reset configuration
   rm .worktree-config.json
   cargo xtask worktree setup --config
   ```

### Debug Mode

```bash
# Enable verbose output
RUST_LOG=debug cargo xtask worktree list --detailed

# Check tool availability
cargo xtask worktree status --format json
```

## Best Practices

1. **Use descriptive branch names**: `feature/user-authentication`, `bugfix/login-crash`
2. **Clean up worktrees**: Remove worktrees when features are merged
3. **Preserve important branches**: Don't remove `main`, `develop`, or `master`
4. **Use hooks**: Leverage post-create and pre-remove hooks for automation
5. **Validate environments**: Run `cargo xtask check-all` after switching worktrees

## Migration from Manual Worktrees

If you have existing worktrees created manually:

```bash
# 1. List existing worktrees
git worktree list

# 2. Setup the new system
cargo xtask worktree setup --all

# 3. Migrate existing worktrees (optional)
# The system will detect and work with existing worktrees

# 4. Use the new commands going forward
cargo xtask worktree create --branch new-feature
```

## References

- [wtp Documentation](https://github.com/satococoa/wtp)
- [git-wt Documentation](https://github.com/branchvincent/git-wt)
- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [Hooksmith Architecture](../ARCHITECTURE.md)
- [Xtask Commands](../CLI_HELP.md) 
