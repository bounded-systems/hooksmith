# Contract Validation Architecture

## Overview

The Contract Validation System is a schema-driven state machine that provides cryptographic proof of integrity for code validation across multiple hierarchical scopes. It ensures deterministic validation, tamper detection, and auditable proof chains using Git Notes and Merkle trees.

## Core Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Contract Validation System                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────┐ │
│  │   State Machine │    │   Merkle Chain  │    │   Git Notes │ │
│  │                 │    │                 │    │             │ │
│  │ • UNTRACKED     │    │ • Char → Line   │    │ • Contracts │ │
│  │ • UNVALIDATED   │    │ • Line → Chunk  │    │ • Transitions│ │
│  │ • VALIDATED     │    │ • Chunk → File  │    │ • Merkle    │ │
│  │ • LOCKED        │    │ • File → Dir    │    │             │ │
│  │                 │    │ • Dir → Repo    │    │             │ │
│  └─────────────────┘    └─────────────────┘    └─────────────┘ │
│           │                       │                    │        │
│           └───────────────────────┼────────────────────┘        │
│                                   │                             │
│  ┌─────────────────────────────────┼─────────────────────────────┐ │
│  │              CI Pipeline        │                             │ │
│  │                                 │                             │ │
│  │ • State Transition Enforcement  │                             │ │
│  │ • Merkle Chain Verification     │                             │ │
│  │ • Tamper Detection              │                             │ │
│  │ • Performance Monitoring        │                             │ │
│  └─────────────────────────────────┴─────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## State Machine

### States

| State | Description | Conditions |
|-------|-------------|------------|
| **UNTRACKED** | File has no contract or codegen attribute | Initial state for all files |
| **UNVALIDATED** | File has contract/codegen attribute but no proof | File matches `.gitattributes` pattern |
| **VALIDATED** | File has contract/codegen attribute + matching note with hash | Validation passed, Git note created |
| **LOCKED** | File is validated + no further modifications allowed | File committed, no pending changes |

### Transitions

| Event | From → To | Action | Tool |
|-------|-----------|--------|------|
| `detect_contract` | UNTRACKED → UNVALIDATED | Tag file via `.gitattributes` | Git filter |
| `validate_contract` | UNVALIDATED → VALIDATED | Run validator → create Git note with hash | `xtask-contract-validate` |
| `lock_contract` | VALIDATED → LOCKED | Post-hook marks file as locked | Git post-commit hook |
| `modify_contract` | LOCKED → UNVALIDATED | Changing file invalidates proof | Git pre-commit hook |
| `regen_codegen` | UNVALIDATED → VALIDATED | Regenerate codegen + hash-check | `xtask-gen-all` |
| `release_proof` | VALIDATED/LOCKED → VALIDATED/LOCKED | CI verifies Merkle hashes and proof chain | CI pipeline |

### State Machine Implementation

```rust
pub struct ContractStateMachine {
    transitions: HashMap<(ContractState, TransitionEvent), StateTransition>,
    current_states: HashMap<String, ContractState>,
}
```

**Key Features:**
- **Deterministic Transitions**: Only valid state transitions are allowed
- **Condition Validation**: Each transition validates specific conditions
- **Audit Trail**: All transitions are logged in Git Notes
- **Schema-Driven**: State machine defined by JSON schema

## Merkle Chain Validation

### Hierarchical Structure

```
Repository (repo)
├── Directory (dir)
│   ├── File (file)
│   │   ├── Chunk (chunk)
│   │   │   ├── Line (line)
│   │   │   │   └── Character (char)
│   │   │   └── Line (line)
│   │   └── Chunk (chunk)
│   └── File (file)
└── Directory (dir)
```

### Hash Computation Rules

#### Character Level (char)
```rust
hash = sha256(content + position + file_path)
```

#### Line Level (line)
```rust
hash = sha256(
    line_number.to_string() + 
    line_content + 
    file_path + 
    char_hashes.join("")
)
```

#### Chunk Level (chunk)
```rust
hash = sha256(
    chunk_id + 
    file_path + 
    line_hashes.join("") + 
    chunk_metadata
)
```

#### File Level (file)
```rust
hash = sha256(
    file_path + 
    file_content + 
    chunk_hashes.join("") + 
    file_metadata
)
```

#### Directory Level (dir)
```rust
hash = sha256(
    dir_path + 
    sorted_file_hashes.join("") + 
    dir_metadata
)
```

#### Repository Level (repo)
```rust
hash = sha256(
    repo_name + 
    sorted_dir_hashes.join("") + 
    repo_metadata + 
    commit_hash
)
```

### Merkle Chain Benefits

1. **Tamper Detection**: Any modification invalidates all parent hashes
2. **Deterministic Aggregation**: Same input always produces same output
3. **Efficient Verification**: Can verify specific scopes without full validation
4. **Audit Trail**: Complete validation history in Git Notes

## Git Notes Integration

### Note References

- `refs/notes/contracts` - Main contract validation notes
- `refs/notes/contracts-log` - Transition audit log
- `refs/notes/merkle-chain` - Merkle tree structure

### Note Content Structure

