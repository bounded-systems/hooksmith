# Repository Structure Validation

## 🎯 **Overview**

Hooksmith includes a comprehensive repository structure validation system that ensures your project follows the expected layout and conventions. This system uses JSONC schemas to define the expected structure and provides automated validation through the `xtask` CLI.

## 📋 **What It Validates**

### **1. Workspace Structure**
- ✅ **Required files**: `Cargo.toml`, `rust-toolchain.toml`, `lefthook.yml`, `.cargo/config.toml`, etc.
- ✅ **Required directories**: `crates/`, `docs/`, `schemas/`, `generated-sources/`, etc.
- ✅ **Naming patterns**: Config files only in `config/`, schema files only in `schemas/`, etc.

### **2. Component Crates**
- ✅ **Location**: Must be in `crates/components/`
- ✅ **Required files**: `Cargo.toml`, `src/lib.rs`
- ✅ **Required directories**: `src/`, `wit/`
- ✅ **Cargo.toml metadata**: Must have `[package.metadata.component]` section
- ✅ **WIT files**: Must have `.wit` files in the `wit/` directory

### **3. CLI Crates**
- ✅ **Location**: Can be in `src/` or `crates/lefthook-rs/`
- ✅ **Required files**: `Cargo.toml`, `src/main.rs`
- ✅ **Forbidden directories**: Cannot have `wit/` directory

### **4. Documentation**
- ✅ **Required files**: `docs/README.md`, `docs/ARCHITECTURE.md`, `docs/PROJECT_STRUCTURE.md`
- ✅ **Component docs**: Each component should have docs in `docs/components/`
- ✅ **Design docs**: Architecture docs in `docs/design/`

### **5. Generated Sources**
- ✅ **Location**: Must be in `generated-sources/`
- ✅ **Registry tracking**: Files must be tracked in `config/generated-files.jsonc`
- ✅ **Checksums**: Files must have checksums in `config/manual-files.jsonc`
- ✅ **File extensions**: Only allowed extensions (`.jsonc`, `.json`, `.md`, etc.)

### **6. Examples**
- ✅ **Location**: Must be in `examples/`
- ✅ **File extensions**: Only `.rs` files
- ✅ **Naming patterns**: Must follow expected patterns

### **7. Tests**
- ✅ **Location**: Must be in `tests/`
- ✅ **File extensions**: Only `.rs` files
- ✅ **Naming patterns**: Integration tests (`*_test.rs`) or unit tests (`integration.rs`)

### **8. Hooks**
- ✅ **Location**: Must be in `hooks/`
- ✅ **File extensions**: Only `.rs` files
- ✅ **Executable**: Files should be executable (Unix systems)

### **9. Scripts**
- ✅ **Location**: Must be in `scripts/`
- ✅ **File extensions**: Only `.rs` files
- ✅ **Naming patterns**: Simple scripts (`*_simple.rs`) or full binaries (`*.rs`)

## 🚀 **Usage**

### **Basic Validation**
```bash
# Validate repository structure
cargo run --bin xtask -- validate-structure

# Show detailed output
cargo run --bin xtask -- validate-structure --verbose

# Exit with error on validation failures
cargo run --bin xtask -- validate-structure --strict
```

### **Output Formats**
```bash
# Text output (default)
cargo run --bin xtask -- validate-structure --format text

# JSON output
cargo run --bin xtask -- validate-structure --format json

# Summary output
cargo run --bin xtask -- validate-structure --format summary
```

### **CI/CD Integration**
```bash
# Use in CI/CD pipelines
cargo run --bin xtask -- validate-structure --strict --format summary
```

## 📊 **Schema Definition**

The validation rules are defined in `schemas/repo-structure.schema.jsonc`:

