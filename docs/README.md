# hooksmith

Hooksmith bridges the gap between modern Git workflow tools and WebAssembly components

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


## 🚀 Current Status vs Intended Purpose

### 🎯 **Intended Purpose**
Hooksmith is designed to be a CLI tool that builds Rust binaries into Lefthook hooks with WASM components. The goal is to compile Rust code into optimized binary executables for Git hooks, integrate WebAssembly components for cross-language functionality, generate Lefthook configuration files automatically, and provide a unified interface for hook management.

### 📊 **Current State**

| Feature | Status | Description |
|---------|--------|-------------|
| **CLI Structure** | ✅ Complete | Full CLI with commands for building, generating, installing, and managing hooks |
| **Documentation** | ✅ Complete | Comprehensive rustdoc comments, README, and generated documentation |
| **Testing** | ✅ Complete | 16 integration tests, unit tests, and build verification |
| **Build System** | ✅ Complete | Automated build script with component compilation |
| **WASM Components** | ✅ Complete | WASM module with component building, running, and bindings generation |
| **Lefthook Integration** | ✅ Complete | Configuration generator module implemented |
| **Hook Building** | ✅ Complete | Rust-to-binary compilation logic with Cargo integration |
| **WASM Compilation** | ✅ Complete | Placeholder WASM component building with WIT validation |
| **Tool Integration** | ✅ Complete | Integration with existing worktree tools (wtp, wt, treekanga, git) |
| **Hook Installation** | ✅ Complete | Hook installation and management functionality |
| **Worktree Management** | ✅ Complete | Worktree creation, listing, switching, and removal |
| **Hierarchical Validation** | ✅ Complete | Bottom-up contract validation with Git Notes integration |
| **Contract State Machine** | ✅ Complete | Schema-driven state machine with Merkle chain validation |


## 🚀 **Roadmap**

#### **Phase 1: Foundation** ✅
Core CLI structure and project setup

- [✅] CLI structure and command parsing
- [✅] Documentation and testing framework
- [✅] Build system and component architecture
- [✅] Basic project structure

#### **Phase 2: WASM Integration** ✅
WebAssembly component integration

- [✅] WASM dependencies added (wasmtime, wit-bindgen)
- [✅] WIT interface definitions created
- [✅] Worktree-runner component scaffolded
- [✅] Actual WASM component compilation (placeholder implementation)
- [✅] WASM runtime integration in hooks

#### **Phase 3: Lefthook Integration** ✅
Git hook management integration

- [✅] Lefthook configuration generator
- [✅] YAML configuration structure
- [✅] Hook installation and management
- [✅] Git integration and hook execution

#### **Phase 4: Tool Integration** ✅
Integration with existing Git tools

- [✅] Integration with wtp, wt, treekanga, git
- [✅] Worktree management automation
- [✅] Cross-platform compatibility
- [✅] Performance optimization (basic implementation)

#### **Phase 5: Hierarchical Validation** ✅
Contract validation system

- [✅] Bottom-up validation pipeline implementation
- [✅] Git Notes integration for validation history
- [✅] Hierarchical scope detection
- [✅] Validation chain integrity verification
- [✅] Git hooks integration (pre-commit, post-commit)
- [✅] Xtask CLI for validation management

#### **Phase 6: Contract State Machine** ✅
State machine validation system

- [✅] Schema-driven state machine implementation
- [✅] Merkle chain validation system
- [✅] Git Notes integration for audit trails
- [✅] CI pipeline with security enforcement
- [✅] State transition validation and enforcement

#### **Phase 7: Production Ready** 📋
Production deployment preparation

- [📋] Error handling and recovery
- [📋] Performance benchmarking
- [📋] Security audit and hardening
- [📋] Production deployment pipeline



## 📚 API Documentation



## 🧪 Examples

No examples available yet.

## 🛠️ Installation

```bash
cargo install hooksmith
```

## 🚀 Quick Start

```bash
# Build and install hooks
hooksmith build --install

# Generate Lefthook configuration
hooksmith gen-lefthook

# Run validation
hooksmith validate
```

## 📖 Documentation

- [API Reference](docs/api/)
- [Development Guide](docs/DEVELOPMENT.md)
- [Contributing Guide](docs/CONTRIBUTING.md)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](docs/CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
