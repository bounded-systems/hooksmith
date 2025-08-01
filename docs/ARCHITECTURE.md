# Pushd CLI Architecture

This document describes the strong, componentized architecture of the pushd-cli with WebAssembly components.

## Overview

The pushd-cli is built as a modular system with:
- **Strong typing** throughout the entire stack
- **Wasm components** for language-agnostic functionality
- **WIT contracts** for stable interfaces
- **Comprehensive testing** at all levels
- **Clear separation of concerns** between CLI and components

## Architecture Layers

### 1. CLI Binary Layer (`src/`)

The CLI binary provides the user interface and orchestrates components:

```
src/
├── cli_main.rs          # CLI entry point (clap-based)
├── lib.rs               # CLI library with traits and types
├── commands/            # Command implementations
│   ├── worktree.rs
│   ├── daemon.rs
│   ├── docker.rs
│   ├── git.rs
│   └── setup.rs
├── components.rs        # Component loading and binding
├── config.rs           # Configuration management
└── utils.rs            # Shared utilities
```

**Key Design Principles:**
- **Trait-based commands**: All commands implement the `Command` trait
- **Strong typing**: No raw strings in internal APIs
- **Component orchestration**: CLI loads and manages Wasm components
- **Error handling**: Comprehensive error types with context

### 2. Component Layer (`components/`)

Each component is a separate crate that compiles to Wasm:

```
components/
├── cli-core/           # Core CLI operations
├── daemon-runner/      # Daemon management
├── worktree-manager/   # Worktree operations
├── docker-manager/     # Docker operations
└── git-validator/      # Git validation
```

**Key Design Principles:**
- **cdylib crate type**: Compiles to Wasm
- **WIT interfaces**: Stable contracts between components
- **Language agnostic**: Can be written in Rust, Go, TypeScript, etc.
- **Isolated**: Each component is self-contained

### 3. Interface Layer (`wit/`)

WIT (WebAssembly Interface Types) defines the contracts:

```wit
package pushd:cli;

interface cli-core {
  record config {
    aws-profile: string,
    target-env: string,
    project-root: string,
    dry-run: bool,
  };

  record operation-result {
    success: bool,
    message: string,
    error: option<string>,
    data: option<string>,
  };

  execute-daemon: func(daemon-name: string, config: config, options: list<string>) -> operation-result;
  // ... more functions
}

world pushd-cli-world {
  export cli-core;
  export daemon-runner;
  export worktree-manager;
  export docker-manager;
  export git-validator;
}
```

## Strong Typing Strategy

### 1. CLI Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    pub aws_profile: String,
    pub target_env: String,
    pub project_root: String,
    pub dry_run: bool,
    pub use_components: bool,
    pub components_path: String,
}
```

### 2. Command Results

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliResult {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
    pub data: Option<String>,
}
```

### 3. Command Trait

```rust
pub trait Command {
    fn run(&self, args: &[String]) -> Result<()>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}
```

### 4. Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Component error: {0}")]
    Component(String),
    
    #[error("Command error: {0}")]
    Command(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("System error: {0}")]
    System(String),
}
```

## Component Architecture

### 1. Component Loading

```rust
pub struct CliApp {
    config: CliConfig,
    component_manager: Option<ComponentManager>,
}

impl CliApp {
    pub async fn execute(&self, command: &str, args: &[String]) -> Result<CliResult> {
        // Try to use component if available
        if let Some(ref manager) = self.component_manager {
            if let Some(component) = manager.get_component(command) {
                return self.execute_with_component(component, args).await;
            }
        }
        
        // Fall back to native command
        self.execute_native(command, args).await
    }
}
```

### 2. WIT Binding

```rust
wasmtime::component::bindgen!({
    path: "../wit/pushd-cli.wit",
    world: "pushd-cli-world",
    async: true,
});
```

### 3. Component Implementation

```rust
// In each component crate
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Engine, Store};

bindgen!({
    path: "../../wit/pushd-cli.wit",
    world: "pushd-cli-world",
    async: true,
});

pub struct DaemonRunnerComponent {
    engine: Engine,
    store: Store<()>,
    component: Component,
    instance: PushdCliWorld,
}

