# CLI Worktree Specification: hooksmith-worktree-rfc@v1

## 🎯 Purpose

To define a unified, automation-first, worktree-centric branching model that supports:
- Clean local development without feature branches in the main repo
- Schema-typed metadata for worktree environments
- Seamless CLI and RPC automation via xtask or spin-driven hooks
- IDE and AI agent compatibility (Cursor/Workbloom)

## 📦 Directory Layout

```
/dev/repos/
├── hooksmith/                    # root repo (worktree-base)
├── hooksmith-spin/              # worktree: feature/spin-integration
└── hooksmith-spin-integration/  # worktree: feature/spin-integration-v2
```

- All feature worktrees live in sibling folders
- Naming convention: `hooksmith-{branch-slug}`
- No feature branches live locally in `.git/refs/heads/`

## 🧠 Naming & Lifecycle Conventions

| Concept | Rule |
|---------|------|
| Base branch | `worktree-base` (renamed from main) |
| Branch pattern | `feature/*`, `bugfix/*`, `hotfix/*` |
| Worktree path | `../hooksmith-{branch-suffix}` |
| Auto naming | `cargo xtask worktree create --branch feature/foo` → `../hooksmith-foo` |
| Cleanup rule | CLI or workbloom cleanup deletes local folder + branch ref |
| Cursor Context | Enabled via `.workbloom` shell launch and env propagation |

## 🧰 Tooling Requirements

| Tool | Role |
|------|------|
| `cargo xtask` | Primary automation runner (worktree create, switch, list) |
| `git worktree` | Base Git primitive; managed via Rust wrappers |
| `workbloom` | Shell+env setup, file copying, directory cleanup |
| `Cursor` | AI integration via `.workbloom` context launcher |

## ⚙️ .workbloom Example (Schema-Aligned)

```jsonc
{
  // Setup copied files automatically
  "copy_files": [".env", ".envrc", ".direnv", "config/dev.toml"],

  // Auto run `direnv allow` or shell launcher
  "auto_shell": true,

  // Attach metadata for Cursor/AI usage
  "labels": ["spin", "integration"],
  "description": "Hooksmith Spin Integration Testbed",

  // Automatically clean up these after switching
  "remove_on_cleanup": ["tmp/", "target/", ".DS_Store"]
}
```

## 🔄 CLI Command Spec

| Command | Description |
|---------|-------------|
| `cargo xtask worktree create --branch feature/foo` | Creates `feature/foo`, adds to `../hooksmith-foo` |
| `cargo xtask worktree list --detailed` | Lists all worktrees and their directories |
| `cargo xtask worktree switch foo` | Switches shell and Cursor to `../hooksmith-foo` |
| `cargo xtask worktree remove foo` | Deletes worktree and branch safely |

## 🔌 Integration Hooks (Optional)

Future-Ready: attach these via WIT-based Spin components or CLI hooks.

| Hook | Event | Example Behavior |
|------|-------|------------------|
| `post_create` | After creating worktree | Copy `.env`, run `cargo build`, link WIT |
| `pre_remove` | Before cleanup | Run test, prompt if uncommitted changes exist |
| `post_switch` | After switching | Load `direnv`, reattach devtools |

## ✅ Invariants

- The only permanent branch is `worktree-base`
- All worktrees are sibling directories to the root repo
- No stale branches remain in `.git/refs/heads/`
- `.workbloom` or equivalent schema is present in all worktrees
- CLI and AI systems share consistent naming and context

## 📚 Future Enhancements

- Add `.worktree-config.jsonc` schema (validate branch→path mapping)
- Link xtask to spin for component validation on each worktree
- Add SLO tracking for dev/merge cycles across worktree branches
- Enable `hooksmith doctor` to audit and auto-repair worktree states

## 🧩 Summary (Reduced Form)

- **Spec Name**: `hooksmith-worktree-rfc@v1`
- **Pattern**: `worktree-base` root + `../hooksmith-{branch}` per feature
- **CLI**: `xtask` + `workbloom` to manage lifecycle
- **Goal**: Schema-typed, AI-compatible, automation-first worktree system

