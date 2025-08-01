# Repository Structure

This document shows the complete file structure of the repository.

## 📁 File Structure

```
.
├── .gitignore
├── .trunk
│   ├── configs
│   └── trunk.yaml
├── build.sh
├── Cargo.toml
├── components
│   ├── cli-core
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   └── worktree-runner
│       ├── Cargo.toml
│       ├── src
│       │   └── lib.rs
│       └── wit
│           └── worktree-runner.wit
├── docs
│   ├── CLI_HELP.md
│   ├── DEVELOPMENT.md
│   ├── README.md
│   ├── STRUCTURE.md
│   └── TEST_SUMMARY.md
├── hooks
│   └── README.md
├── README.md
├── scripts
│   ├── generate-docs.sh
│   └── generate-structure.sh
├── src
│   ├── commands
│   │   └── mod.rs
│   ├── lib.rs
│   ├── main.rs
│   └── modules
│       └── mod.rs
├── STRUCTURE.md
└── tests
    ├── hooks_test.rs
    └── integration.rs

16 directories, 25 files
```

## 📊 File Count Summary

- **Total Files**:       30
- **Rust Files**:        8 (.rs)
- **Configuration Files**:        7 (.toml, .yaml, .rc)
- **Documentation**:        8 (.md)
- **Scripts**:        3 (.sh)

---

*Generated on Fri Aug  1 17:46:36 EDT 2025 using `git ls-tree -r HEAD`*
