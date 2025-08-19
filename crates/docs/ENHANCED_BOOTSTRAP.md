# Enhanced Bootstrap Command

The enhanced `bootstrap` command provides a comprehensive, deterministic way to set up and regenerate your Hooksmith project with all generated files.

## Features

- **🔨 Minimal Build Environment**: Builds `xtask` first to ensure you can always run generation/validation
- **🧹 Smart Cleaning**: Removes all generated files (except `.rs` and `.jsonc`) using robust JSONC parsing
- **🔄 Deterministic Regeneration**: Regenerates everything using your unified generator
- **✅ Comprehensive Validation**: Validates that regenerated files match checksums and registry
- **📝 Structured Logging**: Logs every step in structured JSON for traceability
- **🔍 Dry-Run Mode**: See what would be done without making changes
- **📊 Verbose Output**: Detailed logging for debugging and monitoring

## Usage

### Basic Bootstrap

```bash
# Basic bootstrap with validation
cargo xtask bootstrap --validate

# Bootstrap with validation and commit
cargo xtask bootstrap --validate --commit
```

### Enhanced Options

```bash
# Full bootstrap with all features
cargo xtask bootstrap \
  --validate \
  --commit \
  --clean \
  --build-xtask \
  --verbose

# Dry-run to see what would be done
cargo xtask bootstrap \
  --validate \
  --commit \
  --clean \
  --dry-run \
  --verbose
```

### Command Options

| Option | Description | Default |
|--------|-------------|---------|
| `--validate` | Validate generated files after bootstrap | `false` |
| `--commit` | Commit generated files to git | `false` |
| `--clean` | Clean existing generated files first | `false` |
| `--build-xtask` | Build xtask binary first | `true` |
| `--dry-run` | Show what would be done without making changes | `false` |
| `--verbose` | Show detailed output | `false` |

## Workflow

The enhanced bootstrap follows this deterministic workflow:

1. **🔍 Dry-Run Check**: If `--dry-run` is set, shows what would be done
2. **🔨 Build xtask**: Ensures minimal build environment (unless disabled)
3. **🧹 Clean Files**: Removes old generated files (if `--clean` is set)
4. **🔄 Regenerate**: Uses unified generator to create all files deterministically
5. **✅ Validate**: Checks checksums and registry (if `--validate` is set)
6. **📝 Commit**: Adds and commits files to git (if `--commit` is set)

## Examples

### Development Setup

```bash
# First-time setup with full validation
cargo xtask bootstrap --validate --commit --clean --verbose
```

### Regeneration Only

```bash
# Just regenerate files without cleaning
cargo xtask bootstrap --validate --verbose
```

### Safe Preview

```bash
# See what would happen without making changes
cargo xtask bootstrap --validate --commit --clean --dry-run --verbose
```

### CI/CD Integration

```bash
# CI-friendly bootstrap (no commit, strict validation)
cargo xtask bootstrap --validate --clean --verbose
```

## JSONC Integration

The enhanced bootstrap uses robust JSONC parsing for the generated files registry:

```jsonc
{
  // Generated files registry
  "files": [
    {
      "path": "src/generated/version.rs",
      "checksum": "sha256:abc123...",
      "generated_by": "xtask"
    },
    {
      "path": "docs/README.md", 
      "checksum": "sha256:def456...",
      "generated_by": "xtask"
    }
  ]
}
```

## Structured Logging

All operations are logged in structured JSON format:

```json
{
  "timestamp": "2025-01-02T10:30:00Z",
  "level": "info",
  "action": "bootstrap_start",
  "message": "🚀 Starting enhanced bootstrap process",
  "details": null
}
```

## Error Handling

The bootstrap command provides comprehensive error handling:

- **Build failures**: Detailed error messages with context
- **Validation failures**: Specific checksum mismatches and registry violations
- **File system errors**: Permission and path issues
- **Git operations**: Commit and add failures

## Integration with Existing Commands

The enhanced bootstrap integrates with your existing xtask commands:

- Uses `gen-all-unified` for deterministic regeneration
- Uses `validate-generated-unified` for comprehensive validation
- Uses `file_audit::check_files()` for file type validation

## Best Practices

1. **Always use `--validate`** in production to ensure integrity
2. **Use `--dry-run` first** to understand what will happen
3. **Enable `--verbose`** for debugging and monitoring
4. **Use `--clean`** when you want a fresh start
5. **Use `--commit`** only when you're ready to commit changes

## Troubleshooting

### Common Issues

**Build failures:**
```bash
# Ensure xtask can build
cargo build -p xtask
```

**Validation failures:**
```bash
# Check what files are out of sync
cargo xtask validate-generated-unified --strict --verbose
```

**Permission errors:**
```bash
# Check file permissions
ls -la config/generated-files.jsonc
```

### Debug Mode

```bash
# Full debug output
RUST_LOG=debug cargo xtask bootstrap --verbose --dry-run
```

## Migration from Legacy Bootstrap

The enhanced bootstrap is backward compatible. Your existing commands will work:

```bash
# Old command still works
cargo xtask bootstrap --validate --commit

# New enhanced features available
cargo xtask bootstrap --validate --commit --clean --verbose
```

## Performance

The enhanced bootstrap is optimized for performance:

- **Parallel operations** where possible
- **Incremental builds** (only rebuilds what's necessary)
- **Smart file detection** (only processes changed files)
- **Efficient JSONC parsing** (no manual comment stripping)

## Security

The enhanced bootstrap includes security features:

- **Deterministic generation** (same input = same output)
- **Checksum validation** (prevents tampering)
- **Registry validation** (ensures file integrity)
- **Safe file operations** (preserves `.rs` and `.jsonc` files)

---

*This enhanced bootstrap command replaces shell scripts with pure Rust, providing a robust, cross-platform solution for project setup and regeneration.* 