impl DaemonRunnerComponent {
    pub async fn start_daemon(&mut self, config: DaemonConfig, cli_config: CliConfig) -> Result<OperationResult> {
        let result = self.instance
            .daemon_runner()
            .start_daemon(&mut self.store, config.into(), cli_config.into())
            .await?;
        Ok(result.into())
    }
}
```

## Testing Strategy

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_daemon_config_conversion() {
        let config = DaemonConfig {
            name: "test-daemon".to_string(),
            env_vars: vec!["TEST_VAR=value".to_string()],
            args: vec!["--test".to_string()],
            background: true,
            log_level: "info".to_string(),
        };
        
        let wit_config: daemon_runner::DaemonConfig = config.clone().into();
        let back_to_config: DaemonConfig = wit_config.into();
        
        assert_eq!(config.name, back_to_config.name);
    }
}
```

### 2. Integration Tests

```rust
#[test]
fn test_worktree_create() -> anyhow::Result<()> {
    let mut cmd = assert_cmd::Command::cargo_bin("pushd-cli")?;
    cmd.arg("worktree").arg("create").arg("feature-x");
    cmd.assert().success().stdout(predicates::str::contains("Created worktree"));
    Ok(())
}
```

### 3. Component Tests

```rust
#[tokio::test]
async fn test_component_loading() {
    let manager = ComponentManager::new("./components").await.unwrap();
    let components = manager.list_components().await.unwrap();
    assert!(!components.is_empty());
}
```

## Build System

### 1. Workspace Configuration

```toml
[workspace]
members = [
    "src",                    # CLI binary
    "components/cli-core",    # Core CLI operations component
    "components/daemon-runner", # Daemon management component
    "components/worktree-manager", # Worktree operations component
    "components/docker-manager", # Docker operations component
    "components/git-validator", # Git validation component
]

[workspace.dependencies]
# Shared dependencies across all crates
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full", "process", "fs"] }
wasmtime = { version = "15.0", features = ["component-model"] }
```

### 2. Build Process

```bash
# Build CLI binary
cargo build --release --package pushd-cli

# Build all components
cargo build --release --target wasm32-unknown-unknown --workspace

# Convert to component format
wasm-tools component new target/wasm32-unknown-unknown/release/daemon_runner.wasm \
  -o target/wasm32-unknown-unknown/release/daemon_runner.component.wasm
```

## Development Workflow

### 1. Adding New Commands

1. **Define WIT interface** in `wit/pushd-cli.wit`
2. **Create component crate** in `components/new-command/`
3. **Implement component** with WIT bindings
4. **Add CLI command** in `src/commands/new_command.rs`
5. **Add integration tests** in `tests/integration.rs`

### 2. Component Development

```bash
# Create new component
mkdir components/new-component
cd components/new-component

# Create Cargo.toml with cdylib
cargo init --lib

# Add to workspace
echo '  "components/new-component",' >> ../../Cargo.toml

# Implement component
# Add WIT interface
# Add tests
# Build and test
```

### 3. Testing Workflow

```bash
# Run all tests
cargo test --workspace

# Run integration tests
cargo test --package pushd-cli --test integration

# Run component tests
cargo test --package daemon-runner

# Test CLI end-to-end
cargo run -- daemon list
```

## Benefits of This Architecture

### 1. Strong Typing
- **Compile-time safety**: Type errors caught at build time
- **Refactoring safety**: Changes propagate through type system
- **Documentation**: Types serve as living documentation
- **IDE support**: Better autocomplete and error detection

### 2. Component Isolation
- **Language agnostic**: Components can be written in any language
- **Independent development**: Teams can work on different components
- **Easy testing**: Components can be tested in isolation
- **Deployment flexibility**: Components can be updated independently

### 3. WIT Contracts
- **Stable interfaces**: WIT provides versioned, stable contracts
- **Type safety**: Interfaces are strongly typed
- **Composability**: Components can be composed into larger systems
- **Performance**: No serialization overhead between components

### 4. Testing Strategy
- **Unit tests**: Test individual functions and types
- **Integration tests**: Test component interactions
- **End-to-end tests**: Test complete CLI workflows
- **Golden tests**: Validate component interfaces

## Future Enhancements

### 1. Plugin System
- Runtime component loading
- Hot reloading of components
- Component versioning and compatibility

### 2. Remote Components
- Load components from registries
- Component distribution and updates
- Component signing and verification

### 3. Performance Monitoring
- Component performance metrics
- Resource usage tracking
- Performance optimization

### 4. Multi-language Support
- TypeScript components
- Go components
- C/C++ components
- Python components

## Resources

- [Wasm Component Model](https://component-model.bytecodealliance.org/)
- [WIT Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [Rust CLI Guidelines](https://rust-cli.github.io/book/)
- [Clap Documentation](https://docs.rs/clap/) 
