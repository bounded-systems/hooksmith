# Monorepo Orchestrator Plan

## 🎯 **Vision: Root as Repo-Wide Policy & Tooling Orchestrator**

Transform Hooksmith into a **monorepo** where the root serves as an **orchestrator** for repo-wide policy, tooling, and documentation - not a code location. The root becomes the place for governance, not implementation.

## 🏗️ **Target Root Layout (Monorepo Orchestrator)**

### **Allowed at Root (Repo-Wide Orchestration Only)**
```
/
├── .gitignore          # Required
├── .gitattributes      # Git metadata
├── .github/            # CI/CD
├── .hooksmith/         # All Hooksmith contracts, actors, reports, caches
├── Cargo.toml          # Workspace manifest only (no [package])
├── Cargo.lock          # Workspace lockfile
├── README.md           # Project overview
├── CONTRIBUTING.md     # Contribution guidelines
├── LICENSE             # License
├── crates/             # Shared libraries
├── apps/               # Binaries/CLIs/services
├── tools/              # Dev tooling, analyzers, generators
├── infra/              # Deploy, IaC, pipelines
├── docs/               # Documentation
├── schemas/            # Machine-checked schemas
├── contracts/          # Optional if not under .hooksmith/
├── examples/           # Repo-wide examples
└── tests/              # Integration tests
```

### **Explicitly Rejected**
- `*.rs` - No source code at root
- `src/` - No source directory at root
- `target/` - No build artifacts at root
- `*.log` - No log files at root

## 🗂️ **Subtree Organization**

### **crates/** - Shared Libraries
- `hooksmith-core/` - Core library functionality
- `git-agreement/` - Git agreement handling
- `event-types/` - Event type definitions
- `file-operations/` - File operation handlers
- `git-operations/` - Git operation handlers
- `hooks/` - Git hooks implementation
- `lefthook-rs/` - Lefthook integration
- `snapshot/` - Snapshot functionality
- `tree/` - Tree manipulation utilities
- `xtask/` - Build system tasks

### **apps/** - Binaries/CLIs/Services
- `hooksmith-core/` - Main Hooksmith binary (moved from `src/`)
- `standalone-auditor/` - Standalone contract auditor
- `git-proxy/` - Git proxy service
- `file-operations-server/` - File operations server
- `git-operations-server/` - Git operations server

### **tools/** - Dev Tooling & Generators
- `scripts/` - CLI utilities and scripts
- `lefthook-rs/` - Lefthook implementation (moved from `hooks/`)
- `wit/` - WIT file handling
- `worktree-lifecycle/` - Worktree lifecycle management

### **infra/** - Infrastructure & Deployment
- `config-model/` - Configuration models (moved from `config/`)
- `schemas/` - Machine-checked schemas (moved from `schemas/`)
- `docker-compose.yml` - Docker composition
- `docker-bake.hcl` - Docker build configuration
- `Dockerfile` - Container definition

### **docs/** - Documentation
- `summaries/` - Implementation summaries and reports
- `guides/` - User and developer guides
- `api/` - API documentation
- `design/` - Architecture and design docs

## 🔧 **Implementation Components**

### **1. Updated Monorepo Root Contract**
- **File**: `contracts/object-names@root-minimal.json`
- **Strategy**: Allow orchestration dirs, reject source code, no exceptions needed
- **Status**: ✅ **Ready**

### **2. Enhanced Migration Script**
- **File**: `scripts/migrate-to-minimal-root.sh`
- **Features**: Creates monorepo structure, moves files to appropriate subtrees
- **Status**: ✅ **Ready**

### **3. Workspace-Only Cargo.toml**
- **Strategy**: Remove `[package]` section, keep only `[workspace]` with members
- **Status**: 🔄 **Needs update**

## 🚀 **Migration Commands**

### **Execute Migration**
```bash
# Run migration script
./scripts/migrate-to-minimal-root.sh

# Review changes
git status

# Test monorepo root contract
cd standalone-auditor
cargo run -- HEAD ../contracts/object-names@root-minimal.json

# Commit changes
git commit -m "chore: migrate to monorepo orchestrator root layout"
```

