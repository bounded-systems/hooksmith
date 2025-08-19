# Hooksmith Architecture

## Overview

Hooksmith is a modern, Rust-based Git hook system that provides comprehensive validation, performance optimization, and extensibility through a unified architecture.

## Directory Structure

```
.hooksmith/
└── hooks/
    ├── git/                    # Git client-side hooks
    │   ├── pre-commit         # Pre-commit validation
    │   ├── post-commit        # Post-commit actions
    │   ├── pre-push          # Pre-push validation
    │   ├── prepare-commit-msg # Commit message preparation
    │   ├── commit-msg         # Commit message validation
    │   ├── fsmonitor-watchman # File system monitoring
    │   └── ... (all other Git hooks)
    │
    └── github/                # GitHub Actions event handlers
        ├── github-push        # Push event handler
        ├── github-pull-request # Pull request handler
        ├── github-issues      # Issues handler
        ├── github-release     # Release handler
        ├── github-create      # Create event handler
        └── github-delete      # Delete event handler
```

## Core Components

### 1. Git Hooks (`.hooksmith/hooks/git/`)

All Git client-side hooks are pure Rust binaries that:

- **No shell scripts**: Direct binary execution for maximum performance
- **Contract-driven**: Based on concerns and contract objects
- **Worktree-safe**: Proper path resolution for worktree environments
- **Extensible**: Easy to add new validation logic

#### Available Hooks

| Hook | Purpose | Status |
|------|---------|--------|
| `pre-commit` | Validate staged changes | ✅ Stub |
| `post-index-change` | Handle index changes | ✅ Stub |
| `commit-msg` | Validate commit messages | ✅ Stub |
| `prepare-commit-msg` | Prepare commit messages | ✅ Stub |
| `post-commit` | Post-commit actions | ✅ Stub |
| `pre-push` | Pre-push validation | ✅ Stub |
| `post-merge` | Post-merge actions | ✅ Stub |
| `pre-rebase` | Pre-rebase validation | ✅ Stub |
| `post-rebase` | Post-rebase actions | ✅ Stub |
| `pre-applypatch` | Pre-applypatch validation | ✅ Stub |
| `applypatch-msg` | Apply patch message | ✅ Stub |
| `post-applypatch` | Post-applypatch actions | ✅ Stub |
| `post-checkout` | Post-checkout actions | ✅ Stub |
| `post-rewrite` | Post-rewrite actions | ✅ Stub |
| `reference-transaction` | Reference validation | ✅ Stub |
| `sendemail-validate` | Email validation | ✅ Stub |
| `pre-merge-commit` | Pre-merge validation | ✅ Stub |
| `fsmonitor-watchman` | File system monitoring | ✅ FSMonitor |

### 2. GitHub Actions Hooks (`.hooksmith/hooks/github/`)

GitHub event handlers that integrate with the unified `hooksmith.yml` workflow:

| Event | Handler | Purpose |
|-------|---------|---------|
| `push` | `github-push` | Push event validation |
| `pull_request` | `github-pull-request` | PR validation |
| `issues` | `github-issues` | Issue event handling |
| `release` | `github-release` | Release validation |
| `create` | `github-create` | Create event handling |
| `delete` | `github-delete` | Delete event handling |

## FSMonitor Integration

### Modern File System Monitoring

Hooksmith includes a comprehensive FSMonitor system that provides:

1. **Built-in Git FSMonitor**: Automatically enables Git's built-in daemon (Git 2.37.0+)
2. **Rust-based Alternative**: Custom implementation when built-in isn't available
3. **Performance Optimization**: Significant speed improvements for `git status` and other operations

### Setup

```bash
# Run the FSMonitor setup
cargo run --bin setup-fsmonitor
```

### Features

- **Automatic Detection**: Detects Git version and enables appropriate FSMonitor
- **Performance Monitoring**: Includes performance test scripts
- **Validation Integration**: Can integrate with Hooksmith's validation system
- **Cross-Platform**: Works on macOS, Linux, and Windows

## Configuration

### Git Configuration

```bash
# Set custom hooks directory
git config core.hooksPath .hooksmith/hooks/git

# Enable FSMonitor (automatically done by setup-fsmonitor)
git config core.fsmonitor true
```

### GitHub Actions Configuration

The `hooksmith.yml` workflow is the single source of truth for all GitHub Actions events:

