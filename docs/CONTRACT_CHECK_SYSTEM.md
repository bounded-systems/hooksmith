# Contract Check System

The Contract Check System is the central hub for all validation and status checks in Hooksmith. It provides a single entry point for comprehensive project validation, progress tracking, and quality assurance.

## Overview

The Contract Check System validates:

1. **Generated Files**: Ensures no manual modifications to generated files
2. **Migration Progress**: Tracks progress toward 100% Rust-owned coverage
3. **File Type Analysis**: Complete breakdown of all file types with migration status
4. **Trend Tracking**: Historical progress monitoring

## Quick Start

### Basic Usage

```bash
# Run all contract checks
cargo xtask contract-check

# Strict mode (fails on violations)
cargo xtask contract-check --strict

# Pre-commit check (staged files only)
cargo xtask contract-check --staged-only --strict

# Full validation with trends
cargo xtask contract-check --strict --trend --verbose
```

### Command Options

| Option | Description |
|--------|-------------|
| `--staged-only` | Check only staged files (for pre-commit hooks) |
| `--strict` | Exit with error on violations (for CI) |
| `--trend` | Generate trend data for historical tracking |
| `--trend-output <DIR>` | Output directory for trend data (default: `status-trends`) |
| `--verbose` | Show detailed output |

## System Components

### 1. Generated File Validation

Validates that generated files haven't been manually modified:

```bash
# Check all generated files
cargo xtask validate-generated --strict

# Check only staged files
cargo xtask validate-generated --staged-only --strict
```

**What it checks:**
- Files marked as `codegen` in `.gitattributes`
- Files with generation markers in headers
- Files in generated directories

### 2. Migration Progress Tracking

Tracks progress toward 100% Rust-owned pipeline:

```bash
# Check migration progress
cargo xtask status migration-progress --format markdown

# JSON output for CI
cargo xtask status migration-progress --format json
```

**Current Status:**
- **Progress**: 7.1% (target: 95%)
- **File Types**: 14 (target: ≤8)
- **Generated Files**: 93% (13/14 types)

### 3. File Type Analysis

Complete breakdown of all file types:

```bash
# Detailed analysis
cargo xtask status file-types --format markdown

# JSON for CI integration
cargo xtask status file-types --format json
```

**File Type Categories:**
- **Remove**: Shell scripts (priority 9)
- **Generate**: Documentation, config files (priority 6-8)
- **Consolidate**: Build artifacts (priority 2)
- **Keep**: Rust source files (priority 1)

## CI/CD Integration

### GitHub Actions Workflow

```yaml
name: Contract Check System

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  contract-check:
    name: Contract Check & Validation
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      with:
        fetch-depth: 0  # Full history for trend analysis

    - name: Setup Rust toolchain
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy

    - name: Run contract check
      run: cargo run -p xtask -- contract-check --strict --trend --verbose

    - name: Generate status report
      run: |
        echo "## 📊 Contract Check Status Report" >> $GITHUB_STEP_SUMMARY
        cargo run -p xtask -- status migration-progress --format markdown >> $GITHUB_STEP_SUMMARY
        cargo run -p xtask -- status file-types --format markdown >> $GITHUB_STEP_SUMMARY

    - name: Upload trend data
      uses: actions/upload-artifact@v3
      with:
        name: contract-check-trends
        path: status-trends/
        retention-days: 30
```

### Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run contract check on staged files
cargo xtask contract-check --staged-only --strict
```

### Quality Gates

```bash
# Check migration progress threshold
PROGRESS=$(cargo xtask status migration-progress --format json | jq -r '.progress_percentage')
if (( $(echo "$PROGRESS < 5" | bc -l) )); then
  echo "❌ Migration progress ($PROGRESS%) is below threshold (5%)"
  exit 1
fi

# Check file type count
TYPE_COUNT=$(cargo xtask status file-types --format json | jq '.file_types | length')
if [ "$TYPE_COUNT" -gt 15 ]; then
  echo "❌ File type count ($TYPE_COUNT) is above threshold (15)"
  exit 1
fi
```

## Output Formats

### Markdown Output

```markdown
## 📊 Contract Check Status Report

### Migration Progress
- **Current Progress**: 7.1%
- **Target**: 95.0%
- **Status**: ⚠️ Below target

### File Type Analysis
- **Total Types**: 14
- **Generated**: 13 (93%)
- **Manual**: 1 (7%)
- **Priority Actions**: 2 types need migration
```

### JSON Output

```json
{
  "migration_progress": {
    "progress_percentage": 7.1,
    "target_percentage": 95.0,
    "status": "below_target"
  },
  "file_types": {
    "total_count": 14,
    "generated_count": 13,
    "manual_count": 1,
    "priority_actions": 2
  }
}
```

## Integration Examples

### Local Development

```bash
# Quick check during development
cargo xtask contract-check

