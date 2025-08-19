# WIT-First Architecture Implementation

## Overview

This document describes the implementation of a **WIT-first, minimal-host Rust workspace** architecture in Hooksmith. This approach follows the best practices recommended by the Bytecode Alliance and the Rust/Wasm community for building robust, future-proof systems.

## Architecture Principles

### 1. **Component-First Design**
- All business logic is implemented in WASM components
- Components are defined by WIT (WebAssembly Interface Types) interfaces
- Components are language-agnostic and reusable
- Minimal host surface area for security

### 2. **WIT-First Development**
- WIT interfaces are the source of truth
- Components implement WIT interfaces in `lib.rs`
- CLI host only handles orchestration and I/O
- Clear separation between host and guest logic

### 3. **Deterministic Builds**
- All components built for `wasm32-wasip2` target
- Unified registry tracks all generated files
- Checksum validation ensures reproducibility
- Policy enforcement prevents manual modifications

## Project Structure

```
hooksmith/
├── .cargo/
│   └── config.toml              # Workspace target: wasm32-wasip2
├── src/
│   ├── .cargo/
│   │   └── config.toml          # CLI override: native target
│   └── main.rs                  # Minimal CLI host
├── crates/components/
│   ├── hook-builder/
│   │   ├── Cargo.toml           # [package.metadata.component]
│   │   ├── wit/
│   │   │   └── hook-builder.wit # WIT interface
│   │   └── src/
│   │       └── lib.rs           # Component implementation
│   ├── worktree-runner/
│   ├── git-filter/
│   └── validation-handler/
├── config/
│   └── file-policy.jsonc        # Enforce .rs only in CLI
└── scripts/
    └── run_component.sh         # Shell wrapper for wasmtime
```

## Component Configuration

### Package Metadata

Each component crate includes `[package.metadata.component]` in its `Cargo.toml`:

```toml
[package.metadata.component]
wit = ["wit"]
bindings = ["hooksmith:hook-builder"]
```

This configuration:
- Specifies the WIT directory location
- Defines the component bindings
- Enables `cargo component build` integration

### WIT Interface Example

```wit
package hooksmith:hook-builder;

/// Configuration for building a hook
record build-config {
  source-path: string,
  output-path: string,
  target-triple: option<string>,
  optimization-level: u8,
  debug-symbols: bool,
}

/// Result of a hook build operation
record build-result {
  success: bool,
  binary-path: option<string>,
  build-logs: string,
  error: option<string>,
  duration-ms: u64,
}

/// Hook building interface
interface hook-builder {
  /// Build a hook from source
  build-hook: func(config: build-config) -> result<build-result, string>;
  
  /// Validate source code
  validate-source: func(source-path: string) -> result<bool, string>;
}

export hook-builder;
```

## Build Configuration

### Workspace Configuration (`.cargo/config.toml`)

```toml
[build]
# Default target for all crates
target = "wasm32-wasip2"

[unstable]
build-std = ["std", "panic_abort"]

[component]
enabled = true

[target.wasm32-wasip2]
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "link-arg=--export-table",
    "-C", "link-arg=--export-memory",
]
```

### CLI Override (`src/.cargo/config.toml`)

```toml
[build]
# Override for CLI - build for native platform
target = "x86_64-apple-darwin"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

## Component Development Workflow

### 1. Create a New Component

```bash
# Create component directory
mkdir -p crates/components/my-component
cd crates/components/my-component

# Create Cargo.toml with component metadata
cat > Cargo.toml << EOF
[package]
name = "my-component"
version.workspace = true
edition.workspace = true

[package.metadata.component]
wit = ["wit"]
bindings = ["hooksmith:my-component"]

[dependencies]
# Add dependencies...
EOF

# Create WIT interface
mkdir wit
cat > wit/my-component.wit << EOF
package hooksmith:my-component;

interface my-component {
  my-function: func(input: string) -> result<string, string>;
}

export my-component;
EOF

# Implement component
cat > src/lib.rs << EOF
wit_bindgen::generate!({
    path: "../wit/my-component.wit",
    world: "my-component",
});

struct MyComponent;

impl my_component::MyComponent for MyComponent {
    fn my_function(input: String) -> Result<String, String> {
        Ok(format!("Processed: {}", input))
    }
}

export_my_component!(MyComponent);
EOF
```

### 2. Build Components

```bash
# Add wasm32-wasip2 target
rustup target add wasm32-wasip2

# Build all components
cargo component build --target wasm32-wasip2 --release --workspace --exclude xtask

# Build specific component
cargo component build --target wasm32-wasip2 --release -p my-component
```

### 3. Test Components with wasmtime

```bash
# Direct invocation
wasmtime run --invoke 'my-function("test")' target/wasm32-wasip2/release/my_component.wasm

# Using shell wrapper
./scripts/run_component.sh 'my-function' target/wasm32-wasip2/release/my_component.wasm "test"

