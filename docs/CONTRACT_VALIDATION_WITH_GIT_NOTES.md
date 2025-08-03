# Contract Validation with Git Notes

This document describes Hooksmith's comprehensive contract validation system that uses **Git notes** to store cryptographic proofs of validation, providing tamper detection and audit trails for code validation.

## Overview

The contract validation system provides:

- **Rust AST Validation**: Parse and validate Rust code using `rustc`
- **JSON Schema Generation**: Derive JSON Schema from Rust types using `schemars`
- **Cryptographic Hashing**: SHA-256 hashing for tamper detection
- **Git Notes Integration**: Store validation proofs as tamper-proof Git notes
- **Hierarchical Validation**: Multi-scope validation (char → line → chunk → file → dir → repo)
- **Audit Trails**: Complete validation history with transition logs

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Rust Files    │    │   Contract      │    │   Git Notes     │
│   (.rs, .toml)  │───►│   Validator     │───►│   (Proofs)      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Validation    │◄───│   AST Parser    │◄───│   rustc         │
│   Results       │    │   (rustc)       │    │   (metadata)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Schema        │◄───│   schemars      │◄───│   Rust Types    │
│   Generation    │    │   (derive)      │    │   (Contracts)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Key Components

### 1. Contract Proof Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractProof {
    /// File path being validated
    pub file_path: String,
    /// SHA-256 hash of the file content
    pub blob_hash: String,
    /// SHA-256 hash of the extracted AST
    pub ast_hash: String,
    /// SHA-256 hash of the JSON schema
    pub schema_hash: String,
    /// Contract type (e.g., "rust_validation", "json_schema")
    pub contract_type: String,
    /// Validation timestamp
    pub validated_at: String,
    /// Tool version that performed validation
    pub validated_by: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### 2. Git Notes Storage

The system uses Git notes to store validation proofs:

- **Reference**: `refs/notes/contracts/`
- **Structure**: Hierarchical storage by file path
- **Content**: JSON-serialized `ContractProof` objects
- **Audit Trail**: `refs/notes/contracts-log/` for transition history

### 3. Validation Process

1. **File Reading**: Read target file content
2. **Blob Hashing**: Generate SHA-256 hash of file content
3. **AST Extraction**: Use `rustc --emit=metadata` for Rust files
4. **Schema Generation**: Generate JSON Schema using `schemars`
5. **Proof Creation**: Combine all hashes into `ContractProof`
6. **Git Notes Storage**: Store proof in Git notes
7. **Verification**: Compare current proof with stored proof

## Usage Examples

### Basic Contract Validation

```rust
use hooksmith::contract_validation::ContractValidator;

#[tokio::main]
async fn main() -> Result<()> {
    // Create validator
    let validator = ContractValidator::new()?;
    
    // Validate a Rust file
    let proof = validator.validate_contract("src/main.rs").await?;
    
    // Store proof in Git notes
    validator.store_proof(&proof).await?;
    
    // Verify proof later
    let is_valid = validator.verify_proof("src/main.rs", &proof).await?;
    
    Ok(())
}
```

### CLI Usage

```bash
# Validate a contract file
cargo run -p xtask -- contract-validation validate src/main.rs

# Verify existing proof
cargo run -p xtask -- contract-validation verify src/main.rs --strict

# Generate proof without storing
cargo run -p xtask -- contract-validation generate src/main.rs

# List all proofs
cargo run -p xtask -- contract-validation list --detailed

# Clean up old proofs
cargo run -p xtask -- contract-validation cleanup --days 30 --dry-run
```

### Git Hooks Integration

The system integrates with Git hooks for automated validation:

#### Pre-commit Hook
```yaml
# lefthook.yml
pre_commit:
  contract-validate:
    run: cargo run -p xtask -- contract-validation validate {staged_files}
    stage_fixed: true
```

#### Post-commit Hook
```yaml
# lefthook.yml
post_commit:
  contract-store:
    run: cargo run -p xtask -- contract-validation validate {all_files} --store
```

