# Pushd Worktree CLI

CLI tools for Git worktree management and safety.

## Overview

This CLI provides tools for:
- **Worktree Management**: Create, list, remove, and check worktrees
- **Safety Hooks**: Generate and install Git hooks for worktree safety
- **Status Monitoring**: Check worktree status and safety

## Architecture

This is a Rust workspace with the following structure:

```
pushd-worktree-cli/
├── Cargo.toml               # Workspace manifest
├── build.sh                 # Build script
├── README.md                # This file
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command implementations
│   └── modules/             # Core functionality
├── components/              # Modular components
│   ├── cli-core/            # Core CLI functionality
│   ├── worktree-manager/    # Git worktree operations
│   └── git-validator/       # Git validation tools
├── hooks/                   # Hook scripts
├── docs/                    # Documentation
└── tests/                   # Test files
```

## Installation

```bash
# Build from source
./build.sh

# Install globally (optional)
cargo install --path .
```

## Usage

### Worktree Management

```bash
# Create a new worktree
pushd-worktree-cli worktree create my-feature

# Create from specific branch
pushd-worktree-cli worktree create my-feature --branch develop

# List all worktrees
pushd-worktree-cli worktree list

# Remove a worktree
pushd-worktree-cli worktree remove my-feature

# Check worktree status
pushd-worktree-cli worktree check
pushd-worktree-cli worktree check my-feature
```

### Hook Management

```bash
# Generate worktree safety hooks
pushd-worktree-cli hooks generate

# Install hooks
pushd-worktree-cli hooks install

# Run a specific hook
pushd-worktree-cli hooks run pre-commit

# Check hook status
pushd-worktree-cli hooks status
```

### Status and Safety

```bash
# Check worktree status
pushd-worktree-cli status

# Run safety checks
pushd-worktree-cli status --safety
```

## Development

```bash
# Build workspace
./build.sh

# Build specific component
cargo build --package cli-core

# Test workspace
cargo test --workspace

# Run CLI
cargo run --package pushd-worktree-cli -- test
```

## Components

- **cli-core**: Core CLI functionality and utilities
- **worktree-manager**: Git worktree creation, management, and safety
- **git-validator**: Git validation and safety checks

## Integration

This CLI is designed to be used as a Git submodule in the main Pushd repository:

```bash
git submodule add git@github.com:bdelanghe/pushd-worktree-cli.git .cli-helper
```

## License

MIT 