## 📋 Implementation Status

### ✅ Completed
- [x] Basic worktree CLI commands in `cargo xtask`
- [x] Workbloom integration and configuration
- [x] Git worktree fallback support
- [x] Configuration file management (`.worktree-config.jsonc`)
- [x] Tool detection and selection logic

### 🔄 In Progress
- [ ] Enhanced CLI commands per specification
- [ ] Schema validation for `.workbloom` files
- [ ] Cursor integration improvements
- [ ] Worktree metadata tracking

### 📋 Planned
- [ ] Spin component integration
- [ ] RPC hooks system
- [ ] WASM runtime triggers
- [ ] Workbloom schema integration
- [ ] Automated cleanup and maintenance

## 🔧 Configuration Schema

### .worktree-config.jsonc

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
      "setup": ["cargo build", "cargo xtask gen-all"],
      "labels": ["feature", "development"]
    },
    "bugfix/*": {
      "template": "{repo}-{branch}",
      "setup": ["cargo build"],
      "labels": ["bugfix", "maintenance"]
    },
    "hotfix/*": {
      "template": "{repo}-{branch}",
      "setup": ["cargo build"],
      "labels": ["hotfix", "urgent"]
    }
  },
  "integration": {
    "lefthook": true,
    "xtask": true,
    "wasm_components": true
  },
  "cursor_integration": {
    "auto_open_cursor": true,
    "cursor_config_template": "worktree_integration",
    "shell_aliases": {
      "cbloom": "wb bloom $1 && cursor ./$1",
      "cswitch": "wb switch $1 && cursor ."
    },
    "env_vars": {
      "WORKTREE_MANAGER": "workbloom",
      "CURSOR_INTEGRATION": "enabled"
    }
  },
  "workbloom_metadata": {
    "enabled": true,
    "metadata_dir": ".wb",
    "labels_config": "labels.toml",
    "status_tracking": true
  }
}
```

### .workbloom

```bash
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

# Cursor integration
.cursor/
```

## 🚀 Usage Examples

### Basic Workflow

```bash
# Create a new feature worktree
cargo xtask worktree create --branch feature/new-feature --switch

# List all worktrees with details
cargo xtask worktree list --detailed

# Switch to existing worktree
cargo xtask worktree switch spin-integration

# Remove worktree when done
cargo xtask worktree remove new-feature --with-branch
```

### Advanced Workflow with Workbloom

```bash
# Create with automatic setup and environment sync
cargo xtask worktree create --branch feature/spin-integration --setup --copy-env

# Switch with Cursor integration
cargo xtask worktree switch spin-integration

# Direct workbloom commands for advanced features
workbloom setup feature/test --no-shell
workbloom list
workbloom cleanup --status
```

### CI/CD Integration

```bash
# Setup worktree management in CI
cargo xtask worktree setup --all

# List worktrees in JSON format for automation
cargo xtask worktree list --format json

# Validate worktree configuration
cargo xtask worktree status --detailed
```

## 🔍 Troubleshooting

### Common Issues

1. **Tool Not Found**: Run `cargo xtask worktree setup --install-tools`
2. **Configuration Issues**: Run `cargo xtask worktree setup --config`
3. **Permission Issues**: Check Git aliases with `git config --global --list | grep alias`
4. **Workbloom Issues**: Verify with `workbloom list` and `workbloom cleanup --status`

### Debug Commands

```bash
# Check tool availability
cargo xtask worktree status --detailed

# Validate configuration
cargo xtask worktree setup --config

# Test worktree creation
cargo xtask worktree create --branch test/debug --setup
```

## 📖 Related Documentation

- [Worktree Setup Guide](./WORKTREE_SETUP.md)
- [CLI Help Documentation](./CLI_HELP.md)
- [Lefthook Integration](./GIT_LEFTHOOK_INTEGRATION.md)
- [WASM Components Guide](./COMPONENT_RUNNER_GUIDE.md)
- [Project Architecture](./STRUCTURE.md) 