# Static Hook Definition System

The Hooksmith static hook definition system provides a **zero-dynamic-resolution**, **schema-validated** approach to defining Git hooks with mandatory binary existence validation.

## 🎯 Design Principles

### Static Contract Enforcement
- **No Optional Fields**: All fields are required
- **No Dynamic Inference**: Scope must be explicitly declared
- **No Runtime Conditions**: This is a build-layer contract, not an execution DAG
- **No Silent Fallbacks**: Failures are explicit and immediate

### Binary Existence Validation
- **Mandatory Binary Check**: Binary must exist at build time
- **Path Validation**: Binary must be a file, not a directory
- **Build-Time Contract**: No runtime discovery or fallbacks

## 📋 Hook Definition Format

### JSONC Schema (.jsonc)

```jsonc
{
  // Human-readable name of the hook
  "name": "pre-commit",

  // Hook trigger scope (only one allowed)
  "scope": "git",

  // Required list of concerns (must match schema)
  "concerns": [
    "blob",
    "tree", 
    "contract-violation"
  ],

  // Only one binary per hook, must exist at build time
  "bin": "target/release/hooksmith-validate-tree"
}
```

### Schema Validation

The system enforces strict validation:

1. **Name Format**: Alphanumeric characters, underscores, and hyphens only
2. **Scope Enum**: Must be one of: `git`, `github`, `fsmonitor`, `reference`, `email`, `patch`
3. **Concerns Array**: Must contain valid concerns, no duplicates
4. **Binary Path**: Must exist and be a file

## 🏗️ Directory Structure

```
.hooksmith/
└── hooks/
    └── git/
        └── pre-commit.jsonc
```

### Scope-Based Organization

Hooks are organized by scope in the `.hooksmith/hooks/` directory:

- `git/` - Traditional Git lifecycle hooks
- `github/` - GitHub-specific hooks
- `fsmonitor/` - File system monitoring hooks
- `reference/` - Reference transaction hooks
- `email/` - Email-related hooks
- `patch/` - Patch-related hooks

## 🔍 Hook Concerns

### Available Concerns

| Concern | Description |
|---------|-------------|
| `blob` | Validates Git blob objects |
| `tree` | Validates Git tree objects |
| `ref` | Validates Git references |
| `note` | Validates Git notes |
| `attr` | Validates Git attributes |
| `contract-violation` | Validates contract violations |
| `symbol-analysis` | Performs symbol analysis |

### Git Object Validation

The system focuses on Git objects that can be inspected with:

```bash
git rev-list --all --objects | \
git cat-file --batch-check='%(objectname) %(objecttype) %(rest)'
```

This command provides:
- **Object Hash**: SHA-1 hash of the object
- **Object Type**: `blob`, `tree`, `commit`, or `tag`
- **Additional Data**: Size, path, etc.

## 🛠️ CLI Tools

### Validate Single Hook

```bash
cargo run --bin validate-static-hook -- validate --file .hooksmith/hooks/git/pre-commit.jsonc
```

### Validate Directory

```bash
cargo run --bin validate-static-hook -- validate-dir --dir .hooksmith/hooks/git/
```

### Discover All Hooks

```bash
cargo run --bin validate-static-hook -- discover --root .
```

### Git Object Validator

```bash
cargo run --bin hooksmith-validate-tree
```

This binary validates all Git objects in the repository using the exact command specified:

```bash
git rev-list --all --objects | \
git cat-file --batch-check='%(objectname) %(objecttype) %(rest)'
```

## 🔧 Implementation Details

