# Component Status System

The Component Status System provides comprehensive monitoring and health checking for all WIT components and native crates in the Hooksmith workspace. It offers real-time build status, version information, schema validation, and checksum verification.

## 🎯 Overview

The system consists of:

- **Component Registry**: Single source of truth for all components and crates
- **Status Checker**: Comprehensive health monitoring and validation
- **CLI Integration**: Easy-to-use commands via xtask
- **CI/CD Integration**: Automated status checking in GitHub Actions
- **Multiple Output Formats**: Table, JSON, and CSV reporting

## 📦 Component Registry

The registry (`config/component-registry.jsonc`) serves as the authoritative source for all components:

```jsonc
{
  "metadata": {
    "version": "1.0.0",
    "description": "Hooksmith Component and Crate Registry",
    "last_updated": "2024-01-01T00:00:00Z"
  },
  "wit_components": [
    {
      "name": "git-filter",
      "path": "crates/components/git-filter",
      "wit": "wit/git-filter.wit",
      "description": "Git filter component for blob and tree contract validation",
      "category": "validation",
      "status": "active"
    }
  ],
  "native_crates": [
    {
      "name": "file-operations",
      "path": "crates/file-operations",
      "description": "File system operations and event handlers",
      "category": "system",
      "status": "active"
    }
  ]
}
```

## 🚀 Usage

### Basic Status Check

```bash
# Check status of all components
cargo run -p xtask -- component-status

# Verbose output with detailed information
cargo run -p xtask -- component-status --verbose

# Different output formats
cargo run -p xtask -- component-status --format json
cargo run -p xtask -- component-status --format csv
```

### Example Output

```
🔍 Hooksmith Component Status Report
=====================================
Generated: 2024-01-01 12:00:00 UTC
Build time: 15.2s

📦 WIT Components
----------------
Name                 Category     Status     Build    Version  Schema
--------------------------------------------------------------------------------
git-filter          validation   active     ✅       0.2.1     ✅
hook-builder        build        active     ✅       0.4.0     ✅
validation-handler  validation   active     ❌       —         ❌
worktree-runner     git          active     ✅       0.1.3     ✅

🦀 Native Crates
---------------
Name                 Category     Status     Build    Version  Schema
--------------------------------------------------------------------------------
cli-core            core         active     ✅       0.1.0     —
file-operations     system       active     ✅       0.1.0     —
git-operations      git          active     ✅       0.1.0     —
lefthook-rs         integration  active     ✅       0.3.2     —
xtask               build        active     ✅       0.5.1     —

📊 Summary
----------
Total components: 11
✅ Successful builds: 10
❌ Failed builds: 1
⏸️  Not built: 0
📈 Success rate: 90.9%
```

## 🔧 Features

### Build Status Checking

- **WIT Components**: Uses `cargo component build` to verify compilation
- **Native Crates**: Uses `cargo check` to verify syntax and dependencies
- **Performance Metrics**: Tracks build times and success rates

### Version Information

- Extracts version from `Cargo.toml` using `cargo metadata`
- Displays current version for each component
- Helps track version consistency across the workspace

### Schema Validation

- **WIT Files**: Validates WIT interface definitions
- **JSON Schemas**: Checks schema files for validity
- **Registry Consistency**: Ensures registry matches actual filesystem

### Checksum Verification

- **WIT Components**: SHA256 checksums of compiled `.wasm` files
- **Native Crates**: Directory checksums for source code
- **Integrity Checking**: Detects changes and potential corruption

### Error Reporting

- Detailed error messages for failed builds
- Warning collection for potential issues
- Structured error output for CI/CD integration

## 🏗️ Architecture

### Component Status Checker

```rust
pub struct ComponentStatusChecker {
    registry: ComponentRegistry,
    workspace_root: PathBuf,
    verbose: bool,
    check_checksums: bool,
    check_schemas: bool,
}
```

### Status Information