# Using xtask
cargo run -p xtask -- component-smoke-test --component my-component --build --strict
```

## Integration with CLI Host

### Loading Components

The CLI host uses `wasmtime` to load and invoke components:

```rust
use wasmtime::{Engine, Store, Config};
use wasmtime::component::{Component, Linker};

async fn load_component(component_path: &str) -> Result<()> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    
    let engine = Engine::new(&config)?;
    let mut store = Store::new(&engine, ());
    
    let component = Component::from_file(&engine, component_path)?;
    let linker = Linker::new(&engine);
    
    let (instance, _) = linker.instantiate(&mut store, &component)?;
    
    // Invoke component functions
    let result = instance
        .get_func(&mut store, "my-function")?
        .call(&mut store, &["test"], &mut [])?;
    
    Ok(())
}
```

### Component Registry

Components are registered and managed through a unified system:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct ComponentRegistry {
    components: HashMap<String, ComponentInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentInfo {
    name: String,
    wasm_path: String,
    wit_path: String,
    checksum: String,
    functions: Vec<String>,
}
```

## Testing and Validation

### Component Smoke Tests

The `xtask component-smoke-test` command validates components:

```bash
# Test all components
cargo run -p xtask -- component-smoke-test --component all --build --strict

# Test specific component
cargo run -p xtask -- component-smoke-test --component hook-builder --verbose

# Test without building
cargo run -p xtask -- component-smoke-test --component worktree-runner --no-build
```

### CI/CD Integration

```yaml
# .github/workflows/component-tests.yml
name: Component Tests

on: [push, pull_request]

jobs:
  component-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-wasip2
      
      - name: Install wasmtime
        run: curl https://wasmtime.dev/install.sh -sSf | bash
      
      - name: Build components
        run: cargo component build --target wasm32-wasip2 --release --workspace --exclude xtask
      
      - name: Run smoke tests
        run: cargo run -p xtask -- component-smoke-test --component all --strict
```

## File Policy Enforcement

### Strict Extension Policy

The file policy enforces the WIT-first architecture:

```jsonc
{
  "allowedExtensions": ["rs", "jsonc"],
  "generatedExtensions": [
    "toml", "md", "yml", "wit", "wasm"
  ],
  "generationCommands": {
    "wasm": "cargo component build",
    "wit": "xtask gen-wit"
  }
}
```

### Validation Rules

- `.rs` files only allowed in CLI crate (`src/`) and `xtask`
- All other `.rs` files must be generated
- WIT files must be generated from source
- WASM files must be built from components

## Best Practices

### 1. **Component Design**
- Keep components focused and single-purpose
- Use WIT interfaces as contracts
- Implement proper error handling
- Document all exported functions

### 2. **Host Design**
- Keep CLI minimal - orchestration only
- No business logic in host code
- Use async/await for I/O operations
- Implement proper error propagation

### 3. **Testing Strategy**
- Unit test component logic in isolation
- Integration test with wasmtime
- Smoke test all components in CI
- Validate WIT interfaces

### 4. **Performance Considerations**
- Use release builds for production
- Enable LTO for smaller binaries
- Profile component performance
- Consider caching strategies

## Troubleshooting

### Common Issues

1. **Component Build Failures**
   ```bash
   # Check target installation
   rustup target list | grep wasm32-wasip2
   
   # Reinstall target
   rustup target remove wasm32-wasip2
   rustup target add wasm32-wasip2
   ```

2. **WIT Parsing Errors**
   ```bash
   # Validate WIT syntax
   wit-parser --check wit/my-component.wit
   
   # Generate bindings
   wit-bindgen generate --out-dir src/generated wit/my-component.wit
   ```

3. **wasmtime Invocation Errors**
   ```bash
   # Check component validity
   wasmtime validate target/wasm32-wasip2/release/my_component.wasm
   
   # Debug with verbose output
   wasmtime run --invoke 'my-function' component.wasm --verbose
   ```

### Debugging Tools

- `wasmtime validate` - Validate WASM components
- `wasm-tools objdump` - Inspect component structure
- `wit-bindgen generate` - Generate bindings for debugging
- `cargo component check` - Check component configuration

## Future Enhancements

### Planned Features

1. **Component Versioning**
   - Semantic versioning for WIT interfaces
   - Backward compatibility checking
   - Migration tools for interface changes

2. **Advanced Testing**
   - Property-based testing for components
   - Fuzzing with wasmtime
   - Performance benchmarking

3. **Development Tools**
   - WIT interface generator from Rust types
   - Component dependency management
   - Hot reloading for development

4. **Deployment**
   - Component registry and distribution
   - OCI container integration
   - Cloud deployment automation

## Conclusion

The WIT-first architecture provides a solid foundation for building modular, secure, and maintainable Rust applications. By following these patterns and best practices, you can create components that are:

- **Reusable** across different projects
- **Testable** in isolation
- **Secure** with minimal attack surface
- **Performant** with optimized WASM execution
- **Future-proof** with stable interfaces

This architecture aligns with the broader WebAssembly ecosystem and positions your project for long-term success in the evolving landscape of cross-language, component-based development. 
