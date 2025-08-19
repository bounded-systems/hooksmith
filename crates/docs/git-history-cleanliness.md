# Git History Cleanliness Guide

## Overview

This guide defines the **ideal Git history characteristics** for Hooksmith's contract-based system, ensuring optimal performance, maintainability, and SHA stability.

## 🧼 Clean Git History: Ideal Characteristics

### ✅ 1. Linear & Meaningful Commits

**Goal**: Maintain a clean, linear history with meaningful commit messages.

**Best Practices**:
- Rebase before merging (`git rebase`, not merge)
- Squash WIP or noisy commits (e.g., "fix bug", "oops")
- Use conventional commits or semantic messages

**Example Commit Messages**:
```
feat: add modularization analyzer
fix: deduplicate config hashing
chore: clean up blob inspector logging
```

**Conventional Commit Types**:
- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### ✅ 2. Blob-Deduplicated

**Goal**: Minimize repository size and improve performance through efficient blob storage.

**Best Practices**:
- Avoid repeated large files (media, binaries, vendored deps)
- Track large assets with Git LFS
- Use `git gc` and packfile tuning (`--depth`, `--window`) to keep `.git/objects` lean

**Git LFS Candidates**:
- Files larger than 100KB
- Binary files (images, videos, executables)
- Generated files that change frequently

**Packfile Optimization**:
```bash
# Aggressive garbage collection
git gc --aggressive

# Optimize packfiles
git repack -a -d --depth=250 --window=250
```

### ✅ 3. Subtree-Split Ready

**Goal**: Maintain modular layout using stable crate/package boundaries.

**Best Practices**:
- Modular layout using stable crate/package boundaries
- Avoid deep cross-crate imports
- Can use `git subtree split` or `filter-repo` to extract crates cleanly

**Ideal Structure**:
```
crates/
├── core/           # Core functionality
├── tooling/        # Development tools
├── contracts/      # Contract definitions
└── utils/          # Shared utilities
```

### ✅ 4. Compact Yet Reproducible

**Goal**: Balance repository size with build reproducibility.

**Best Practices**:
- Lock files (e.g. `Cargo.lock`) tracked when reproducibility matters
- Generated files always ignored
- No cache, log, or local config files tracked

**`.gitignore` Examples**:
```gitignore
# Build artifacts
target/
dist/
build/

# Generated files
*.generated.rs
*.wit.h

# Logs and caches
*.log
.cache/
```

### ✅ 5. Bounded by Releases

**Goal**: Tag known-good states and optionally squash after major cycles.

**Best Practices**:
- Tag known-good states (`v1.0.0`, `stable/hooksmith`, etc.)
- Optionally squash after each major cycle to reduce depth
- Example: squash 100+ commits into 1 per release phase

**Release Workflow**:
```bash
# Create release branch
git checkout -b release/v1.0.0

# Tag the release
git tag -a v1.0.0 -m "Release v1.0.0"

# Squash commits for clean history
git rebase -i HEAD~100
```

### ✅ 6. Immutable After Release

**Goal**: No history rewrite after contracts or SHAs are pinned.

**Best Practices**:
- No history rewrite after contracts or SHAs are pinned
- Use `main` or `release/` branches for stable SHAs
- Prefer side branches or `dev/` for experimentation

**Branch Strategy**:
```
main          # Stable, immutable after release
├── release/  # Release branches
├── feature/  # Feature development
└── dev/      # Experimental work
```

### ✅ 7. Low Tree Rehashing

**Goal**: Use stable directory layouts to minimize tree SHA changes.

**Best Practices**:
- Use stable directory layouts
- Avoid moving folders often (tree SHA changes)
- Minimize deep nesting of changing files
- Group volatile files by crate/module

**Stable Layout Example**:
```
src/
├── core/          # Stable core modules
├── contracts/     # Contract definitions
├── tooling/       # Development tools
└── utils/         # Shared utilities
```

### ✅ 8. Sparse Commit Trees

**Goal**: Avoid committing large sets of unrelated changes.

**Best Practices**:
- Avoid committing large sets of unrelated changes
- Commit one concern per commit (enables easier blame/diff/log)
- Use atomic commits for better bisection

**Good Commit Example**:
```bash
# Single concern per commit
git add src/contracts/validation.rs
git commit -m "feat: add contract validation logic"

git add src/tooling/analyzer.rs
git commit -m "feat: add modularization analyzer"
```

## 🧠 For Hooksmith/Contract-Based Systems

### Key Traits and Benefits

