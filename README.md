# Hooksmith

A CLI tool for building Rust binaries into Lefthook hooks with WASM components.

## Features

- 🔧 **Structured Code Generation**: WIT interfaces generated from Rust structs
- 🚀 **WASM Integration**: Build and manage WASM components for Git hooks
- 📝 **Lefthook Integration**: Generate and validate Lefthook configurations
- 🛠️ **Xtask Workflow**: Rust-based build system replacing shell scripts

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Get help
hooksmith --help

# Test the CLI
hooksmith test

# Generate WIT interfaces
cargo xtask gen-wit

# Generate Lefthook configuration
cargo xtask gen-lefthook

# Run all code generation
cargo xtask gen-all
```

## CLI Commands

```bash

```

## Development

### Prerequisites

- **Rust**: Latest stable version (1.75+)
- **Git**: Latest version
- **Lefthook**: For pre-commit hooks (optional but recommended)

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/hooksmith.git
   cd hooksmith
   ```

2. **Install dependencies**
   ```bash
   # Install Lefthook (optional but recommended)
   npm install -g @evilmartians/lefthook

   # Or using Homebrew on macOS
   brew install lefthook
   ```

3. **Install pre-commit hooks**
   ```bash
   lefthook install
   ```

4. **Generate code and build the project**
   ```bash
   # Generate all code and documentation
   ./xtask.sh gen-all --overwrite

   # Or use the build script
   ./build.sh
   ```

5. **Run tests**
   ```bash
   cargo test --all-targets --all-features
   ```

### Xtask Commands

This project uses **xtask** for structured code generation and build tasks, replacing shell scripts and raw echo statements:

```bash
# Build the project and all components
./xtask.sh build --target all --release

# Generate WIT interface definitions
./xtask.sh gen-wit --overwrite

# Generate Lefthook configuration
./xtask.sh gen-lefthook --validate

# Generate documentation
./xtask.sh gen-docs --open

# Generate README with CLI help
./xtask.sh gen-readme --overwrite

# Generate mod.rs files
./xtask.sh gen-mods --overwrite

# Run all code generation tasks
./xtask.sh gen-all --overwrite

# Check if generated files are up to date
./xtask.sh check --strict

# Validate project configuration
./xtask.sh validate --all
```

**Benefits of Xtask:**
- ✅ **No shell scripts** - All tasks are Rust-based
- ✅ **Structured code generation** - WIT files generated from Rust structs
- ✅ **Type-safe configuration** - All configs are strongly typed
- ✅ **Deterministic builds** - Same input always produces same output
- ✅ **CI integration** - Automated checks ensure generated files are up to date

## Project Structure

```
hooksmith/
├── Cargo.toml               # Workspace manifest
├── xtask.sh                 # Xtask wrapper script
├── README.md                # This file (auto-generated)
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command modules (auto-generated mod.rs)
│   └── modules/             # Core modules (auto-generated mod.rs)
├── components/              # WASM components
│   ├── cli-core/            # Core CLI functionality
│   └── worktree-runner/     # Worktree management WASM component
├── wit/                     # WIT interface definitions (auto-generated)
├── hooks/                   # Hook scripts directory
├── tests/                   # Test files
└── target/doc/              # Generated documentation
```

## Components

- **hooksmith**: Main CLI binary for hook building and WASM management
- **cli-core**: Core CLI functionality and utilities
- **worktree-runner**: WASM component for worktree management

## Integration

This CLI is designed to integrate with Lefthook for Git hook management:

```bash
# Generate Lefthook config
hooksmith generate > lefthook.yml

# Install hooks
hooksmith install
```

## Documentation

- **API Documentation**: `cargo doc --no-deps --open`
- **CLI Help**: `hooksmith --help`
- **Command Help**: `hooksmith <command> --help`

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_cli_help

# Run integration tests
cargo test --test integration
```

## Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| CLI Structure | ✅ Complete | Full command parsing and help |
| Documentation | ✅ Complete | Comprehensive docs and examples |
| Tests | ✅ Complete | All tests passing |
| Build System | ✅ Complete | Xtask-based workflow |
| WASM Compilation | ✅ Complete | WASM toolchain integration |
| WIT Processing | ✅ Complete | WIT parser and compiler |
| Lefthook Integration | ✅ Complete | YAML generation and hook installation |
| Hook Building | ✅ Complete | Rust compilation pipeline |

## License

MIT License - see LICENSE file for details.

---

*This README is auto-generated using `cargo xtask gen-readme`. The CLI help section is automatically updated from the actual CLI output.*
