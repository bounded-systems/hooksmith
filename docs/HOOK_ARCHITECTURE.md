# Hook Architecture: Rust Structs for Dynamic Generation

This document describes the advanced hook architecture that uses Rust structs to define and generate Lefthook configurations dynamically.

## 🎯 **Core Architecture**

### **1. HookConfig Struct**

The foundation of the system is the `HookConfig` struct that defines a single Git hook:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub name: String,                    // Hook name (used as key in lefthook.yml)
    pub stage: String,                   // Git hook stage (pre-commit, pre-push, etc.)
    pub command: String,                 // Command to run
    pub description: String,             // Human-readable description
    pub parallel: bool,                  // Whether this hook can run in parallel
    pub glob: Option<String>,            // File glob pattern for this hook
    pub stage_fixed: bool,               // Whether to stage fixed files
    pub conditional: bool,               // Whether this hook is conditional
    pub required_feature: Option<String>, // Feature required for this hook
}
```

### **2. Builder Pattern**

Hooks are created using a fluent builder pattern:

```rust
let hook = HookConfig::new("format", "pre-commit", "cargo fmt --all -- --check", "Check code formatting")
    .parallel(true)
    .glob("*.rs")
    .stage_fixed(true)
    .conditional("rust_workspace");
```

### **3. HookGenerator**

The `HookGenerator` manages all hooks and generates configurations:

```rust
pub struct HookGenerator {
    hooks: Vec<HookConfig>,
}

impl HookGenerator {
    pub fn new() -> Self {
        Self {
            hooks: Self::default_hooks(),
        }
    }
    
    pub fn add_hook(&mut self, hook: HookConfig) {
        self.hooks.push(hook);
    }
    
    pub fn generate_config(&self, features: &ProjectFeatures) -> Result<String> {
        // Generate YAML based on features
    }
}
```

## 🚀 **Default Hooks**

The system comes with a comprehensive set of default hooks:

### **CLI-Based Hooks (Always Included)**

```rust
HookConfig::new(
    "cli-pre-commit",
    "pre-commit",
    "cargo run --bin pushd-cli -- hooks pre-commit",
    "Run all Rust CLI pre-commit validations",
).parallel(true).glob("*"),

HookConfig::new(
    "cli-pre-push",
    "pre-push",
    "cargo run --bin pushd-cli -- hooks pre-push",
    "Run all Rust CLI pre-push validations",
),
```

### **Rust-Specific Hooks (Conditional)**

```rust
HookConfig::new(
    "format",
    "pre-commit",
    "cargo fmt --all -- --check",
    "Check code formatting",
).parallel(true).glob("*.rs").stage_fixed(true).conditional("rust_workspace"),

HookConfig::new(
    "lint",
    "pre-commit",
    "cargo clippy --workspace --all-targets -- -D warnings",
    "Lint with Clippy",
).parallel(true).glob("*.rs").conditional("rust_workspace"),
```

### **WASM Component Hooks**

```rust
HookConfig::new(
    "wit-validate",
    "pre-commit",
    "cargo component build --release",
    "Validate WIT interfaces",
).glob("wit/*.wit").conditional("wit_interfaces"),
```

### **Safety and Worktree Hooks**

```rust
HookConfig::new(
    "worktree-guard",
    "pre-commit",
    "./scripts/git/hooks/worktree_guard.sh",
    "Enforce worktree safety rules",
).glob("*").conditional("safety_checks"),

HookConfig::new(
    "prevent-main-files",
    "pre-commit",
    "./scripts/git/hooks/prevent_main_files.sh",
    "Prevent creation of files named 'main'",
).glob("*").conditional("safety_checks"),
```

## 🔧 **Feature Detection**

The system uses `ProjectFeatures` to determine which hooks to include:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFeatures {
    pub has_rust_workspace: bool,
    pub has_wasm_components: bool,
    pub has_wit_interfaces: bool,
    pub has_integration_tests: bool,
    pub has_daemons: bool,
    pub has_docker: bool,
    pub has_worktree_management: bool,
    pub has_safety_checks: bool,
    pub has_documentation: bool,
    pub has_commitlint: bool,
}
```

### **Feature Detection Logic**

```rust
async fn analyze_project_features(repo_root: &Path) -> Result<ProjectFeatures> {
    let mut features = ProjectFeatures::default();
    
    // Check for Rust workspace
    let cargo_toml = repo_root.join("Cargo.toml");
    if cargo_toml.exists() {
        features.has_rust_workspace = true;
        
        // Check for components
        let components_dir = repo_root.join("components");
        if components_dir.exists() {
            features.has_wasm_components = true;
            
            let wit_dir = repo_root.join("wit");
            if wit_dir.exists() {
                features.has_wit_interfaces = true;
            }
        }
    }
    
    // Check for other features...
    Ok(features)
}
```

## 📊 **Dynamic Generation**

### **YAML Generation Process**

1. **Filter Hooks**: Only include hooks that match detected features
2. **Group by Stage**: Organize hooks by Git hook stage
3. **Generate YAML**: Create properly formatted lefthook.yml
4. **Add Metadata**: Include generation timestamp and feature info

### **Generated YAML Example**