### **Post-Migration Usage**
```bash
# Test with new contract paths
cargo run -- HEAD .hooksmith/agreements/object-names@v1.json
cargo run -- HEAD .hooksmith/agreements/object-names@root-minimal.json
```

## 🔧 **Cargo.toml Workspace Conversion**

### **Current (Mixed Package/Workspace)**
```toml
[package]
name = "hooksmith"
version = "0.1.0"
# ... package config

[workspace]
members = [
    "crates/*",
    # ... other members
]
```

### **Target (Workspace Only)**
```toml
[workspace]
members = [
    "crates/*",
    "apps/*",
    "tools/*",
    "infra/*",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Pushd Team"]
license = "MIT"

[workspace.dependencies]
# ... shared dependencies
```

## 🎯 **Benefits of Monorepo Orchestrator**

### **Root Stability**
- **No source code at root**: Prevents "root as crate" anti-pattern
- **Stable tree SHA**: Root becomes extremely stable, better cache performance
- **Clean history**: Easy to filter when splitting repos later

### **Clear Separation of Concerns**
- **Orchestration vs Implementation**: Root handles policy, subtrees handle code
- **Predictable organization**: Each subtree has a clear purpose
- **Easy discovery**: Developers know where to find different types of artifacts

### **Subtree Break-Off Ready**
- **Independent subtrees**: Each subtree can become its own repository
- **Clean boundaries**: No cross-contamination between different concerns
- **Stable interfaces**: Root provides stable orchestration layer

### **Performance Benefits**
- **Faster builds**: Workspace-only root, focused subtrees
- **Better caching**: Stable root SHA improves contract cache performance
- **Reduced churn**: Root changes signal policy/tooling, not product code

## 📊 **Current Status**

### **Pre-Migration**
- **Root files**: 70 total (including source code and mixed concerns)
- **Monorepo root contract**: ❌ **70 violations** (expected)
- **Rejected**: 8 files (source code: `*.rs`, `src/`)
- **Not allowed**: 62 files (mixed concerns, config, etc.)

### **Post-Migration Target**
- **Root files**: ~20 total (orchestration only)
- **Monorepo root contract**: ✅ **0 violations**
- **Clear organization**: Each subtree has focused responsibility

## 🔄 **Migration Steps**

### **Phase 1: Preparation**
1. ✅ Create updated monorepo root contract
2. ✅ Enhance migration script with monorepo structure
3. 🔄 Update Cargo.toml to workspace-only
4. 🔄 Update CI/CD to use new structure

### **Phase 2: Execution**
1. **Run migration script** to create monorepo structure
2. **Move source code** to `apps/` and `crates/`
3. **Move tools** to `tools/`
4. **Move infrastructure** to `infra/`
5. **Test contract validation**

### **Phase 3: Integration**
1. **Update CI/CD** to use new contract paths
2. **Update documentation** to reference monorepo structure
3. **Train team** on new organization
4. **Plan subtree break-offs** for individual components

## 🛡️ **Enforcement Strategy**

### **CI/CD Integration**
```yaml
# .github/workflows/contract-validation.yml
- name: Validate monorepo root
  run: |
    cd apps/standalone-auditor
    cargo run -- HEAD ../../.hooksmith/agreements/object-names@root-minimal.json
```

### **Protected References**
- Maintain `refs/hooksmith/scopes/project-root` pointing to root tree SHA
- Compare new PRs against this reference for drift detection

### **Developer Experience**
- Clear documentation of monorepo structure
- Workspace-aware tooling and scripts
- Migration script for bulk operations

## 🎉 **Success Metrics**

### **Immediate**
- ✅ Root reduced from 70 to ~20 files
- ✅ Clear separation of orchestration vs implementation
- ✅ Contract validation passes with 0 violations

### **Long-term**
- **Stable root SHA**: Better cache performance
- **Clean break-offs**: Easy subtree extraction
- **Developer clarity**: Clear organization of concerns
- **Performance**: Faster builds and better caching

This monorepo orchestrator approach transforms Hooksmith into a **clean, organized, and break-off ready repository** with a root that serves as a true orchestrator for repo-wide policy and tooling.
