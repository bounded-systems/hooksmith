# Contract Validation State Machine Specification

## 🎯 Overview

This document formalizes the entire contract validation pipeline as a **schema-driven state machine** that acts as both:
- A specification of valid states and transitions (what must happen at each phase)
- A proof mechanism (producing verifiable artifacts like Git notes with hashes)

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Contract State Machine                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   State Schema  │  │ Transition Log  │  │  Proof Chain    │ │
│  │   (JSON Schema) │  │   (Git Notes)   │  │  (Merkle Tree)  │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Validation Pipeline                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Contract       │  │  State          │  │  CI             │ │
│  │  Validator      │  │  Auditor        │  │  Enforcement    │ │
│  │  (xtask)        │  │  (xtask)        │  │  (Git Hooks)    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 📋 1. State Machine Model

### States

| State | Meaning | Description |
|-------|---------|-------------|
| `UNTRACKED` | File has no contract or codegen attribute | File exists but no contract validation has been attempted |
| `UNVALIDATED` | File has contract/codegen attribute but no proof | File is marked for validation but hasn't been validated yet |
| `VALIDATED` | File has contract/codegen attribute + matching note with hash | File has been validated and proof is stored in Git notes |
| `LOCKED` | File is validated + no further modifications allowed | File is locked and requires re-validation for any changes |

### Transitions

| Event | From → To | Action | Trigger |
|-------|-----------|--------|---------|
| `detect_contract` | `UNTRACKED` → `UNVALIDATED` | Tag file via `.gitattributes` | File modification detected |
| `validate_contract` | `UNVALIDATED` → `VALIDATED` | Run validator → create Git note with hash | Manual validation or CI |
| `lock_contract` | `VALIDATED` → `LOCKED` | Post-hook marks file as locked | Post-validation hook |
| `modify_contract` | `LOCKED` → `UNVALIDATED` | Changing file invalidates proof | File modification |
| `regen_codegen` | `UNVALIDATED` → `VALIDATED` | Regenerate codegen + hash-check | Code generation |
| `release_proof` | `VALIDATED/LOCKED` → `VALIDATED/LOCKED` | CI verifies Merkle hashes | Release process |

## 📊 2. Schema Structure

### Contract State Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Contract State",
  "type": "object",
  "properties": {
    "file": { 
      "type": "string",
      "description": "Relative path to the file from repository root"
    },
    "contract": { 
      "type": "string",
      "description": "Type of contract (blob, tree, commit, tag, etc.)"
    },
    "state": { 
      "enum": ["UNTRACKED", "UNVALIDATED", "VALIDATED", "LOCKED"],
      "description": "Current state of the contract"
    },
    "hash": { 
      "type": "string", 
      "pattern": "^sha256:[a-f0-9]{64}$",
      "description": "SHA-256 hash of the validated content"
    },
    "validated_by": { 
      "type": "string",
      "description": "Tool and version that performed validation"
    },
    "timestamp": { 
      "type": "string", 
      "format": "date-time",
      "description": "ISO 8601 timestamp of validation"
    },
    "parent_scope": { 
      "type": "string",
      "description": "Parent scope identifier (for Merkle tree)"
    },
    "parent_hash": { 
      "type": "string",
      "description": "Hash of parent scope (for Merkle tree)"
    },
    "metadata": {
      "type": "object",
      "description": "Additional contract-specific metadata",
      "additionalProperties": true
    }
  },
  "required": ["file", "contract", "state", "hash", "validated_by", "timestamp"]
}
```

### Transition Log Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Contract Transition Log",
  "type": "object",
  "properties": {
    "transition": {
      "type": "string",
      "enum": ["detect_contract", "validate_contract", "lock_contract", "modify_contract", "regen_codegen", "release_proof"]
    },
    "from": {
      "type": "string",
      "enum": ["UNTRACKED", "UNVALIDATED", "VALIDATED", "LOCKED"]
    },
    "to": {
      "type": "string",
      "enum": ["UNTRACKED", "UNVALIDATED", "VALIDATED", "LOCKED"]
    },
    "file": {
      "type": "string"
    },
    "hash": {
      "type": "string",
      "pattern": "^sha256:[a-f0-9]{64}$"
    },
    "tool": {
      "type": "string"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "reason": {
      "type": "string",
      "description": "Human-readable reason for transition"
    }
  },
  "required": ["transition", "from", "to", "file", "hash", "tool", "timestamp"]
}
```

