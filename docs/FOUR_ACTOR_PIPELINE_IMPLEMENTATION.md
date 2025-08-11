# Four-Actor Git Pipeline Implementation

## Overview

This document describes the implementation of a four-actor Git pipeline that replaces the shell script `scripts/pre-receive-object-names.sh` with a Rust-based solution. The pipeline operates entirely within Git's object database without touching the worktree, following the design principles outlined in the original specification.

## Architecture

The pipeline consists of four distinct actors, each with a specific responsibility:

### 1. Researcher
- **Purpose**: Analyzes Git objects (trees, blobs) and extracts relevant information
- **Input**: Git object metadata (OID, type, path)
- **Output**: Analysis blob with deterministic encoding
- **Example**: Tree analyzer that extracts entry names, types, and counts

### 2. Reporter
- **Purpose**: Normalizes multiple analyses into a single standardized report
- **Input**: Multiple analysis blobs for the same object
- **Output**: Single report blob with canonical format
- **Example**: Combines tree analysis results into a normalized structure

### 3. Mandator
- **Purpose**: Creates expectations based on the contract specification
- **Input**: Contract blob + object metadata
- **Output**: Mandate blob (expectation for the object)
- **Example**: Compiles contract rules into per-object expectations

### 4. Auditor
- **Purpose**: Compares report against mandate using domain-aware diff
- **Input**: Report blob + mandate blob
- **Output**: Verdict blob + optional diff blob
- **Example**: Validates object names against contract rules

## Implementation Details

### Core Data Structures

```rust
/// Git object metadata for pipeline processing
pub struct GitObject {
    pub oid: String,
    pub kind: ObjectType,
    pub logical_path: Option<PathBuf>,
    pub parent_tree_oid: Option<String>,
    pub size: usize,
}

/// Analysis result from a researcher
pub struct Analysis {
    pub tool_fingerprint: ToolFingerprint,
    pub object_oid: String,
    pub analysis_data: serde_json::Value,
    pub cache_key: String,
}

/// Report combining multiple analyses
pub struct Report {
    pub domain: String,
    pub version: String,
    pub object_oid: String,
    pub normalized_data: serde_json::Value,
    pub analysis_oids: Vec<String>,
    pub cache_key: String,
}

/// Mandate (expectation) for an object based on contract
pub struct Mandate {
    pub contract_name: String,
    pub contract_oid: String,
    pub version: String,
    pub object_selector: String,
    pub logical_path: Option<PathBuf>,
    pub expectation: serde_json::Value,
    pub cache_key: String,
}

/// Audit verdict
pub struct Verdict {
    pub contract_name: String,
    pub version: String,
    pub pass: bool,
    pub summary_code: String,
    pub report_oid: String,
    pub mandate_oid: String,
    pub diff_oid: Option<String>,
    pub cache_key: String,
}
```

### Contract Specification

The object-names contract defines rules for Git tree structures:

```json
{
  "name": "object-names",
  "version": "1.0.0",
  "spec": {
    "git": {
      "tree": {
        "objects": {
          "names": {
            "required": [".gitignore", "projects"],
            "allowed": [".gitignore", ".gitattributes", ".meta", "docs", "generated", "projects", "src", "tests", "tools", "wit"],
            "rejected": ["README.md", "Cargo.toml", "rustfmt.toml"],
            "ignored": [".DS_Store", "Thumbs.db", ".idea", ".vscode"]
          }
        }
      }
    }
  }
}
```

### Pipeline Flow

1. **Scope Resolution**: Resolve Git reference to get commit OID and root tree
2. **Object Selection**: Use concern selector to determine which objects to validate
3. **Research Phase**: Analyze selected objects using appropriate researchers
4. **Report Phase**: Normalize analyses into standardized reports
5. **Mandate Phase**: Create expectations based on contract rules
6. **Audit Phase**: Compare reports against mandates and generate verdicts

### Cache Management

