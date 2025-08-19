# Refactoring Plan: From WIT-First to Hybrid Architecture

## 🚨 **Current Problems**

The current implementation incorrectly tries to make system operations into WIT components:

### **❌ Current Incorrect Components**
- `hook-builder` - Tries to do file I/O and Git operations in WASM
- `worktree-runner` - Tries to manage Git worktrees in WASM
- `git-filter` - Tries to do file filtering in WASM
- `validation-handler` - Tries to do file validation in WASM

### **❌ Why This Is Wrong**
- **WASM components cannot access the file system** (except through WASI)
- **WASM components cannot execute Git commands**
- **WASM components cannot manage processes**
- **WASM components are sandboxed by design**

## ✅ **Correct Hybrid Architecture**

### **WIT Components (Pure Computation Only)**
```
crates/components/
├── validation-engine/      # ✅ Pure validation logic
├── contract-checker/       # ✅ Contract validation algorithms
├── checksum-calculator/    # ✅ Checksum computation
├── policy-evaluator/       # ✅ Policy evaluation
└── data-transformer/       # ✅ Data transformation
```

### **Native Rust Crates (System Operations)**
```
crates/
├── cli-core/              # ✅ CLI framework and utilities
├── git-operations/        # ✅ Git operations (commit, push, etc.)
├── file-system/           # ✅ File I/O operations
├── process-manager/       # ✅ Process orchestration
└── network-client/        # ✅ Network operations
```

## 🔄 **Refactoring Steps**

### **Step 1: Audit Current Components**

#### **hook-builder** → Split into Native + WIT
```rust
// ❌ Current: Tries to do everything in WASM
impl hook_builder::HookBuilder for HookBuilderComponent {
    fn build_hook(config: BuildConfig) -> Result<BuildResult, String> {
        // ❌ This tries to do file I/O and Git operations
        let source = read_file(&config.source_path)?;  // ❌ Can't do this in WASM
        let binary = compile_source(&source)?;         // ❌ Can't do this in WASM
        write_file(&config.output_path, &binary)?;     // ❌ Can't do this in WASM
        Ok(result)
    }
}

// ✅ Correct: Split into Native + WIT
// Native Rust (crates/git-operations/src/hook_builder.rs)
pub async fn build_hook(config: BuildConfig) -> Result<BuildResult> {
    // ✅ Native Rust handles file I/O
    let source = std::fs::read_to_string(&config.source_path)?;
    
    // ✅ Load WIT component for pure computation
    let validator = load_component("validation-engine.wasm")?;
    let is_valid = validator.validate_source(&source)?;
    
    if !is_valid {
        return Err("Source validation failed".into());
    }
    
    // ✅ Native Rust handles compilation
    let binary = compile_source(&source)?;
    std::fs::write(&config.output_path, &binary)?;
    
    Ok(BuildResult { success: true, binary_path: Some(config.output_path) })
}

// WIT Component (crates/components/validation-engine/src/lib.rs)
impl validation_engine::ValidationEngine for ValidationEngineComponent {
    fn validate_source(source: String) -> Result<bool, String> {
        // ✅ Pure computation only
        let parsed = parse_rust_source(&source)?;
        let rules = get_validation_rules();
        let result = apply_rules(&parsed, &rules);
        Ok(result.is_valid)
    }
}
```

#### **worktree-runner** → Split into Native + WIT
```rust
// ❌ Current: Tries to manage Git worktrees in WASM
impl worktree_runner::WorktreeRunner for WorktreeRunnerComponent {
    fn create_worktree(branch_name: String) -> Result<WorktreeResult, String> {
        // ❌ This tries to execute Git commands
        let output = execute_git_command(&["worktree", "add", &branch_name])?;  // ❌ Can't do this in WASM
        Ok(WorktreeResult { success: true, output })
    }
}

// ✅ Correct: Split into Native + WIT
// Native Rust (crates/git-operations/src/worktree.rs)
pub async fn create_worktree(branch_name: &str) -> Result<WorktreeResult> {
    // ✅ Native Rust executes Git commands
    let output = std::process::Command::new("git")
        .args(["worktree", "add", branch_name])
        .output()?;
    
    // ✅ Load WIT component for result processing
    let processor = load_component("data-transformer.wasm")?;
    let processed_result = processor.process_git_output(&output.stdout)?;
    
    Ok(WorktreeResult { 
        success: output.status.success(),
        output: processed_result
    })
}

// WIT Component (crates/components/data-transformer/src/lib.rs)
impl data_transformer::DataTransformer for DataTransformerComponent {
    fn process_git_output(output: String) -> Result<String, String> {
        // ✅ Pure computation only
        let lines: Vec<&str> = output.lines().collect();
        let filtered = lines.iter()
            .filter(|line| !line.contains("warning"))
            .collect::<Vec<_>>();
        Ok(filtered.join("\n"))
    }
}
```

