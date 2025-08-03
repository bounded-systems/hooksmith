# Hooksmith

A CLI tool for building Rust binaries into Lefthook hooks with WASM components.

## Features

- 🔧 **Structured Code Generation**: WIT interfaces generated from Rust structs
- 🚀 **WASM Integration**: Build and manage WASM components for Git hooks
- 📝 **Lefthook Integration**: Generate and validate Lefthook configurations
- 🛠️ **Xtask Workflow**: Rust-based build system replacing shell scripts

## Installation

```bash
# Build from source
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Usage

```bash
# Get help
./target/release/hooksmith --help

# Test the CLI
./target/release/hooksmith test

# Generate WIT interfaces
cd xtask && cargo run -- gen-wit

# Generate Lefthook configuration
cd xtask && cargo run -- gen-lefthook

# Run all code generation
cd xtask && cargo run -- gen-all
```

## CLI Commands

```bash
# Main commands
hooksmith test                    # Test command to verify CLI functionality
hooksmith build                   # Build Rust binaries for Git hooks
hooksmith generate                # Generate Lefthook configuration
hooksmith generate-comprehensive  # Generate comprehensive Lefthook configuration
hooksmith generate-code           # Generate structured code and documentation
hooksmith install                 # Install hooks into Git repository
hooksmith list                    # List available hooks
hooksmith validate                # Validate Lefthook configuration
hooksmith verify-hooks            # Verify Hooksmith hooks registration
hooksmith wasm                    # WASM component management
hooksmith worktree                # Worktree management
hooksmith contract                # Contract validation with JSON Schema and Git notes
```

## Development

### Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Git**: Latest version
- **Lefthook**: For pre-commit hooks (optional but recommended)

### Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/bdelanghe/hooksmith.git
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
   # Build the project
   cargo build

   # Generate all code and documentation
   cd xtask && cargo run -- gen-all
   ```

5. **Run tests**
   ```bash
   # Run library tests (some examples may have compilation issues)
   cargo test --lib
   ```

### Xtask Commands

This project uses **xtask** for structured code generation and build tasks, replacing shell scripts and raw echo statements:

```bash
# Navigate to xtask directory first
cd xtask

# Build the project and all components
cargo run -- build --target all --release

# Generate WIT interface definitions
cargo run -- gen-wit --overwrite

# Generate Lefthook configuration
cargo run -- gen-lefthook --validate

# Generate documentation
cargo run -- gen-docs --open

# Generate README with CLI help
cargo run -- gen-readme --overwrite

# Generate mod.rs files
cargo run -- gen-mods --overwrite

# Run all code generation tasks
cargo run -- gen-all --overwrite

# Check if generated files are up to date
cargo run -- check --strict

# Validate project configuration
cargo run -- validate --all
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
├── README.md                # This file
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command modules
│   │   ├── mod.rs           # Auto-generated mod.rs
│   │   └── contract_validation.rs
│   ├── modules/             # Core modules
│   │   ├── mod.rs           # Auto-generated mod.rs
│   │   ├── contract_validation.rs
│   │   ├── git_model.rs
│   │   ├── wasm.rs
│   │   ├── contract_state_machine.rs
│   │   ├── hierarchical_validation.rs
│   │   ├── generator.rs
│   │   ├── hook_builder.rs
│   │   └── lefthook.rs
│   └── orchestrator/        # Orchestration layer
├── components/              # WASM components
│   ├── cli-core/            # Core CLI functionality
│   ├── git-filter/          # Git filtering components
│   ├── hook-builder/        # Hook building components
│   └── worktree-runner/     # Worktree management WASM component
├── xtask/                   # Build system and code generation
│   ├── Cargo.toml
│   ├── src/
│   └── README.md
├── wit/                     # WIT interface definitions
├── hooks/                   # Hook scripts directory
├── tests/                   # Test files
├── examples/                # Example code
├── docs/                    # Documentation
└── target/                  # Build artifacts
```

## Components

- **hooksmith**: Main CLI binary for hook building and WASM management
- **cli-core**: Core CLI functionality and utilities
- **git-filter**: Git filtering and validation components
- **hook-builder**: Hook building and compilation components
- **worktree-runner**: WASM component for worktree management

## Integration

This CLI is designed to integrate with Lefthook for Git hook management:

```bash
# Generate Lefthook config
./target/release/hooksmith generate > lefthook.yml

# Install hooks
./target/release/hooksmith install
```

## Documentation

- **API Documentation**: `cargo doc --no-deps --open`
- **CLI Help**: `./target/release/hooksmith --help`
- **Command Help**: `./target/release/hooksmith <command> --help`

## Testing

```bash
# Run library tests
cargo test --lib

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
| Tests | ⚠️ Partial | Library tests pass, some examples need fixes |
| Build System | ✅ Complete | Xtask-based workflow |
| WASM Compilation | ✅ Complete | WASM toolchain integration |
| WIT Processing | ✅ Complete | WIT parser and compiler |
| Lefthook Integration | ✅ Complete | YAML generation and hook installation |
| Hook Building | ✅ Complete | Rust compilation pipeline |

## Known Issues

- Some example files have compilation errors due to API changes
- Tests need to be updated to match current API signatures
- Xtask commands must be run from the `xtask/` directory

## License

MIT License - see LICENSE file for details.

---

*This README is auto-generated using `cargo xtask gen-readme`. The CLI help section is automatically updated from the actual CLI output.*
