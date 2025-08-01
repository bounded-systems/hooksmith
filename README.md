# Hooksmith

A CLI tool for building Rust binaries into Lefthook hooks with WASM components.

## 🎯 Purpose

Hooksmith bridges the gap between modern Git workflow tools and WebAssembly components, enabling:

- **High-performance Git hooks** written in Rust
- **Cross-language functionality** via WASM components
- **Type-safe interfaces** using WIT (WebAssembly Interface Types)
- **Seamless integration** with Lefthook for Git workflow management

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Lefthook      │    │   Hooksmith     │    │   WASM          │
│   (Git Hooks)   │◄──►│   (CLI Tool)    │◄──►│   Components    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │   Rust          │
                       │   Binaries      │
                       └─────────────────┘
```

## 🚀 Current Status

**⚠️ This is a prototype/placeholder implementation**

The tool currently provides:
- ✅ CLI structure and command parsing
- ✅ Comprehensive documentation
- ✅ Test suite
- ✅ Build system
- ❌ **Actual WASM compilation** (TODO)
- ❌ **Lefthook integration** (TODO)
- ❌ **Hook building logic** (TODO)

## 📦 Installation

```bash
# Build from source
./build.sh

# Install globally (optional)
cargo install --path .
```

## 🛠️ Usage

### Basic Commands

```bash
# Test the CLI
hooksmith test

# Test with custom message
hooksmith test --message "Custom test message"

# List available hooks
hooksmith list
```

### Hook Building (Planned)

```bash
# Build a hook binary
hooksmith build my-hook

# Build with custom output directory
hooksmith build my-hook --output target/custom-hooks
```

### Lefthook Integration (Planned)

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

### WASM Component Management (Planned)

```bash
# Build WASM component from WIT
hooksmith wasm build interface.wit

# Build with custom output
hooksmith wasm build interface.wit --output target/wasm

# Run WASM component
hooksmith wasm run component.wasm --function validate --args arg1,arg2

# Generate bindings from WIT
hooksmith wasm bindings interface.wit

# Generate bindings with custom output
hooksmith wasm bindings interface.wit --output target/bindings
```

## 🏗️ Development

```bash
# Build workspace
./build.sh

# Build specific component
cargo build --package cli-core

# Test workspace
cargo test --workspace

# Run CLI
cargo run -- test

# Generate documentation
cargo doc --no-deps --open
```

## 📁 Project Structure

```
hooksmith/
├── Cargo.toml               # Workspace manifest
├── build.sh                 # Build script
├── README.md                # This file
├── src/                     # Main CLI binary
│   ├── main.rs              # CLI entry point (documented)
│   ├── lib.rs               # Library exports
│   ├── commands/            # Command structure
│   └── modules/             # Module structure
├── components/              # Modular components
│   └── cli-core/            # Core CLI functionality
├── hooks/                   # Hook scripts directory
├── tests/                   # Test files
└── target/doc/              # Generated documentation
```

## 🔧 Components

- **hooksmith**: Main CLI binary for hook building and WASM management
- **cli-core**: Core CLI functionality and utilities

## 🔗 Integration

This CLI is designed to integrate with Lefthook for Git hook management:

```bash
# Generate Lefthook config
hooksmith generate > lefthook.yml

# Install hooks
hooksmith install
```

## 📚 Documentation

- **API Documentation**: `cargo doc --no-deps --open`
- **CLI Help**: `hooksmith --help`
- **Command Help**: `hooksmith <command> --help`

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_cli_help

# Run integration tests
cargo test --test integration
```

## 🚧 Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| CLI Structure | ✅ Complete | Full command parsing and help |
| Documentation | ✅ Complete | Comprehensive docs and examples |
| Tests | ✅ Complete | 14 tests passing |
| Build System | ✅ Complete | Workspace builds successfully |
| WASM Compilation | ❌ TODO | Need WASM toolchain integration |
| WIT Processing | ❌ TODO | Need WIT parser and compiler |
| Lefthook Integration | ❌ TODO | Need YAML generation and hook installation |
| Hook Building | ❌ TODO | Need Rust compilation pipeline |

## 🎯 Next Steps

To make this a fully functional Lefthook + WASM integration tool:

1. **Add WASM toolchain dependencies** (wasmtime, wit-bindgen, etc.)
2. **Implement WIT parsing and compilation**
3. **Add Rust compilation pipeline for hooks**
4. **Implement Lefthook YAML generation**
5. **Add hook installation logic**
6. **Create example WIT interfaces and WASM components**

## 📄 License

MIT
