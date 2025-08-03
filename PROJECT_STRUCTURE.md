# Hooksmith Project Structure

## Overview

Hooksmith is a modular Rust workspace designed for git hooks, contract validation, and development automation. The project follows a strict file type policy where only `.rs` and `.jsonc` files are allowed as source files, while all other file types must be code-generated.

## File Type Policy

### Allowed Source Files
- **`.rs`** - Rust source files (manually maintained)
- **`.jsonc`** - JSON with comments configuration files (manually maintained)

### Generated File Types
All other file types must be code-generated with appropriate markers:
- **`.toml`** - Cargo configuration files
- **`.md`** - Markdown documentation
- **`.yml`/`.yaml`** - YAML configuration files
- **`.wit`** - WebAssembly Interface Types
- **`.json`** - JSON schemas and data
- **`.hbs`** - Handlebars templates
- **`.dot`** - Graphviz dot files
- **`.css`** - Stylesheets
- **`.html`** - HTML documentation
- **`.pdf`/`.epub`** - Documentation formats
- **`.sh`/`.bash`/`.zsh`** - Shell scripts
- **`.gitattributes`/`.gitignore`** - Git configuration
- **`CODEOWNERS`** - GitHub ownership rules

## Directory Structure

```
hooksmith/
├── 📁 components/                    # Modular crates
│   ├── 📁 cli-core/                 # Core CLI functionality
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── 📁 git-filter/               # Git filtering and contract validation
│   │   ├── src/
│   │   ├── wit/                     # WebAssembly interface definitions
│   │   └── Cargo.toml
│   ├── 📁 hook-builder/             # Hook construction and compilation
│   │   ├── src/
│   │   ├── wit/
│   │   └── Cargo.toml
│   ├── 📁 validation-handler/       # Validation logic
│   │   ├── src/
│   │   └── Cargo.toml
│   └── 📁 worktree-runner/          # Git worktree management
│       ├── src/
│       ├── wit/
│       └── Cargo.toml
├── 📁 src/                          # Main crate sources
│   ├── 📁 bin/                      # Binary entry points
│   ├── 📁 commands/                 # CLI command implementations
│   ├── 📁 generated/                # Auto-generated code
│   ├── 📁 modules/                  # Core modules
│   ├── 📁 orchestrator/             # Component orchestration
│   └── main.rs
├── 📁 xtask/                        # Build automation and tooling
│   ├── 📁 docs/                     # Documentation generation
│   ├── 📁 examples/                 # Example implementations
│   ├── 📁 hooks/                    # Custom git hooks
│   ├── 📁 src/                      # Tooling source code
│   └── Cargo.toml
├── 📁 schemas/                      # JSON schemas and contracts
├── 📁 templates/                    # Handlebars templates
├── 📁 docs/                         # Generated documentation
├── 📁 examples/                     # Usage examples
├── 📁 scripts/                      # Custom scripts and automation
│   └── 📁 jql_queries/              # JQL analysis queries
├── 📁 lefthook-rs/                  # Lefthook integration crate
├── 📁 wit/                          # WebAssembly interface definitions
├── 📁 config/                       # Configuration files
├── 📁 hooks/                        # Git hooks
└── 📁 tests/                        # Integration tests
```

## Component Architecture

### Core Components

#### `components/cli-core/`
- **Purpose**: Core CLI functionality and command parsing
- **Dependencies**: Minimal, focused on CLI operations
- **Interfaces**: Provides command trait implementations

#### `components/git-filter/`
- **Purpose**: Git filtering, contract validation, and blob/tree processing
- **Dependencies**: Git operations, contract validation
- **WIT Interfaces**: WebAssembly interface for filtering operations
- **Key Features**:
  - Blob contract validation
  - Tree contract processing
  - Filename character validation
  - Unified contract system

#### `components/hook-builder/`
- **Purpose**: Hook construction, compilation, and optimization
- **Dependencies**: Hook building logic, optimization
- **WIT Interfaces**: WebAssembly interface for hook building
- **Key Features**:
  - Hook compilation
  - Code optimization
  - Validation

