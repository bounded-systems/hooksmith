# Git Blob Analysis Tools

Fast, workspace-style Rust tools for inspecting Git storage, performance, and repo hygiene. Built for large monorepos and CI.

## Quick start

```bash
# Run a tool directly (Cargo)
cargo run --bin repository_size_auditor

# Or use the meta CLI (recommended)
cargo run -p gba -- --help

# Reproducible (Nix)
nix run .#gba -- --help
```

## Tool matrix

| Binary | What it answers | Typical use | Example |
|--------|----------------|-------------|---------|
| `repository_size_auditor` | "Is this repo within healthy size limits?" | CI gate | `cargo run --bin repository_size_auditor -- --fail-on 1` |
| `rust_blob_analyzer` | "Which .rs files are biggest/hottest?" | Dev focus | `cargo run --bin rust_blob_analyzer` |
| `git_delta_analyzer` | "Where can delta compression save space?" | Storage tuning | `cargo run --bin git_delta_analyzer` |
| `git_hygiene_reporter` | "What should be ignored / moved to LFS?" | Hygiene | `cargo run --bin git_hygiene_reporter --format md` |
| `git_lfs_analyzer` / `git_lfs_auto_tracker` | "What should be LFS?" | LFS migration | `cargo run --bin git_lfs_auto_tracker` |
| `packfile_delta_analyzer` | "What's actually in .pack?" | Low-level audit | `cargo run --bin packfile_delta_analyzer` |
| `file_churn_analyzer` | "What files churn the most?" | Extraction picks | `cargo run --bin file_churn_analyzer "6 months ago"` |
| `tree_object_stability_auditor` | "Which trees are unstable?" | Boundaries | `cargo run --bin tree_object_stability_auditor` |
| `tree_to_repo_extractor` | "Split a dir into a repo (with history)?" | Extraction | `cargo run --bin tree_to_repo_extractor scripts ../new-repo` |

Many tools support `--format table|json|md` and `--fail-on <N>` to fail CI when thresholds are exceeded.

## Installation

- **Rust**: 1.88+ (stable).
- **Optional (recommended)**: Nix flakes for hermetic builds and caching.

```bash
# Dev shell with pinned toolchain
nix develop
# Then use Cargo normally
cargo build
```

## Project Structure

```
hooksmith/
├── .config/                    # Real configuration files
│   ├── .editorconfig          # Editor configuration
│   ├── lefthook.yml           # Git hooks configuration
│   └── lefthook-example.yml   # Example hooks configuration
├── crates/                    # Rust crates
│   ├── gba/                   # Git Blob Analysis meta CLI
│   ├── xtask/                 # Build system and tools
│   │   └── src/config/        # Xtask configuration files
│   │       ├── component-registry.jsonc
│   │       ├── event-registry.jsonc
│   │       ├── github-actions.jsonc
│   │       └── ...
│   └── ...                    # Other crates
├── data/                      # Large data files
│   └── languages.yml          # Language definitions
├── docs/                      # Documentation
│   └── examples/              # Example outputs
│       └── enhanced-contract-validation-results.sarif
└── .github/                   # GitHub-specific files
    └── inputs/                # Test input files for workflows
```

## Workspace layout & defaults

Make one crate your default target (meta CLI `gba`):

```toml
# Cargo.toml (root)
[workspace]
members = [
  "crates/gba",                # meta CLI (recommended)
  "crates/repository_size_auditor",
  "crates/rust_blob_analyzer",
  "crates/git_delta_analyzer",
  # ...others
]
default-members = ["crates/gba"]
resolver = "2"
```

## Meta CLI (optional but nice)

Create `crates/gba` that re-exports subcommands (clap) so users can run:

```bash
cargo run -p gba -- rust-blob
cargo run -p gba -- repo-audit --fail-on 1
```

## Justfile (developer ergonomics)

```makefile
MAIN_PKG := gba

default: build

build:
    cargo build -p {{MAIN_PKG}}

run *ARGS:
    cargo run -p {{MAIN_PKG}} -- {{ARGS}}

test:
    cargo test --workspace

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

nix-build:
    nix build .#{{MAIN_PKG}}
```

## Nix flake (default package/app)

Point Nix defaults to the meta CLI (or your chosen main crate):

```nix
# flake.nix (excerpt)
packages.${system}.gba = craneLib.buildPackage {
  pname = "gba";
  src = craneLib.cleanCargoSource (craneLib.path ./.);
  cargoExtraArgs = "--package gba";
};
packages.${system}.default = packages.${system}.gba;
apps.${system}.default = flake-utils.lib.mkApp {
  drv = packages.${system}.gba;
  exePath = "/bin/gba";
};
```

## Configuration (env overrides)

All tools read sensible defaults and accept flags. You can also override via env:

- **Repo/paths**
  - `GBA_REPO_PATH` (default: `.`)
  - `GBA_GIT_ARGS` (extra args for internal git calls)
- **Thresholds**
  - `GBA_RUST_FILE_WARN_KB` (default: 100)
  - `GBA_LFS_MIN_MB` (default: 50)
  - `GBA_MAX_REPO_SIZE_MB` (for repository_size_auditor)
  - `GBA_CHURN_SINCE` (e.g., `3 months ago`)
- **Output/behavior**
  - `GBA_OUTPUT = table|json|md`
  - `GBA_FAIL_ON = integer` → non-zero exit if findings ≥ N

Flags always win over env.

## CI usage

```yaml
# .github/workflows/ci.yml (snippet)
- name: Build
  run: cargo build --workspace --locked

- name: Lints (fail on warnings)
  run: cargo clippy --workspace --all-targets -- -D warnings

- name: Repo audit (CI gate)
  run: cargo run -p repository_size_auditor -- --fail-on 1 --format json
```

## Tips & best practices

- Keep `.rs` files < ~100KB where possible; split hot `main.rs`.
- Track big binaries with LFS; mark truly binary types `-delta` in `.gitattributes`.
- Repack periodically: `git repack -Ad --window=250 --depth=50`.
- Use `--fail-on` in CI to keep things green and fast.

## Roadmap

- SARIF export for code scanning
- Mermaid graphs for stability trees
- Git notes for inline findings
- Auto-apply: optional "fix-it" mode for hygiene
