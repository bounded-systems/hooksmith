# Xtask Commands Quick Reference

This guide shows the correct ways to run xtask commands in the Hooksmith project.

## 🚀 **Correct Ways to Run Xtask**

### 1. **From Project Root (Recommended)**
```bash
# Run xtask with any command
cargo run -p xtask -- <command>

# Examples:
cargo run -p xtask -- --help
cargo run -p xtask -- build
cargo run -p xtask -- gen-docs
cargo run -p xtask -- auto-push
```

### 2. **Using Convenient Aliases (Available in direnv)**
```bash
# These aliases are automatically available when direnv is loaded
xtask --help
xtask build
xtask gen-docs
xtask auto-push

# Other aliases:
xtask-check    # cargo check -p xtask
xtask-build    # cargo build -p xtask
xtask-test     # cargo test -p xtask
```

### 3. **From xtask Directory**
```bash
cd xtask
cargo run -- <command>
cargo check
cargo build
```

## ❌ **What Doesn't Work**

```bash
# ❌ This fails - Cargo looks for binary named "xtask" in root package
cargo run --bin xtask

# ❌ This also fails - same reason
cargo check --bin xtask
```

## 🔧 **Why This Happens**

The xtask binary is defined in the `xtask/` workspace member, not in the root package. When you run `cargo run --bin xtask` from the root, Cargo looks for a binary target named `xtask` in the current package's `Cargo.toml`.

## ✅ **Solutions Implemented**

### 1. **Added `default-run` to xtask/Cargo.toml**
```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"
default-run = "xtask"  # ← This allows cargo run to work from xtask/ directory
```

### 2. **Added Convenient Aliases in .envrc**
```bash
alias xtask="cargo run -p xtask"
alias xtask-check="cargo check -p xtask"
alias xtask-build="cargo build -p xtask"
alias xtask-test="cargo test -p xtask"
```

## 🎯 **Common Commands**

### Build & Development
```bash
xtask build                    # Build project and components
xtask check                    # Check if generated files are up to date
xtask validate                 # Validate project configuration
```

### Code Generation
```bash
xtask gen-docs                 # Generate documentation
xtask gen-wit                  # Generate WIT interfaces
xtask gen-lefthook            # Generate Lefthook configuration
xtask gen-all                 # Generate all code-generated files
```

### Git Workflow
```bash
xtask auto-push               # Automated git workflow
xtask clean-auto-push         # Clean auto-push with porcelain output
xtask structured-auto-push    # Structured auto-push with JSONL output
xtask setup-git-aliases       # Set up git aliases
```

### Validation
```bash
xtask contract-check          # Comprehensive contract validation
xtask validate-generated      # Validate generated files
xtask validate-commit-msg     # Validate commit messages
```

### Monitoring & Dashboard
```bash
xtask dashboard               # Interactive dashboard
xtask event-stream init       # Initialize event stream
xtask event-bus init          # Initialize event bus
```

## 🔍 **Getting Help**

```bash
# Show all available commands
xtask --help

# Show help for specific command
xtask build --help
xtask auto-push --help
xtask dashboard --help
```

## 🛠️ **Troubleshooting**

### Environment Not Loaded
```bash
# Make sure direnv is loaded
direnv allow
direnv status

# Or manually source the environment
source .envrc
```

### Xtask Not Found
```bash
# Check if xtask is built
cargo build -p xtask

# Check if you're in the right directory
pwd  # Should be in project root
ls xtask/Cargo.toml  # Should exist
```

### Permission Issues
```bash
# Make sure .envrc is executable
chmod +x .envrc

# Allow direnv to run it
direnv allow
```

## 📚 **Related Documentation**

- [DIRENV_SETUP.md](./DIRENV_SETUP.md) - Modern direnv setup
- [xtask/README.md](./xtask/README.md) - Detailed xtask documentation
- [Cargo Workspace Documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html) 