#### `components/validation-handler/`
- **Purpose**: Validation logic and contract checking
- **Dependencies**: Validation rules, contract definitions
- **Key Features**:
  - Contract validation
  - Rule enforcement
  - Error reporting

#### `components/worktree-runner/`
- **Purpose**: Git worktree management and execution
- **Dependencies**: Git worktree operations
- **WIT Interfaces**: WebAssembly interface for worktree operations
- **Key Features**:
  - Worktree creation and management
  - Execution environment setup
  - State management

### Main Application

#### `src/`
- **Purpose**: Main application logic and orchestration
- **Structure**:
  - `bin/` - Binary entry points for different use cases
  - `commands/` - CLI command implementations
  - `generated/` - Auto-generated code (docs, features, version)
  - `modules/` - Core business logic modules
  - `orchestrator/` - Component coordination and routing

### Build Automation

#### `xtask/`
- **Purpose**: Custom build tasks, automation, and tooling
- **Key Features**:
  - Documentation generation
  - Schema validation
  - File type enforcement
  - Code generation
  - Contract validation
  - Event bus management
  - Structured logging

## Configuration and Schemas

### `config/`
- **`file_types.yaml`** - File type policy configuration
- **`file-policy.jsonc`** - Strict file extension policy
- **`docs_manifest.yaml`** - Documentation generation manifest
- **`lefthook.yml`** - Lefthook configuration

### `schemas/`
- **Contract schemas** - JSON schemas for contract validation
- **State schemas** - State machine definitions
- **Transition schemas** - State transition rules

## Documentation Structure

### `docs/`
- **Component documentation** - Per-component guides
- **API documentation** - Generated API docs
- **Architecture docs** - System design and patterns
- **Examples** - Usage examples and tutorials
- **Generated files** - Auto-generated documentation

## Scripts and Automation

### `scripts/`
- **Shell scripts** - Development and deployment automation
- **JQL queries** - Analysis and reporting queries
- **CI/CD scripts** - Continuous integration automation
- **Development scripts** - Local development helpers

## WebAssembly Integration

### WIT Files
- **Purpose**: Define WebAssembly interface types
- **Location**: `wit/` and component-specific `wit/` directories
- **Usage**: Enable cross-language component communication

## Development Workflow

### File Type Enforcement
1. **Source files** (`.rs`, `.jsonc`) are manually maintained
2. **Generated files** must have appropriate markers
3. **Validation** ensures compliance with file type policy

### Code Generation
```bash
# Generate all files
cargo xtask gen-all

# Generate specific file types
cargo xtask gen-docs --all
cargo xtask gen-config --all
cargo xtask gen-schema --all

# Validate generated files
cargo xtask validate-files
```

### Component Development
```bash
# Build specific component
cargo build -p cli-core

# Test component
cargo test -p git-filter

# Run examples
cargo run -p xtask -- examples
```

## Validation and Quality

### File Validation
- **Extension validation** - Ensures only allowed file types
- **Generated file validation** - Checks for proper markers
- **Contract validation** - Validates against schemas

### Code Quality
- **Clippy** - Rust linting and best practices
- **Formatting** - Consistent code formatting
- **Testing** - Comprehensive test coverage

## Integration Points

### Git Integration
- **Lefthook** - Git hook management
- **Git filters** - Content filtering and validation
- **Worktree management** - Multi-worktree operations

### External Tools
- **JQL** - Query language for analysis
- **SARIF** - Static analysis results
- **CodeQL** - Security analysis integration

## Best Practices

### Module Organization
- **Clear separation** of concerns across components
- **Consistent naming** conventions
- **Minimal dependencies** between components
- **WebAssembly interfaces** for cross-language support

### Documentation
- **Auto-generated** documentation from source
- **Component-specific** guides and examples
- **Architecture documentation** for system design
- **API documentation** for external interfaces

### Testing
- **Unit tests** within each component
- **Integration tests** for component interaction
- **Contract validation** tests
- **End-to-end** workflow tests

This structure provides a solid foundation for a modular, maintainable, and extensible Rust workspace focused on git hooks and development automation. 