```jsonc
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://hooksmith.dev/schemas/repo-structure.schema.jsonc",
  "title": "Hooksmith Repository Structure Schema",
  "description": "Schema defining the expected structure and validation rules for the Hooksmith repository",
  "type": "object",
  "properties": {
    "workspace": {
      "type": "object",
      "description": "Top-level workspace structure and requirements",
      "properties": {
        "requiredFiles": {
          "type": "array",
          "description": "Critical files that must be present at the workspace root",
          "default": [
            "Cargo.toml",
            "rust-toolchain.toml",
            "lefthook.yml",
            ".cargo/config.toml",
            "clippy.toml",
            "deny.toml",
            "CODEOWNERS"
          ]
        },
        "requiredDirectories": {
          "type": "array",
          "description": "Directories that must be present at the workspace root",
          "default": [
            "crates/",
            "docs/",
            "schemas/",
            "generated-sources/",
            "config/",
            "src/",
            "examples/",
            "tests/",
            "hooks/",
            "scripts/",
            "wit/"
          ]
        }
      }
    }
  }
}
```

## 🔧 **Customization**

### **Modifying Validation Rules**

1. **Edit the schema**: Modify `schemas/repo-structure.schema.jsonc`
2. **Update the validator**: Modify `crates/xtask/src/repo_structure_validator.rs`
3. **Test changes**: Run validation to ensure your changes work

### **Adding New Validation Categories**

1. **Add to schema**: Add new properties to the JSONC schema
2. **Add to validator**: Implement validation logic in the Rust validator
3. **Add to CLI**: Update the command handler in `crates/xtask/src/main.rs`

### **Example: Adding Custom File Validation**

```rust
// In repo_structure_validator.rs
fn validate_custom_files(&self, result: &mut ValidationResult) -> Result<()> {
    // Your custom validation logic here
    Ok(())
}
```

## 📈 **Benefits**

### **Consistency**
- ✅ **Enforces standards**: Ensures all projects follow the same structure
- ✅ **Prevents drift**: Catches structural issues before they become problems
- ✅ **Documentation**: Schema serves as living documentation of expected layout

### **Automation**
- ✅ **CI/CD integration**: Automatically validate structure in pipelines
- ✅ **Pre-commit hooks**: Catch issues before they're committed
- ✅ **Developer feedback**: Immediate feedback on structural issues

### **Maintainability**
- ✅ **Clear expectations**: Everyone knows what the structure should be
- ✅ **Easy onboarding**: New contributors can understand the layout quickly
- ✅ **Tooling support**: IDEs and tools can understand the structure

## 🚨 **Troubleshooting**

### **Common Issues**

#### **Missing Required Files**
```
❌ Required workspace file not found: Cargo.toml
```
**Solution**: Ensure all required files are present in the workspace root.

#### **Component Crate Issues**
```
❌ Missing required metadata section: package.metadata.component
```
**Solution**: Add the required metadata section to component crate `Cargo.toml` files.

#### **WIT File Issues**
```
⚠️ No WIT files found in WIT directory
```
**Solution**: Ensure component crates have `.wit` files in their `wit/` directory.

#### **Generated Sources Issues**
```
❌ Generated sources registry file not found
```
**Solution**: Ensure `config/generated-files.jsonc` exists and tracks generated files.

### **Validation Errors vs Warnings**

- **Errors**: Critical issues that must be fixed (e.g., missing required files)
- **Warnings**: Issues that should be addressed but don't break validation (e.g., missing optional documentation)

### **Strict Mode**

Use `--strict` flag to exit with error code when validation fails:
```bash
cargo run --bin xtask -- validate-structure --strict
```

## 🔄 **Integration with Other Tools**

### **Lefthook Integration**
```yaml
# lefthook.yml
pre-commit:
  validate-structure:
    run: cargo run --bin xtask -- validate-structure --strict
```

### **Git Hooks**
```bash
#!/bin/sh
# .git/hooks/pre-commit
cargo run --bin xtask -- validate-structure --strict
```

### **CI/CD Pipelines**
```yaml
# .github/workflows/validate.yml
- name: Validate Repository Structure
  run: cargo run --bin xtask -- validate-structure --strict --format summary
```

## 📚 **Related Documentation**

- [WIT-First Architecture](./WIT_FIRST_ARCHITECTURE.md)
- [Project Structure](./PROJECT_STRUCTURE.md)
- [Component Development](./components/README.md)
- [Generated Files System](./GENERATED_FILE_VALIDATION.md)

## 🎉 **Conclusion**

The repository structure validation system ensures your Hooksmith project maintains a consistent, well-organized layout that follows best practices for WIT-first, minimal-host architectures. By validating structure automatically, you can focus on building great components rather than worrying about organizational issues. 