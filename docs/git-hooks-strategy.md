# Git Hooks Strategy for .gitattributes Validation

This document explains the dual enforcement strategy for validating `.gitattributes` files using both local Git hooks and GitHub Actions CI.

## Overview

The strategy implements a **dual enforcement approach**:

1. **Local Development**: Pre-commit hooks using Lefthook
2. **CI/CD**: GitHub Actions workflows that simulate hook behavior

This ensures developers get fast feedback locally while maintaining strict enforcement at the repository level.

## Hook Strategy Breakdown

### ✅ Pre-commit Hook (Local)

**Purpose**: Validate staged files before committing

**Location**: `lefthook.yml` → `pre-commit.gitattributes`

**Logic**:
```bash
# Get staged files
STAGED_FILES=$(git diff --cached --name-only)

# Check each file with extension
for file in $STAGED_FILES; do
  if [[ "$file" =~ \.[a-zA-Z0-9]+$ ]]; then
    # Check if file has linguist-language entry
    if ! grep -q "^$file linguist-language=" .gitattributes; then
      # Detect language with hyperpolyglot
      LANGUAGE=$(hyply --breakdown "$file" | awk '{print $2}')
      echo "Missing: $file (detected: $LANGUAGE)"
    fi
  fi
done
```

**Benefits**:
- Fast local feedback
- Prevents committing files without proper classification
- Integrates with existing development workflow

### 🔁 Pre-push Hook (Local)

**Purpose**: Ensure `.gitattributes` is up to date before pushing

**Location**: `lefthook.yml` → `pre-push.gitattributes-sync`

**Logic**:
```bash
# Check if .gitattributes needs updating
if ./scripts/ci-gitattributes.sh -c; then
  echo "✅ .gitattributes is up to date"
else
  echo "⚠️  .gitattributes needs updating"
  exit 1
fi
```

**Benefits**:
- Prevents pushing outdated `.gitattributes`
- Ensures repository consistency
- Catches issues before they reach CI

### 🚀 GitHub Actions (CI)

**Purpose**: Simulate hook behavior in CI environment

**Location**: `.github/workflows/language-validation.yml`

**Logic**:
```yaml
- name: Validate staged files (PR only)
  run: |
    # Get files changed in PR
    CHANGED_FILES=$(git diff --name-only origin/main...HEAD)
    
    # Check each changed file
    for file in $CHANGED_FILES; do
      if [[ "$file" =~ \.[a-zA-Z0-9]+$ ]]; then
        # Same validation logic as pre-commit
      fi
    done
```

**Benefits**:
- Enforces policy at repository level
- Works for all contributors
- Provides detailed feedback in PRs

## Implementation Details

### 1. Local Hook Setup

The hooks are configured in `lefthook.yml`:

```yaml
pre-commit:
  commands:
    gitattributes:
      run: |
        # Validation logic here
        # Uses git diff --cached --name-only
        # Runs hyperpolyglot on staged files
        # Checks .gitattributes entries

pre-push:
  commands:
    gitattributes-sync:
      run: |
        # Sync validation logic here
        # Uses ./scripts/ci-gitattributes.sh -c
        # Ensures .gitattributes is up to date
```

### 2. CI Workflow Setup

The CI workflow simulates the same logic:

```yaml
- name: Validate staged files (PR only)
  if: github.event_name == 'pull_request'
  run: |
    # Same logic as pre-commit hook
    # But uses git diff origin/main...HEAD
    # For files changed in the PR
```

### 3. Dual Enforcement Benefits

| Aspect | Local Hook | CI Workflow |
|--------|------------|-------------|
| **Speed** | ✅ Instant feedback | ⚠️ Slower (CI time) |
| **Scope** | Staged files only | All PR changes |
| **Enforcement** | Developer choice | Mandatory |
| **Integration** | Development workflow | Repository policy |

## Usage Examples

### Local Development Workflow

```bash
# 1. Add files to staging
git add new-file.rs

# 2. Pre-commit hook runs automatically
# Validates new-file.rs has .gitattributes entry

# 3. If validation fails, fix and re-add
./scripts/generate-gitattributes.sh
git add .gitattributes

# 4. Commit again
git commit -m "Add new Rust file with proper .gitattributes"

# 5. Pre-push hook validates sync
git push
```

### CI/CD Workflow

