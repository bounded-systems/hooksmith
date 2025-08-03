# Hooksmith Bootstrap Refactor

## Overview

The bootstrap process has been refactored into a two-layer system that provides structured logging, SARIF output, and git state validation. This enables better CI integration and programmatic error handling.

## Architecture

### Layer 1: Minimal Bootstrapper (`bootstrap-minimal.rs`)

**Purpose**: Guarantee xtask exists and is buildable.

**Responsibilities**:
- Check git repository state (ensures clean working directory)
- Create minimal xtask structure if missing
- Build xtask binary
- Delegate to `cargo run -p xtask -- bootstrap`

**Features**:
- JSONL structured logging
- SARIF error output
- Git state validation
- Minimal dependencies (<50 lines)

### Layer 2: Full Bootstrap (`xtask bootstrap`)

**Purpose**: Implement complete project bootstrap logic.

**Responsibilities**:
- Generate all Cargo.toml files
- Create component structures
- Set up .gitattributes markers
- Configure direnv environment
- Generate documentation
- Validate generated files

**Features**:
- Structured JSONL logging
- SARIF error/warning output
- Comprehensive validation
- Git integration

## Usage

### Basic Bootstrap

```bash
# Run the minimal bootstrapper (recommended)
cargo run --bin bootstrap-minimal

# Or run the full bootstrap directly
cargo run -p xtask -- bootstrap
```

### Bootstrap with Options

```bash
# Bootstrap with validation
cargo run -p xtask -- bootstrap --validate

# Bootstrap with auto-commit
cargo run -p xtask -- bootstrap --commit

# Both validation and commit
cargo run -p xtask -- bootstrap --validate --commit
```

## Structured Logging

### JSONL Output Format

All bootstrap operations emit structured JSONL events:

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "level": "info",
  "action": "bootstrap_start",
  "message": "🚀 Bootstrapping project with all generated files",
  "details": null,
  "file": null,
  "line": null
}
```

### Event Types

- `bootstrap_start`: Bootstrap process initiated
- `generate_files`: File generation started
- `generate_success`: Files generated successfully
- `validation_start`: Validation process started
- `validation_success`: Validation passed
- `file_check_start`: File type checking started
- `file_check_success`: File type validation passed
- `commit_start`: Git commit process started
- `git_add_success`: Files added to git
- `git_commit_success`: Files committed successfully
- `bootstrap_complete`: Bootstrap completed
- `next_step`: Next step instructions

### Error Events

- `generate_failed`: File generation failed
- `validation_failed`: Validation failed
- `file_check_failed`: File type validation failed
- `git_add_failed`: Git add failed
- `git_commit_failed`: Git commit failed

## SARIF Integration

### Error Output

When failures occur, SARIF format errors are emitted:

```json
{
  "$schema": "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "hooksmith-bootstrap",
          "version": "0.1.0"
        }
      },
      "results": [
        {
          "level": "error",
          "message": {
            "text": "File generation failed: Permission denied"
          },
          "locations": [
            {
              "physical_location": {
                "artifact_location": {
                  "uri": "xtask/src/main.rs"
                },
                "region": {
                  "start_line": 3492,
                  "start_column": 1
                }
              }
            }
          ]
        }
      ]
    }
  ]
}
```

### Warning Output

Warnings are also emitted in SARIF format with `"level": "warning"`.

## Git State Validation

### Clean Working Directory Requirement

The minimal bootstrapper requires a clean git working directory:

```bash
# Check if working directory is clean
git status --porcelain

# If not clean, commit or stash changes
git add . && git commit -m "WIP: Save changes before bootstrap"
# OR
git stash
```

### Validation Process

1. Opens git repository
2. Checks for uncommitted changes
3. Reports changed files if dirty
4. Emits SARIF error if not clean
5. Proceeds only if clean

## CI Integration

### GitHub Actions Example

```yaml
name: Bootstrap

on: [push, pull_request]

jobs:
  bootstrap:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run Bootstrap
        run: cargo run --bin bootstrap-minimal
        continue-on-error: true
        
      - name: Parse JSONL Output
        run: |
          # Extract JSONL events for analysis
          cargo run --bin bootstrap-minimal 2>&1 | grep '^{' > bootstrap-events.jsonl
          
      - name: Parse SARIF Output
        run: |
          # Extract SARIF results for CI reporting
          cargo run --bin bootstrap-minimal 2>&1 | grep -A 100 '^{.*"$schema"' > bootstrap-results.sarif
          
      - name: Upload SARIF Results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: bootstrap-results.sarif
```

### Error Handling

```bash
# Check for bootstrap errors
if ! cargo run --bin bootstrap-minimal; then
  echo "Bootstrap failed"
  # Parse SARIF output for detailed error reporting
  exit 1
fi
```

## File Structure

```
hooksmith/
├── bootstrap-minimal.rs          # Layer 1: Minimal bootstrapper
├── bootstrap.rs                  # Legacy bootstrap (deprecated)
├── xtask/
│   ├── src/
│   │   ├── main.rs              # Contains bootstrap command
│   │   └── structured_logging.rs # JSONL + SARIF utilities
│   └── Cargo.toml
└── README.md
```

## Migration from Legacy Bootstrap

### Before (Legacy)

```bash
# Old way
cargo eval bootstrap.rs
```

### After (New)

```bash
# New way (recommended)
cargo run --bin bootstrap-minimal

# Or direct xtask
cargo run -p xtask -- bootstrap
```

## Benefits

1. **Structured Logging**: JSONL output enables programmatic parsing
2. **SARIF Integration**: Standard error format for CI/CD tools
3. **Git State Validation**: Prevents accidental overwrites
4. **Two-Layer Architecture**: Fast bootstrap + comprehensive setup
5. **CI-Friendly**: Machine-readable output for automation
6. **Error Localization**: File and line number reporting
7. **Extensible**: Easy to add new validation steps

## Troubleshooting

### Common Issues

1. **Git Working Directory Not Clean**
   ```bash
   # Solution: Commit or stash changes
   git add . && git commit -m "Save changes"
   ```

2. **Xtask Build Failure**
   ```bash
   # Check xtask dependencies
   cargo check -p xtask
   ```

3. **Permission Errors**
   ```bash
   # Check file permissions
   ls -la xtask/
   ```

### Debug Mode

```bash
# Enable verbose output
RUST_LOG=debug cargo run --bin bootstrap-minimal

# Parse JSONL events
cargo run --bin bootstrap-minimal | jq '.'
```

## Future Enhancements

1. **Schema Validation**: Validate JSONL events against schema
2. **Metrics Collection**: Performance timing for bootstrap steps
3. **Rollback Support**: Automatic rollback on failure
4. **Parallel Processing**: Concurrent file generation
5. **Incremental Bootstrap**: Only regenerate changed files
6. **Remote Validation**: Validate against remote repository state 