### Rust Schema

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct StaticHook {
    pub name: String,
    pub scope: HookScope,
    pub concerns: Vec<HookConcern>,
    pub bin: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HookScope {
    Git,
    Github,
    FsMonitor,
    Reference,
    Email,
    Patch,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum HookConcern {
    Blob,
    Tree,
    Ref,
    Note,
    Attr,
    ContractViolation,
    SymbolAnalysis,
}
```

### Validation Rules

1. **Name Validation**: Regex pattern `^[a-zA-Z0-9_-]+$`
2. **Concerns Uniqueness**: No duplicate concerns allowed
3. **Binary Existence**: File must exist and be readable
4. **Schema Compliance**: Must match JSON schema exactly

### Error Handling

The system provides explicit error messages for:

- Invalid hook names
- Duplicate concerns
- Missing binaries
- Invalid file paths
- Schema violations
- JSONC parsing errors

## 🚀 Usage Examples

### Create a Pre-commit Hook

1. **Define the hook**:

```jsonc
// .hooksmith/hooks/git/pre-commit.jsonc
{
  "name": "pre-commit",
  "scope": "git",
  "concerns": ["blob", "tree", "contract-violation"],
  "bin": "target/release/hooksmith-validate-tree"
}
```

2. **Build the binary**:

```bash
cargo build --release --bin hooksmith-validate-tree
```

3. **Validate the hook**:

```bash
cargo run --bin validate-static-hook -- validate --file .hooksmith/hooks/git/pre-commit.jsonc
```

### Create a GitHub Hook

```jsonc
// .hooksmith/hooks/github/pull-request.jsonc
{
  "name": "pull-request",
  "scope": "github",
  "concerns": ["blob", "contract-violation"],
  "bin": "target/release/hooksmith-github-validator"
}
```

## 🔒 Security Features

### Build-Time Validation

- **No Runtime Discovery**: All binaries must exist at build time
- **Explicit Dependencies**: No dynamic loading or fallbacks
- **Schema Enforcement**: Strict JSON schema validation
- **Path Validation**: Binary paths are validated for existence

### Contract Enforcement

- **Zero Dynamic Resolution**: All values are static
- **No Silent Failures**: All errors are explicit
- **Mandatory Fields**: No optional fields allowed
- **Type Safety**: Strong typing with Rust enums

## 📊 Validation Output

### Success Example

```
🔍 Validating static hook: .hooksmith/hooks/git/pre-commit.jsonc
✅ Hook 'pre-commit' is valid
   Scope: git
   Concerns: [Blob, Tree, ContractViolation]
   Binary: target/release/hooksmith-validate-tree
```

### Error Example

```
🔍 Validating static hook: .hooksmith/hooks/git/pre-commit.jsonc
❌ Hook validation failed: Missing hook binary: target/release/hooksmith-validate-tree
```

## 🎯 Integration with Git Workflow

### Lefthook Integration

The static hooks can be integrated with Lefthook:

```yaml
# lefthook.yml
pre-commit:
  commands:
    validate-git-objects:
      run: target/release/hooksmith-validate-tree
```

### Git Hooks Integration

Direct integration with Git hooks:

```bash
#!/bin/sh
# .git/hooks/pre-commit
target/release/hooksmith-validate-tree
```

## 🔄 Migration from Dynamic Hooks

### Before (Dynamic)

```rust
// Old dynamic approach
let hook = DynamicHook {
    name: "pre-commit".to_string(),
    scope: infer_scope_from_context(), // Dynamic
    concerns: discover_concerns(), // Dynamic
    bin: find_binary_or_fallback(), // Dynamic with fallback
};
```

### After (Static)

```jsonc
// New static approach
{
  "name": "pre-commit",
  "scope": "git", // Explicit
  "concerns": ["blob", "tree", "contract-violation"], // Explicit
  "bin": "target/release/hooksmith-validate-tree" // Explicit
}
```

## 🎉 Benefits

1. **Deterministic**: Same input always produces same output
2. **Fast**: No runtime discovery or validation
3. **Secure**: No dynamic code execution
4. **Reliable**: Failures are explicit and immediate
5. **Maintainable**: Clear contracts and validation
6. **Type-Safe**: Strong typing with Rust enums
7. **Schema-Validated**: JSON schema enforcement
8. **Build-Time Verified**: All dependencies verified at build time

This system provides a **rock-solid foundation** for Git hook management with **zero runtime surprises** and **explicit failure modes**.