#### Contract State Note
```json
{
  "file": "src/modules/wasm.rs",
  "contract": "validation",
  "state": "VALIDATED",
  "scope": "file",
  "hash": "sha256:...",
  "validated_by": "xtask-contract-validate 0.2.0",
  "timestamp": "2025-01-02T15:20:00Z",
  "parent_scope": "dir",
  "parent_hash": "sha256:...",
  "child_scopes": [
    {
      "scope": "chunk",
      "hash": "sha256:...",
      "file": "src/modules/wasm.rs:1-100"
    }
  ],
  "metadata": {
    "validation_duration_ms": 150,
    "contract_type": "rust_validation",
    "tool_version": "0.2.0"
  }
}
```

#### Transition Log Note
```json
{
  "transition": "validate_contract",
  "from": "UNVALIDATED",
  "to": "VALIDATED",
  "file": "src/modules/wasm.rs",
  "hash": "sha256:...",
  "tool": "xtask-contract-validate 0.2.0",
  "timestamp": "2025-01-02T15:20:00Z",
  "commit_hash": "abc123...",
  "metadata": {
    "validation_errors": [],
    "duration_ms": 150
  }
}
```

## CI Integration

### GitHub Actions Workflow

The CI pipeline consists of multiple jobs that enforce different aspects of the validation system:

1. **validate-contracts**: Core validation and Merkle chain verification
2. **verify-git-notes**: Git Notes structure and schema validation
3. **enforce-state-transitions**: State machine compliance checking
4. **security-audit**: Tamper detection and access control verification
5. **performance-check**: Validation performance monitoring
6. **report-generation**: Comprehensive report generation and PR comments

### Pre-commit Hook Integration

```yaml
# .lefthook.yml
pre-commit:
  parallel: true
  commands:
    contract-validate:
      glob: "*.{rs,toml,md,yml,json}"
      run: ./xtask.sh contract-validate pre-commit
```

### Post-commit Hook Integration

```yaml
post-commit:
  commands:
    contract-lock:
      run: ./xtask.sh contract-validate post-commit
```

## Validation Pipeline

### 1. Change Detection
```bash
# Detect modified scopes
./xtask.sh contract-validate detect --range HEAD~1..HEAD
```

### 2. Bottom-Up Validation
```bash
# Validate from smallest to largest scope
./xtask.sh contract-validate validate --scope char,line,chunk,file,dir,repo
```

### 3. Merkle Chain Construction
```bash
# Build Merkle tree from validation results
./xtask.sh contract-validate merkle --commit HEAD
```

### 4. Proof Verification
```bash
# Verify entire chain integrity
./xtask.sh contract-validate verify --commit HEAD
```

## Security Considerations

### Tamper Detection
- Any modification to a file invalidates all parent hashes
- Hash mismatches indicate potential tampering
- Full audit trail in Git notes provides non-repudiation

### Deterministic Validation
- Validation order is fixed: char → line → chunk → file → dir → repo
- Same input always produces same output
- No external dependencies affect hash computation

### Access Control
- Git notes are read-only after creation
- Only authorized tools can create/update notes
- CI verifies note integrity on every commit

## Performance Optimization

### Caching Strategy
- Cache validation results by content hash
- Skip re-validation if content hasn't changed
- Parallel validation of independent scopes

### Incremental Updates
- Only validate modified scopes
- Recompute parent hashes incrementally
- Batch Git note operations

### Memory Management
- Stream large files instead of loading entirely
- Use memory-mapped files for large repositories
- Garbage collect old validation notes

## Error Handling

### Validation Failures
- Store errors in Git notes with full context
- Prevent state transitions on validation failure
- Provide detailed error messages for debugging

### Hash Mismatches
- Detect and report hash inconsistencies
- Rollback to last known good state
- Trigger re-validation of affected scopes

### Network Issues
- Retry Git note operations with exponential backoff
- Cache validation results locally
- Provide offline validation capabilities

## Benefits

### ✅ Schema + State Machine
- Specification that doubles as validation logic
- Enforces correct validation workflow
- Prevents invalid state transitions

### ✅ Git Notes = Auditable Proofs
- Tamper-proof validation history
- Tied to specific commits
- Full audit trail for compliance

### ✅ Merkle Hashing
- Guarantees deterministic aggregation
- Efficient verification of large repositories
- Cryptographic proof of integrity

### ✅ CI Enforcement
- Prevents stale, tampered, or unvalidated contracts
- Automated compliance checking
- Performance monitoring and optimization

## Future Enhancements

### Advanced Merkle Trees
- Support for sparse Merkle trees
- Efficient proof generation for large repositories
- Incremental tree updates

### Cross-Repository Validation
- Validate contracts across multiple repositories
- Shared validation rules and policies
- Federated validation authorities

### Performance Monitoring
- Validation performance metrics
- Bottleneck identification
- Automated optimization suggestions

### Machine Learning Integration
- Predictive validation based on code patterns
- Automated rule generation
- Anomaly detection in validation results

## Conclusion

The Contract Validation System provides a comprehensive, secure, and auditable approach to code validation. By combining state machines, Merkle chains, and Git Notes, it ensures that validation is deterministic, tamper-proof, and verifiable across all hierarchical scopes.

The system is designed to scale from small projects to large repositories while maintaining performance and security. The CI integration ensures that validation rules are enforced automatically, preventing invalid code from being merged into the repository.

This architecture serves as both a specification and implementation, providing a solid foundation for building reliable, auditable validation systems. 
