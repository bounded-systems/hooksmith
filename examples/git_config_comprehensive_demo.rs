//! Comprehensive Git Configuration Demo
//! 
//! This example demonstrates the complete Git configuration system including:
//! - Tracked Git config files (.gitattributes, .gitignore, .gitmodules, .mailmap)
//! - Git hooks with Rust binaries and Git aliases
//! - Git configuration management and conversion to JSONC
//! - includeIf conditional configuration

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Demo function to show the complete Git configuration system
fn run_comprehensive_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Comprehensive Git Configuration Demo");
    println!("=====================================");
    
    // 1. Tracked Git Configuration Files
    println!("\n📁 1. Tracked Git Configuration Files");
    println!("=====================================");
    
    let tracked_configs = vec![
        (".gitattributes", "File behavior: diff, merge, binary, eol", "line-based"),
        (".gitignore", "Ignore untracked files", "glob"),
        (".gitmodules", "Submodule configuration", "ini"),
        (".mailmap", "Canonical author mapping", "line-based"),
    ];
    
    for (file, purpose, format) in tracked_configs {
        println!("• {}: {} ({})", file, purpose, format);
    }
    
    // 2. Git Hooks with Rust Binaries
    println!("\n🔗 2. Git Hooks with Rust Binaries");
    println!("===================================");
    
    let hooks = vec![
        ("pre-commit", "Run linting and tests before commit"),
        ("commit-msg", "Validate commit message format"),
        ("pre-push", "Run security checks before push"),
        ("post-merge", "Update dependencies after merge"),
    ];
    
    for (hook, description) in hooks {
        println!("• {}: {}", hook, description);
        println!("  Binary: hook-{}", hook.replace('-', "_"));
        println!("  Alias: hook-{}", hook);
        println!("  Stub: exec git hook-{}\"$@\"", hook);
        println!();
    }
    
    // 3. Git Configuration Categories
    println!("\n⚙️ 3. Git Configuration Categories");
    println!("=================================");
    
    let config_categories = vec![
        ("Identity", "User identity and commit behavior", vec!["user", "commit", "gpg"]),
        ("Remote", "Remotes, branches, and syncing", vec!["remote", "branch", "push", "pull"]),
        ("Behavior", "Behavior customization and safety", vec!["core", "merge", "rebase", "diff"]),
        ("Alias", "Aliases and custom commands", vec!["alias"]),
        ("Tooling", "Tooling integration and custom sections", vec!["vscode", "github", "xtask"]),
    ];
    
    for (category, description, sections) in config_categories {
        println!("• {}: {}", category, description);
        for section in sections {
            println!("  - [{}]", section);
        }
        println!();
    }
    
    // 4. includeIf Conditional Configuration
    println!("\n🔀 4. includeIf Conditional Configuration");
    println!("=======================================");
    
    let include_if_examples = vec![
        ("gitdir:~/work/", "Work repositories", "~/.gitconfig-work"),
        ("gitdir:~/personal/", "Personal repositories", "~/.gitconfig-personal"),
        ("gitdir:~/repos/**/", "All repositories in repos/", "~/.gitconfig-default"),
    ];
    
    for (condition, description, path) in include_if_examples {
        println!("• {}: {} → {}", condition, description, path);
    }
    
    // 5. Generate Example Files
    println!("\n📝 5. Generating Example Files");
    println!("=============================");
    
    generate_example_files()?;
    
    // 6. Show JSONC Manifest
    println!("\n📋 6. JSONC Manifest Example");
    println!("============================");
    
    let manifest = generate_jsonc_manifest();
    println!("{}", manifest);
    
    println!("\n✅ Comprehensive demo completed successfully!");
    println!("\nTo use this system:");
    println!("1. cargo xtask git-tracked-config scan");
    println!("2. cargo xtask git-hooks install");
    println!("3. cargo xtask git-config convert");
    println!("4. cargo xtask git-attributes convert");
    
    Ok(())
}

