# Object Names Contract System

This document describes the object-names contract validation system that enforces naming rules for Git tree structures.

## Overview

The object-names contract defines rules for what files and directories are allowed, required, rejected, or ignored at the root of a Git repository. This system provides:

- **Validation**: Check if a repository's root structure complies with the contract
- **CI Enforcement**: GitHub Actions workflow to validate on pull requests
- **Server-side Enforcement**: Pre-receive hooks to block non-compliant pushes
- **Migration Tools**: Scripts to analyze and fix violations

## Contract Definition

The contract is defined in `contracts/object-names@v1.json` with the following structure:

```json
{
  "spec": {
    "git": {
      "tree": {
        "objects": {
          "names": {
            "required": [".gitignore", "projects"],
            "allowed": [".gitignore", ".gitattributes", ".meta", "docs", "generated", "projects", "src", "tests", "tools", "wit"],
            "rejected": ["README.md", "Cargo.toml", "rustfmt.toml", "*.md", "*.toml"],
            "ignored": [".DS_Store", "Thumbs.db", ".idea", ".vscode"]
          }
        }
      }
    }
  }
}
```

### Rule Types

1. **Required**: Files/directories that MUST exist at root
2. **Allowed**: Files/directories that are permitted at root
3. **Rejected**: Files/directories that are NOT allowed at root (glob patterns supported)
4. **Ignored**: Files/directories that are excluded from validation (glob patterns supported)

### Validation Precedence

1. **Ignored** → Skip validation for these entries
2. **Required** → Check that all required entries exist
3. **Rejected** → Check that no rejected patterns match
4. **Allowed** → Check that all non-ignored entries are in the allowed list

## Current Status

As of the latest analysis, the repository has **68 violations**:

- ❌ **Missing required**: `.gitignore`, `projects`
- 🚫 **Rejected files**: 4 files (README.md, *.md files)
- ⚠️ **Unexpected files**: 64 files (mostly .rs scripts)

## Validation Tools

### 1. Rust Validation Script

```bash
cd scripts
cargo run --bin validate_object_names_contract
```

This script:
- Fetches the latest `origin/main`
- Reads the contract from `contracts/object-names@v1.json`
- Validates the root tree structure
- Reports violations with detailed error messages

### 2. Migration Analysis

```bash
cd scripts
cargo run --bin analyze_object_names_migration
```

This script:
- Analyzes current violations
- Suggests file moves to fix issues
- Provides a migration plan

### 3. Bash Validation (Quick Check)

```bash
# One-liner validation
req=(".gitignore" "projects")
allowed=(".gitignore" ".gitattributes" ".meta" "docs" "generated" "projects" "src" "tests" "tools" "wit")
rejected_globs=("README.md" "Cargo.toml" "rustfmt.toml" "*.md" "*.toml")
ignored_globs=(".DS_Store" "Thumbs.db" ".idea" ".vscode")

mapfile -t root < <(git ls-tree --name-only origin/main)
# ... validation logic
```

## CI/CD Integration

### GitHub Actions Workflow

The workflow `.github/workflows/validate-object-names-contract.yml` runs on:
- Pull request events (opened, synchronize, reopened, ready_for_review)
- Push to main branch

**Features:**
- Validates the merged state for PRs
- Provides detailed failure reports
- Integrates with GitHub step summaries

### Server-side Enforcement

The pre-receive hook `scripts/pre-receive-object-names.sh` can be installed on the server to:
- Block pushes to main that violate the contract
- Provide immediate feedback to developers
- Ensure compliance at the repository level

## Migration Strategies

### Option 1: Strict Compliance (Current Contract)

Move all non-compliant files to appropriate directories:

```
Root Structure (After Migration):
├── .gitignore          # Required
├── projects/           # Required
│   └── hooksmith/
│       ├── Cargo.toml
│       ├── README.md
│       └── scripts/
│           └── *.rs files
├── docs/               # Allowed
│   └── *.md files
├── src/                # Allowed
├── tests/              # Allowed
└── [other allowed dirs]
```

### Option 2: Rust Workspace Variant

Use the alternative contract `contracts/object-names@v1-rust-workspace.json` that allows:
- `Cargo.toml`, `Cargo.lock` at root
- `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml` at root
- `README.md` at root
- Only requires `.gitignore`

This is more suitable for Rust workspace repositories.

## Implementation Steps

### For Strict Compliance:

1. **Create required directories**:
   ```bash
   mkdir -p projects/hooksmith/scripts
   ```

2. **Move rejected files**:
   ```bash
   # Move README files to docs/
   mv README*.md docs/
   
   # Move Rust scripts to projects/hooksmith/scripts/
   mv *.rs projects/hooksmith/scripts/
   ```

3. **Verify compliance**:
   ```bash
   cd scripts && cargo run --bin validate_object_names_contract
   ```

### For Rust Workspace Variant:

1. **Switch to the workspace contract**:
   ```bash
   cp contracts/object-names@v1-rust-workspace.json contracts/object-names@v1.json
   ```

2. **Verify compliance**:
   ```bash
   cd scripts && cargo run --bin validate_object_names_contract
   ```

## Caching Strategy

The validation system supports caching by root tree SHA:

```
Cache Key: contract:object-names@root_tree=<sha1>
```

This ensures:
- Fast validation for unchanged root trees
- Automatic invalidation when root structure changes
- Efficient CI/CD performance

## Integration with Hooksmith

The object-names contract integrates with the broader Hooksmith system:

- **Contract State Machine**: Tracks validation state changes
- **Event System**: Emits validation events for monitoring
- **Component Registry**: Registers validation components
- **Documentation**: Auto-generates contract documentation

## Troubleshooting

### Common Issues

1. **Missing required files**: Create the missing files/directories
2. **Rejected files at root**: Move to allowed directories
3. **Unexpected files**: Add to allowed list or move to appropriate location

### Debug Commands

```bash
# Check current root structure
git ls-tree --name-only origin/main

# Validate with detailed output
cd scripts && cargo run --bin validate_object_names_contract

# Analyze migration needs
cd scripts && cargo run --bin analyze_object_names_migration
```

## Future Enhancements

- **Schema Validation**: Validate contract JSON against schema
- **Custom Rules**: Support for repository-specific rule overrides
- **Visual Reports**: Generate visual tree structure reports
- **Integration APIs**: REST APIs for external validation
- **Rule Templates**: Predefined rule sets for common project types
