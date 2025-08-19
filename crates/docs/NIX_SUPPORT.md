# Nix Support for Hooksmith

This document describes the comprehensive Nix support added to Hooksmith for reproducible builds and development environments.

## Overview

Hooksmith now includes a complete Nix flake that provides:

- **Reproducible builds** for all Hooksmith tools
- **Development environment** with all necessary dependencies
- **Multiple packages** for different use cases
- **Cross-platform support** (Linux, macOS)
- **Integration with direnv** for automatic environment loading

## Quick Start

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- Optional: [direnv](https://direnv.net/) for automatic environment loading

### Basic Usage

```bash
# Enter development environment
nix develop

# Build all Hooksmith tools
nix build .#hooksmith-suite

# Run a specific analysis tool
nix run .#repository_size_auditor

# Quick analysis suite
nix run .#analyze
```

### With direnv (Recommended)

```bash
# Enable automatic environment loading
echo "use flake" > .envrc
direnv allow

# Now the environment loads automatically when you cd into the directory
```

## Available Packages

The flake provides several packages for different use cases:

### Main Packages

- **`hooksmith-suite`** (default) - Complete suite with all tools and convenience scripts
- **`analysis-tools`** - All Git repository analysis tools
- **`xtask`** - Build system and orchestration tool
- **`core-tools`** - Core validation and inspection tools

### Individual Tool Packages

For convenience, individual analysis tools are also available:

- `repository_size_auditor`
- `rust_blob_analyzer`
- `git_delta_analyzer`
- `git_lfs_auto_tracker`
- `file_churn_analyzer`
- `tree_object_stability_auditor`
- `git_history_cleanliness_analyzer`

## Available Apps

Apps provide direct execution of tools:

```bash
# Main suite information
nix run .#hooksmith-suite

# Quick analysis
nix run .#analyze

# Build system
nix run .#xtask -- --help

# Individual analysis tools
nix run .#repository_size_auditor
nix run .#rust_blob_analyzer
nix run .#git_delta_analyzer
nix run .#file_churn_analyzer
```

## Development Environment

The development shell includes comprehensive tooling:

### Rust Development
- Latest stable Rust toolchain via fenix
- cargo-nextest, cargo-watch, cargo-edit
- cargo-audit, cargo-deny, bacon

### Git and Analysis Tools
- Git with Git LFS support
- git-filter-repo for repository extraction
- jq, tree, ripgrep, fd for data processing

### Build Tools
- just for convenient task running
- gnumake for traditional builds
- wasmtime for WASM development

### Documentation
- mdbook for documentation generation

## Integration with Existing Workflows

### With Cargo

Nix doesn't replace Cargo - it enhances it:

```bash
# Standard Cargo still works
cargo build --release
cargo test

# Nix provides reproducible builds
nix build .#analysis-tools
```

### With Xtask

The xtask build system works seamlessly with Nix:

```bash
# Via Cargo
cargo run -p xtask -- build

# Via Nix (in dev shell)
xtask build

# Direct Nix execution
nix run .#xtask -- build
```

### With Just

The provided justfile integrates Nix commands:

```bash
# Cargo-based commands
just build
just test

# Nix-based commands
just nix-build
just nix-analyze

# Mixed workflows
just analyze-all  # Uses cargo
```

## Reproducible Builds

### Benefits

1. **Consistent Dependencies**: All dependencies pinned to specific versions
2. **Cross-Platform**: Same build on Linux and macOS
3. **Isolated Environment**: No interference with system packages
4. **Cacheable**: Nix can cache builds and share them

### Build Configuration

All packages use optimized release builds:

```nix
CARGO_PROFILE_RELEASE_LTO = "true";
CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
CARGO_PROFILE_RELEASE_OPT_LEVEL = "s";
```

### Platform Support

- **x86_64-linux**: Full support
- **aarch64-linux**: Full support  
- **x86_64-darwin**: Full support with macOS frameworks
- **aarch64-darwin**: Full support with Apple Silicon optimizations

## Convenience Scripts

The `hooksmith-suite` package includes helpful scripts:

### `hooksmith-suite`

Shows available tools and usage information:

```bash
nix run .#hooksmith-suite
# Shows comprehensive tool listing and usage examples
```

### `hooksmith-analyze`

Runs a quick analysis suite:

```bash
nix run .#analyze
# Automatically runs:
# - Repository size audit
# - Rust blob analysis  
# - Git delta analysis
```

## direnv Integration

For the best development experience, use direnv:

```bash
# Setup (one time)
echo "use flake" > .envrc
direnv allow

# Now the environment auto-loads
cd /path/to/hooksmith
# 🔨 Hooksmith development environment loaded via Nix
# 📚 See WARP.md for development workflows

# All tools are automatically available
repository_size_auditor
xtask --help
```

## Customization

### Local Overrides

Create a `flake.local.nix` for local customizations:

```nix
{
  # Add extra packages to the dev shell
  extraPackages = with pkgs; [
    your-favorite-editor
    custom-tool
  ];
  
  # Override environment variables
  extraEnvVars = {
    CUSTOM_VAR = "value";
  };
}
```

### Alternative Toolchains

To use a different Rust toolchain:

```bash
# In your dev shell
rustup override set nightly
# or
rustup override set 1.70.0
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Setup Nix
  uses: cachix/install-nix-action@v22
  with:
    github_access_token: ${{ secrets.GITHUB_TOKEN }}

- name: Build with Nix
  run: nix build .#hooksmith-suite

- name: Run analysis
  run: nix run .#analyze
```

### Cachix Support

Consider setting up Cachix for faster CI builds:

```yaml
- name: Setup Cachix
  uses: cachix/cachix-action@v12
  with:
    name: your-cache
    authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
```

## Troubleshooting

### Common Issues

**"flakes are not enabled"**
```bash
# Enable flakes in your Nix configuration
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

**"permission denied"**
```bash
# Make sure direnv is allowed
direnv allow
```

**"build fails with git dependency"**
```bash
# Ensure git is available in the build environment
# (This should be automatic with our flake)
```

### Getting Help

1. Check the development shell help: `nix develop`
2. Review WARP.md for comprehensive workflows
3. Use `just` for convenient commands
4. Run `nix run .#hooksmith-suite` for tool overview

## Migration Guide

### From Pure Cargo

```bash
# Before
cargo build --release
cargo run --bin repository_size_auditor

# After (equivalent)
nix build .#analysis-tools
nix run .#repository_size_auditor

# Or in dev shell
nix develop
cargo build --release  # Still works!
repository_size_auditor  # Now in PATH
```

### From Shell Scripts

```bash
# Before
./build.sh
./analyze.sh

# After
just build      # or nix build
just analyze-all  # or nix run .#analyze
```

## Performance Comparison

| Method | First Build | Incremental | Reproducible |
|--------|-------------|-------------|--------------|
| Cargo  | ✅ Fast     | ✅ Fastest  | ❌ No        |
| Nix    | ⚠️ Slower*  | ⚠️ Slower*  | ✅ Yes       |

*Nix builds are slower initially but benefit from caching and can be shared across machines.

## Future Enhancements

- [ ] Cross-compilation support for more targets
- [ ] Nix-based CI cache optimization
- [ ] Container image generation
- [ ] NixOS module for system-wide installation

## See Also

- [WARP.md](../WARP.md) - Comprehensive development guide
- [BUILD_SYSTEM.md](BUILD_SYSTEM.md) - Platform-aware build documentation
- [Nix Flakes Documentation](https://nixos.wiki/wiki/Flakes)
- [direnv Documentation](https://direnv.net/)
