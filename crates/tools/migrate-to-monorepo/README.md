# Migrate to Monorepo

A Rust tool for migrating the Hooksmith repository to a monorepo orchestrator layout.

## Overview

This tool transforms the repository from a mixed structure to a clean monorepo layout where:

- **Root** serves as an orchestrator for repo-wide policy and tooling
- **Subtrees** are organized by concern (crates, apps, tools, infra, docs, etc.)
- **No source code** exists at the root level

## Usage

### Dry Run (Preview)
```bash
cargo run -- --dry-run
```

Shows what files and directories would be moved without making any changes.

### Execute Migration
```bash
cargo run -- --execute
```

Performs the actual migration, moving files and creating the new structure.

## Migration Plan

### Directories Created
- `crates/docs/summaries/` - Implementation summaries and reports
- `.hooksmith/schemas/` - Machine-checked schemas
- `crates/apps/` - Binaries/CLIs/services
- `tools/` - Dev tooling, analyzers, generators
- `infra/` - Deploy, IaC, pipelines
- `crates/examples/` - Repo-wide examples
- `crates/tests/` - Integration tests
- `.hooksmith/` - Hooksmith configuration (with subdirectories)

### Files Moved
- **Documentation**: `*_SUMMARY.md` → `crates/docs/summaries/`
- **Configuration**: `languages.yml` → `.hooksmith/schemas/`, `lefthook.yml` → `.github/`
- **Hooksmith**: `contracts/` → `.hooksmith/git/agreements/`, `contract_snapshots/` → `.hooksmith/snapshots/`
- **Source Code**: `src/` → `crates/apps/hooksmith-core`, `standalone-auditor/` → `crates/apps/`
- **Tools**: `scripts/` → `crates/scripts/`, `hooks/` → `.git/hooks/` (automatically installed)
- **Infrastructure**: `config/` → `infra/config-model`, `Dockerfile` → `infra/`

### Files Created
- `.hooksmith/.gitignore` - Local ignore for cache and logs

## Target Structure

After migration, the root will contain only orchestration files:

```
/
├── .gitignore          # Required
├── .gitattributes      # Git metadata
├── .github/            # CI/CD
├── .hooksmith/         # All Hooksmith configuration
├── Cargo.toml          # Workspace manifest only
├── Cargo.lock          # Workspace lockfile
├── README.md           # Project overview
├── CONTRIBUTING.md     # Contribution guidelines
├── LICENSE             # License
├── crates/             # Shared libraries
├── crates/apps/        # Binaries/CLIs/services
├── tools/              # Dev tooling, analyzers, generators
├── infra/              # Deploy, IaC, pipelines
├── crates/docs/        # Documentation
├── .hooksmith/schemas/ # Machine-checked schemas
├── contracts/          # Optional if not under .hooksmith/
├── crates/examples/    # Repo-wide examples
└── crates/tests/       # Integration tests
```

## Benefits

1. **Root Stability**: No source code at root, stable tree SHA
2. **Clear Organization**: Each subtree has focused responsibility
3. **Break-Off Ready**: Each subtree can become its own repository
4. **Performance**: Better caching and faster builds
5. **Developer Clarity**: Clear separation of orchestration vs implementation

## Safety

- The tool uses `git mv` to preserve file history
- Dry run mode allows previewing changes
- Graceful handling of missing files (skips with warning)
- Creates backup structure before moving files

## Post-Migration

After running the migration:

1. Review changes: `git status`
2. Test contract validation:
   ```bash
   cd crates/apps/standalone-auditor
   cargo run -- HEAD ../../.hooksmith/git/agreements/object-names@root-minimal.json
cargo run -- HEAD ../../.hooksmith/git/agreements/object-names@v1.json
   ```
3. Commit changes: `git commit -m 'chore: migrate to monorepo orchestrator root layout'`