#### **git-filter** → Split into Native + WIT
```rust
// ❌ Current: Tries to do file filtering in WASM
impl git_filter::GitFilter for GitFilterComponent {
    fn validate_blob(blob_content: String, config: FilterConfig) -> Result<FilterResult, String> {
        // ❌ This tries to do file operations
        let file_path = extract_file_path(&blob_content)?;  // ❌ Can't access files in WASM
        let content = read_file(&file_path)?;               // ❌ Can't do this in WASM
        Ok(result)
    }
}

// ✅ Correct: Split into Native + WIT
// Native Rust (crates/file-system/src/git_filter.rs)
pub async fn validate_blob(blob_content: &str, config: FilterConfig) -> Result<FilterResult> {
    // ✅ Native Rust handles file operations
    let file_path = extract_file_path(blob_content)?;
    let content = std::fs::read_to_string(&file_path)?;
    
    // ✅ Load WIT component for validation logic
    let validator = load_component("validation-engine.wasm")?;
    let validation_result = validator.validate_content(&content, &config.rules)?;
    
    Ok(FilterResult {
        success: validation_result.is_valid,
        content: Some(content),
        error: validation_result.error,
        details: validation_result.messages
    })
}

// WIT Component (crates/components/validation-engine/src/lib.rs)
impl validation_engine::ValidationEngine for ValidationEngineComponent {
    fn validate_content(content: String, rules: Vec<String>) -> Result<ValidationResult, String> {
        // ✅ Pure computation only
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        for rule in rules {
            match apply_validation_rule(&content, &rule) {
                Ok(_) => {},
                Err(ValidationError::Error(msg)) => errors.push(msg),
                Err(ValidationError::Warning(msg)) => warnings.push(msg),
            }
        }
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            error: if errors.is_empty() { None } else { Some(errors.join("; ")) },
            messages: warnings
        })
    }
}
```

#### **validation-handler** → Split into Native + WIT
```rust
// ❌ Current: Tries to do file validation in WASM
impl validation_handler::ValidationHandler for ValidationHandlerComponent {
    fn validate_file(file_path: String, config: ValidationConfig) -> Result<ValidationResult, String> {
        // ❌ This tries to read files in WASM
        let content = read_file(&file_path)?;  // ❌ Can't do this in WASM
        Ok(result)
    }
}

// ✅ Correct: Split into Native + WIT
// Native Rust (crates/file-system/src/validation.rs)
pub async fn validate_file(file_path: &str, config: ValidationConfig) -> Result<ValidationResult> {
    // ✅ Native Rust reads the file
    let content = std::fs::read_to_string(file_path)?;
    
    // ✅ Load WIT component for validation logic
    let validator = load_component("validation-engine.wasm")?;
    let validation_result = validator.validate_content(&content, &config.rules)?;
    
    Ok(validation_result)
}

// WIT Component (crates/components/validation-engine/src/lib.rs)
impl validation_engine::ValidationEngine for ValidationEngineComponent {
    fn validate_content(content: String, rules: Vec<ValidationRule>) -> Result<ValidationResult, String> {
        // ✅ Pure computation only
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut messages = Vec::new();
        
        for rule in rules {
            if rule.enabled {
                match apply_rule(&content, &rule) {
                    Ok(_) => messages.push(format!("Rule '{}' passed", rule.name)),
                    Err(ValidationError::Error(_)) => {
                        error_count += 1;
                        messages.push(format!("Rule '{}' failed", rule.name));
                    },
                    Err(ValidationError::Warning(_)) => {
                        warning_count += 1;
                        messages.push(format!("Rule '{}' warning", rule.name));
                    }
                }
            }
        }
        
        Ok(ValidationResult {
            success: error_count == 0,
            error_count,
            warning_count,
            messages,
            details: Some(format!("Validated with {} rules", rules.len()))
        })
    }
}
```

### **Step 2: Create New Native Rust Crates**

#### **crates/git-operations/Cargo.toml**
```toml
[package]
name = "git-operations"
version.workspace = true
edition.workspace = true

[dependencies]
git2.workspace = true
tokio.workspace = true
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
```

#### **crates/file-system/Cargo.toml**
```toml
[package]
name = "file-system"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
walkdir.workspace = true
```

#### **crates/process-manager/Cargo.toml**
```toml
[package]
name = "process-manager"
version.workspace = true
edition.workspace = true

[dependencies]
tokio.workspace = true
anyhow.workspace = true
serde.workspace = true
serde_json.workspace = true
which.workspace = true
```

### **Step 3: Create New WIT Components**

#### **crates/components/validation-engine/wit/validation-engine.wit**
```wit
package hooksmith:validation-engine;

/// Validation rule
record validation-rule {
  name: string,
  description: string,
  enabled: bool,
  severity: validation-severity,
}

/// Validation severity
enum validation-severity {
  info,
  warning,
  error,
  critical,
}

/// Validation result
record validation-result {
  success: bool,
  error-count: u32,
  warning-count: u32,
  messages: list<string>,
  details: option<string>,
}

/// Validation engine interface
interface validation-engine {
  /// Validate content against rules
  validate-content: func(content: string, rules: list<validation-rule>) -> result<validation-result, string>;
  
  /// Validate source code
  validate-source: func(source: string) -> result<bool, string>;
  
  /// Apply single validation rule
  apply-rule: func(content: string, rule: validation-rule) -> result<bool, string>;
}

export validation-engine;
```

