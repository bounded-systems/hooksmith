# Development Guide

This guide provides comprehensive information for developers working on Hooksmith, including safe development practices to avoid breaking changes.

## 🎯 Development Approaches

When developing Hooksmith in its own repository, you have several approaches to avoid breaking your development environment:

### ✅ Option 1: Build & Run Directly from Source (Recommended for Dev)

Since Hooksmith is a workspace crate, you can build and run it directly:

```bash
# From the root of the repo
cargo build --bin hooksmith
cargo run --bin hooksmith -- <args>
```

This uses the current version in the repo, ensuring you test changes immediately.

### ✅ Option 2: Use a Released Version of Hooksmith as a Dependency

To avoid breaking the dev environment while testing new changes:

1️⃣ **Install a released version globally:**

```bash
cargo install hooksmith --version 0.1.0
```

2️⃣ **Run it from anywhere:**

```bash
hooksmith <args>
```

This ensures the version you use for daily workflows is stable.

### ✅ Option 3: Pin a Version in the Workspace (Using a Local Override)

Inside `Cargo.toml`, you can force the workspace to depend on a specific version of hooksmith while you develop locally:

```toml
[patch.crates-io]
hooksmith = { path = "." }
```

Or, for testing an older version:

```toml
[dependencies]
hooksmith = "=0.1.0"
```

This way, the workspace always builds with the chosen version unless you override it.

## 🔑 Recommended Dev Setup

### Day-to-Day Development

```bash
cargo run --bin hooksmith -- test
```

### Ensuring No Breakages

1. **Install a stable version globally:**

```bash
cargo install hooksmith --version 0.1.0
```

2. **Use that version for daily workflows:**

```bash
hooksmith list
```

3. **When testing new changes, use:**

```bash
cargo run --bin hooksmith -- <args>
```

## 🛠️ Prerequisites

- **Rust**: Latest stable version (1.75+)
- **Git**: Latest version
- **Lefthook**: For pre-commit hooks (optional but recommended)
- **Pandoc**: For documentation generation (optional)

## 🚀 Setup

### 1. Clone the Repository

```bash
git clone https://github.com/bdelanghe/hooksmith.git
cd hooksmith
```

### 2. Install Dependencies

```bash
# Install Lefthook (optional but recommended)
npm install -g @evilmartians/lefthook

# Or using Homebrew on macOS
brew install lefthook

# Install Pandoc for documentation (optional)
# macOS: brew install pandoc
# Ubuntu: sudo apt-get install pandoc
# Windows: Download from https://pandoc.org/installing.html
```

### 3. Install Pre-commit Hooks

```bash
lefthook install
```

### 4. Set Up Git Filters for Contract Validation

```bash
# Set up Git filters and diffs for contract validation
./scripts/setup-git-filters.sh

# Verify the configuration
git config --list | grep contract
```

This sets up the hierarchical contract validation system that integrates with Git's filter and diff mechanisms.

### 5. Generate Code and Build the Project

```bash
# Generate all code and documentation
./xtask.sh gen-all --overwrite

# Or use the build script
./build.sh
```

### 6. Run Tests

```bash
cargo test --all-targets --all-features
```

## 🔧 Xtask Commands

This project uses **xtask** for structured code generation and build tasks, replacing shell scripts and raw echo statements:

### Basic Commands

```bash
# Build the project and all components
./xtask.sh build --target all --release

# Generate WIT interface definitions
./xtask.sh gen-wit --overwrite

# Generate Lefthook configuration
./xtask.sh gen-lefthook --validate

# Generate documentation
./xtask.sh gen-docs --open

# Run all code generation tasks
./xtask.sh gen-all --overwrite

# Check if generated files are up to date
./xtask.sh check --strict
```

### Hierarchical Contract Validation

```bash
# Validate changes in a commit range
./xtask.sh contract-validate validate --range HEAD~1..HEAD

# Verify validation chain integrity
./xtask.sh contract-validate verify <commit-hash>

# Show validation notes for a commit
./xtask.sh contract-validate show <commit-hash>

# Run pre-commit validation
./xtask.sh contract-validate pre-commit

# Run post-commit validation
./xtask.sh contract-validate post-commit
```

### Git Filter Operations

```bash
# Git filter operations
./xtask.sh contract-validate clean <file>
./xtask.sh contract-validate smudge <file>
./xtask.sh contract-validate diff <file>
```

### State Machine Operations

```bash
# State machine operations
./xtask.sh contract-validate audit --strict --commit HEAD
./xtask.sh contract-validate merkle --verify --commit HEAD
./xtask.sh contract-validate report --comprehensive --commit HEAD
```

## 🛡️ Safety Commands

### Check Stable Version Compatibility

```bash
# Check if current changes are compatible with the last release
./xtask.sh check-stable

# Check against a specific version
./xtask.sh check-stable --version 0.1.0

# Run comprehensive compatibility tests
./xtask.sh check-stable --comprehensive
```

### Validate Against Released Version

```bash
# Build current branch
cargo build --bin hooksmith

# Run tests using the last released version
./xtask.sh test-with-release

# Compare behavior to ensure no breaking changes
./xtask.sh compare-with-release
```

## 📁 Project Structure

