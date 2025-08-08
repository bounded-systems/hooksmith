# Git Blob Analysis Tools

A comprehensive suite of Rust-based tools for analyzing Git repository storage, performance, and optimization opportunities.

## 🚀 Quick Start

```bash
# Analyze Rust files by blob sizes
cargo run --bin rust_blob_analyzer

# Analyze Git delta compression opportunities
cargo run --bin git_delta_analyzer

# Generate Git hygiene recommendations
cargo run --bin git_hygiene_reporter

# Run comprehensive modularization analysis
cargo run --bin modularization_analyzer

# Analyze Git LFS optimization opportunities
cargo run --bin git_lfs_analyzer

# Optimize binary hooks with LFS
cargo run --bin lfs_hook_optimizer

# Analyze actual packfile delta compression
cargo run --bin packfile_delta_analyzer
```

## 📊 Analysis Tools

### 🔍 Rust Blob Analyzer (`rust_blob_analyzer`)
**Purpose**: Analyze Rust files by actual blob sizes rather than line counts for more accurate Git storage insights.

**Features**:
- **Blob Size Distribution**: Categorizes files as tiny (<1KB), small (1-10KB), medium (10-100KB), large (100KB-1MB), huge (>1MB)
- **Performance Impact Assessment**: Evaluates compilation speed and IDE performance impact
- **Module Analysis**: Groups files by module path and identifies optimization opportunities
- **Complexity Scoring**: Combines blob size and line count for complexity assessment

**Example Output**:
```
🦀 Rust Blob Analysis Report
=============================

📊 Blob Size Distribution:
  • large: 2 files
  • small: 247 files
  • medium: 158 files
  • tiny: 55 files

🔍 Largest Rust Files (Top 10):
  • crates/xtask/src/main.rs (332725 bytes) - High impact
    Recommendation: Consider splitting main.rs into smaller modules
```

### 🔗 Git Delta Analyzer (`git_delta_analyzer`)
**Purpose**: Analyze literal byte-level similarity for delta compression opportunities.

**Features**:
- **Delta Candidate Detection**: Identifies files with high similarity potential
- **Compression Group Formation**: Groups files by extension and size for delta analysis
- **Savings Calculation**: Estimates potential storage savings from delta compression
- **File Type Analysis**: Breaks down savings by file type (.rs, .md, .json, etc.)

**Example Output**:
```
🔍 Git Delta Compression Analysis
=================================

🔍 Top Delta Candidates (Top 10):
  • .github/workflows/hooksmith.yml (2504 bytes, 100.0% similarity)

🔗 Delta Compression Groups:
  • Base: crates/xtask/src/worktree.rs (80338 bytes)
    Delta files: 158
    Savings: 975.5 KB (33.9% compression)
```

### 🧹 Git Hygiene Reporter (`git_hygiene_reporter`)
**Purpose**: Comprehensive repository hygiene analysis with actionable recommendations.

**Features**:
- **Frequent Write Detection**: Identifies files that should be ignored (logs, cache, build artifacts)
- **Large File Analysis**: Finds candidates for Git LFS tracking
- **Git Attributes Suggestions**: Recommends `.gitattributes` rules for binary files
- **Optimization Commands**: Provides ready-to-run Git optimization commands

**Example Output**:
```
🧹 Git Hygiene Report
====================

📋 Files to Ignore (4 issues, High priority):
  • .cargo/aliases.toml - File may change frequently
    Recommendation: Review if this should be tracked

📝 .gitignore Suggestions:
  echo "target/" >> .gitignore
  echo "*.log" >> .gitignore

🔧 Optimization Commands:
  git repack -Ad --window=250 --depth=50
  git gc --prune=now
```

### 🔧 Modularization Analyzer (`modularization_analyzer`)
**Purpose**: Identify code modularization opportunities using Git's delta compression insights.

**Features**:
- **Modularization Candidates**: Identifies files that could be split into smaller modules
- **Code Pattern Analysis**: Detects common patterns (Test, Config, Utility, etc.)
- **Similarity Scoring**: Calculates code similarity for refactoring decisions
- **Delta Compression Groups**: Analyzes how similar files could benefit from delta compression

### 📦 Git LFS Analyzer (`git_lfs_analyzer`)
**Purpose**: Detect large files and suggest Git LFS tracking.

**Features**:
- **Large File Detection**: Identifies files >50MB for LFS consideration
- **Binary Hook Analysis**: Finds potential reuse of binary hooks
- **LFS Command Generation**: Provides ready-to-run LFS tracking commands
- **Gitattributes Templates**: Generates appropriate `.gitattributes` rules

### 🔄 LFS Hook Optimizer (`lfs_hook_optimizer`)
**Purpose**: Specialized optimization for binary hooks with Git LFS, focusing on reuse.

**Features**:
- **Hook Candidate Detection**: Identifies hooks >1MB or with specific binary extensions
- **Shared Binary Analysis**: Groups hooks by hash to find identical binaries
- **Optimization Planning**: Generates LFS rules and migration commands
- **Deduplication Planning**: Recommends symbolic links for shared binaries