Each actor produces content-addressed blobs with deterministic cache keys:

- **Analysis Cache**: `H(analysis-tool@ver, object_oid)`
- **Report Cache**: `H(report@ver, [analysis_blob_oid...])`
- **Mandate Cache**: `H(mandate@ver, contract_oid, object_selector, logical_path)`
- **Audit Cache**: `H(audit@ver, report_oid, mandate_oid)`

### Validation Logic

The auditor implements domain-aware validation:

1. **Required Entries**: Check that all required entries are present
2. **Rejected Entries**: Ensure no rejected entries exist (skip ignored)
3. **Allowed Entries**: Verify all entries are in the allowed list (skip ignored)
4. **Root vs Sub-tree**: Apply different rules for root trees vs sub-trees

## Files Created

### Core Implementation
- `crates/core/src/git_pipeline.rs` - Main pipeline orchestrator
- `crates/core/src/object_names_validator.rs` - Four-actor implementation
- `src/bin/pre-receive-object-names.rs` - CLI binary replacement for shell script
- `src/bin/hooksmith-pipeline.rs` - CLI tool for pipeline operations

### Examples
- `examples/object_names_pipeline_demo.rs` - Demo of the four-actor pipeline
- `examples/standalone_pipeline_demo.rs` - Standalone example

### Documentation
- `docs/FOUR_ACTOR_PIPELINE_IMPLEMENTATION.md` - This document

## Usage Examples

### CLI Pipeline Tool

```bash
# Resolve scope
cargo run --bin hooksmith-pipeline scope refs/heads/main

# Select objects
cargo run --bin hooksmith-pipeline select refs/heads/main --selector root-names

# Validate with contract
cargo run --bin hooksmith-pipeline validate refs/heads/main --contract contracts/object-names@v1.json
```

### Pre-receive Hook Replacement

The Rust binary `pre-receive-object-names` replaces the shell script:

```bash
# Install as Git hook
cp target/release/pre-receive-object-names .git/hooks/pre-receive
chmod +x .git/hooks/pre-receive
```

### Standalone Demo

```bash
# Run the demo
cargo run --example standalone_pipeline_demo
```

## Benefits Over Shell Script

1. **Performance**: Content-addressed caching eliminates redundant work
2. **Reliability**: Strong typing and error handling
3. **Extensibility**: Modular design allows easy addition of new researchers/auditors
4. **Determinism**: Canonical JSON output ensures consistent results
5. **Git-native**: Operates entirely within Git's object database
6. **Parallelization**: Can process multiple objects concurrently

## Cache Strategy

The pipeline implements a sophisticated caching strategy:

- **Object-level caching**: Analysis results cached per object
- **Tool-version invalidation**: Cache keys include tool fingerprints
- **Contract-based caching**: Mandates cached per contract version
- **Content-addressed storage**: All blobs stored with SHA-256 hashes

## Future Enhancements

1. **Pattern Matching**: Implement glob pattern support for object selection
2. **Diff Engine**: Enhanced domain-aware diff with structured output
3. **Parallel Processing**: Concurrent processing of multiple objects
4. **Git Notes Integration**: Store results in Git notes for persistence
5. **WebAssembly Components**: Deploy researchers as WASM components
6. **Real-time Validation**: Continuous validation during development

## Migration from Shell Script

The shell script `scripts/pre-receive-object-names.sh` has been replaced with:

1. **Rust binary**: `src/bin/pre-receive-object-names.rs`
2. **Pipeline framework**: `crates/core/src/git_pipeline.rs`
3. **Object names validator**: `crates/core/src/object_names_validator.rs`

The new implementation provides the same validation logic but with:
- Better performance through caching
- More robust error handling
- Extensible architecture
- Git-native operation

## Conclusion

The four-actor pipeline successfully replaces the shell script with a more sophisticated, performant, and maintainable solution. The architecture provides a solid foundation for future enhancements while maintaining the core validation functionality of the original implementation.