| Trait | Benefit |
|-------|---------|
| 💡 Stable SHAs | Contract reproducibility |
| 🧱 Modular boundaries | Clear repair scopes |
| 🚀 Fast clone/builds | Good CI performance |
| 🔍 Traceable blame | Debuggable regressions |
| 🧼 Low object churn | Git hygiene + smaller packfiles |

### Contract-Specific Considerations

**SHA Stability**:
- Pinned contracts depend on stable commit SHAs
- Avoid history rewrites after contracts are deployed
- Use semantic versioning for contract releases

**Modular Boundaries**:
- Each crate should have a single contract responsibility
- Clear separation between core, tooling, and contract modules
- Independent testability for each module

## ✅ Ideal Commit Hygiene Summary

| Rule | Description |
|------|-------------|
| 🧹 Clean commits | Rebase + squash into meaningful commits |
| 📦 No junk | Ignore build, log, and cache artifacts |
| 🔁 Stable layout | Avoid directory reshuffling |
| ⛔ No rewrites after pin | Don't rewrite SHAs tied to contracts |
| 🧱 Modular structure | Use crates/packages cleanly |
| 🏷️ Tag releases | Mark stable points explicitly |

## 🔧 Tools and Automation

### Git History Cleanliness Analyzer

Use the built-in analyzer to assess your repository:

```bash
cargo run --bin git_history_cleanliness_analyzer
```

This tool provides:
- Commit quality analysis
- Blob health assessment
- Subtree readiness evaluation
- Tree stability measurement
- Contract safety verification

### Tree-to-Repo Extractor

Safely extract subdirectories into standalone repositories:

```bash
cargo run --bin tree_to_repo_extractor <source_path> <target_repo>
```

Example:
```bash
cargo run --bin tree_to_repo_extractor crates/tooling/dircheck hooksmith-dircheck
```

### Git History Rewriter

Analyze the safety and impact of history rewrites:

```bash
cargo run --bin git_history_rewriter
```

## 🚨 Critical Issues to Avoid

### 1. Large Blobs
- **Problem**: Large files bloat repository size
- **Solution**: Use Git LFS for files > 100KB

### 2. Poor Commit Messages
- **Problem**: Unclear history and difficult debugging
- **Solution**: Use conventional commit format

### 3. Frequent Directory Restructuring
- **Problem**: Tree SHA changes affect contract stability
- **Solution**: Plan stable directory structure early

### 4. History Rewrites After Contract Deployment
- **Problem**: Invalidates pinned contract SHAs
- **Solution**: Freeze history after contract deployment

### 5. Cross-Crate Dependencies
- **Problem**: Difficult extraction and maintenance
- **Solution**: Clear module boundaries and minimal coupling

## 📊 Monitoring and Metrics

### Key Metrics to Track

1. **Commit Quality Score**: Percentage of conventional commits
2. **Blob Deduplication Score**: Efficiency of blob storage
3. **Subtree Readiness Score**: Modularity assessment
4. **Tree Stability Score**: Directory structure stability
5. **Contract Safety Score**: SHA stability for contracts

### Health Thresholds

| Metric | Excellent | Good | Needs Improvement | Critical |
|--------|-----------|------|-------------------|----------|
| Overall Score | > 80% | 60-80% | 40-60% | < 40% |
| Commit Quality | > 70% | 50-70% | 30-50% | < 30% |
| Blob Health | > 80% | 60-80% | 40-60% | < 40% |

## 🔄 Continuous Improvement

### Regular Maintenance

1. **Weekly**: Run git history cleanliness analyzer
2. **Monthly**: Review and optimize blob storage
3. **Quarterly**: Assess subtree readiness for extraction
4. **Release**: Tag stable points and consider squashing

### Automation

Consider integrating these checks into your CI/CD pipeline:

```yaml
# .github/workflows/git-hygiene.yml
name: Git History Hygiene
on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Git History Cleanliness Analyzer
        run: cargo run --bin git_history_cleanliness_analyzer
```

## 📚 Additional Resources

- [Conventional Commits](https://www.conventionalcommits.org/)
- [Git LFS Documentation](https://git-lfs.github.com/)
- [Git Filter-Repo](https://github.com/newren/git-filter-repo)
- [Git Subtree](https://git-scm.com/book/en/v2/Git-Tools-Subtree-Merging)

## 🤝 Contributing

When contributing to this guide:

1. Follow the established patterns
2. Test changes with the analyzers
3. Update metrics and thresholds as needed
4. Document new best practices

---

*This guide ensures optimal Git history for Hooksmith's contract-based system, promoting maintainability, performance, and SHA stability.*
