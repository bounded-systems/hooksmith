# Hooksmith

CLI tools for building Rust binaries into Lefthook hooks with WASM components.

## Overview

This CLI provides tools for:
- **Hook Building**: Build Rust binaries for Git hooks
- **WASM Components**: Build and manage WASM components from WIT interfaces
- **Lefthook Integration**: Generate and install Lefthook configurations
- **Hook Management**: List and manage available hooks

## Architecture

This is a Rust workspace with the following structure:

```
hooksmith/
├── Cargo.toml               # Workspace manifest
├── build.sh                 # Build script
├── README.md                # This file
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command implementations
│   └── modules/             # Core functionality
├── components/              # Modular components
│   └── cli-core/            # Core CLI functionality
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

### Basic Commands

```bash
# Test the CLI
hooksmith test

# Test with custom message
hooksmith test --message "Custom test message"

# List available hooks
hooksmith list
```

### Hook Building

```bash
# Build a hook binary
hooksmith build my-hook

# Build with custom output directory
hooksmith build my-hook --output target/custom-hooks
```

### Lefthook Integration

```bash
# Generate Lefthook configuration
hooksmith generate

# Generate with custom output file
hooksmith generate --output custom-lefthook.yml

# Install hooks
hooksmith install

# Install specific hooks
hooksmith install --hooks pre-commit,pre-push
```

### WASM Component Management

```bash
# Build WASM component from WIT
hooksmith wasm build interface.wit

# Build with custom output
hooksmith wasm build interface.wit --output target/wasm

# Run WASM component
hooksmith wasm run component.wasm --function validate --args arg1 arg2

# Generate bindings from WIT
hooksmith wasm bindings interface.wit

# Generate bindings with custom output
hooksmith wasm bindings interface.wit --output target/bindings
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
cargo run -- test
```

## Components

- **cli-core**: Core CLI functionality and utilities
- **hooksmith**: Main CLI binary for hook building and WASM management

## Integration

This CLI is designed to integrate with Lefthook for Git hook management:

```bash
# Generate Lefthook config
hooksmith generate > lefthook.yml

# Install hooks
hooksmith install
```

## License

MIT