### 📦 Packfile Delta Analyzer (`packfile_delta_analyzer`)
**Purpose**: Analyze actual packfile delta compression using git-pack and gix crates.

**Features**:
- **Real Packfile Analysis**: Uses git-pack to parse actual .pack files
- **Delta Chain Analysis**: Identifies delta chains and their compression ratios
- **Object Type Distribution**: Shows blob, tree, and commit distribution
- **Size Distribution**: Categorizes objects by size (small, medium, large)
- **Compression Statistics**: Calculates actual compression ratios and savings

**Example Output**:
```
📦 Git Packfile Delta Analysis
===============================

📊 Pack Statistics:
  • Total objects: 14770
  • Delta chains: 2
  • Average chain length: 5.0
  • Total compressed size: 14.43 MB
  • Total uncompressed size: 28.85 MB
  • Overall compression ratio: 50.0%
  • Delta savings: 14.43 MB (50.0%)

📋 Object Type Distribution:
  • commit: 1476 (10.0%)
  • tree: 1476 (10.0%)
  • blob: 11815 (80.0%)
```

## 🎯 Key Concepts

### Git Blob Sizes
- **Ideal Range**: 8-200 KB for efficient delta compression
- **Too Small**: <1 KB files often cost more to delta than store raw
- **Too Large**: >1 MB files are often skipped for delta compression
- **Deduplication**: Identical blobs are stored only once, regardless of how many files reference them

### Delta Compression
- **Window Size**: Number of objects compared for similarity (default: 10)
- **Depth**: Maximum delta chain length (default: 50)
- **Path Similarity**: Files in similar paths compress better together
- **Size Similarity**: Similar-sized files compress better than very different sizes

### Git LFS (Large File Storage)
- **Pointer Files**: Small text files (~130 bytes) that reference actual content
- **External Storage**: Actual files stored on LFS server, not in Git repo
- **Benefits**: Keeps repo lightweight, supports versioning of large binaries
- **Caveats**: No delta compression, requires LFS server setup

### Rust-Specific Considerations
- **Incremental Compilation**: Larger files take more time to hash
- **IDE Performance**: rust-analyzer performance degrades with large files
- **Cache Efficiency**: Bigger files = bigger hashes = longer dedup checks
- **Modularity**: Many small files = better parallelism and cache use

## 🛠️ Usage Examples

### Analyze Rust Project Blob Sizes
```bash
cargo run --bin rust_blob_analyzer
```

### Find Delta Compression Opportunities
```bash
cargo run --bin git_delta_analyzer
```

### Generate Hygiene Recommendations
```bash
cargo run --bin git_hygiene_reporter
```

### Analyze Modularization Opportunities
```bash
cargo run --bin modularization_analyzer
```

### Optimize Binary Hooks with LFS
```bash
cargo run --bin lfs_hook_optimizer
```

## 📈 Performance Insights

### Rust Compilation Impact
- **Files >100KB**: May impact compilation speed
- **Files >50KB**: May slow down IDE features
- **Files >20KB**: Monitor for performance impact
- **Recommendation**: Keep .rs files under 100KB for optimal performance

### Git Storage Optimization
- **Delta Compression**: Most effective for 8-200 KB files
- **Deduplication**: Identical files cost almost nothing
- **LFS**: Best for files >50MB that don't benefit from delta compression
- **Gitattributes**: Use `-delta` for binary files that don't compress well

### Repository Hygiene
- **Ignore Patterns**: target/, build/, *.log, *.cache, *.lock
- **LFS Candidates**: *.exe, *.dll, *.so, *.zip, *.tar.gz
- **Gitattributes**: *.zip -delta, *.exe -delta, *.so -delta

## 🔧 Integration with Hooksmith

These tools integrate with the Hooksmith contract system to provide:

1. **Blob Size Contracts**: Enforce maximum file sizes for optimal Git performance
2. **Delta Compression Analysis**: Identify files that could benefit from better organization
3. **LFS Integration**: Automatically suggest LFS tracking for large binaries
4. **Hygiene Enforcement**: Ensure repository follows best practices for Git storage

## 📚 Related Tools

- **Integrated Git Analyzer**: Single-pass analysis using `git ls-files`
- **Frequent Write Analyzer**: Identifies files that should be ignored
- **Git Attributes Analyzer**: Recommends optimal `.gitattributes` rules
- **Rust Git Analyzer**: Specialized analysis for Rust projects and Cargo workflows

## 🎯 Best Practices

1. **Keep Rust files under 100KB** for optimal compilation and IDE performance
2. **Use Git LFS for files >50MB** that don't benefit from delta compression
3. **Group similar files together** to improve delta compression efficiency
4. **Ignore frequently changing files** to prevent history bloat
5. **Use `.gitattributes`** to exclude binary files from delta compression
6. **Regular repacking** with `git repack -Ad --window=250 --depth=50`

## 🚀 Future Enhancements

- **SARIF Output**: Export analysis results in SARIF format for CI integration
- **Mermaid Diagrams**: Visual representation of module relationships
- **Git Notes Integration**: Store analysis results as Git notes
- **Automated Optimization**: Auto-apply recommended changes
- **CI/CD Integration**: Enforce blob size limits in CI pipelines
