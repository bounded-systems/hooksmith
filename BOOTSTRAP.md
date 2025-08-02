# Hooksmith Bootstrap

This directory contains a bootstrap script that can generate the complete Hooksmith project structure from scratch, even without an existing `Cargo.toml`.

## Prerequisites

1. **Install cargo-eval** (required to run the bootstrap script):
   ```bash
   cargo install cargo-eval
   ```

2. **Ensure you have Rust and Cargo installed** (the bootstrap script will use these to build the project)

## Usage

### Option 1: Run the bootstrap script directly

```bash
cargo eval bootstrap.rs
```

### Option 2: Make it executable and run

```bash
chmod +x bootstrap.rs
./bootstrap.rs
```

## What the Bootstrap Script Does

The bootstrap script performs the following steps:

1. **Generates the main `Cargo.toml`** - Creates the workspace configuration with all dependencies
2. **Creates component directories** - Sets up the modular component structure
3. **Generates component `Cargo.toml` files** - Creates individual package configurations for each component
4. **Generates `xtask/Cargo.toml`** - Sets up the build tool configuration
5. **Builds xtask** - Compiles the build tool so it's ready to use
6. **Generates documentation** - Uses the existing documentation generation system to create README.md and other docs

## Project Structure Generated

After running the bootstrap script, you'll have:

```
hooksmith/
├── Cargo.toml                    # Main workspace configuration
├── README.md                     # Generated from source using doc gen system
├── components/
│   ├── cli-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── worktree-runner/
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── git-filter/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── hook-builder/
│       ├── Cargo.toml
│       └── src/
├── lefthook-rs/
│   ├── Cargo.toml
│   └── src/
└── xtask/
    ├── Cargo.toml
    └── src/
```

## Dependencies Included

The bootstrap script generates a complete workspace with all necessary dependencies:

### Workspace Dependencies
- **CLI Framework**: `clap`, `tokio`, `anyhow`, `console`, `indicatif`
- **File System**: `serde`, `serde_json`, `serde_yaml`, `toml`
- **Git Operations**: `git2`, `gix-filter`
- **WASM Support**: `wasmtime`, `wit-bindgen`, `wit-parser`, `wit-component`
- **Validation**: `jsonschema`, `reqwest`
- **Utilities**: `sha2`, `chrono`, `thiserror`, `once_cell`, `regex`, `futures-io`

### Component-Specific Dependencies
Each component gets its own `Cargo.toml` with workspace dependencies, ensuring consistent versions across the project.

## Documentation Generation

The bootstrap script uses Hooksmith's existing documentation generation system:

- **README.md** - Generated from source code using `xtask gen-readme`
- **Comprehensive docs** - Uses `xtask gen-docs-comprehensive` for full documentation
- **Source-based** - All documentation is generated from actual source code, not templates
- **Consistent** - Uses the same generation system as the main project

## Next Steps

After running the bootstrap script:

1. **Build the project**:
   ```bash
   cargo build
   ```

2. **Run tests**:
   ```bash
   cargo test
   ```

3. **Use xtask commands**:
   ```bash
   ./target/debug/xtask --help
   ```

4. **Regenerate documentation** (if needed):
   ```bash
   ./target/debug/xtask gen-docs-comprehensive --all
   ```

## Troubleshooting

### If cargo-eval is not found
Make sure you have installed it:
```bash
cargo install cargo-eval
```

### If the build fails
The bootstrap script requires a working Rust toolchain. Make sure:
- Rust is installed and up to date: `rustup update`
- Cargo is available: `cargo --version`

### If you get permission errors
Make sure the script is executable:
```bash
chmod +x bootstrap.rs
```

### If dependencies are missing
The bootstrap script includes all dependencies used by the existing source code. If you encounter missing dependency errors, they may be from newer code that wasn't included in the bootstrap template.

### If documentation generation fails
The bootstrap script will continue even if documentation generation fails. You can manually regenerate docs later:
```bash
./target/debug/xtask gen-docs-comprehensive --all
```

## Manual Setup (Alternative)

If you prefer not to use the bootstrap script, you can manually:

1. Copy the existing `Cargo.toml` from a working Hooksmith repository
2. Create the component directories and their `Cargo.toml` files
3. Run `cargo build` to verify everything works
4. Use `xtask gen-docs-comprehensive --all` to generate documentation

The bootstrap script just automates this process and ensures consistency.

## Script Files

- `bootstrap.rs` - The working bootstrap script using `cargo-eval` 