#### **crates/components/contract-checker/wit/contract-checker.wit**
```wit
package hooksmith:contract-checker;

/// Contract validation result
record contract-result {
  is-valid: bool,
  errors: list<string>,
  warnings: list<string>,
  details: option<string>,
}

/// Contract checker interface
interface contract-checker {
  /// Validate contract
  validate-contract: func(content: string) -> result<contract-result, string>;
  
  /// Check contract against rules
  check-contract: func(content: string, rules: list<string>) -> result<bool, string>;
  
  /// Parse contract
  parse-contract: func(content: string) -> result<string, string>;
}

export contract-checker;
```

### **Step 4: Create Integration Layer**

#### **src/wasm_runner.rs**
```rust
use wasmtime::{Engine, Store, Config};
use wasmtime::component::{Component, Linker};

pub struct WasmRunner {
    engine: Engine,
    store: Store<()>,
}

impl WasmRunner {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        
        let engine = Engine::new(&config)?;
        let store = Store::new(&engine, ());
        
        Ok(Self { engine, store })
    }
    
    pub async fn load_component(&mut self, path: &str) -> Result<ComponentInstance> {
        let component = Component::from_file(&self.engine, path)?;
        let linker = Linker::new(&self.engine);
        let (instance, _) = linker.instantiate(&mut self.store, &component)?;
        
        Ok(ComponentInstance { instance, store: &mut self.store })
    }
}

pub struct ComponentInstance<'a> {
    instance: wasmtime::component::Instance,
    store: &'a mut Store<()>,
}

impl<'a> ComponentInstance<'a> {
    pub fn call_validation_engine(&mut self, content: &str, rules: &[ValidationRule]) -> Result<ValidationResult> {
        // Call WIT component for pure validation
        let func = self.instance.get_func(self.store, "validate-content")?;
        let result = func.call(self.store, &[content, rules], &mut [])?;
        
        // Convert result back to Rust types
        Ok(result.into())
    }
}
```

### **Step 5: Update CLI Integration**

#### **src/commands/contract_validation.rs**
```rust
use crate::wasm_runner::WasmRunner;
use git_operations::GitOperations;
use file_system::FileSystem;

pub async fn validate_contract(file_path: &str) -> Result<()> {
    // ✅ Native Rust reads the file
    let content = file_system::read_to_string(file_path).await?;
    
    // ✅ Load WIT component for validation
    let mut runner = WasmRunner::new()?;
    let validator = runner.load_component("validation-engine.wasm").await?;
    
    // ✅ WIT component performs pure validation
    let rules = load_validation_rules()?;
    let result = validator.call_validation_engine(&content, &rules)?;
    
    // ✅ Native Rust handles the result
    if !result.success {
        let report = generate_validation_report(&result)?;
        file_system::write("validation-report.json", &report).await?;
        
        if result.error_count > 0 {
            git_operations::commit("Add validation report").await?;
        }
    }
    
    Ok(())
}
```

## 🎯 **Migration Timeline**

### **Week 1: Audit and Plan**
- [ ] Audit all current components
- [ ] Identify pure computation vs. system operations
- [ ] Create detailed refactoring plan
- [ ] Set up new crate structure

### **Week 2: Create Native Crates**
- [ ] Create `crates/git-operations/`
- [ ] Create `crates/file-system/`
- [ ] Create `crates/process-manager/`
- [ ] Move system operations to native crates

### **Week 3: Create WIT Components**
- [ ] Create `crates/components/validation-engine/`
- [ ] Create `crates/components/contract-checker/`
- [ ] Create `crates/components/checksum-calculator/`
- [ ] Extract pure computation to WIT components

### **Week 4: Integration Layer**
- [ ] Create `src/wasm_runner.rs`
- [ ] Update CLI commands
- [ ] Add comprehensive testing
- [ ] Update documentation

### **Week 5: Testing and Optimization**
- [ ] Comprehensive testing of hybrid system
- [ ] Performance optimization
- [ ] Error handling improvements
- [ ] Documentation updates

## 🎉 **Expected Results**

After refactoring:

### **✅ Correct Architecture**
- **WIT components** handle only pure computation
- **Native Rust** handles all system operations
- **Clear boundaries** between sandboxed and system code

### **✅ Better Performance**
- **Native I/O** for file and network operations
- **Optimized computation** in WASM components
- **Efficient integration** between layers

### **✅ Improved Security**
- **Sandboxed components** cannot access system
- **Controlled system access** through native Rust
- **Clear security boundaries**

### **✅ Enhanced Maintainability**
- **Testable components** (pure functions in WIT)
- **Flexible deployment** (components can be distributed)
- **Language agnostic** (WIT components in any language)

## 🚀 **Next Steps**

1. **Review this plan** and provide feedback
2. **Start with Week 1** - audit current components
3. **Create new crate structure** for native operations
4. **Extract pure computation** to WIT components
5. **Implement integration layer** between WIT and native
6. **Test thoroughly** and optimize performance

This refactoring will transform Hooksmith from an incorrectly architected "everything as WIT" system into a properly designed hybrid architecture that follows WebAssembly best practices. 
