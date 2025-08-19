# Hooksmith Development Environment

This document explains how to set up and use the Hooksmith development environment using Nix, DevContainers, and various development tools.

## Quick Start

### Option 1: Local Development with Nix (Recommended)

**Prerequisites:**
- [Nix](https://nixos.org/download.html) installed with flakes enabled
- [direnv](https://direnv.net/) installed (optional but recommended)

**Setup:**
```bash
git clone https://github.com/bdelanghe/hooksmith.git
cd hooksmith

# Option A: With direnv (automatic environment)
direnv allow
just bootstrap

# Option B: Manual Nix shell
nix develop
just bootstrap
```

### Option 2: DevContainer/DevPod

**Prerequisites:**
- [DevPod](https://devpod.sh/) or VS Code with DevContainers extension
- Docker Desktop

**Setup:**
```bash
# With DevPod
devpod up github.com/bdelanghe/hooksmith

# With VS Code
# Open in VS Code and select "Reopen in Container"
```

### Option 3: GitHub Codespaces

Click the "Code" button on GitHub and select "Create codespace on main".

## Development Workflows

### Initial Setup

After entering any development environment:

```bash
# Bootstrap the development environment
just bootstrap

# Verify everything works
just info
cargo --version
rustc --version
```

### Code Quality

```bash
# Format all code
just fmt

# Lint all code  
just lint

# Run all pre-commit checks
pre-commit run --all-files

# Run tests
cargo test --all
```

### Building and Testing

```bash
# Build the entire workspace
cargo build

# Build via xtask
cargo run -p xtask -- build

# Run specific analysis tool
cargo run --bin repository_size_auditor

# Using Nix builds (optimized)
nix build .#analysis-tools
nix run .#repository_size_auditor
```

## Environment Details

### Nix Flake Configuration

The `flake.nix` provides:

- **Rust toolchain**: Stable Rust with cargo, rustfmt, clippy
- **Development tools**: just, git, jq, ripgrep, fd, tree
- **Analysis tools**: Custom Hooksmith binaries via Nix builds  
- **Pre-commit hooks**: Automatic code quality checks
- **Cross-platform**: Linux (x86_64, aarch64) and macOS (Intel, Apple Silicon)

### Available Commands

| Command | Description |
|---------|-------------|
| `just bootstrap` | Initial environment setup |
| `just info` | Show environment information |
| `just fmt` | Format all code |
| `just lint` | Lint all code |
| `just test` | Run all tests |
| `cargo build` | Build workspace |
| `cargo test` | Run tests |
| `nix build .#analysis-tools` | Build analysis tools via Nix |
| `nix run .#repository_size_auditor` | Run specific tool |
| `pre-commit run --all-files` | Run all pre-commit checks |

### Pre-commit Hooks

Automatically configured hooks include:

- **Rust**: rustfmt, clippy with warnings as errors
- **Nix**: alejandra (formatter), statix (linter), deadnix (dead code)
- **Shell**: shellcheck (linter), shfmt (formatter)
- **General**: prettier (JS/JSON/YAML/MD), trailing whitespace, end-of-file fixes

### DevContainer Features

The `.devcontainer/devcontainer.json` provides:

- **Base image**: `ghcr.io/cachix/devenv:latest` (Nix pre-installed)
- **VS Code extensions**: Rust Analyzer, Nix IDE, Docker, GitHub Actions
- **Port forwarding**: 3000, 8080 for development servers
- **Post-create command**: Automatic `just bootstrap` execution

## Troubleshooting

### Nix Issues

```bash
# Clear Nix cache if builds fail
nix-collect-garbage -d
nix develop --refresh

# Update flake inputs
nix flake update
```

### direnv Issues

```bash
# Allow direnv for the project
direnv allow

# Reload direnv
direnv reload
```

### DevContainer Issues

```bash
# Rebuild container
# In VS Code: Cmd/Ctrl + Shift + P → "Dev Containers: Rebuild Container"

# With DevPod
devpod delete hooksmith
devpod up github.com/bdelanghe/hooksmith
```

### Pre-commit Issues

```bash
# Install pre-commit hooks
pre-commit install

# Update hooks
pre-commit autoupdate

# Skip hooks temporarily
git commit --no-verify
```

## CI/CD

### GitHub Actions

The `.github/workflows/ci.yml` runs:

- **Nix checks**: Build all packages, run pre-commit checks
- **Format checking**: Rust, Nix, and shell formatting validation
- **Testing**: Unit tests, integration tests, documentation tests
- **Security**: Cargo audit and deny checks
- **Cross-platform**: Ubuntu and macOS testing
- **DevContainer**: Container build verification

### Cachix Setup

For faster CI builds, we use Cachix for binary caching:

1. Create account at [cachix.org](https://cachix.org)
2. Create cache named "hooksmith"
3. Add `CACHIX_AUTH_TOKEN` to GitHub repository secrets

## Performance Tips

### Nix Development

- Use `nix develop` for consistent environments
- Cache builds with Cachix for faster CI
- Use `nix run` for one-off tool execution
- Leverage `nix build` for optimized release builds

### Local Development

- Use `direnv` for automatic environment activation
- Run `just bootstrap` only once per environment
- Use `cargo check` for faster feedback during development
- Use `bacon` for continuous compilation checking

### DevContainer Development

- Use volume mounts for better performance
- Prebuild images for faster startup
- Use DevPod for local container management

## Advanced Usage

### Custom Nix Configuration

```bash
# Override inputs locally
nix develop --override-input nixpkgs github:NixOS/nixpkgs/nixos-23.11

# Use different Rust version
nix develop --override-input fenix github:nix-community/fenix/monthly
```

### Pre-commit Customization

Edit `.pre-commit-config.yaml` to customize hooks or add new ones.

### justfile Customization

The `justfile` can be extended with project-specific commands. See existing commands for examples.

## Getting Help

- **Documentation**: Check `WARP.md` for comprehensive project documentation
- **Issues**: Open GitHub issues for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make changes following the code quality standards
4. Run `just lint` and `just test` to verify changes
5. Submit a pull request

The CI will automatically run all checks including formatting, linting, testing, and security audits.
