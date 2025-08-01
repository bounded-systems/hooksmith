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
│   └── cli-core
│       ├── Cargo.toml
│       └── src
│           └── lib.rs
├── hooks
│   └── README.md
├── README.md
├── src
│   ├── commands
│   │   └── mod.rs
│   ├── lib.rs
│   ├── main.rs
│   └── modules
│       └── mod.rs
└── tests
    ├── hooks_test.rs
    └── integration.rs

11 directories, 14 files
```

## 📊 File Count Summary

- **Total Files**:       18
- **Rust Files**:        7 (.rs)
- **Configuration Files**:        5 (.toml, .yaml, .rc)
- **Documentation**:        2 (.md)
- **Scripts**:        1 (.sh)

---

*Generated on Fri Aug  1 17:41:20 EDT 2025 using `git ls-tree -r HEAD`*