## 🌳 3. Merkle Validation Chain

### Hierarchy Structure

```
Repository (Root Hash)
├── Tree Objects
│   ├── TreeEntry (filename + mode + hash)
│   └── TreeEntry (filename + mode + hash)
├── Blob Objects
│   ├── BlobLine (char validation)
│   ├── BlobLine (char validation)
│   └── BlobLine (char validation)
├── Commit Objects
│   ├── Commit (tree + parents + author + message)
│   └── Commit (tree + parents + author + message)
└── Tag Objects
    └── Tag (object + tagger + message)
```

### Merkle Proof Specification

Each child scope (char/line/chunk) has a hash stored in a parent note:

```json
{
  "merkle_proof": {
    "root_hash": "sha256:abcd...",
    "path": [
      {
        "direction": "left",
        "hash": "sha256:efgh..."
      },
      {
        "direction": "right", 
        "hash": "sha256:ijkl..."
      }
    ],
    "leaf_hash": "sha256:mnop...",
    "leaf_index": 3
  }
}
```

## 🔧 4. Implementation Components

### State Machine Definition

```yaml
# contract-state-machine.yaml
states:
  UNTRACKED:
    description: "File has no contract or codegen attribute"
    allowed_transitions: ["UNVALIDATED"]
    
  UNVALIDATED:
    description: "File has contract/codegen attribute but no proof"
    allowed_transitions: ["VALIDATED"]
    
  VALIDATED:
    description: "File has contract/codegen attribute + matching note with hash"
    allowed_transitions: ["LOCKED", "UNVALIDATED"]
    
  LOCKED:
    description: "File is validated + no further modifications allowed"
    allowed_transitions: ["UNVALIDATED"]

transitions:
  detect_contract:
    from: ["UNTRACKED"]
    to: "UNVALIDATED"
    action: "Tag file via .gitattributes"
    
  validate_contract:
    from: ["UNVALIDATED"]
    to: "VALIDATED"
    action: "Run validator → create Git note with hash"
    
  lock_contract:
    from: ["VALIDATED"]
    to: "LOCKED"
    action: "Post-hook marks file as locked"
    
  modify_contract:
    from: ["LOCKED"]
    to: "UNVALIDATED"
    action: "Changing file invalidates proof"
    
  regen_codegen:
    from: ["UNVALIDATED"]
    to: "VALIDATED"
    action: "Regenerate codegen + hash-check"
    
  release_proof:
    from: ["VALIDATED", "LOCKED"]
    to: ["VALIDATED", "LOCKED"]
    action: "CI verifies Merkle hashes and proof chain"
```

### Validator Runner (xtask contract-validate)

```rust
// xtask/src/contract_validator.rs
pub struct ContractValidator {
    state_machine: StateMachine,
    git_notes: GitNotesManager,
    merkle_tree: MerkleTreeBuilder,
}

impl ContractValidator {
    pub fn validate_file(&self, file_path: &Path) -> Result<ContractState> {
        // 1. Check current state
        let current_state = self.get_current_state(file_path)?;
        
        // 2. Run appropriate validation based on contract type
        let validation_result = self.run_validation(file_path)?;
        
        // 3. Create new state
        let new_state = ContractState {
            file: file_path.to_string_lossy().to_string(),
            contract: self.detect_contract_type(file_path)?,
            state: "VALIDATED".to_string(),
            hash: validation_result.hash,
            validated_by: format!("xtask-contract-validate {}", env!("CARGO_PKG_VERSION")),
            timestamp: chrono::Utc::now().to_rfc3339(),
            parent_scope: validation_result.parent_scope,
            parent_hash: validation_result.parent_hash,
            metadata: validation_result.metadata,
        };
        
        // 4. Store state in Git notes
        self.git_notes.store_state(&new_state)?;
        
        // 5. Log transition
        self.log_transition(&current_state, &new_state, "validate_contract")?;
        
        Ok(new_state)
    }
}
```

### State Auditor (xtask contract-audit)