```
hooksmith/
├── Cargo.toml               # Workspace manifest
├── build.sh                 # Build script
├── xtask.sh                 # Xtask wrapper script
├── README.md                # This file
├── .gitattributes           # Hierarchical validation configuration
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point (documented)
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command structure
│   └── modules/             # Module structure
│       ├── wasm.rs          # WASM component management
│       ├── hook_builder.rs  # Hook building and compilation
│       ├── hierarchical_validation.rs # Hierarchical contract validation
│       └── contract_state_machine.rs # Contract state machine implementation
├── components/              # Modular components
│   ├── cli-core/            # Core CLI functionality
│   ├── worktree-runner/     # Worktree management WASM component
│   └── git-filter/          # Git filter components
├── xtask/                   # Xtask build system
│   ├── src/main.rs          # Xtask CLI
│   └── src/hierarchical_validation.rs # Validation CLI
├── docs/                    # Documentation
│   ├── git-notes-schema.json # Git Notes JSON schema
│   ├── contract-state-machine-schema.json # Contract state machine schema
│   ├── state-transitions.yaml # State transition definitions
│   ├── merkle-chain-spec.md # Merkle chain validation specification
│   └── contract-validation-architecture.md # Complete architecture documentation
├── hooks/                   # Hook scripts directory
├── tests/                   # Test files
└── target/doc/              # Generated documentation
```

## 🔧 Components

- **hooksmith**: Main CLI binary for hook building and WASM management
- **cli-core**: Core CLI functionality and utilities
- **worktree-runner**: WASM component for worktree management
- **git-filter**: Git filter components for hierarchical validation
- **xtask**: Build system for code generation and validation tasks

## 🧪 Testing

### Run All Tests

```bash
cargo test --all-targets --all-features
```

### Run Specific Tests

```bash
# Run specific test
cargo test test_cli_help

# Run integration tests
cargo test --test integration

# Run hierarchical validation tests
cargo test --test hierarchical_validation
```

### Test Against Stable Version

```bash
# Install stable version for comparison
cargo install hooksmith --version 0.1.0

# Run current version tests
cargo test

# Compare outputs
./xtask.sh compare-outputs --stable-version 0.1.0
```

## 📚 Documentation

### Generate Documentation

```bash
# Generate API documentation
cargo doc --no-deps --open

# Generate comprehensive documentation
./xtask.sh gen-docs --open

# Generate schema documentation
./xtask.sh gen-schema-docs --pdf --html --epub
```

### View Documentation

- **API Documentation**: `cargo doc --no-deps --open`
- **CLI Help**: `hooksmith --help`
- **Command Help**: `hooksmith <command> --help`
- **Project Docs**: `docs/` directory

## 🔄 Development Workflow

### 1. Start Development

```bash
# Clone and setup
git clone https://github.com/bdelanghe/hooksmith.git
cd hooksmith
./xtask.sh gen-all --overwrite

# Set up Git filters for contract validation
./scripts/setup-git-filters.sh

cargo test
```

### 2. Install Stable Version for Daily Use

```bash
# Install stable version globally
cargo install hooksmith --version 0.1.0

# Use stable version for daily workflows
hooksmith list
hooksmith test
```

### 3. Develop and Test Changes

```bash
# Make changes to the code
# ...

# Test changes with local build
cargo run --bin hooksmith -- test

# Run all tests
cargo test --all-targets --all-features

# Check for breaking changes
./xtask.sh check-stable
```

### 4. Validate Against Release

```bash
# Build current version
cargo build --bin hooksmith

# Compare with stable version
./xtask.sh compare-with-release

# Run compatibility tests
./xtask.sh test-compatibility
```

### 5. Commit and Push

```bash
# Run pre-commit hooks
lefthook run pre-commit

# Commit changes
git add .
git commit -m "feat: add new feature"

# Push changes
git push
```

## 🛡️ Extra Safety Measures

### Add Safety Commands to Xtask

The following commands can be added to `xtask/src/main.rs` for additional safety:

```rust
/// Check if current changes are compatible with the last release
CheckStable {
    /// Version to check against
    #[arg(long, default_value = "0.1.0")]
    version: String,
    /// Run comprehensive compatibility tests
    #[arg(long)]
    comprehensive: bool,
},

/// Test current version against released version
TestWithRelease {
    /// Version to test against
    #[arg(long, default_value = "0.1.0")]
    version: String,
},

/// Compare outputs between current and released version
CompareWithRelease {
    /// Version to compare against
    #[arg(long, default_value = "0.1.0")]
    version: String,
},
```

### Safety Checklist

Before committing changes:

- [ ] Run `cargo test --all-targets --all-features`
- [ ] Run `./xtask.sh check-stable`
- [ ] Run `./xtask.sh compare-with-release`
- [ ] Test with stable version: `hooksmith test`
- [ ] Test with local version: `cargo run --bin hooksmith -- test`
- [ ] Ensure outputs match between versions

## 🚨 Troubleshooting

### Common Issues

1. **Build fails after changes**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --bin hooksmith
   ```

2. **Generated files are outdated**
   ```bash
   # Regenerate all files
   ./xtask.sh gen-all --overwrite
   ```

3. **Tests fail after dependency changes**
   ```bash
   # Update dependencies and rebuild
   cargo update
   cargo test
   ```

4. **Stable version conflicts with local version**
   ```bash
   # Uninstall stable version temporarily
   cargo uninstall hooksmith
   
   # Or use specific version for testing
   cargo run --bin hooksmith -- test
   ```

### Getting Help

- **CLI Help**: `hooksmith --help`
- **Command Help**: `hooksmith <command> --help`
- **Xtask Help**: `./xtask.sh --help`
- **API Docs**: `cargo doc --no-deps --open`

## 📋 Code Style

This project uses standard Rust formatting and linting:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features -- -D warnings
```

## 🔗 Integration

This CLI is designed to integrate with Lefthook for Git hook management:

```bash
# Generate Lefthook config
hooksmith generate > lefthook.yml

# Install hooks
hooksmith install
```

---

*This development guide ensures you can safely develop Hooksmith while maintaining a stable environment for daily workflows.*
