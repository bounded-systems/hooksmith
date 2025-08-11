# Hooksmith Consolidation Plan: .hooksmith/ Approach

## 🎯 **Vision: Minimal Root with .hooksmith/ Consolidation**

Transform Hooksmith into a repository with a **truly minimal root** by consolidating all Hooksmith-specific configuration into a single, well-organized `.hooksmith/` directory. This approach keeps the root clean while providing a focused namespace for all Hooksmith artifacts.

## 🏗️ **Target Root Layout (Ultra-Minimal)**

### **Allowed at Root (Essential Only)**
```
/
├── .gitignore          # Required
├── .gitattributes      # Git metadata
├── .hooksmith/         # All Hooksmith configuration
├── Cargo.toml          # Workspace root
├── README.md           # Project overview
├── LICENSE*            # License files
├── rust-toolchain.toml # Toolchain config
├── deny.toml           # Security policy
├── clippy.toml         # Linting config
├── rustfmt.toml        # Formatting config
├── crates/             # All first-party code
├── docs/               # All human documentation
├── scripts/            # CLI utilities
└── schemas/            # Machine-checked schemas
```

### **Explicitly Rejected**
- `contracts/` → moves to `.hooksmith/agreements/`
- `contract_snapshots/` → moves to `.hooksmith/snapshots/`

## 🗂️ **.hooksmith/ Directory Structure**

```
.hooksmith/
├── agreements/         # JSON contracts (e.g., object-names@v1.json)
├── policies/           # schemas & policy assets (optional)
├── actors/             # researcher/reporter/mandator/auditor/triage config
├── snapshots/          # immutable mandate/report outputs
├── cache/              # memoized fix plans, LRU files (gitignored)
├── hooks/              # hook templates or generated hooks (optional)
├── logs/               # NDJSON/proto reports (gitignored)
├── refs/               # small text pointers to git refs/tree SHAs (optional)
└── .gitignore          # local ignore for cache/logs
```

## 🔧 **Implementation Components**

### **1. Updated Minimal Root Contract**
- **File**: `contracts/object-names@root-minimal.json`
- **Strategy**: Default reject, allow exceptions, explicitly reject `contracts/` and `contract_snapshots/`
- **Status**: ✅ **Ready**

### **2. Enhanced Migration Script**
- **File**: `scripts/migrate-to-minimal-root.sh`
- **Features**: Creates `.hooksmith/` structure, moves contracts, sets up `.gitignore`
- **Status**: ✅ **Ready**

### **3. Rust Helper Module**
- **File**: `crates/hooksmith-core/src/hooksmith_dir.rs`
- **Features**: Environment-aware path resolution, subdirectory helpers
- **Status**: ✅ **Ready**

### **4. Updated Auditor**
- **File**: `standalone-auditor/src/main.rs`
- **Features**: Updated examples to use `.hooksmith/agreements/` paths
- **Status**: ✅ **Ready**

## 🚀 **Migration Commands**

### **Execute Migration**
```bash
# Run migration script
./scripts/migrate-to-minimal-root.sh

# Review changes
git status

# Test minimal root contract
cd standalone-auditor
cargo run -- HEAD ../contracts/object-names@root-minimal.json

# Commit changes
git commit -m "chore(hooksmith): consolidate repo config into .hooksmith/"
```

### **Post-Migration Usage**
```bash
# Test with new contract paths
cargo run -- HEAD .hooksmith/agreements/object-names@v1.json
cargo run -- HEAD .hooksmith/agreements/object-names@root-minimal.json
```

## 🔧 **Rust Integration**

### **Hooksmith Directory Resolution**
```rust
use hooksmith_core::hooksmith_dir;

// Get the Hooksmith directory (respects HOOKSMITH_DIR env var)
let hooksmith_dir = hooksmith_dir::hooksmith_dir();

// Get specific subdirectories
let agreements_dir = hooksmith_dir::agreements_dir();
let snapshots_dir = hooksmith_dir::snapshots_dir();
let actors_dir = hooksmith_dir::actors_dir();
let cache_dir = hooksmith_dir::cache_dir();
let logs_dir = hooksmith_dir::logs_dir();
let hooks_dir = hooksmith_dir::hooks_dir();
let refs_dir = hooksmith_dir::refs_dir();

// Ensure all directories exist
hooksmith_dir::ensure_hooksmith_dirs()?;
```

