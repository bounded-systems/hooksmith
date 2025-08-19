# Build System Guide

This document describes Hooksmith's platform-aware build system that automatically detects the host platform and builds accordingly.

## 🎯 Overview

The build system is designed to:
- **Auto-detect platform**: Automatically determine the host architecture and OS
- **Use native targets**: Build for the host platform by default for maximum performance
- **Avoid cross-compilation**: Only use cross-compilation when explicitly needed
- **Cache efficiently**: Cache builds per platform to avoid redundant work
- **Support CI/CD**: Matrix builds in GitHub Actions for multiple platforms

## 🛠️ Local Development

### Standard Build (Recommended)

For local development, use the smart build script that auto-detects your platform:

```bash
# Build for your current platform
./scripts/build_xtask.sh

# Build with additional cargo arguments
./scripts/build_xtask.sh --release
./scripts/build_xtask.sh --debug
```

### Platform Detection

The build script automatically detects:
- **Apple Silicon Mac**: `aarch64-apple-darwin`
- **Intel Mac**: `x86_64-apple-darwin`
- **Linux x86_64**: `x86_64-unknown-linux-gnu`
- **Linux ARM64**: `aarch64-unknown-linux-gnu`
- **Other platforms**: Uses default target

### Cross-Compilation (When Needed)

If you need to build for a different platform:

```bash
# Build for Apple Silicon from Intel Mac
./scripts/build_xtask_cross.sh aarch64-apple-darwin

# Build for Linux from Mac
./scripts/build_xtask_cross.sh x86_64-unknown-linux-gnu

# Build with release optimizations
./scripts/build_xtask_cross.sh aarch64-apple-darwin --release
```

## 🚀 CI/CD Integration

### GitHub Actions Matrix Builds

The CI system uses matrix builds to test multiple platforms:

```yaml
jobs:
  build-xtask:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: aarch64-apple-darwin
```

### Platform-Specific Builds

Each platform builds using its native target:

- **Ubuntu**: Uses default target (no `--target` flag)
- **macOS**: Uses `--target aarch64-apple-darwin` for Apple Silicon

### Caching Strategy

Builds are cached per platform to avoid redundant work:

```yaml
- uses: actions/cache@v4
  with:
    path: target
    key: xtask-${{ runner.os }}-${{ matrix.target }}-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      xtask-${{ runner.os }}-${{ matrix.target }}-
      xtask-${{ runner.os }}-
```

## 📋 Build Scripts

### `scripts/build_xtask.sh`

**Purpose**: Smart platform-aware build for local development

**Features**:
- Auto-detects host platform
- Uses native targets by default
- Passes through additional cargo arguments
- Provides clear feedback about target selection

**Usage**:
```bash
./scripts/build_xtask.sh [cargo-args...]
```

### `scripts/build_xtask_cross.sh`

**Purpose**: Cross-compilation when needed

**Features**:
- Installs target automatically if missing
- Validates target triple
- Provides clear usage instructions
- Shows binary location after build

**Usage**:
```bash
./scripts/build_xtask_cross.sh <target-triple> [cargo-args...]
```

## 🔧 Configuration

### Cargo Configuration

The `.cargo/config.toml` is configured to:
- Use native targets by default
- Only use `wasm32-wasip2` when explicitly building WIT components
- Avoid toolchain conflicts

### Toolchain Management

- **Local**: Uses `rustup override set stable` for consistent builds
- **CI**: Uses `dtolnay/rust-toolchain@stable` action
- **Targets**: Installed on-demand for cross-compilation

## 🎯 Best Practices

### For Local Development

1. **Use native builds**: Always use `./scripts/build_xtask.sh` for local development
2. **Avoid cross-compilation**: Only use cross-compilation when distributing binaries
3. **Cache builds**: Let Cargo handle incremental builds automatically

### For CI/CD

1. **Matrix builds**: Test on multiple platforms
2. **Cache artifacts**: Cache builds per platform
3. **Reuse binaries**: Download and reuse built artifacts between jobs

### For Distribution

1. **Cross-compile only when needed**: Use `./scripts/build_xtask_cross.sh`
2. **Test on target platform**: Always test cross-compiled binaries
3. **Document targets**: Clearly document which platforms are supported

## 🐛 Troubleshooting

### Common Issues

**Issue**: Build fails with "target not found"
**Solution**: Use `./scripts/build_xtask_cross.sh` which auto-installs targets

**Issue**: Slow builds on CI
**Solution**: Check that caching is working properly

**Issue**: Toolchain conflicts
**Solution**: Use `rustup override set stable` locally

### Platform-Specific Notes

**Apple Silicon Mac**:
- Uses `aarch64-apple-darwin` target
- May need Rosetta 2 for some dependencies

**Intel Mac**:
- Uses `x86_64-apple-darwin` target
- Generally faster builds than cross-compilation

**Linux**:
- Uses default target (no `--target` flag)
- Most efficient builds

## 📊 Performance Comparison

| Platform | Native Build | Cross-Compile | Notes |
|----------|-------------|---------------|-------|
| Apple Silicon | ✅ Fast | ❌ Slow | Native is 3-5x faster |
| Intel Mac | ✅ Fast | ❌ Slow | Native is 2-3x faster |
| Linux | ✅ Fast | ❌ Slow | Native is 2-4x faster |

## 🔄 Migration Guide

### From Old Build System

If you were previously using hardcoded targets:

**Before**:
```bash
cargo build -p xtask --target aarch64-apple-darwin
```

**After**:
```bash
./scripts/build_xtask.sh
```

### For CI/CD

**Before**:
```yaml
- run: cargo build -p xtask --target aarch64-apple-darwin
```

**After**:
```yaml
- run: |
    if [[ "${{ runner.os }}" == "macOS" ]]; then
      cargo build -p xtask --target aarch64-apple-darwin
    else
      cargo build -p xtask
    fi
```

## 📚 Related Documentation

- [Component Status System](./COMPONENT_STATUS_SYSTEM.md)
- [WIT Component Development](./WIT_FIRST_ARCHITECTURE.md)
- [CI/CD Integration](./CI_CD_INTEGRATION.md) 