```bash
# 1. Create PR with new files
git checkout -b feature/new-files
git add new-file.py
git commit -m "Add Python file"
git push origin feature/new-files

# 2. CI automatically validates
# - Checks if new-file.py has .gitattributes entry
# - Validates .gitattributes format
# - Shows language statistics

# 3. If validation fails, CI blocks merge
# Developer must fix .gitattributes first
```

## Hook Comparison

### Pre-commit vs Pre-push

| Hook | When | Purpose | Use Case |
|------|------|---------|----------|
| **pre-commit** | Before commit | Validate staged files | Prevent committing unclassified files |
| **pre-push** | Before push | Validate repository state | Ensure .gitattributes is up to date |

### Local vs CI

| Environment | Trigger | Scope | Enforcement |
|-------------|---------|-------|-------------|
| **Local** | Git hooks | Staged files | Developer choice |
| **CI** | GitHub Actions | All PR changes | Mandatory |

## Best Practices

### 1. Hook Design Principles

- **Fast**: Pre-commit hooks should be quick (< 5 seconds)
- **Focused**: Each hook has a single responsibility
- **Informative**: Provide clear error messages and fixes
- **Non-blocking**: Allow bypass for emergencies

### 2. CI Integration

- **Simulate hooks**: CI should run the same logic as local hooks
- **Detailed feedback**: Provide actionable error messages
- **Automatic fixes**: When possible, auto-generate fixes
- **Policy enforcement**: Make CI the source of truth

### 3. Developer Experience

- **Clear messages**: Explain what's wrong and how to fix it
- **Quick fixes**: Provide one-liner solutions
- **Documentation**: Link to relevant docs
- **Fallbacks**: Allow emergency bypasses

## Troubleshooting

### Common Issues

**1. Hook not running:**
```bash
# Check if Lefthook is installed
lefthook --version

# Check if hooks are installed
lefthook install

# Check hook configuration
lefthook run pre-commit
```

**2. CI failing but local passes:**
```bash
# Check if local .gitattributes is up to date
./scripts/ci-gitattributes.sh -c

# Update if needed
./scripts/generate-gitattributes.sh
git add .gitattributes
git commit -m "Update .gitattributes"
```

**3. Performance issues:**
```bash
# Check hook execution time
time lefthook run pre-commit

# Optimize by limiting file scope
# Only check files with extensions
```

### Debug Mode

Enable verbose output for debugging:

```bash
# Local hooks
lefthook run pre-commit --verbose

# CI workflow
# Add -v flag to scripts
./scripts/ci-gitattributes.sh -v
```

## Advanced Configuration

### Custom Hook Logic

You can customize the hook behavior:

```yaml
# lefthook.yml
pre-commit:
  commands:
    gitattributes:
      run: |
        # Custom validation logic
        # Add your specific requirements here
```

### Conditional Execution

Hooks can be made conditional:

```bash
# Only run if .gitattributes exists
if [[ -f ".gitattributes" ]]; then
  # Validation logic
fi

# Only run on specific file types
if echo "$file" | grep -q "\.(rs|js|py)$"; then
  # Validation logic
fi
```

### Integration with Other Tools

The hooks integrate with existing tools:

```yaml
# Lefthook with other tools
pre-commit:
  parallel: true
  commands:
    rustfmt:
      glob: "*.rs"
      run: cargo fmt --check {staged_files}
    gitattributes:
      run: # .gitattributes validation
    clippy:
      glob: "*.rs"
      run: cargo clippy -- -D warnings
```

## Migration Guide

### From Manual to Automated

1. **Start with CI**: Implement GitHub Actions first
2. **Add local hooks**: Add Lefthook configuration
3. **Test thoroughly**: Ensure both environments work
4. **Document**: Create clear documentation
5. **Train team**: Ensure everyone understands the workflow

### From Other Hook Systems

If migrating from other hook systems:

```bash
# From pre-commit (Python)
# Replace with Lefthook configuration

# From husky (Node.js)
# Replace with Lefthook configuration

# From custom scripts
# Integrate logic into Lefthook commands
```

## Conclusion

The dual enforcement strategy provides:

- **Fast local feedback** for developers
- **Strict CI enforcement** for repository policy
- **Consistent validation** across environments
- **Clear error messages** and fixes
- **Integration** with existing workflows

This ensures `.gitattributes` files are always accurate and up to date, providing proper language statistics and syntax highlighting on GitHub.
