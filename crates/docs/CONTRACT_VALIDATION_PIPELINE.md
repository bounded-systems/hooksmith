# Contract Validation Pipeline

This document describes the comprehensive contract validation pipeline that enforces repository structure and content rules across multiple scopes with intelligent caching and reporting.

## 🏗️ Architecture Overview

The contract validation pipeline implements a production-grade validation system with the following key features:

- **Tree-aware caching** for fast validation of stable trees
- **Scope detection** to validate only changed areas
- **Matrix execution** for parallel validation across scopes
- **SARIF reporting** for GitHub integration
- **Merge-aware validation** for accurate PR checks

## 🔄 Pipeline Flow

### 1. Triggers

The pipeline runs on multiple triggers:

```yaml
# Local development
pre-commit: Fast checks on staged paths
pre-push: Validate current branch against origin/main

# CI/CD
pull_request: Validate synthetic merge (HEAD ⊕ origin/main)
push: Validate new main tip
scheduled: Nightly drift detection
```

### 2. Scope Discovery

The pipeline automatically detects what needs validation:

```rust
// Root tree scope (always validated)
root_tree = git rev-parse <merge>^{tree}

// Changed subtrees (from diff analysis)
changed_dirs = git diff --name-only base head | extract_top_level_dirs()

// Contract mapping
contracts = map_path_to_contracts(changed_dirs)
```

### 3. Cache Strategy

Tree-aware caching with SHA-stable keys:

```rust
cache_key = hash({
    tree_sha,          // Scope tree SHA
    contract_id,       // e.g., object-names@1.0.0
    contract_rev,      // Schema hash or version
    fix_hash,          // Tool+config fingerprint
})
```

**Cache Behavior:**
- **Hit**: Return prior results (SARIF, violations, fix plans)
- **Miss**: Run validation, store with TTL + tree invalidation
- **TTL**: 24 hours for unchanged trees
- **Invalidation**: When tree SHA changes or contract updates

### 4. Execution Graph

Validation runs in order of increasing cost:

```rust
// 1. Structure (fast & cheap)
object_names_contract.validate(root_tree)
tree_stability_analyzer.analyze(changed_trees)

// 2. Semantics (medium cost)
schema_validator.validate(json_files)
license_checker.validate(headers)

// 3. Heavy analysis (if needed)
pack_analyzer.analyze_deltas()
sbom_generator.generate()
```

## 🛠️ Implementation Components

### Core Validator

```bash
# Main pipeline entry point
cargo run --bin contract_validation_pipeline origin/main HEAD

# Scope-specific validation
cargo run --bin contract_validation_pipeline origin/main HEAD --scope root
```

**Features:**
- Automatic scope detection
- Tree-aware caching
- SARIF output generation
- Fix plan generation

### Contract Variants

```bash
# Switch between contract variants
cargo run --bin switch_object_names_contract rust    # Rust workspace
cargo run --bin switch_object_names_contract strict  # Strict compliance
```

**Available Contracts:**
- `object-names@v1-strict.json`: Original strict rules
- `object-names@v1-rust-workspace-fixed.json`: Rust-friendly rules

### GitHub Actions Integration

The pipeline integrates with GitHub Actions through:

1. **Matrix Strategy**: Parallel validation across scopes
2. **SARIF Upload**: Code scanning integration
3. **PR Comments**: Automated feedback
4. **Artifact Sharing**: Cache persistence between jobs

## 📊 Validation Results

### Success Case
```
🚀 Contract Validation Pipeline
Base: origin/main
Head: HEAD

🔍 Detecting validation scopes...
📋 Found 3 scopes to validate:
  - root (contracts: ["object-names@v1"])
  - subtree:crates (contracts: ["crate-structure@v1"])
  - subtree:docs (contracts: ["documentation@v1"])

🔍 Validating root...
  ✅ Cache hit (5ms)
  ✅ Validation passed

🔍 Validating subtree:crates...
  ⚡ Fresh validation (150ms)
  ✅ Validation passed

🔍 Validating subtree:docs...
  ✅ Cache hit (3ms)
  ✅ Validation passed

📊 Pipeline Summary:
  - Total scopes: 3
  - Cache hits: 2
  - Total execution time: 158ms
  - Failed validations: 0

🎉 All contract validations passed!
```

### Failure Case
```
❌ Contract validation failed for 1 scopes:
  - root: 7 violations

📄 SARIF report saved to: contract-validation-results.sarif
```

## 🔧 Configuration

### Contract Definitions