/// Generate example configuration files
fn generate_example_files() -> Result<(), Box<dyn std::error::Error>> {
    // Create examples directory
    fs::create_dir_all("examples/git-config")?;
    
    // 1. .gitattributes example
    let gitattributes = r#"# Git Attributes Example
# 
# This file controls how Git handles different file types.

# Text files with specific line endings
*.sh text eol=lf
*.bat text eol=crlf
*.md text eol=lf

# Binary files
*.jpg binary -diff -merge
*.png binary -diff -merge
*.zip binary -diff -merge

# Custom diff drivers
*.json diff=json
*.yaml diff=yaml
*.toml diff=toml

# Merge strategies
*.lock merge=union
config/*.conf merge=ours

# Export filtering
secret.key export-ignore
.env.local export-ignore

# Linguist overrides (GitHub)
*.ts linguist-language=TypeScript
*.jsx linguist-language=JavaScript
docs/** linguist-documentation
"#;
    fs::write("examples/git-config/.gitattributes", gitattributes)?;
    
    // 2. .gitignore example
    let gitignore = r#"# Git Ignore Example
# 
# This file specifies files that should be ignored by Git.

# Build artifacts
target/
dist/
build/
*.o
*.so
*.dll

# Dependencies
node_modules/
vendor/
Cargo.lock

# IDE files
.vscode/
.idea/
*.swp
*.swo
*~

# OS files
.DS_Store
Thumbs.db
desktop.ini

# Logs
*.log
logs/
*.log.*

# Environment files
.env
.env.local
.env.production
.env.staging

# Temporary files
*.tmp
*.temp
temp/
tmp/

# Generated files
*.generated.*
generated/
"#;
    fs::write("examples/git-config/.gitignore", gitignore)?;
    
    // 3. .gitmodules example
    let gitmodules = r#"# Git Submodules Example
# 
# This file declares submodules for this repository.
# Currently, no submodules are used.
# 
# Example submodule declaration:
# [submodule "vendor/libfoo"]
#   path = vendor/libfoo
#   url = https://github.com/foo/libfoo.git
#   branch = main
# 
# [submodule "docs/examples"]
#   path = docs/examples
#   url = https://github.com/company/examples.git
#   branch = master
"#;
    fs::write("examples/git-config/.gitmodules", gitmodules)?;
    
    // 4. .mailmap example
    let mailmap = r#"# Mailmap Example
# 
# This file maps author names and emails to canonical forms.
# Format: <canonical-email> <canonical-name> <alias-email> <alias-name>
# 
# Examples:
# john.doe@company.com John Doe john.doe@gmail.com John Doe
# jane.smith@company.com Jane Smith jane@example.com Jane
# bob.wilson@company.com Bob Wilson bob.wilson@old-company.com Bob Wilson
# 
# Multiple aliases for same person:
# alice@company.com Alice Johnson alice.johnson@gmail.com Alice Johnson
# alice@company.com Alice Johnson alice@personal.com Alice
"#;
    fs::write("examples/git-config/.mailmap", mailmap)?;
    
    // 5. Git config example
    let git_config = r#"[user]
	name = Robert DeLanghe
	email = bobbit@example.com

[core]
	editor = nvim
	filemode = true
	autocrlf = input
	ignorecase = true

[remote "origin"]
	url = git@github.com:user/repo.git
	fetch = +refs/heads/*:refs/remotes/origin/*

[branch "main"]
	remote = origin
	merge = refs/heads/main

[alias]
	co = checkout
	st = status
	ci = commit
	l = log --oneline --graph --decorate
	amend = commit --amend --no-edit
	safe-push = !./scripts/safe-push.sh
	safe-commit = !./scripts/safe-commit.sh

[pull]
	rebase = true

[diff "contract_diff"]
	textconv = /path/to/xtask contract-validate diff
	cachetextconv = true

[vscode]
	merge-base = origin/main

[github]
	pr-number = 42
"#;
    fs::write("examples/git-config/.git-config", git_config)?;
    
    // 6. includeIf example
    let include_if = r#"# Global Git Config with includeIf
# 
# This file shows how to use includeIf for conditional configuration.

[user]
	name = Robert DeLanghe
	email = bobbit@personal.com

[core]
	editor = nvim
	filemode = true

[alias]
	co = checkout
	st = status
	ci = commit

# Work repositories
[includeIf "gitdir:~/work/"]
	path = ~/.gitconfig-work

# Personal repositories  
[includeIf "gitdir:~/personal/"]
	path = ~/.gitconfig-personal

# All repositories in repos/
[includeIf "gitdir:~/repos/**/"]
	path = ~/.gitconfig-default
"#;
    fs::write("examples/git-config/.gitconfig-global", include_if)?;
    
    println!("✅ Example files generated in examples/git-config/");
    Ok(())
}

