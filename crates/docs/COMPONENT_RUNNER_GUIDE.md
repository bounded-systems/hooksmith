# Component Runner Guide

## 🎯 **Overview**

Hooksmith provides multiple ways to run WIT components using `wasmtime --invoke`. Each approach has different trade-offs in terms of complexity, dependencies, and ease of use.

## 🚀 **Available Runners**

### **1. Simple Rust Script (Recommended for Quick Testing)**

**File:** `scripts/run_component_simple.rs`

**Usage:**
```bash
# Compile and run in one step (with nightly toolchain)
rustup run nightly rustc --edition=2021 -o /tmp/run-component scripts/run_component_simple.rs && /tmp/run-component <function> <component.wasm> [args...]

# Or compile once and reuse
rustup run nightly rustc --edition=2021 -o /tmp/run-component scripts/run_component_simple.rs
/tmp/run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

**Pros:**
- ✅ **No complex dependencies** - only standard library
- ✅ **Fast compilation** - minimal dependencies
- ✅ **Works with nightly Rust** - supports WASI Preview 2
- ✅ **Simple to understand** - straightforward code
- ✅ **Easy to modify** - self-contained script

**Cons:**
- ❌ **Basic CLI** - no advanced argument parsing
- ❌ **No subcommands** - simple positional arguments
- ❌ **Manual compilation** - need to compile before use

**Best for:** Quick testing, CI/CD scripts, simple automation

### **2. Full Cargo Binary (Recommended for Development)**

**File:** `scripts/run_component.rs`

**Usage:**
```bash
# Run directly with cargo (requires nightly toolchain)
cargo run --bin run-component -- <function> <component.wasm> [args...]

# Or build and run
cargo build --bin run-component
./target/debug/run-component <function> <component.wasm> [args...]
```

**Pros:**
- ✅ **Advanced CLI** - full clap argument parsing
- ✅ **Subcommands** - organized command structure
- ✅ **Environment variables** - WASMTIME_ARGS, VERBOSE, STRICT
- ✅ **Better error handling** - comprehensive error messages
- ✅ **Integrated with project** - uses workspace dependencies

**Cons:**
- ❌ **Requires nightly Rust** - for WASI Preview 2 support
- ❌ **Complex dependencies** - clap, anyhow, etc.
- ❌ **Slower compilation** - more dependencies to compile
- ❌ **Sccache issues** - may have build cache problems

**Best for:** Development, complex automation, integration with other tools

### **3. Shell Script (Legacy - Removed)**

**File:** `scripts/run_component.sh` - **REMOVED**

**Status:** This shell script has been **removed** and replaced with Rust implementations.

**Reason:** Better integration with project architecture, type safety, and maintainability.

## 📋 **Component Functions**

### **hook-builder Component**
```bash
# Validate source code
run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm --source-path src/main.rs

# Build hook
run-component build-hook target/wasm32-wasip2/release/hook_builder.wasm --config hook-config.json

# Check configuration
run-component check-config target/wasm32-wasip2/release/hook_builder.wasm --config hook-config.json

# Generate bindings
run-component generate-bindings target/wasm32-wasip2/release/hook_builder.wasm --output bindings.rs
```

### **worktree-runner Component**
```bash
# List worktrees
run-component list-worktrees target/wasm32-wasip2/release/worktree_runner.wasm

# Create worktree
run-component create-worktree target/wasm32-wasip2/release/worktree_runner.wasm --branch feature/new-feature

# Remove worktree
run-component remove-worktree target/wasm32-wasip2/release/worktree_runner.wasm --branch feature/old-feature

# Check status
run-component check-status target/wasm32-wasip2/release/worktree_runner.wasm
```

### **git-filter Component**
```bash
# Validate blob
run-component validate-blob target/wasm32-wasip2/release/git_filter.wasm --blob test-data

# Filter object
run-component filter-object target/wasm32-wasip2/release/git_filter.wasm --object git-object

# Check contract
run-component check-contract target/wasm32-wasip2/release/git_filter.wasm --contract contract-name

# Transform content
run-component transform-content target/wasm32-wasip2/release/git_filter.wasm --content input-content
```

### **validation-handler Component**
```bash
# Validate content
run-component validate target/wasm32-wasip2/release/validation_handler.wasm --input test-input

# Validate file
run-component validate-file target/wasm32-wasip2/release/validation_handler.wasm --file path/to/file

# Check if valid
run-component is-valid target/wasm32-wasip2/release/validation_handler.wasm --input test-input

# Get rules
run-component get-rules target/wasm32-wasip2/release/validation_handler.wasm

# Add rule
run-component add-rule target/wasm32-wasip2/release/validation_handler.wasm --rule rule-definition
```

## 🔧 **Environment Variables**

### **WASMTIME_ARGS**
Additional arguments to pass to wasmtime:
```bash
export WASMTIME_ARGS="--wasm-features=component-model"
run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

### **VERBOSE**
Enable verbose output:
```bash
export VERBOSE=1
run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

### **STRICT**
Exit on errors:
```bash
export STRICT=1
run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

## 🎯 **Recommendations**

### **For Quick Testing**
Use the simple Rust script with nightly:
```bash
rustup run nightly rustc --edition=2021 -o /tmp/run-component scripts/run_component_simple.rs && /tmp/run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

### **For Development**
Use the cargo binary with nightly:
```bash
cargo run --bin run-component -- validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

### **For CI/CD**
Use the simple Rust script with nightly for reliability:
```bash
rustup run nightly rustc --edition=2021 -o /tmp/run-component scripts/run_component_simple.rs
/tmp/run-component validate-source target/wasm32-wasip2/release/hook_builder.wasm
```

### **For Legacy Support**
The shell script has been removed. Use the Rust implementations instead.

## 🚨 **Troubleshooting**

### **wasmtime not found**
```bash
# Install wasmtime
curl https://wasmtime.dev/install.sh -sSf | bash
source ~/.zshrc  # or ~/.bashrc
```

### **Component not found**
```bash
# Build the component first
cargo component build --target wasm32-wasip2 --release
```

### **Function not found**
```bash
# Check available functions
run-component --help
```

### **Compilation errors**
```bash
# Use nightly for simple script
rustup run nightly rustc --edition=2021 -o /tmp/run-component scripts/run_component_simple.rs

# Use nightly for full binary
cargo run --bin run-component
```

### **Sccache issues**
If you encounter sccache problems with the full binary:
```bash
# Use the simple script instead
rustup run nightly rustc --edition=2021 -o /tmp/run-component scripts/run_component_simple.rs && /tmp/run-component <function> <component.wasm> [args...]
```

## 🎉 **Conclusion**

The **simple Rust script** (`scripts/run_component_simple.rs`) is recommended for most use cases because it:
- ✅ Works with nightly Rust (required for WASI Preview 2)
- ✅ Has minimal dependencies
- ✅ Is fast to compile
- ✅ Is easy to understand and modify
- ✅ Is reliable for CI/CD

Use the **full cargo binary** (`scripts/run_component.rs`) when you need advanced CLI features or integration with the project's tooling.

**Note:** The shell script has been removed and replaced with Rust implementations for better integration and maintainability. 
