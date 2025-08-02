# Hooksmith Bootstrap

This directory contains a bootstrap script that can generate the complete Hooksmith project structure from scratch, even without an existing `Cargo.toml`.

## Prerequisites

1. **Install cargo-script** (required to run the bootstrap script):
   ```bash
   cargo install cargo-script
   ```

2. **Ensure you have Rust and Cargo installed** (the bootstrap script will use these to build the project)

## Usage

### Option 1: Run the bootstrap script directly

```bash
cargo script bootstrap.rs
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

## Project Structure Generated

After running the bootstrap script, you'll have:

```
hooksmith/
├── Cargo.toml                    # Main workspace configuration
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

## Troubleshooting

### If cargo-script is not found
Make sure you have installed it:
```bash
cargo install cargo-script
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

## Manual Setup (Alternative)

If you prefer not to use the bootstrap script, you can manually:

1. Copy the existing `Cargo.toml` from a working Hooksmith repository
2. Create the component directories and their `Cargo.toml` files
3. Run `cargo build` to verify everything works

The bootstrap script just automates this process and ensures consistency. 