```rust
pub struct ComponentStatus {
    pub name: String,
    pub path: String,
    pub category: String,
    pub status: String,
    pub build_status: BuildStatus,
    pub version: Option<String>,
    pub checksum: Option<String>,
    pub schema_valid: Option<bool>,
    pub last_build: Option<String>,
    pub build_duration: Option<Duration>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

## 🔄 CI/CD Integration

### GitHub Actions Workflow

The system includes a comprehensive GitHub Actions workflow (`component-status.yml`) that:

1. **Validates Registry**: Ensures registry file exists and is valid
2. **Checks Components**: Builds all WIT components in parallel
3. **Tests Crates**: Runs tests for all native crates
4. **Generates Reports**: Creates detailed status reports
5. **Comments on PRs**: Provides status feedback on pull requests

### Workflow Jobs

- `check-component-status`: Main status checking
- `validate-registry`: Registry file validation
- `build-components`: Parallel WIT component builds
- `test-native-crates`: Native crate testing
- `status-summary`: Final summary and PR comments

## 📊 Output Formats

### Table Format (Default)

Human-readable table with status icons and key metrics.

### JSON Format

Structured JSON output for programmatic consumption:

```json
{
  "total_components": 11,
  "successful_builds": 10,
  "failed_builds": 1,
  "not_built": 0,
  "build_time": "15.2s",
  "timestamp": "2024-01-01T12:00:00Z",
  "wit_components": [...],
  "native_crates": [...]
}
```

### CSV Format

Comma-separated values for spreadsheet analysis:

```csv
Name,Category,Status,Build Status,Version,Schema Valid,Errors
git-filter,validation,active,Success,0.2.1,Yes,
validation-handler,validation,active,Failed,,No,"Build failed: missing dependency"
```

## 🎯 Use Cases

### Development Workflow

1. **Pre-commit Checks**: Ensure all components build before committing
2. **Debugging**: Quickly identify which components have issues
3. **Version Tracking**: Monitor version consistency across the workspace

### CI/CD Pipeline

1. **Build Verification**: Ensure all components compile successfully
2. **Quality Gates**: Block deployments if critical components fail
3. **Performance Monitoring**: Track build times and identify bottlenecks

### Documentation

1. **Component Inventory**: Maintain up-to-date list of all components
2. **Dependency Tracking**: Understand relationships between components
3. **Health Monitoring**: Track overall system health over time

## 🔧 Configuration

### Registry Management

To add a new component to the registry:

1. Add entry to `config/component-registry.jsonc`
2. Include required fields: `name`, `path`, `description`, `category`, `status`
3. For WIT components: add `wit` field pointing to WIT file
4. For native crates: add `dependencies`, `targets`, `features` as needed

### Customization

The system can be customized through:

- **Verbose Mode**: Detailed output for debugging
- **Format Selection**: Choose output format based on use case
- **Selective Checking**: Focus on specific components or categories
- **Performance Tuning**: Adjust timeouts and parallelization

## 🚀 Getting Started

### Prerequisites

- Rust toolchain with `wasm32-wasip2` target
- `cargo-component` installed
- Hooksmith workspace cloned

### Quick Start

```bash
# 1. Build xtask
cargo build -p xtask

# 2. Check component status
cargo run -p xtask -- component-status

# 3. Run demo
cargo run --example component_status_demo
```

### Demo Script

The `examples/component_status_demo.rs` provides a comprehensive demonstration:

```bash
cargo run --example component_status_demo
```

## 🔍 Troubleshooting

### Common Issues

1. **Missing Dependencies**: Ensure `cargo-component` is installed
2. **Build Failures**: Check component dependencies and WIT files
3. **Registry Errors**: Validate JSONC syntax in registry file
4. **Permission Issues**: Ensure write access to target directories

### Debug Mode

Use verbose mode for detailed debugging:

```bash
cargo run -p xtask -- component-status --verbose
```

### Manual Validation

Check individual components manually:

```bash
# Check specific WIT component
cargo component build -p git-filter --target wasm32-wasip2

# Check specific native crate
cargo check -p file-operations
```

## 📈 Performance

### Optimization Tips

1. **Parallel Builds**: Components are built in parallel for speed
2. **Caching**: Leverage cargo's built-in caching mechanisms
3. **Selective Checking**: Only check components that have changed
4. **Incremental Builds**: Use `cargo check` for faster native crate validation

### Benchmarks

Typical performance metrics:

- **Full Status Check**: 15-30 seconds for 11 components
- **WIT Component Build**: 5-10 seconds per component
- **Native Crate Check**: 1-3 seconds per crate
- **Registry Validation**: <1 second

## 🔮 Future Enhancements

### Planned Features

1. **Dependency Graph**: Visual representation of component relationships
2. **Performance Trends**: Historical build time tracking
3. **Health Scoring**: Automated health scoring system
4. **Integration APIs**: REST API for external tooling
5. **Dashboard**: Web-based status dashboard

### Extensibility

The system is designed for extensibility:

- **Custom Validators**: Add custom validation logic
- **Plugin System**: Support for third-party status checkers
- **Event Integration**: Integration with event bus system
- **Metrics Export**: Export metrics to monitoring systems

## 📚 Related Documentation

- [WIT Component Architecture](../WIT_FIRST_ARCHITECTURE.md)
- [Host-Mediated Communication](../HOST_MEDIATED_COMMUNICATION_GUIDE.md)
- [Event-Driven Architecture](../STRUCTURED_LOGGING.md)
- [CI/CD Integration](../CI_CD_INTEGRATION_VERIFICATION.md)

## 🤝 Contributing

To contribute to the Component Status System:

1. **Add Tests**: Include tests for new features
2. **Update Registry**: Keep registry file current
3. **Document Changes**: Update this documentation
4. **Follow Patterns**: Maintain consistency with existing code

## 📄 License

This component status system is part of the Hooksmith project and follows the same licensing terms. 