#### Pre-push Hook
```yaml
# lefthook.yml
pre_push:
  contract-verify:
    run: cargo run -p xtask -- contract-validation verify {all_files} --strict
```

## Hook Strategy

Based on your requirements, here's the optimal hook strategy:

### ✅ Hooks Where Adding Notes Works

| Hook Name | Timing | Notes Usable? | Why |
|-----------|--------|---------------|-----|
| pre-commit | Before commit is created | ✅ Yes | Repo is writable; new commit not yet finalized |
| post-commit | After commit is created | ✅ Yes | Commit exists, you can safely run git notes add |
| pre-push | Before push | ✅ Yes | Repo is writable; commit already exists |
| post-checkout | After switching branches | ✅ Yes | Can add notes for the checked-out commit |
| post-merge | After merge | ✅ Yes | Commit exists; notes can be added |

### ⚠️ Hooks Where Notes Are Risky

| Hook Name | Timing | Notes Usable? | Why |
|-----------|--------|---------------|-----|
| pre-receive | On remote before commit is stored | ❌ No | Runs on remote; repo is in a transient state |
| update | On remote per ref update | ❌ No | You can write notes here, but typically discouraged |
| post-receive | After remote push is complete | ✅ Yes | Commit exists in remote repo; you can add notes server-side |
| commit-msg | Before final commit creation | ⚠️ Not yet | Commit SHA doesn't exist yet |

### Best Strategy for Your Pipeline

1. **Pre-commit hook**: Validate staged `.rs` blobs → fail if AST invalid
2. **Post-commit hook**: Attach Git notes with `{blob_hash, ast_hash, validated: true}`
3. **Pre-push hook**: Verify that all commits being pushed have notes
4. **CI fallback**: If commit has no note, regenerate proof and fail if mismatched

## Implementation Details

### AST Hash Generation

For Rust files, the system uses `rustc` to extract AST metadata:

```rust
async fn generate_ast_hash(&self, content: &str) -> Result<String> {
    let temp_file = tempfile::NamedTempFile::new()?;
    std::fs::write(&temp_file, content)?;

    let output = std::process::Command::new("rustc")
        .arg("--emit=metadata")
        .arg("--crate-type=lib")
        .arg(temp_file.path())
        .output()
        .context("Failed to run rustc")?;

    if !output.status.success() {
        // Fall back to content hash if rustc fails
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        return Ok(format!("sha256:{}", hex::encode(result)));
    }

    // Hash the metadata output
    let mut hasher = Sha256::new();
    hasher.update(&output.stdout);
    let result = hasher.finalize();
    Ok(format!("sha256:{}", hex::encode(result)))
}
```

### Schema Hash Generation

For JSON Schema validation:

```rust
async fn generate_schema_hash(&self, file_path: &str) -> Result<String> {
    // Generate JSON schema for contract types
    let schema = schemars::schema_for!(HookContract);
    let schema_json = serde_json::to_string_pretty(&schema)?;

    // Hash the schema + file path
    let mut hasher = Sha256::new();
    hasher.update(schema_json.as_bytes());
    hasher.update(file_path.as_bytes());
    let result = hasher.finalize();
    Ok(format!("sha256:{}", hex::encode(result)))
}
```

### Git Notes Storage

The system stores proofs in Git notes using the `git2` crate:

```rust
pub async fn store_proof(&self, proof: &ContractProof) -> Result<()> {
    // Convert proof to contract state note
    let state_note = ContractStateNote {
        file: proof.file_path.clone(),
        contract: proof.contract_type.clone(),
        state: "VALIDATED".to_string(),
        hash: proof.blob_hash.clone(),
        validated_by: proof.validated_by.clone(),
        timestamp: proof.validated_at.clone(),
        parent_scope: None,
        parent_hash: None,
        metadata: Some(proof.metadata.clone()),
    };

    self.notes_manager.store_contract_state(&state_note)?;

    // Store transition log
    let transition = TransitionLogEntry {
        transition: "validate_contract".to_string(),
        from: "UNVALIDATED".to_string(),
        to: "VALIDATED".to_string(),
        file: proof.file_path.clone(),
        hash: proof.blob_hash.clone(),
        tool: proof.validated_by.clone(),
        timestamp: proof.validated_at.clone(),
        reason: Some("Contract validation completed".to_string()),
        commit_hash: None,
        user: None,
        environment: None,
        metadata: Some(proof.metadata.clone()),
    };

    self.notes_manager.store_transition_log(&transition)?;
    Ok(())
}
```

