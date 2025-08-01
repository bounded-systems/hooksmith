# Pushd CLI Prototype Worktree

This document describes the CLI prototype worktree and its safety measures to prevent accidental merges into main.

## Overview

The CLI prototype has been moved to a dedicated worktree to keep the main repository clean and organized. This worktree contains a Rust-based CLI tool that demonstrates WebAssembly Components for modular, language-agnostic functionality.

## Worktree Structure

```
/Users/bobby/dev/repos/pushd-cli-prototype/
├── README.md                    # Comprehensive CLI documentation
├── docs/                        # Detailed documentation
│   ├── COMPONENT_ARCHITECTURE.md
│   ├── COMPONENT_DEVELOPMENT.md
│   ├── WIT_REFERENCE.md
│   └── CLI_DESIGN.md
├── src/                         # CLI source code
├── components/                  # Wasm components
├── wit/                         # WIT interface definitions
├── scripts/git/hooks/          # Git hooks and safety scripts
└── lefthook.yml                # Lefthook configuration
```

## Safety Measures

### 1. Dedicated Branch

The CLI prototype uses its own branch (`pushd-cli-prototype`) that tracks `origin/pushd-cli-prototype`, ensuring it never accidentally uses main as upstream.

### 2. Pre-push Safety Hook

A custom pre-push hook prevents pushing to main from the CLI prototype worktree:

```bash
# This will be blocked:
git push origin pushd-cli-prototype:main

# This is the correct way:
git push origin pushd-cli-prototype
```

The safety hook provides helpful error messages and suggestions for proper workflow.

### 3. GitHub Branch Protection

The main branch is protected on GitHub, preventing force pushes and requiring pull requests for merges.

## Usage

### Working with the CLI Prototype

```bash
# Switch to the CLI prototype worktree
cd ../pushd-cli-prototype

# Make changes and commit
git add .
git commit -m "feat: add new CLI feature"

# Push to the correct branch
git push origin pushd-cli-prototype

# Create a pull request (if needed)
gh pr create --title "CLI Prototype: new feature" --body "..."
```

### Safety Check Verification

The safety check runs automatically on every push attempt:

```bash
# This will show the safety check in action
git push origin pushd-cli-prototype:main
# ❌ ERROR: Cannot push to main from pushd-cli-prototype branch!
```

## Development Workflow

### 1. Feature Development

1. Work in the CLI prototype worktree
2. Make changes and test locally
3. Commit changes to the `pushd-cli-prototype` branch
4. Push to `origin/pushd-cli-prototype`

### 2. Integration with Main

1. Create a pull request from `pushd-cli-prototype` to `main`
2. Get code review and approval
3. Merge through GitHub's protected workflow

### 3. Emergency Override

If absolutely necessary (emergency only):

```bash
git push origin pushd-cli-prototype:main --force
```

This will be blocked by GitHub's branch protection, requiring admin override.

## Benefits

### 1. Clean Separation

- CLI prototype development is isolated from main repository
- No risk of accidental merges into main
- Clear boundaries between experimental and production code

### 2. Safety First

- Multiple layers of protection prevent mistakes
- Clear error messages guide developers to correct workflow
- GitHub protection provides final safety net

### 3. Organized Development

- Dedicated worktree for CLI development
- Comprehensive documentation in one place
- Easy to manage and maintain

## Documentation

The CLI prototype includes comprehensive documentation:

- **README.md**: Overview, installation, usage, and examples
- **docs/COMPONENT_ARCHITECTURE.md**: Detailed Wasm component architecture
- **docs/COMPONENT_DEVELOPMENT.md**: Guide for creating new components
- **docs/WIT_REFERENCE.md**: WIT interface documentation
- **docs/CLI_DESIGN.md**: CLI design decisions and patterns

## Future Enhancements

1. **Plugin System**: Runtime plugin loading
2. **Remote Components**: Load components from registries
3. **Hot Reloading**: Update components without restart
4. **Performance Monitoring**: Track component performance
5. **Multi-language Support**: Components in TypeScript, Go, C

## Resources

- [Wasm Component Model](https://component-model.bytecodealliance.org/)
- [WIT Specification](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md)
- [Wasmtime Documentation](https://docs.wasmtime.dev/)
- [Bytecode Alliance](https://bytecodealliance.org/) 