/// Generate JSONC manifest example
fn generate_jsonc_manifest() -> String {
    r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Git Configuration Manifest",
  "description": "Comprehensive manifest of Git configuration files and settings",
  "type": "object",
  "properties": {
    "tracked_files": {
      "type": "array",
      "description": "Tracked Git configuration files",
      "items": {
        "type": "object",
        "properties": {
          "file": {
            "type": "string",
            "description": "File name"
          },
          "purpose": {
            "type": "string", 
            "description": "Purpose/description"
          },
          "format": {
            "type": "string",
            "description": "File format"
          },
          "optional": {
            "type": "boolean",
            "description": "Whether file is optional"
          }
        }
      }
    },
    "hooks": {
      "type": "array",
      "description": "Git hooks configuration",
      "items": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "description": "Hook name"
          },
          "binary": {
            "type": "string",
            "description": "Rust binary name"
          },
          "alias": {
            "type": "string",
            "description": "Git alias name"
          },
          "enabled": {
            "type": "boolean",
            "description": "Whether hook is enabled"
          }
        }
      }
    },
    "config_sections": {
      "type": "object",
      "description": "Git configuration sections by category",
      "properties": {
        "identity": {
          "type": "array",
          "description": "User identity and commit behavior",
          "items": {
            "type": "string"
          }
        },
        "remote": {
          "type": "array", 
          "description": "Remotes, branches, and syncing",
          "items": {
            "type": "string"
          }
        },
        "behavior": {
          "type": "array",
          "description": "Behavior customization and safety", 
          "items": {
            "type": "string"
          }
        },
        "alias": {
          "type": "array",
          "description": "Aliases and custom commands",
          "items": {
            "type": "string"
          }
        },
        "tooling": {
          "type": "array",
          "description": "Tooling integration and custom sections",
          "items": {
            "type": "string"
          }
        }
      }
    },
    "include_if": {
      "type": "array",
      "description": "Conditional configuration includes",
      "items": {
        "type": "object",
        "properties": {
          "condition": {
            "type": "string",
            "description": "includeIf condition"
          },
          "path": {
            "type": "string",
            "description": "Path to included config"
          },
          "description": {
            "type": "string",
            "description": "Purpose of this include"
          }
        }
      }
    }
  }
}

// Example manifest data:
{
  "tracked_files": [
    {
      "file": ".gitattributes",
      "purpose": "File behavior: diff, merge, binary, eol",
      "format": "line-based",
      "optional": false
    },
    {
      "file": ".gitignore", 
      "purpose": "Ignore untracked files",
      "format": "glob",
      "optional": false
    },
    {
      "file": ".gitmodules",
      "purpose": "Submodule configuration", 
      "format": "ini",
      "optional": true
    },
    {
      "file": ".mailmap",
      "purpose": "Canonical author mapping",
      "format": "line-based", 
      "optional": true
    }
  ],
  "hooks": [
    {
      "name": "pre-commit",
      "binary": "hook-pre_commit",
      "alias": "hook-pre-commit",
      "enabled": true
    },
    {
      "name": "commit-msg",
      "binary": "hook-commit_msg", 
      "alias": "hook-commit-msg",
      "enabled": true
    },
    {
      "name": "pre-push",
      "binary": "hook-pre_push",
      "alias": "hook-pre-push", 
      "enabled": true
    }
  ],
  "config_sections": {
    "identity": ["user", "commit", "gpg"],
    "remote": ["remote", "branch", "push", "pull"],
    "behavior": ["core", "merge", "rebase", "diff"],
    "alias": ["alias"],
    "tooling": ["vscode", "github", "xtask"]
  },
  "include_if": [
    {
      "condition": "gitdir:~/work/",
      "path": "~/.gitconfig-work",
      "description": "Work repositories"
    },
    {
      "condition": "gitdir:~/personal/",
      "path": "~/.gitconfig-personal", 
      "description": "Personal repositories"
    }
  ]
}"#.to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_comprehensive_demo()
}
