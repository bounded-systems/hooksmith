# Phase 4: File Type Normalization
## Goal: 100% Rust-Owned Pipeline with ≤8 File Types

### 📊 Current State Analysis

**22 File Extensions** (Too Many!)
```
rs         106  ✅ Rust source (keep)
md         74   🔄 Generate from Rust/xtask
yml/yaml   15   🔄 Generate & mark as codegen
toml       8    🔄 Generate & mark as codegen
json       9    🔄 Generate & mark as codegen
wit        5    🔄 Generate from Rust types
gitattributes* 9 🔄 Generate once, rarely changes
gitignore  2    🔄 Could be generated/templated
shellcheckrc/sh 2 🔄 Replace with Rust xtask subcommands
CODEOWNERS 1    🔄 Could generate CODEOWNERS.md
pdf/html/epub/hbs/css/dot 7 🔄 Generate from Rust schemas
generated_file_demo 1 🔄 Keep as example or remove
```

### 🎯 Target State: 6-8 File Types

**✅ Core Types (Keep)**
- `rs` - Rust source code
- `toml` - Workspace + package manifests
- `yml` - CI/automation configs (optionally generate)
- `md` - Documentation (all other formats generated in CI)
- `wit` - Generated WASM interfaces (optional)

**🔄 Generated Only (Not checked in)**
- `.gitignore` + `.gitattributes` (generated at bootstrap)
- `json` as build artifacts only
- `pdf/html/epub` generated in CI from `.md`

### 📋 Migration Strategy

#### 1. **Bootstrap Script Enhancements**
```rust
// xtask/src/bootstrap.rs
pub async fn generate_project_files() -> Result<()> {
    // Generate all config files from Rust structs
    generate_cargo_toml_files()?;
    generate_git_files()?;
    generate_ci_configs()?;
    generate_codeowners()?;
    
    // Mark as codegen in .gitattributes
    mark_generated_files()?;
    
    Ok(())
}
```

#### 2. **Documentation Pipeline Consolidation**
```rust
// xtask/src/docs/mod.rs
pub async fn generate_all_docs() -> Result<()> {
    // Generate .md from Rust doc comments
    generate_api_docs()?;
    generate_examples()?;
    generate_architecture_docs()?;
    
    // CI will generate PDF/HTML/EPUB from .md
    Ok(())
}
```

#### 3. **Diagram/Asset Generation**
```rust
// xtask/src/diagrams/mod.rs
pub async fn generate_diagrams() -> Result<()> {
    // Convert .dot/.hbs into Rust schema-based generators
    generate_mermaid_diagrams()?;
    generate_plantuml_diagrams()?;
    
    // Output as .md with embedded diagrams
    Ok(())
}
```

#### 4. **Shell → Rust Migration**
```rust
// xtask/src/commands/mod.rs
#[derive(Subcommand)]
pub enum Commands {
    // Replace shell scripts with xtask commands
    Validate { /* was validate.sh */ },
    Setup { /* was setup.sh */ },
    Deploy { /* was deploy.sh */ },
    // ... existing commands
}
```

### 🛠️ Implementation Plan

#### **Phase 4.1: Bootstrap File Generation**
- [ ] Generate `Cargo.toml` files from workspace config
- [ ] Generate `.gitignore` from project patterns
- [ ] Generate `.gitattributes` with codegen markers
- [ ] Generate `CODEOWNERS.md` from team config
- [ ] Generate CI configs (`lefthook.yml`, GitHub Actions)

#### **Phase 4.2: Documentation Consolidation**
- [ ] Convert PDFs/HTML/EPUB to CI-generated from `.md`
- [ ] Generate all `.md` from Rust doc comments
- [ ] Replace `.dot` diagrams with Mermaid in `.md`
- [ ] Replace `.hbs` templates with Rust template engine
- [ ] Remove `.css` files, inline styles in generated HTML

#### **Phase 4.3: Config File Generation**
- [ ] Generate JSON configs from Rust structs with serde
- [ ] Generate YAML configs from Rust structs
- [ ] Mark all generated configs with codegen markers
- [ ] Add validation to prevent manual edits

#### **Phase 4.4: Shell Script Elimination**
- [ ] Move `validate.sh` logic to `cargo xtask validate`
- [ ] Move `setup.sh` logic to `cargo xtask setup`
- [ ] Remove `shellcheckrc` (no more shell scripts)
- [ ] Update CI to use xtask commands instead of shell

#### **Phase 4.5: WASM Interface Generation**
- [ ] Generate `.wit` files from Rust types using wit-bindgen
- [ ] Add validation to ensure `.wit` files match Rust types
- [ ] Mark `.wit` files as generated

### 📈 Progress Tracking Integration