```yaml
# Lefthook configuration for pushd-cli-prototype
# Auto-generated by pushd-cli -- hooks generate
# This enforces CLI usage and validates the component architecture
# Lefthook is now just a dumb runner - all logic is in the CLI

pre-commit:
  parallel: true
  commands:
    cli-pre-commit:
      run: cargo run --bin pushd-cli -- hooks pre-commit
      glob: "*"

    format:
      run: cargo fmt --all -- --check
      glob: "*.rs"
      stage_fixed: true

    lint:
      run: cargo clippy --workspace --all-targets -- -D warnings
      glob: "*.rs"

    wit-validate:
      run: cargo component build --release
      glob: "wit/*.wit"

    worktree-guard:
      run: ./scripts/git/hooks/worktree_guard.sh
      glob: "*"

pre-push:
  commands:
    cli-pre-push:
      run: cargo run --bin pushd-cli -- hooks pre-push

    integration-tests:
      run: cargo test --package pushd-cli --test integration

# Configuration metadata
# Generated on: 2024-01-15T10:30:00Z
# Features detected:
# - Rust workspace: true
# - WASM components: true
# - WIT interfaces: true
# - Integration tests: true
# - Safety checks: true
```

## 🧪 **Testing**

### **Comprehensive Test Suite**

The system includes extensive tests to ensure reliability:

```rust
#[test]
fn test_hook_config_builder() {
    let hook = HookConfig::new("test-hook", "pre-commit", "echo 'test'", "Test hook")
        .parallel(true)
        .glob("*.rs")
        .stage_fixed(true)
        .conditional("rust_workspace");

    assert_eq!(hook.name, "test-hook");
    assert!(hook.parallel);
    assert_eq!(hook.glob, Some("*.rs".to_string()));
    assert!(hook.conditional);
}

#[test]
fn test_hook_generator_with_rust_workspace() {
    let generator = HookGenerator::new();
    let mut features = ProjectFeatures::default();
    features.has_rust_workspace = true;
    features.has_wit_interfaces = true;
    
    let config = generator.generate_config(&features).unwrap();
    
    // Should include CLI hooks (always included)
    assert!(config.contains("cli-pre-commit"));
    
    // Should include Rust-specific hooks
    assert!(config.contains("format"));
    assert!(config.contains("lint"));
    
    // Should include WIT validation
    assert!(config.contains("wit-validate"));
}
```

### **Test Coverage**

- ✅ Hook configuration builder pattern
- ✅ Default hooks validation
- ✅ Feature-based filtering
- ✅ YAML structure validation
- ✅ Conditional hook logic
- ✅ Parallel execution flags
- ✅ Glob pattern handling
- ✅ Custom hook addition
- ✅ Statistics generation

## 🔄 **Usage Workflow**

### **1. Auto-Installation**

```bash
# Any CLI command triggers auto-installation
pushd-cli --help
# Output: 📌 Auto-installing Lefthook hooks...
# Output: ✅ Hooks auto-installed successfully
```

### **2. Manual Generation**

```bash
# Generate hooks based on current project structure
pushd-cli hooks generate

# Update hooks when project structure changes
pushd-cli hooks update

# Check what features were detected
pushd-cli hooks status
```

### **3. Custom Hooks**

```rust
// Add custom hooks programmatically
let mut generator = HookGenerator::new();

let custom_hook = HookConfig::new(
    "custom-test",
    "pre-commit",
    "echo 'custom test'",
    "Custom test hook"
).glob("*.txt");

generator.add_hook(custom_hook);
```

## 🎯 **Benefits**

### **1. Type Safety**

- **Compile-time validation**: All hook configurations are validated at compile time
- **Structured data**: No string manipulation or YAML parsing errors
- **IDE support**: Full autocomplete and error detection

### **2. Maintainability**

- **Single source of truth**: All hooks defined in Rust code
- **Version controlled**: Hook logic tracked in Git
- **Easy refactoring**: Change hook logic, regenerate everywhere

### **3. Extensibility**

- **Add hooks easily**: Just add a new `HookConfig`
- **Custom features**: Extend `ProjectFeatures` for new detection
- **Plugin system**: Future support for external hook plugins

### **4. Reliability**

- **Consistent generation**: Same logic everywhere
- **No config drift**: Always generated fresh
- **Comprehensive testing**: Full test coverage

## 🚀 **Future Enhancements**

### **1. Component-Based Hooks**

```rust
// Future: WIT components for hooks
#[component]
pub trait HookGenerator {
    fn generate_hooks(&self, features: &ProjectFeatures) -> Vec<HookConfig>;
    fn validate_hooks(&self, hooks: &[HookConfig]) -> Result<()>;
}
```

### **2. Plugin System**

```rust
// Future: External hook plugins
pub trait HookPlugin {
    fn name(&self) -> &str;
    fn hooks(&self) -> Vec<HookConfig>;
    fn features(&self) -> Vec<String>;
}
```

### **3. Advanced Features**

- **Hook dependencies**: Define hook execution order
- **Performance optimization**: Parallel execution strategies
- **Caching**: Cache feature detection results
- **Incremental updates**: Only update changed sections

## 📚 **Integration**

### **CLI Commands**

```bash
# Generate and install hooks
pushd-cli hooks install

# Generate configuration only
pushd-cli hooks generate

# Update existing hooks
pushd-cli hooks update

# Check hook status
pushd-cli hooks status

# Run specific hooks
pushd-cli hooks pre-commit
pushd-cli hooks pre-push
```

### **CI/CD Integration**

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    steps:
      - uses: actions/checkout@v4
      - run: cargo install lefthook-cli
      - run: cargo run --bin pushd-cli -- hooks pre-push
```

This architecture provides a robust, type-safe, and maintainable foundation for dynamic hook generation that scales with project complexity while ensuring consistency across all environments. 
