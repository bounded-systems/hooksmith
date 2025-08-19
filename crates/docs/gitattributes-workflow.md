# GitAttributes Workflow with Hyperpolyglot

This document describes the automated workflow for generating and maintaining `.gitattributes` files using [hyperpolyglot](https://github.com/monkslc/hyperpolyglot) for accurate language detection.

## Overview

The workflow automatically detects file types in your repository using hyperpolyglot and generates appropriate `.gitattributes` entries for GitHub Linguist integration. This ensures accurate language statistics and proper syntax highlighting on GitHub.

## Components

### 1. Rust Script (`scripts/generate-gitattributes.rs`)

The core language detection and `.gitattributes` generation script.

**Features:**
- Uses `git ls-files` to get all tracked files
- Runs hyperpolyglot on each file for language detection
- Generates comprehensive `.gitattributes` with:
  - `linguist-language` attributes for each detected language
  - Common exclusions for generated, documentation, and vendored files
  - Detectability settings for configuration files

**Usage:**
```bash
# Compile and run
rustc scripts/generate-gitattributes.rs -o scripts/generate-gitattributes
./scripts/generate-gitattributes [output_file]
```

### 2. Shell Wrapper (`scripts/generate-gitattributes.sh`)

User-friendly wrapper script with various options.

**Usage:**
```bash
# Basic usage
./scripts/generate-gitattributes.sh

# Custom output file
./scripts/generate-gitattributes.sh -o custom.gitattributes

# Force overwrite with backup
./scripts/generate-gitattributes.sh -f -b

# Dry run to see what would be generated
./scripts/generate-gitattributes.sh -d
```

**Options:**
- `-o, --output FILE` - Output file (default: `.gitattributes`)
- `-f, --force` - Overwrite existing file without confirmation
- `-b, --backup` - Create backup of existing `.gitattributes`
- `-d, --dry-run` - Show what would be generated without writing
- `-h, --help` - Show help message

### 3. CI Integration (`scripts/ci-gitattributes.sh`)

CI/CD focused script that integrates with automated workflows.

**Usage:**
```bash
# Check if update is needed
./scripts/ci-gitattributes.sh -c

# Force update
./scripts/ci-gitattributes.sh -f

# Verbose output
./scripts/ci-gitattributes.sh -v
```

**Features:**
- Checksum-based change detection
- CI environment awareness (no colors, no prompts)
- Automatic backup creation
- Integration with GitHub Actions

### 4. GitHub Actions Workflow (`.github/workflows/gitattributes.yml`)

Automated workflow that runs on repository changes.

**Triggers:**
- Push to `main` or `develop` branches (excluding `.gitattributes` changes)
- Pull requests to `main` or `develop` branches
- Manual workflow dispatch with force option

**Features:**
- Automatic language detection using hyperpolyglot
- Conditional updates based on file type changes
- Automatic commit and push of changes
- Pull request creation for manual review (when triggered manually)

## Installation

### Prerequisites

1. **Rust toolchain** (for compiling the script)
2. **hyperpolyglot** (automatically installed by scripts)
3. **Git repository** (for file tracking)

### Setup

1. **Clone or copy the scripts:**
   ```bash
   # Ensure scripts are executable
   chmod +x scripts/generate-gitattributes.sh
   chmod +x scripts/ci-gitattributes.sh
   ```

2. **Install hyperpolyglot:**
   ```bash
   cargo install hyperpolyglot
   ```

3. **Test the setup:**
   ```bash
   ./scripts/generate-gitattributes.sh -d
   ```

## Usage Examples

### Local Development

**Generate `.gitattributes` for the first time:**
```bash
./scripts/generate-gitattributes.sh
```

**Preview what would be generated:**
```bash
./scripts/generate-gitattributes.sh -d
```

**Force update with backup:**
```bash
./scripts/generate-gitattributes.sh -f -b
```

### CI/CD Integration

**Check if update is needed:**
```bash
./scripts/ci-gitattributes.sh -c
```

**Force update in CI environment:**
```bash
export CI=true
./scripts/ci-gitattributes.sh -f
```

### GitHub Actions

The workflow automatically runs when:
- New files are added to the repository
- File extensions change
- Manual trigger via GitHub Actions UI

## Generated .gitattributes Structure

The generated `.gitattributes` file includes:

### 1. Language Detection Results
```gitattributes
# Rust files
src/main.rs linguist-language=Rust
src/lib.rs linguist-language=Rust

# Shell files
scripts/setup.sh linguist-language=Shell
```

### 2. Common Exclusions
```gitattributes
# Generated files - mark as linguist-generated
target/ linguist-generated=true
dist/ linguist-generated=true

# Documentation files - exclude from language stats
*.md linguist-documentation=true
*.txt linguist-documentation=true

# Vendored files - exclude from language stats
vendor/ linguist-vendored=true
third_party/ linguist-vendored=true
```

### 3. Detectability Settings
```gitattributes
# Make certain file types detectable in language stats
*.yaml linguist-detectable=true
*.yml linguist-detectable=true
*.json linguist-detectable=true
*.toml linguist-detectable=true
```

## Integration with Your Pipeline

### 1. Pre-commit Hook

Add to your `.git/hooks/pre-commit`:
```bash
#!/bin/bash
# Check if .gitattributes needs updating
if ./scripts/ci-gitattributes.sh -c; then
    echo "✅ .gitattributes is up to date"
else
    echo "⚠️  .gitattributes needs updating"
    echo "Run: ./scripts/generate-gitattributes.sh"
    exit 1
fi
```

### 2. Lefthook Integration

Add to your `lefthook.yml`:
```yaml
pre-commit:
  commands:
    gitattributes:
      run: ./scripts/ci-gitattributes.sh -c || (echo "Update .gitattributes needed" && exit 1)
```

### 3. CI Pipeline Integration

Add to your CI pipeline:
```yaml
- name: Update .gitattributes
  run: |
    ./scripts/ci-gitattributes.sh -f
    git add .gitattributes
    git commit -m "Update .gitattributes" || echo "No changes"
```

## Language Detection Accuracy

The workflow uses hyperpolyglot, which is based on GitHub's Linguist library and provides:

- **High accuracy** for common programming languages
- **Fast detection** using multiple strategies
- **Support for 400+ languages**
- **Heuristic-based classification** for ambiguous files

### Detection Strategies

1. **File extension matching**
2. **Filename patterns**
3. **Content analysis**
4. **Shebang detection**
5. **Heuristic classification**

## Troubleshooting

### Common Issues

**1. hyperpolyglot not found:**
```bash
cargo install hyperpolyglot
```

**2. Rust compilation errors:**
```bash
rustc --version  # Ensure Rust is installed
rustc scripts/generate-gitattributes.rs -o scripts/generate-gitattributes
```

**3. Git repository not found:**
```bash
git status  # Ensure you're in a git repository
```

**4. Permission denied:**
```bash
chmod +x scripts/*.sh
```

### Debug Mode

Enable verbose output for debugging:
```bash
./scripts/ci-gitattributes.sh -v
```

### Manual Language Override

If hyperpolyglot misclassifies a file, you can manually override it in `.gitattributes`:
```gitattributes
# Manual override
specific-file.ext linguist-language=CorrectLanguage
```

## Performance

- **Fast detection**: hyperpolyglot is significantly faster than Linguist
- **Incremental updates**: Only processes changed files
- **Caching**: Checksum-based change detection
- **Parallel processing**: Multi-threaded language detection

## Contributing

To contribute to this workflow:

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Test with various file types**
5. **Submit a pull request**

## License

This workflow is part of the Hooksmith project and follows the same licensing terms.

## References

- [hyperpolyglot](https://github.com/monkslc/hyperpolyglot) - Fast programming language detector
- [GitHub Linguist](https://github.com/github/linguist) - Language detection library
- [Git Attributes](https://git-scm.com/docs/gitattributes) - Git attributes documentation
- [GitHub Linguist Attributes](https://github.com/github/linguist#overrides) - Linguist override documentation
