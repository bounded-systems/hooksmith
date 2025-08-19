# Git Configuration System

This document describes the comprehensive Git configuration system built for the Hooksmith project, which provides structured management of all Git-related configuration files and settings.

## Overview

The Git configuration system consists of four main components:

1. **Tracked Git Config Files** - Repository-level configuration files
2. **Git Hooks** - Rust-based hooks with Git aliases
3. **Git Configuration** - Local `.git/config` management
4. **Git Attributes** - File behavior configuration

## 1. Tracked Git Configuration Files

### What Are Tracked Config Files?

These are Git-related files that are committed to the repository and tracked by Git:

| File | Purpose | Format | Required |
|------|---------|--------|----------|
| `.gitattributes` | File behavior: diff, merge, binary, eol | Line-based | ✅ Yes |
| `.gitignore` | Ignore untracked files | Glob | ✅ Yes |
| `.gitmodules` | Submodule configuration | INI | ❌ Optional |
| `.mailmap` | Canonical author mapping | Line-based | ❌ Optional |

### Example `.gitattributes`

```gitattributes
# Text files with specific line endings
*.sh text eol=lf
*.bat text eol=crlf

# Binary files
*.jpg binary -diff -merge

# Custom diff drivers
*.json diff=json

# Linguist overrides (GitHub)
*.ts linguist-language=TypeScript
```

### Example `.gitignore`

```gitignore
# Build artifacts
target/
dist/

# Dependencies
node_modules/

# IDE files
.vscode/
.idea/

# Environment files
.env
.env.local
```

### Example `.gitmodules`

```ini
# This repository has no submodules
# 
# This file is tracked to explicitly indicate that no submodules are used.
# If submodules are added in the future, they should be declared here.
```

## 2. Git Hooks with Rust Binaries

### Architecture

Each Git hook has:
- **Rust Binary**: `hook-<hook-name>` (e.g., `hook-pre-commit`)
- **Git Alias**: `hook-<hook-name>` (e.g., `hook-pre-commit`)
- **Hook Stub**: `.git/hooks/<hook-name>` that delegates to the alias

### Standard Hooks

| Hook | Purpose | Binary | Alias |
|------|---------|--------|-------|
| `pre-commit` | Run linting and tests | `hook-pre_commit` | `hook-pre-commit` |
| `commit-msg` | Validate commit message | `hook-commit_msg` | `hook-commit-msg` |
| `pre-push` | Security checks | `hook-pre_push` | `hook-pre-push` |
| `post-merge` | Update dependencies | `hook-post_merge` | `hook-post-merge` |

### Hook Stub Example

```bash
#!/bin/sh
exec git hook-pre-commit "$@"
```

### Rust Hook Example

```rust
//! Pre-commit Hook
//! 
//! This hook runs before each commit to ensure code quality.

use std::env;
use std::process;

fn main() {
    println!("🔍 Running pre-commit checks...");
    
    // Run cargo check
    let check_status = std::process::Command::new("cargo")
        .args(["check"])
        .status();
    
    if let Ok(status) = check_status {
        if !status.success() {
            eprintln!("❌ Cargo check failed");
            process::exit(1);
        }
    }
    
    println!("✅ Pre-commit checks passed");
    process::exit(0);
}
```

## 3. Git Configuration Management

### Configuration Categories

Git configuration is organized into five categories:

#### 1. Identity
User identity and commit behavior:
```ini
[user]
    name = Robert DeLanghe
    email = bobbit@example.com

[commit]
    gpgsign = true
```

#### 2. Remote
Remotes, branches, and syncing:
```ini
[remote "origin"]
    url = git@github.com:user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*

[branch "main"]
    remote = origin
    merge = refs/heads/main
```

#### 3. Behavior
Behavior customization and safety:
```ini
[core]
    editor = nvim
    filemode = true
    autocrlf = input

[merge]
    tool = meld

[rebase]
    autosquash = true
```

#### 4. Alias
Aliases and custom commands:
```ini
[alias]
    co = checkout
    st = status
    ci = commit
    l = log --oneline --graph --decorate
    safe-push = !./scripts/safe-push.sh
```

#### 5. Tooling
Tooling integration and custom sections:
```ini
[vscode]
    merge-base = origin/main

[github]
    pr-number = 42

[xtask]
    validation-mode = strict
```