# Validate before commit
cargo xtask contract-check --staged-only --strict

# Full validation with trends
cargo xtask contract-check --strict --trend --verbose
```

### CI/CD Pipeline

```yaml
# GitHub Actions
- name: Contract Check
  run: cargo xtask contract-check --strict --trend

# Generate status report
- name: Status Report
  run: cargo xtask status migration-progress --format markdown
```

### Pre-commit Hooks

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🔗 Running contract check..."

# Run contract check on staged files
if ! cargo xtask contract-check --staged-only --strict; then
  echo "❌ Contract check failed. Please fix the issues above."
  exit 1
fi

echo "✅ Contract check passed!"
```

## Troubleshooting

### Common Issues

**"Generated files validation failed"**
```bash
# Regenerate all files
cargo xtask gen-all

# Or regenerate specific files
cargo xtask gen-lefthook
cargo xtask gen-docs
cargo xtask gen-mods
```

**"Migration progress below threshold"**
- Review file types that need migration
- Prioritize high-priority types (shell scripts, documentation)
- Use `cargo xtask status file-types` for detailed analysis

**"File type count above threshold"**
- Consolidate similar file types
- Generate more files from Rust source
- Remove unnecessary file types

### Debugging

```bash
# Verbose output
cargo xtask contract-check --verbose

# Check specific components
cargo xtask validate-generated --strict
cargo xtask status migration-progress --format json
cargo xtask status file-types --format markdown

# Test individual validations
cargo xtask validate-headers --strict
cargo xtask check-files --strict --verbose
```

## Advanced Usage

### Custom Validation Rules

```rust
// Add custom validation to contract check
pub async fn run_custom_validation() -> Result<()> {
    // Your custom validation logic
    Ok(())
}
```

### Trend Analysis

```bash
# Generate trend data
cargo xtask contract-check --trend

# View trend data
ls status-trends/
cat status-trends/status-$(date +%Y-%m-%d).json
```

### Integration with External Tools

```bash
# Export data for external analysis
cargo xtask status migration-progress --format json > migration-data.json
cargo xtask status file-types --format json > file-types.json

# Use in scripts
PROGRESS=$(cargo xtask status migration-progress --format json | jq -r '.progress_percentage')
echo "Migration progress: $PROGRESS%"
```

## Best Practices

### For Developers

1. **Run contract check before commits**
   ```bash
   cargo xtask contract-check --staged-only --strict
   ```

2. **Use strict mode in CI**
   ```bash
   cargo xtask contract-check --strict --trend
   ```

3. **Monitor migration progress**
   ```bash
   cargo xtask status migration-progress --format markdown
   ```

### For Teams

1. **Set up pre-commit hooks**
   ```bash
   # Install pre-commit hook
   cp scripts/pre-commit .git/hooks/
   chmod +x .git/hooks/pre-commit
   ```

2. **Configure CI quality gates**
   ```yaml
   # GitHub Actions quality gates
   - name: Quality Gates
     run: |
       cargo xtask contract-check --strict
       # Check thresholds
   ```

3. **Regular progress reviews**
   ```bash
   # Weekly progress report
   cargo xtask status migration-progress --format markdown
   cargo xtask status file-types --format markdown
   ```

## Future Enhancements

### Planned Features

1. **Custom Validation Rules**: Project-specific checks
2. **Performance Metrics**: Build time and efficiency tracking
3. **Dependency Analysis**: Rust crate dependency validation
4. **Security Scanning**: Vulnerability detection
5. **Compliance Checking**: License and legal validation

### Integration Opportunities

1. **IDE Integration**: VS Code extensions
2. **Slack/Discord**: Status notifications
3. **Dashboard**: Web-based progress visualization
4. **API**: REST endpoints for external tools
5. **Webhooks**: Real-time status updates

## Conclusion

The Contract Check System provides a comprehensive solution for maintaining project quality and tracking progress toward the 100% Rust-owned pipeline goal. It integrates seamlessly with existing workflows and provides clear feedback for both developers and CI/CD systems.

**Key Benefits:**
- ✅ **Single Entry Point**: One command for all validation
- ✅ **CI/CD Ready**: Complete automation pipeline
- ✅ **Progress Tracking**: Real-time migration status
- ✅ **Quality Assurance**: Automated validation
- ✅ **Flexible Integration**: Works with existing tools

The system is designed to grow with your project and can be extended with custom validation rules and integration points as needed. 
