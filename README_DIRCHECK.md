# DirCheck - Minimal CI-Optimized Directory Validation

A fast, minimal Rust CLI for validating directory and file structure in Git repositories. Designed for CI/CD with minimal boot time and zero dependencies in the core validation logic.

## 🚀 Features

- **Minimal Boot Time**: Zero dependencies in core validation logic
- **Single Git Command**: One `git ls-tree` or `git ls-files` call per validation
- **CI Optimized**: Fast compilation and execution for GitHub Actions
- **Declarative Rules**: YAML-based configuration for easy customization
- **Structured Output**: Clear violation reporting with suggestions

## 📦 Architecture

```
dircheck/
├── crates/
│   ├── core/           # Zero-dependency validation logic
│   ├── tree/           # dircheck-tree binary
│   └── files/          # dircheck-files binary
├── config/
│   └── rules.yml       # Declarative rule configuration
├── hooks/              # Git hooks for local validation
└── .github/workflows/  # GitHub Actions integration
```

## 🛠️ Usage

### Local Development

```bash
# Validate directory structure
cargo run -p dircheck-tree

# Validate file structure  
cargo run -p dircheck-files

# Build optimized release binaries
cargo build --release -p dircheck-tree
cargo build --release -p dircheck-files
```

### Git Hooks

```bash
# Install hooks
chmod +x hooks/pre-commit
chmod +x hooks/pre-push
cp hooks/pre-commit .git/hooks/
cp hooks/pre-push .git/hooks/
```

### GitHub Actions

The `.github/workflows/validate.yml` workflow automatically runs validation on:
- Push to main/develop branches
- Pull requests to main/develop branches

## ⚙️ Configuration

### Default Rules

The system includes sensible defaults for Rust projects:

**Directory Structure:**
- Allowed root dirs: `src`, `docs`, `crates`, `examples`, `tests`, etc.
- Forbidden dirs: `node_modules`, `vendor`, `tmp`, `temp`
- Required dirs: `src`

**File Structure:**
- Forbidden root extensions: `.md`, `.toml`, `.rs`
- Allowed extensions by directory (e.g., `.rs` only in `src/`)
- Required files: `Cargo.toml`, `README.md`

### Custom Rules

Edit `config/rules.yml` to customize validation rules:

```yaml
tree_rules:
  allowed_root_dirs:
    - src
    - docs
    # Add your directories...

file_rules:
  forbidden_root_extensions:
    - .md
    - .toml
    # Add forbidden extensions...
```

## 🔧 Performance Optimizations

### Build Configuration

```toml
[profile.dev]
opt-level = 1
debug = false
incremental = true

[profile.release]
opt-level = "z"
strip = "debuginfo"
lto = true
codegen-units = 1
panic = "abort"
```

### CI Integration

For GitHub Actions with act:

```yaml
# .github/workflows/validate.yml
- name: Setup Rust toolchain
  uses: dtolnay/rust-toolchain@stable

- name: Validate directory structure
  run: cargo run -p dircheck-tree

- name: Validate file structure
  run: cargo run -p dircheck-files
```

## 🧪 Testing

```bash
# Test with current repository
cargo run -p dircheck-tree
cargo run -p dircheck-files

# Build release binaries for fast execution
cargo build --release -p dircheck-tree
./target/release/dircheck-tree
```

## 📊 Output Format

### Success
```
✅ All directory structure rules passed
```

### Violations
```
❌ Found 3 directory structure violations:
  Rule: allowed_root_dirs
  Path: temp_file.md
  Error: Root directory 'temp_file.md' is not in allowed list
  Suggestion: Add 'temp_file.md' to allowed_root_dirs or remove it
```

## 🎯 Design Principles

1. **Single Pass**: Each Git command runs exactly once
2. **Minimal Dependencies**: Core logic has zero external dependencies
3. **Fast Boot**: Optimized for CI execution speed
4. **Declarative**: Rules defined in YAML, not code
5. **Extensible**: Easy to add new validation rules

## 🔄 Integration with Act

For local CI testing with act:

```bash
# Install act
brew install act

# Run GitHub Actions locally
act push

# Use rust:slim image for faster builds
act push --container-architecture linux/amd64
```

## 📈 Performance Metrics

- **Compile Time**: ~1s for core crate
- **Boot Time**: ~10ms for validation logic
- **Memory Usage**: <5MB per binary
- **Git Command**: Single subprocess call per validation

## 🛡️ Error Handling

- Graceful Git command failure handling
- Clear error messages with suggestions
- Non-zero exit codes for CI integration
- Structured violation reporting

## 🔧 Development

```bash
# Add new validation rule
# 1. Edit crates/core/src/lib.rs
# 2. Add rule to TreeRuleSet or FileRuleSet
# 3. Implement validation logic
# 4. Update config/rules.yml

# Test changes
cargo test -p dircheck-core
cargo run -p dircheck-tree
cargo run -p dircheck-files
```

## 📝 License

MIT License - see LICENSE file for details.