### includeIf Conditional Configuration

Use `includeIf` for conditional configuration based on repository path:

```ini
# Global config
[user]
    name = Robert DeLanghe
    email = bobbit@personal.com

# Work repositories
[includeIf "gitdir:~/work/"]
    path = ~/.gitconfig-work

# Personal repositories  
[includeIf "gitdir:~/personal/"]
    path = ~/.gitconfig-personal
```

## 4. Git Attributes Management

### Attribute Categories

Git attributes are organized into categories:

#### 1. Text Handling
```gitattributes
*.sh text eol=lf
*.bat text eol=crlf
*.jpg binary
```

#### 2. Diff Drivers
```gitattributes
*.json diff=json
*.yaml diff=yaml
```

#### 3. Merge Strategies
```gitattributes
*.lock merge=union
config/*.conf merge=ours
```

#### 4. Export Filtering
```gitattributes
secret.key export-ignore
```

#### 5. Linguist Overrides
```gitattributes
*.ts linguist-language=TypeScript
docs/** linguist-documentation
```

## Usage

### Command Line Interface

The system provides comprehensive CLI tools through `cargo xtask`:

```bash
# Tracked config files
cargo xtask git-tracked-config scan
cargo xtask git-tracked-config validate
cargo xtask git-tracked-config list

# Git hooks
cargo xtask git-hooks install
cargo xtask git-hooks list
cargo xtask git-hooks test pre-commit

# Git configuration
cargo xtask git-config convert
cargo xtask git-config analyze
cargo xtask git-config template

# Git attributes
cargo xtask git-attributes convert
cargo xtask git-attributes analyze
cargo xtask git-attributes template
```

### JSONC Output

All components can export to JSONC format for structured management:

```jsonc
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Git Configuration Manifest",
  "description": "Comprehensive manifest of Git configuration",
  "type": "object",
  "properties": {
    "tracked_files": {
      "type": "array",
      "description": "Tracked Git configuration files"
    },
    "hooks": {
      "type": "array", 
      "description": "Git hooks configuration"
    },
    "config_sections": {
      "type": "object",
      "description": "Git configuration sections by category"
    }
  }
}
```

## Benefits

### 1. Centralized Management
- All Git configuration in one place
- Structured JSONC format for tooling
- Version-controlled configuration

### 2. Rust-Based Hooks
- Type-safe hook implementations
- Easy testing and debugging
- Consistent behavior across platforms

### 3. Git Alias Integration
- Hooks are Git aliases, making them discoverable
- Easy to test hooks manually
- Consistent with Git workflow

### 4. Validation and Safety
- Automatic validation of configuration files
- Format checking for different file types
- Error reporting and suggestions

### 5. Tooling Integration
- Works with existing Git tools
- IDE integration through Git aliases
- CI/CD pipeline integration

## Examples

### Complete Setup Example

```bash
# 1. Generate tracked config files
cargo xtask git-tracked-config generate-templates

# 2. Install Git hooks
cargo xtask git-hooks install --config git-hooks.jsonc

# 3. Convert Git config to JSONC
cargo xtask git-config convert --input .git/config --output git-config.jsonc

# 4. Convert Git attributes to JSONC
cargo xtask git-attributes convert --input .gitattributes --output git-attributes.jsonc

# 5. Validate everything
cargo xtask git-tracked-config validate
cargo xtask git-hooks list
```

### Custom Hook Example

```rust
// hooks/custom_hook.rs
use std::env;
use std::process;

fn main() {
    println!("Running custom hook...");
    
    // Get hook arguments
    let args: Vec<String> = env::args().collect();
    
    // Custom logic here
    if args.len() > 1 {
        println!("Hook arguments: {:?}", &args[1..]);
    }
    
    // Always succeed for now
    println!("✅ Custom hook completed successfully");
    process::exit(0);
}
```

## Conclusion

This Git configuration system provides:

1. **Structured Management** - All Git config in JSONC format
2. **Rust Integration** - Type-safe hooks and tooling
3. **Git Native** - Uses Git aliases and standard mechanisms
4. **Validation** - Automatic checking and error reporting
5. **Tooling** - Comprehensive CLI for all operations

The system maintains Git's flexibility while adding structure, validation, and tooling integration.