```yaml
name: Hooksmith Validation
on:
  push:
  pull_request:
  issues:
  release:
  create:
  delete:
  schedule:
    - cron: '0 0 * * *'  # Daily validation

jobs:
  github_push:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run push validation
        run: cargo run --bin github-push
```

## Performance Optimizations

### 1. Parallel Processing

The `safe-worktree-cleanup` binary uses parallel processing for worktree operations:

```rust
use rayon::prelude::*;

worktrees.par_iter().for_each(|worktree_path| {
    // Parallel-safe worktree processing
});
```

### 2. FSMonitor Acceleration

- **Built-in Daemon**: Uses Git's native FSMonitor when available
- **Rust Implementation**: Custom file watcher for older Git versions
- **Performance Monitoring**: Built-in performance testing

### 3. Efficient Git Operations

- **`--porcelain`**: Uses structured output for parsing
- **`--untracked-files=no`**: Skips untracked file scanning when possible
- **Short-circuiting**: Early exit for empty results

## Contract-Driven Architecture

### GitHookSurface Model

Hooksmith uses a unified model that maps all Git extension points:

```rust
pub enum GitHookSurface {
    Lifecycle(LifecycleHookType),
    Lfs(LfsHookType),
    AttributeFilter(AttributeFilterType),
    Config(ConfigHookType),
    Notes(NotesHookType),
    CustomRef(CustomRefType),
    Worktree(WorktreeHookType),
    ExternalTool(ExternalToolType),
    RuntimeProxy(RuntimeProxyType),
}
```

### Hook Categories

1. **Lifecycle Hooks**: Standard Git hooks (pre-commit, post-commit, etc.)
2. **LFS Hooks**: Git Large File Storage integration
3. **Attribute Filters**: `.gitattributes` clean/smudge filters
4. **Config Hooks**: `includeIf` and dynamic configuration
5. **Notes Hooks**: Git notes for metadata
6. **Custom Refs**: Custom reference management
7. **Worktree Hooks**: Worktree lifecycle management
8. **External Tools**: Integration with external tools
9. **Runtime Proxies**: Custom remote/proxy implementations

## Development Workflow

### Adding New Hooks

1. **Create Binary**: Add new binary in `src/bin/`
2. **Update Hooks Directory**: Copy to `.hooksmith/hooks/git/`
3. **Test**: Verify hook execution
4. **Document**: Update this documentation

### Adding GitHub Event Handlers

1. **Create Handler**: Add new binary in `src/bin/github-*.rs`
2. **Update Workflow**: Add event to `hooksmith.yml`
3. **Copy Binary**: Copy to `.hooksmith/hooks/github/`
4. **Test**: Verify with `act`

### Performance Testing

```bash
# Run performance test
.hooksmith/performance-test.sh

# Test specific binary
cargo run --bin safe-worktree-cleanup

# Test GitHub workflow
act push --dryrun
```

## Benefits

### 1. Performance
- **Pure Rust**: No shell script overhead
- **Parallel Processing**: Multi-threaded operations
- **FSMonitor**: Accelerated file system operations
- **Caching**: Intelligent result caching

### 2. Maintainability
- **Single Source**: Unified configuration
- **Type Safety**: Rust's type system
- **Modular Design**: Clear separation of concerns
- **Documentation**: Comprehensive docs

### 3. Extensibility
- **Contract-Driven**: Based on concerns and objects
- **Plugin Architecture**: Easy to add new hooks
- **Cross-Platform**: Works everywhere
- **Version Control**: All hooks tracked in Git

### 4. Developer Experience
- **Fast Setup**: One-command installation
- **Clear Structure**: Organized directory layout
- **Performance Monitoring**: Built-in testing
- **Error Handling**: Comprehensive error messages

## Future Enhancements

### Planned Features

1. **WASM Integration**: WebAssembly components for hooks
2. **Plugin System**: Dynamic hook loading
3. **Advanced FSMonitor**: Real-time file watching
4. **Contract Validation**: Schema-based validation
5. **Performance Analytics**: Detailed performance metrics

### Integration Opportunities

1. **Git LFS**: Enhanced large file handling
2. **Git Notes**: Metadata management
3. **Custom Remotes**: Proxy implementations
4. **External Tools**: Third-party integrations

## Conclusion

Hooksmith provides a modern, performant, and extensible Git hook system that leverages Rust's strengths while maintaining compatibility with Git's ecosystem. The formalized directory structure, FSMonitor integration, and contract-driven architecture make it an ideal foundation for sophisticated Git workflows.
