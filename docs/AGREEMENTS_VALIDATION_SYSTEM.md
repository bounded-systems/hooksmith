# Agreements Validation System

A production-ready, streaming validation system that enforces repository structure through Git-only introspection with order-independent digest verification.

## Overview

The Agreements Validation System provides:
- **Top-level only validation**: No recursion, only root entries
- **Git-only operations**: Pure Git object inspection via libgit2
- **Order-independent digests**: Commutative SHA-256 sum over names + rules
- **SARIF output**: Standardized results for CI integration
- **Streaming architecture**: Single-pass, constant-memory validation
- **No overrides**: Allow-list only, default reject

## Architecture

### Five Actors

1. **Dispatcher** (Intake)
   - Discovers `.hooksmith/agreements/*.json` files
   - Loads and validates JSON schema
   - Enforces invariants (mode=tree, scope=top-level, etc.)

2. **Researcher** (Subject Materialization)
   - Uses git2 to materialize top-level Git tree
   - Returns map of name → ObjectType (no recursion)

3. **Reporter** (Canonical Summary)
   - Produces sorted list of names for digest input
   - Counts files/directories for logging

4. **Mandator** (Policy Enforcement)
   - Applies allow-overrides-reject decision logic
   - Directories: exact name match
   - Files: exact name or `*.ext` root pattern

5. **Auditor** (Digest Verification)
   - Recomputes SHA-256 digest over subject + rules
   - Compares to recorded digest value
   - Ensures tamper-evidence

6. **Triage Officer** (Orchestration)
   - Coordinates all actors per agreement
   - Produces SARIF 2.1.0 output
   - Sets exit code based on results

## Agreement Format

```json
{
  "version": "1",
  "mode": "tree",
  "precedence": "allow-overrides-reject",
  "default-action": "reject",
  "allow-dirs": [".github", ".hooksmith", "crates", "src", "tests"],
  "allow-files": ["README.md", "Cargo.toml", "*.md"],
  "subject": {
    "scope": "top-level"
  },
  "digest": {
    "algo": "sha256",
    "value": "<computed-digest>"
  }
}
```

## Usage

### Local Validation
```bash
# Validate all agreements against HEAD
cargo run -p hooksmith-validate -- --ref HEAD

# Validate against specific ref
cargo run -p hooksmith-validate -- --ref main --root .
```

### CI Integration
```yaml
- name: Validate agreements
  run: cargo run -p hooksmith-validate -- --ref ${{ github.sha }} --root .
```

## Digest Computation

The digest ensures both repository structure and validation rules are tamper-evident:

1. **Subject**: Sorted root entry names joined by `\n`
2. **Rules**: Canonical JSON of mode, precedence, defaultAction, allow_dirs, allow_files
3. **Digest**: `SHA256(hex(SHA256(subject)) + "\n" + hex(SHA256(rules)))`

## Key Features

- **Deterministic**: Same input always produces same result
- **Git-only**: No working tree required, pure Git object inspection
- **Top-level only**: No recursion, no path globs beyond simple `*.ext`
- **SARIF output**: Standardized results for GitHub Code Scanning
- **Fail-fast**: CI stops on first error
- **Tamper-evident**: Digest changes if structure or rules change

## File Structure

```
.hooksmith/agreements/
├── root.json              # Main top-level agreement
└── top-level-tree.json    # Alternative agreement

crates/hooksmith-validate/
├── src/
│   ├── lib.rs             # Five-actor implementation
│   └── bin/
│       └── hooksmith-validate.rs  # CLI entrypoint
└── Cargo.toml
```

## Acceptance Criteria

✅ **Only top-level names considered** (no `/` in evaluated names)  
✅ **New root files/dirs fail CI** unless added to allow_* AND digest rotated  
✅ **Deterministic digest** reproduces across machines  
✅ **Git-only operations** via libgit2  
✅ **SARIF 2.1.0 output** for CI integration  
✅ **Five-actor architecture** with clear separation of concerns  

## Benefits

- **Repository governance**: Enforce clean root structure
- **CI integration**: Automated validation on every PR
- **Tamper-evidence**: Digest ensures rules haven't been modified
- **Deterministic**: Reproducible results across environments
- **Extensible**: Easy to add new scopes or validation rules