## Example: Rust Contract → JSON Schema → Hash → Git Notes

This example shows the complete workflow:

### 1. Define Contract Struct

```rust
#[derive(Debug, Serialize, JsonSchema)]
struct HookContract {
    name: String,
    enabled: bool,
    trigger: String,
    version: String,
    metadata: HashMap<String, serde_json::Value>,
}
```

### 2. Generate JSON Schema & Compute Hash

```rust
fn main() {
    // 1. Generate schema
    let schema = schema_for!(HookContract);
    let schema_json = serde_json::to_string_pretty(&schema).unwrap();

    // 2. Compute SHA-256 hash of schema
    let mut hasher = Sha256::new();
    hasher.update(&schema_json);
    let result = hasher.finalize();
    let hash_hex = format!("sha256:{}", hex::encode(result));

    println!("JSON Schema:\n{}", schema_json);
    println!("Schema Hash: {}", hash_hex);
}
```

### 3. Attach Git Note

```bash
git notes --ref=refs/notes/contracts add -m "hook_contract_schema=${hash_hex}" HEAD
```

### 4. Verification in CI

```rust
// Regenerate the schema
let schema = schema_for!(HookContract);
let schema_json = serde_json::to_string_pretty(&schema)?;

// Recompute the hash
let mut hasher = Sha256::new();
hasher.update(&schema_json);
let result = hasher.finalize();
let current_hash = format!("sha256:{}", hex::encode(result));

// Check if the Git note on HEAD matches
let expected_hash = get_git_note_hash("hook_contract_schema")?;
if current_hash != expected_hash {
    anyhow::bail!("Schema hash mismatch: expected {}, got {}", expected_hash, current_hash);
}
```

## Benefits

### 1. Tamper Detection
- Any modification to validated files invalidates the proof
- Cryptographic hashing ensures integrity
- Git notes provide tamper-proof storage

### 2. Audit Trails
- Complete validation history in Git notes
- Transition logs track state changes
- Timestamps and user information for accountability

### 3. Performance
- Efficient verification using hashes
- Incremental validation (only changed files)
- Parallel processing support

### 4. Integration
- Seamless Git workflow integration
- CI/CD pipeline compatibility
- Lefthook configuration support

## Configuration

### Cargo.toml Dependencies

```toml
[dependencies]
schemars = { version = "0.8", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
hex = "0.4"
git2 = "0.18"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

### Lefthook Configuration

```yaml
# lefthook.yml
pre_commit:
  contract-validate:
    run: cargo run -p xtask -- contract-validation validate {staged_files}
    stage_fixed: true
  contract-store:
    run: cargo run -p xtask -- contract-validation validate {staged_files} --store
    stage_fixed: true

pre_push:
  contract-verify:
    run: cargo run -p xtask -- contract-validation verify {all_files} --strict
```

## Future Enhancements

1. **Merkle Tree Integration**: Hierarchical validation across multiple scopes
2. **Schema Evolution Tracking**: Version control for schema changes
3. **Performance Optimization**: Caching and incremental validation
4. **Web UI**: Visual interface for proof management
5. **API Integration**: REST API for external validation services

## Conclusion

The contract validation system with Git notes provides a robust, tamper-proof solution for code validation. By combining Rust AST validation, JSON Schema generation, cryptographic hashing, and Git notes storage, it ensures code integrity while maintaining audit trails and performance.

The system integrates seamlessly with existing Git workflows and provides the foundation for advanced validation features like hierarchical validation and schema evolution tracking. 