### **Environment Variable Support**
```bash
# Use custom Hooksmith directory
export HOOKSMITH_DIR=/custom/hooksmith
cargo run --bin hooksmith-auditor

# Use default (.hooksmith)
cargo run --bin hooksmith-auditor
```

## 🎯 **Benefits of .hooksmith/ Consolidation**

### **Root Stability**
- **Ultra-minimal root**: Only essential files at repository root
- **Stable tree SHA**: Root becomes extremely stable, better cache performance
- **Clean history**: Easy to filter when splitting repos later

### **Focused Namespace**
- **Single location**: All Hooksmith artifacts in one place
- **Clear organization**: Logical subdirectory structure
- **Easy discovery**: Developers know where to find Hooksmith config

### **Sandboxing Support**
- **Git object access**: Can read config as Git objects (no FS access)
- **Ref-based operation**: Store tree SHA in `refs/hooksmith/config`
- **Isolated execution**: Actors can run against Git objects only

### **Future Flexibility**
- **Subtree break-offs**: `.hooksmith/` can move with a crate or stay at workspace root
- **Multi-repo support**: Each repo can have its own `.hooksmith/` configuration
- **Centralized config**: Can reference shared policies from a central location

## 📊 **Current Status**

### **Pre-Migration**
- **Root files**: 78 total (including contracts and contract_snapshots)
- **Minimal root contract**: ❌ **78 violations** (expected)
- **Missing required**: 2 files (`.gitignore`, `crates`)
- **Rejected**: 76 files (including `contracts/`, `contract_snapshots/`)

### **Post-Migration Target**
- **Root files**: ~15 total (minimal set)
- **Minimal root contract**: ✅ **0 violations**
- **Hooksmith config**: Consolidated in `.hooksmith/`

## 🔄 **Migration Steps**

### **Phase 1: Preparation**
1. ✅ Create updated minimal root contract
2. ✅ Enhance migration script with `.hooksmith/` structure
3. ✅ Create Rust helper module
4. ✅ Update auditor examples

### **Phase 2: Execution**
1. **Run migration script** to create `.hooksmith/` structure
2. **Move contracts** to `.hooksmith/agreements/`
3. **Move snapshots** to `.hooksmith/snapshots/`
4. **Set up `.gitignore`** for cache and logs
5. **Test contract validation**

### **Phase 3: Integration**
1. **Update CI/CD** to use new contract paths
2. **Update documentation** to reference `.hooksmith/` structure
3. **Train team** on new organization
4. **Plan subtree break-offs** for individual crates

## 🛡️ **Enforcement Strategy**

### **CI/CD Integration**
```yaml
# .github/workflows/contract-validation.yml
- name: Validate minimal root
  run: |
    cd standalone-auditor
    cargo run -- HEAD .hooksmith/agreements/object-names@root-minimal.json
```

### **Protected References**
- Maintain `refs/hooksmith/config` pointing to `.hooksmith/` tree SHA
- Compare new PRs against this reference for drift detection

### **Developer Experience**
- Clear documentation of `.hooksmith/` structure
- Rust helper functions for path resolution
- Migration script for bulk operations

## 🎉 **Success Metrics**

### **Immediate**
- ✅ Root reduced from 78 to ~15 files
- ✅ All Hooksmith config in single location
- ✅ Contract validation passes with 0 violations

### **Long-term**
- **Stable root SHA**: Better cache performance
- **Clean break-offs**: Easy subtree extraction
- **Developer clarity**: Clear organization of Hooksmith artifacts
- **Sandboxing**: Support for Git object-only operation

This consolidation approach transforms Hooksmith into a **clean, focused, and break-off ready repository** with a truly minimal root and a well-organized configuration namespace.