Contracts are defined in JSON format:

```json
{
  "name": "object-names",
  "version": "1.0.0",
  "spec": {
    "git": {
      "tree": {
        "objects": {
          "names": {
            "required": [".gitignore"],
            "allowed": [".gitignore", "*.rs", "Cargo.toml"],
            "rejected": [],
            "ignored": [".DS_Store"]
          }
        }
      }
    }
  }
}
```

### Cache Configuration

```toml
# .contract_cache/config.toml
[cache]
ttl_hours = 24
max_size_mb = 100
invalidation_strategy = "tree_sha"

[validation]
parallel_jobs = 4
timeout_seconds = 300
```

## 🚀 Usage Examples

### Local Development

```bash
# Quick validation of current state
cd scripts
cargo run --bin validate_object_names_contract

# Full pipeline validation
cargo run --bin contract_validation_pipeline origin/main HEAD

# Migration analysis
cargo run --bin analyze_object_names_migration
```

### CI/CD Integration

```yaml
# .github/workflows/contract-validation-pipeline.yml
name: Contract Validation Pipeline
on: [pull_request, push]

jobs:
  detect-scopes:
    # Scope detection job
  validate-contracts:
    # Matrix validation job
  generate-report:
    # Reporting job
```

### Server-side Enforcement

```bash
#!/usr/bin/env bash
# .git/hooks/pre-receive
read old new ref
[[ "$ref" == "refs/heads/main" ]] || exit 0

cd scripts
cargo run --bin contract_validation_pipeline "$old" "$new" || {
  echo "Contract validation failed for new main tip"
  exit 1
}
```

## 📈 Performance Optimization

### Caching Strategy

1. **Tree SHA Keys**: Cache by actual tree content, not commit SHA
2. **TTL Management**: Automatic expiration for stale results
3. **Parallel Execution**: Matrix strategy for concurrent validation
4. **Early Termination**: Fail fast on structure violations

### Cache Hit Rates

Typical cache hit rates:
- **Stable trees**: 90%+ hit rate
- **Active development**: 60-80% hit rate
- **Major refactoring**: 20-40% hit rate

### Performance Metrics

```bash
# Cache statistics
ls -la .contract_cache/ | wc -l  # Cache entries
du -sh .contract_cache/          # Cache size

# Validation timing
time cargo run --bin contract_validation_pipeline origin/main HEAD
```

## 🔍 Monitoring and Debugging

### Log Analysis

```bash
# Enable debug logging
RUST_LOG=debug cargo run --bin contract_validation_pipeline origin/main HEAD

# Cache debugging
ls -la .contract_cache/
cat .contract_cache/<cache_key>.json
```

### SARIF Reports

SARIF reports provide:
- **Violation details**: Exact location and description
- **Fix suggestions**: Machine-readable remediation steps
- **GitHub integration**: Annotations in PR interface

### Performance Monitoring

```bash
# Cache hit rate analysis
grep "Cache hit" validation.log | wc -l
grep "Fresh validation" validation.log | wc -l

# Execution time analysis
grep "execution_time_ms" validation.log | jq '.execution_time_ms' | jq -s 'add/length'
```

## 🔮 Future Enhancements

### Planned Features

1. **Contract Packs**: Predefined rule sets for common project types
2. **Visual Reports**: Tree structure visualization
3. **Integration APIs**: REST endpoints for external validation
4. **Custom Rules**: Repository-specific rule overrides
5. **Auto-remediation**: Automatic fixes for safe violations

### Integration Opportunities

- **IDE Plugins**: Real-time validation in editors
- **Git Hooks**: Local pre-commit validation
- **CI/CD Platforms**: Generic pipeline integration
- **Monitoring**: Contract compliance dashboards

## 📚 Related Documentation

- [Object Names Contract System](./OBJECT_NAMES_CONTRACT_SYSTEM.md)
- [Contract Validation Summary](./OBJECT_NAMES_CONTRACT_SUMMARY.md)
- [GitHub Actions Workflows](../.github/workflows/)
- [Contract Definitions](../contracts/)

## 🤝 Contributing

To extend the contract validation pipeline:

1. **Add new contracts**: Create JSON definitions in `contracts/`
2. **Implement validators**: Add validation logic for new contract types
3. **Update scope detection**: Extend path-to-contract mapping
4. **Enhance caching**: Optimize cache keys and invalidation
5. **Improve reporting**: Add new output formats or integrations

The pipeline is designed to be extensible and maintainable, with clear separation of concerns and comprehensive testing.
