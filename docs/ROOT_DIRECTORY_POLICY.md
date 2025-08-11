# Root Directory Policy

## 🎯 **Objective**
Maintain a clean, organized root directory with only essential files and directories that must be at the root level.

## 📋 **Root Directory Structure**

### **✅ Allowed at Root Level**

#### **Essential Configuration Files**
- `Cargo.toml` - Main workspace manifest
- `Cargo.lock` - Dependency lock file
- `rust-toolchain.toml` - Rust toolchain specification
- `rustfmt.toml` - Rust formatting configuration
- `clippy.toml` - Clippy linting configuration
- `deny.toml` - Dependency deny rules
- `build.rs` - Build script
- `.gitignore` - Git ignore patterns
- `.gitattributes` - Git attributes
- `.gitmodules` - Git submodules
- `CODEOWNERS` - Code ownership rules

#### **Docker/Infrastructure Files**
- `Dockerfile` - Docker container definition
- `docker-compose.yml` - Docker compose configuration
- `docker-bake.hcl` - Docker build configuration
- `.dockerignore` - Docker ignore patterns

#### **Project Configuration**
- `README.md` - Main project documentation
- `.worktree-config.json` - Worktree configuration
- `.worktree-config.jsonc` - Worktree configuration (JSONC)
- `.workbloom` - Workbloom configuration

#### **Essential Directories**
- `.cargo/` - Cargo configuration
- `.github/` - GitHub configuration and workflows
- `.git/` - Git repository data
- `.hooksmith/` - Hooksmith internal files
- `.trunk/` - Trunk configuration
- `.wb/` - Workbloom configuration
- `.contract_cache/` - Contract cache (consider moving to .hooksmith/)

#### **Core Project Directories**
- `crates/` - Rust crates and components
- `src/` - Main source code
- `contracts/` - Contract definitions
- `docs/` - Documentation
- `tests/` - Test files
- `examples/` - Example code
- `tools/` - Development tools
- `scripts/` - Scripts and utilities
- `schemas/` - Schema definitions
- `config/` - Configuration files
- `hooks/` - Git hooks
- `wit/` - WIT definitions
- `gen/` - Generated files
- `generated-sources/` - Generated sources
- `target/` - Build artifacts
- `worktree-lifecycle/` - Worktree lifecycle (consider integration)

## 🚫 **NOT Allowed at Root Level**

### **Summary Files**
- All `*_SUMMARY.md` files → `docs/summaries/`
- Progress reports → `docs/summaries/`
- Implementation summaries → `docs/summaries/`

### **Test Files**
- Test files → `tests/`
- Test data → `tests/`
- Test configurations → `tests/`

### **Configuration Files**
- Language definitions → `config/`
- Hooksmith configurations → `config/`
- Validation results → `config/`

### **Documentation Files**
- Additional README files → `docs/`
- Architecture documents → `docs/`
- Implementation guides → `docs/`

### **Tool Files**
- Utility scripts → `tools/` or `scripts/`
- Mapping files → `tools/`
- Analysis scripts → `scripts/`

### **Schema Files**
- JSON schemas → `schemas/`
- Contract definitions → `schemas/` or `contracts/`

## 🔄 **Migration Guidelines**

### **When Adding New Files**

1. **Ask**: "Does this file absolutely need to be at root level?"
2. **Categorize**: Determine the file's purpose and category
3. **Place**: Put in the appropriate subdirectory
4. **Document**: Update this policy if new categories are needed

### **File Categories**

| Category | Location | Examples |
|----------|----------|----------|
| Summaries | `docs/summaries/` | `*_SUMMARY.md`, progress reports |
| Tests | `tests/` | `test_*.rs`, `test-*.txt` |
| Configs | `config/` | `*.yml`, `*.json`, validation results |
| Docs | `docs/` | README variants, guides, architecture |
| Tools | `tools/` | Utilities, mappings, analysis tools |
| Scripts | `scripts/` | Automation scripts, analysis scripts |
| Schemas | `schemas/` | JSON schemas, contract definitions |
| Hooks | `hooks/` | Git hooks, validation scripts |

## 🛠️ **Enforcement**

### **Pre-commit Checks**
- Add linting rules to prevent summary files at root
- Add checks for test files in wrong locations
- Validate configuration file placement

### **Code Review**
- Reviewers should check file placement
- Reject PRs that add files to root unnecessarily
- Suggest appropriate subdirectories

### **Automated Tools**
- Consider adding a script to validate root structure
- Add CI checks for file organization
- Create templates for new file categories

## 📊 **Monitoring**

### **Regular Audits**
- Monthly review of root directory
- Check for new files that should be moved
- Update this policy as needed

### **Metrics**
- Track number of files at root level
- Monitor directory organization
- Report on policy compliance

## 🎯 **Goals**

1. **Keep root directory under 40 files/directories**
2. **Maintain clear separation of concerns**
3. **Improve developer experience**
4. **Follow standard project organization patterns**
5. **Make the project easier to navigate and understand**

## ✅ **Success Criteria**

- Root directory remains clean and organized
- New files are placed in appropriate subdirectories
- Developers can easily find what they're looking for
- Project structure follows established patterns
- Documentation and configuration are well-organized

---

**Last Updated**: 2025-01-XX
**Next Review**: Monthly
**Policy Owner**: Development Team
