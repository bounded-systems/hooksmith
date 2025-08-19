# Minimal Root Migration Plan

## 🎯 **Vision: Break-Off Ready Repository**

Transform Hooksmith into a repository with a **minimal, stable, and boring root** that's ready for clean subtree break-offs. The root should be like an API surface: tiny, stable, and predictable.

## 🏗️ **Target Root Layout**

### **Allowed at Root (Minimal Set)**
```
/
├── .gitignore          # Required
├── .gitattributes      # Git metadata
├── .github/            # CI/CD only
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

### **Everything Else Gets Pushed Down**
- **config** → `crates/config-model/`
- **generated** → `generated-sources/` or under each crate
- **summaries/reports** → `docs/summaries/`
- **hooks** → `crates/lefthook-rs/` (code) + CI glue in `.github/`
- **examples** → `examples/` or `crates/*/examples/`
- **tools** → `scripts/` or `crates/tools-*`

## 📋 **Implementation Strategy**

### **Phase 1: Baseline Establishment**
1. ✅ **Create baseline contract** - `object-names@current-baseline.json`
   - Matches current tree exactly
   - Provides safe starting point
   - Zero-diff validation

2. ✅ **Create minimal root contract** - `object-names@root-minimal.json`
   - Strict policy with exceptions
   - Default stance: reject everything
   - Allow only essential files
   - Exception carve-outs for specific files

3. ✅ **Enhanced auditor** - Supports exceptions field
   - Handles complex validation logic
   - Provides clear violation reporting
   - CI/CD friendly output

### **Phase 2: Migration Execution**
1. **Run migration script** - `scripts/migrate-to-minimal-root.sh`
   ```bash
   ./scripts/migrate-to-minimal-root.sh
   ```

2. **File movements**:
   ```
   *_SUMMARY.md → docs/summaries/
   *_IMPLEMENTATION_*.md → docs/summaries/
   languages.yml → schemas/
   lefthook.yml → .github/
   agreement.json → docs/
   contract_snapshots → docs/
   contracts → docs/
   generated-sources → crates/
   examples → crates/
   test-* → docs/test-files/
   scripts → crates/scripts-cli
   hooks → crates/lefthook-rs
   src → crates/
   wit → crates/
   worktree-lifecycle → crates/
   config → crates/config-model
   schemas → crates/schemas
   docs → crates/docs
   ```

### **Phase 3: Validation & Enforcement**
1. **Pre-merge CI**: Run auditor on `origin/main^{tree}` and `HEAD^{tree}`
2. **Protected ref**: Keep `refs/hooksmith/scopes/project-root` pointing to last compliant tree
3. **Local pre-commit**: Warn, don't block (for developer learning)

## 🔧 **Contracts**

### **Current Baseline Contract**
- **File**: `contracts/object-names@current-baseline.json`
- **Purpose**: Safe starting point, matches current tree exactly
- **Status**: ✅ **PASSING**

### **Minimal Root Contract**
- **File**: `contracts/object-names@root-minimal.json`
- **Purpose**: Enforce minimal root layout
- **Strategy**: Default reject, allow exceptions
- **Status**: ❌ **77 violations** (expected during migration)

## 🚀 **Migration Commands**

### **Test Current State**
```bash
# Test baseline contract (should pass)
cd standalone-auditor
cargo run -- HEAD ../contracts/object-names@current-baseline.json

# Test minimal root contract (will show violations)
cargo run -- HEAD ../contracts/object-names@root-minimal.json
```

### **Execute Migration**
```bash
# Run migration script
./scripts/migrate-to-minimal-root.sh

# Review changes
git status

# Test minimal root contract again
cd standalone-auditor
cargo run -- HEAD ../contracts/object-names@root-minimal.json

# Commit changes
git commit -m "Migrate to minimal root layout"
```

## 🎯 **Benefits of Minimal Root**

### **Break-Off Ready**
- Each folder under `crates/` becomes a clean subtree boundary
- Stable tree SHA for each component
- Clean history filtering when splitting repos

### **Performance**
- Root SHA becomes extremely stable
- Better contract cache hit rates
- Fewer packfile rewrites
- Fewer merge conflicts

### **Maintainability**
- Clear separation of concerns
- Predictable file locations
- Easier onboarding for new contributors

## 📊 **Current Status**

### **Baseline Contract**
- ✅ **PASSING** - Matches current tree exactly
- **Violations**: 0
- **Files**: 77 total at root

### **Minimal Root Contract**
- ❌ **FAILING** - 77 violations (expected)
- **Missing required**: 2 files (`.gitignore`, `crates`)
- **Rejected**: 75 files (everything not in minimal set)
- **Exceptions**: 6 files (Cargo.toml, README.md, etc.)

## 🔄 **Next Steps**

1. **Execute migration script** to move files to proper locations
2. **Test minimal root contract** to verify compliance
3. **Set up CI enforcement** for pre-merge validation
4. **Document new structure** for team adoption
5. **Plan subtree break-offs** for individual crates

## 🛡️ **Enforcement Strategy**

### **CI/CD Integration**
```yaml
# .github/workflows/contract-validation.yml
- name: Validate minimal root
  run: |
    cd standalone-auditor
    cargo run -- HEAD ../contracts/object-names@root-minimal.json
```

### **Protected References**
- Maintain `refs/hooksmith/scopes/project-root` pointing to last compliant tree
- Compare new PRs against this reference for drift detection

### **Developer Experience**
- Local pre-commit hooks (warn, don't block)
- Clear error messages with actionable guidance
- Migration script for bulk operations

This migration will transform Hooksmith into a clean, break-off ready repository with a minimal root that serves as a stable foundation for future development and repository splits.