#### **Enhanced xtask status Command**
```rust
// xtask/src/status.rs
pub struct FileTypeStatus {
    pub total_files: usize,
    pub rust_files: usize,
    pub generated_files: usize,
    pub manual_files: usize,
    pub file_types: HashMap<String, FileTypeInfo>,
    pub migration_progress: f64,
}

pub struct FileTypeInfo {
    pub count: usize,
    pub is_generated: bool,
    pub migration_status: MigrationStatus,
    pub target_action: String,
}

pub enum MigrationStatus {
    Keep,           // .rs files
    Generate,       // .md, .toml, .yml
    Remove,         // .sh, .shellcheckrc
    Consolidate,    // .pdf -> .md + CI
}
```

#### **New xtask Commands**
```bash
# Show file type breakdown
cargo xtask status file-types

# Show migration progress
cargo xtask status migration-progress

# Validate no manual edits to generated files
cargo xtask validate-generated

# Generate all project files
cargo xtask bootstrap generate-files
```

### 🎯 Success Metrics

#### **File Type Budget**
- **Target**: ≤8 file types total
- **Current**: 22 file types
- **Progress**: 64% reduction needed

#### **Generation Coverage**
- **Target**: 95% of non-Rust files generated
- **Current**: ~30% generated
- **Progress**: 65% improvement needed

#### **Manual File Count**
- **Target**: ≤10 manual files total
- **Current**: ~50 manual files
- **Progress**: 80% reduction needed

### 🔧 Technical Implementation

#### **File Type Registry**
```rust
// xtask/src/file_types.rs
#[derive(Debug, Clone)]
pub struct FileTypeRegistry {
    pub approved_types: HashSet<String>,
    pub generated_types: HashSet<String>,
    pub deprecated_types: HashSet<String>,
    pub migration_targets: HashMap<String, MigrationTarget>,
}

#[derive(Debug, Clone)]
pub struct MigrationTarget {
    pub target_type: String,
    pub generator: String,
    pub priority: u8,
    pub estimated_effort: String,
}
```

#### **Validation Rules**
```rust
// xtask/src/validation.rs
pub async fn validate_file_types() -> Result<()> {
    let registry = FileTypeRegistry::load()?;
    let violations = find_file_type_violations(&registry).await?;
    
    if !violations.is_empty() {
        anyhow::bail!("File type violations found: {:?}", violations);
    }
    
    Ok(())
}
```

### 📊 Migration Timeline

#### **Week 1-2: Foundation**
- [ ] Implement file type registry
- [ ] Enhance xtask status with file type tracking
- [ ] Create bootstrap file generators

#### **Week 3-4: Documentation**
- [ ] Convert PDFs/HTML/EPUB to CI-generated
- [ ] Implement Rust-based doc generation
- [ ] Replace diagram files with Mermaid

#### **Week 5-6: Configuration**
- [ ] Generate JSON/YAML from Rust structs
- [ ] Implement config validation
- [ ] Mark all configs as generated

#### **Week 7-8: Shell Elimination**
- [ ] Move shell scripts to xtask commands
- [ ] Update CI pipelines
- [ ] Remove shell-related files

#### **Week 9-10: Validation & Polish**
- [ ] Implement strict validation
- [ ] Add CI enforcement
- [ ] Update documentation

### 🚀 Expected Outcomes

#### **Immediate Benefits**
- **Reduced complexity**: 22 → 8 file types
- **Better maintainability**: Everything generated from Rust
- **Consistent tooling**: All operations via xtask
- **Improved CI**: Faster, more reliable builds

#### **Long-term Benefits**
- **100% Rust-owned pipeline**: No external dependencies
- **Self-documenting**: All configs derived from code
- **Version consistency**: Generated files always match code
- **Easier onboarding**: Single toolchain (Rust + xtask)

### 🔄 Integration with Existing Systems

#### **Status System Enhancement**
```rust
// Extend existing status command
Commands::Status {
    #[command(subcommand)]
    command: StatusCommands,
}

#[derive(Subcommand)]
pub enum StatusCommands {
    Report { /* existing */ },
    Badge { /* existing */ },
    Trend { /* existing */ },
    FileTypes { /* new */ },
    MigrationProgress { /* new */ },
}
```

#### **Validation Integration**
```rust
// Add to existing validation pipeline
pub async fn run_validation() -> Result<()> {
    validate_contracts()?;
    validate_schemas()?;
    validate_generated_files()?;
    validate_file_types()?; // New
    Ok(())
}
```

### 📝 Next Steps

1. **Review and approve** this Phase 4 plan
2. **Implement file type registry** in xtask
3. **Enhance status system** with file type tracking
4. **Begin bootstrap file generation** implementation
5. **Start documentation consolidation** process

This plan will transform Hooksmith into a truly Rust-centric project with minimal file type complexity while maintaining all functionality through generated artifacts and inline Rust logic. 
