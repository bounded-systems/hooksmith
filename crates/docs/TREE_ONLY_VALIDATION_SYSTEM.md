# Tree-Only Validation System

## Overview

This document describes the comprehensive tree-only validation system implemented for enforcing a minimal, clean repository root structure. The system provides deterministic, Git-only validation without requiring a full working tree checkout.

## Architecture

### Core Components

1. **Tree Directory Checker** (`scripts/tree_dircheck.rs`)
   - Validates only top-level entries using `git ls-tree --name-only HEAD`
   - No recursion, no deep traversal
   - Simple allow/reject patterns with glob support

2. **Production Agreement Validator** (`scripts/dircheck.rs`)
   - End-to-end validation: schema → subject → decision → digest
   - Uses libgit2 for pure Git operations
   - Computes deterministic digests over subject + rules
   - Optional signature verification support

3. **Tree Cleanup Tool** (`crates/tools/tree_cleanup.rs`)
   - High-impact, low-churn file reorganization
   - Git-aware operations (uses `git mv` for tracked files)
   - Creates canonical directory structure

4. **Configuration Files**
   - `config/dircheck.tree.yml` - Tree-mode configuration
   - `agreement.json` - Tree-only agreement with digest
   - `schemas/dir-agreement.schema.json` - JSON Schema validation

## Key Features

### 🎯 Tree-Only Validation
- **No recursion**: Only validates top-level entries
- **Deterministic**: Uses `git ls-tree --name-only HEAD` (no `-r` flag)
- **Git-only**: No working tree required, pure Git object inspection
- **Simple patterns**: Literal names or root-glob patterns (`*.ext`)

### 🔒 Agreement-Based Enforcement
- **Schema validation**: JSON Schema ensures contract structure
- **Subject materialization**: Extracts actual tree state from Git
- **Decision validation**: Applies allow/reject rules deterministically
- **Digest verification**: SHA-256 over subject + canonicalized rules
- **Signature support**: Optional GPG/Minisign verification

### 🚀 CI/CD Integration
- **GitHub Actions**: Automated validation on PRs and pushes
- **SARIF output**: Integration with GitHub Code Scanning
- **Fail-fast**: Immediate feedback on violations
- **Tree structure display**: Shows current state for debugging

## Usage Examples

### Basic Tree Validation
```bash
# Validate current HEAD against tree config
cargo run --bin tree_dircheck config/dircheck.tree.yml

# Validate against agreement with digest verification
cargo run --bin dircheck -- --agreement agreement.json --ref HEAD --ci
```

### Tree Cleanup
```bash
# Run high-impact cleanup (creates canonical structure)
cargo run --bin tree_cleanup

# Review changes before committing
git status
git diff --cached
```

### CI Integration
```yaml
# .github/workflows/validate-tree-agreement.yml
- name: Validate tree-only agreement
  run: |
    cd scripts
    cargo run --bin dircheck \
      --agreement ../agreement.json \
      --ref ${{ github.sha }} \
      --ci
```

## Configuration

### Tree Mode Configuration (`config/dircheck.tree.yml`)
```yaml
version: 1
mode: tree               # Only top-level entries
default: reject
precedence: allow-overrides-reject

allow_dirs:
  - .cargo
  - .github
  - .hooksmith
  - crates
  - docs
  - examples
  - generated
  - hooks
  - schemas
  - scripts
  - tests
  - tools
  - wit
  - worktree

allow_files:
  - README.md
  - Cargo.toml
  - rust-toolchain.toml
  - rustfmt.toml
  - clippy.toml
  - deny.toml
  - .gitignore
  - .gitattributes
  - "*.md"             # Root-only markdown reports
```

### Agreement Structure (`agreement.json`)
```json
{
  "version": "1.0.0",
  "mode": "tree",
  "precedence": "allow-overrides-reject",
  "default-action": "reject",
  "allow-dirs": [...],
  "allow-files": [...],
  "subject": {
    "ref": "HEAD",
    "scope": "top-level"
  },
  "digest": {
    "algo": "sha256",
    "value": "40bac063a665d5bcccdc9d137bcc64a5ab1ba0613675b4e4f1f6ee75d6a73b96"
  }
}
```

## Validation Process

### 1. Schema Validation
- Validates agreement.json against JSON Schema
- Ensures required fields and correct types
- Enforces tree-only mode constraints

### 2. Subject Materialization
- Uses libgit2 to extract tree from specified ref
- Walks tree in pre-order, collecting only root entries
- Separates files and directories
- Ensures no slashes in names (tree-only invariant)

### 3. Decision Validation
- **Directories**: Exact name matching against allow list
- **Files**: Literal match or root-glob pattern matching
- **Precedence**: allow-overrides-reject (default: reject)
- **CI mode**: Fail-fast on first violation

### 4. Digest Verification
- **Subject**: Sorted, newline-joined entry names
- **Rules**: Canonicalized JSON (sorted keys)
- **Algorithm**: SHA-256(subject) + SHA-256(rules) → SHA-256
- **Comparison**: Computed vs. recorded digest

## Benefits

### 🎯 **Deterministic Validation**
- Same input always produces same result
- No file system dependencies
- Pure Git object inspection

### 🔒 **Strong Enforcement**
- Schema validation prevents contract drift
- Digest verification ensures integrity
- CI integration prevents violations

### 🚀 **Developer Experience**
- Clear error messages with specific violations
- Tree structure display for debugging
- Automated cleanup tools

### 📊 **Compliance Tracking**
- SARIF output for GitHub integration
- Digest changes track structural evolution
- Optional cryptographic signatures

## Migration Path

### Phase 1: Setup
1. Create tree configuration (`config/dircheck.tree.yml`)
2. Generate initial agreement with computed digest
3. Set up CI validation workflow

### Phase 2: Cleanup
1. Run tree cleanup tool (`crates/tools/tree_cleanup.rs`)
2. Review and commit structural changes
3. Update agreement digest

### Phase 3: Enforcement
1. Enable required status checks
2. Monitor compliance in CI
3. Iterate on configuration as needed

## Future Enhancements

### 🔐 **Signature Verification**
- GPG integration for agreement signing
- Minisign support for lightweight signatures
- Key management and rotation

### 📈 **Analytics**
- Violation trend analysis
- Structure evolution tracking
- Compliance reporting

### 🔧 **Advanced Patterns**
- Regular expression support
- Conditional rules based on context
- Hierarchical policy inheritance

## Conclusion

The tree-only validation system provides a robust, deterministic approach to maintaining clean repository structure. By focusing on top-level entries and using pure Git operations, it delivers strong enforcement without the complexity of recursive validation.

The agreement-based approach with digest verification ensures that both the repository structure and the validation rules themselves are tamper-evident, providing a solid foundation for long-term repository governance.