```rust
// xtask/src/contract_auditor.rs
pub struct ContractAuditor {
    state_machine: StateMachine,
    git_notes: GitNotesManager,
    merkle_verifier: MerkleVerifier,
}

impl ContractAuditor {
    pub fn audit_repository(&self) -> Result<AuditReport> {
        let mut report = AuditReport::new();
        
        // 1. Find all files with contract attributes
        let contract_files = self.find_contract_files()?;
        
        // 2. Verify each file's state
        for file in contract_files {
            let state = self.get_current_state(&file)?;
            let validation = self.validate_state(&state)?;
            report.add_file_validation(file, validation);
        }
        
        // 3. Verify Merkle proof chain
        let merkle_validation = self.verify_merkle_chain()?;
        report.set_merkle_validation(merkle_validation);
        
        // 4. Verify transition history
        let transition_validation = self.verify_transitions()?;
        report.set_transition_validation(transition_validation);
        
        Ok(report)
    }
}
```

## 🚀 5. CI Enforcement Logic

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# 1. Parse all modified files → check .gitattributes
modified_files=$(git diff --cached --name-only)

for file in $modified_files; do
    # Check if file has contract attributes
    if git check-attr contract "$file" | grep -q "contract: set"; then
        # 2. Query Git notes for state
        current_state=$(git notes --ref=refs/notes/contracts show "$file" 2>/dev/null || echo "UNTRACKED")
        
        # 3. Verify state transitions are valid
        if ! xtask contract-audit --file "$file" --strict; then
            echo "❌ Contract validation failed for $file"
            exit 1
        fi
    fi
done
```

### CI Pipeline

```yaml
# .github/workflows/contract-validation.yml
name: Contract Validation

on: [push, pull_request]

jobs:
  validate-contracts:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Full history for Git notes
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Build xtask
        run: cargo build --bin xtask
      
      - name: Audit Contract States
        run: ./target/debug/xtask contract-audit --strict
      
      - name: Verify Merkle Proofs
        run: ./target/debug/xtask contract-audit --merkle-only
      
      - name: Validate Transitions
        run: ./target/debug/xtask contract-audit --transitions-only
```

## 📝 6. Git Notes Storage

### Notes Reference Structure

```
refs/notes/contracts/          # Contract states
refs/notes/contracts-log/      # Transition history
refs/notes/merkle-proofs/      # Merkle tree proofs
```

### Example Git Note Content

```json
{
  "file": "src/modules/git_model.rs",
  "contract": "blob",
  "state": "VALIDATED",
  "hash": "sha256:a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456",
  "validated_by": "xtask-contract-validate 0.2.0",
  "timestamp": "2025-01-02T15:20:00Z",
  "parent_scope": "src/modules",
  "parent_hash": "sha256:fedcba0987654321098765432109876543210fedcba098765432109876543210",
  "metadata": {
    "line_count": 1250,
    "char_contracts": 1250,
    "validation_errors": 0
  }
}
```

## 🎯 7. Benefits

### ✅ Schema + State Machine
- Specification that doubles as validation logic
- Deterministic state transitions
- Clear audit trail

### ✅ Git Notes = Auditable Proofs
- Proofs tied to commits
- Immutable history
- Distributed verification

### ✅ Merkle Hashing
- Guarantees deterministic aggregation from char → repo level
- Efficient verification of large repositories
- Tamper-evident structure

### ✅ CI Enforcement
- Prevents stale, tampered, or unvalidated contracts
- Automated compliance checking
- Early detection of issues

## 🔄 8. Migration Path

### Phase 1: Foundation
1. Implement JSON schemas for contract states
2. Create basic state machine definition
3. Add xtask contract-validate command

### Phase 2: Git Integration
1. Implement Git notes storage
2. Add transition logging
3. Create contract-audit command

### Phase 3: Merkle Proofs
1. Implement Merkle tree builder
2. Add proof verification
3. Integrate with CI pipeline

### Phase 4: Enforcement
1. Add pre-commit hooks
2. Implement CI validation
3. Create monitoring dashboard

## 📚 9. References

- [JSON Schema Specification](https://json-schema.org/)
- [Git Notes Documentation](https://git-scm.com/docs/git-notes)
- [Merkle Tree Implementation](https://en.wikipedia.org/wiki/Merkle_tree)
- [State Machine Patterns](https://en.wikipedia.org/wiki/State_machine)
- [Hooksmith Architecture](ARCHITECTURE.md) 
