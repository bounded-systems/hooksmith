# Git-Native Hook System

## Overview

The Hooksmith static hook definition system has been updated to use **only Git-native object types** that map directly to `git2` and `gix` (gitoxide) enums. This ensures zero dynamic resolution, no silent fallbacks, and mandatory binary existence validation.

## 🔒 Git-Native Object Types

### Core Git Objects (from `.git/objects/`)

| Git Object | Description | git2::ObjectType | gix::ObjectKind |
|------------|-------------|------------------|-----------------|
| `blob` | File contents | ✅ Blob | ✅ Blob |
| `tree` | Directory structure | ✅ Tree | ✅ Tree |
| `commit` | Commit history | ✅ Commit | ✅ Commit |
| `tag` | Annotated tag | ✅ Tag | ✅ Tag |

### Git Metadata Types (tracked by Git)

| Concept | Description | Backing Location | git2/gix Support |
|---------|-------------|------------------|------------------|
| `ref` | References (heads, tags, etc.) | `.git/refs/`, `.git/packed-refs` | ✅ (as reference types) |
| `note` | Commit-attached metadata | `.git/refs/notes/`, objects/ | ✅ (as refs + blob) |
| `attr` | Git attributes | Working tree file or `.git/info` | ❌* (handled as files) |
| `index` | Git index (staging area) | `.git/index` | ✅ (gix has support) |

*Note: `attr` is not a true Git object but a file Git interprets, like `.gitignore`. It's still tracked as a "concern" for policy enforcement.

## 🧱 Implementation

### HookConcern Enum (Git-Native Only)

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum HookConcern {
    // Core Git object types (from .git/objects/)
    Blob,      // File contents
    Tree,      // Directory structure  
    Commit,    // Commit history
    Tag,       // Annotated tag
    
    // Git namespaced metadata (tracked in refs or pseudo-objects)
    Ref,       // References (heads, tags, etc.)
    Note,      // Notes (commit-attached metadata)
    Attr,      // Attributes (file-based config)
    Index,     // Index (staging area)
}
```

### Git-Native Validator

```rust
pub struct GitNativeValidator {
    object_counts: HashMap<GitObjectType, u32>,
    metadata_counts: HashMap<GitMetadataType, u32>,
}

impl GitNativeValidator {
    /// Validate that all concerns are Git-native
    pub fn validate_concerns(concerns: &[String]) -> Result<()> {
        let canonical_objects = Self::canonical_object_types();
        let canonical_metadata = Self::canonical_metadata_types();
        let all_canonical: Vec<&str> = canonical_objects.iter()
            .chain(canonical_metadata.iter())
            .cloned()
            .collect();
        
        for concern in concerns {
            if !all_canonical.contains(&concern.as_str()) {
                bail!("Non-Git-native concern '{}' not allowed", concern);
            }
        }
        Ok(())
    }
}
```

## 📁 Example Configuration

### Static Hook Definition (`.hooksmith/hooks/git/pre-commit.jsonc`)

```jsonc
{
  "name": "pre-commit",
  "scope": "git",
  "concerns": [
    "blob",
    "tree", 
    "commit"
  ],
  "bin": "hooksmith-validate-tree"
}
```

### JSON Schema (Updated)

```jsonc
{
  "$id": "https://hooksmith.dev/schemas/static-hook.schema.json",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Hooksmith Static Hook Definition",
  "description": "Static, JSONC-based build spec with zero dynamic resolution, no silent fallbacks, and mandatory binary existence validation",
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string",
      "description": "Human-readable name of the hook",
      "minLength": 1,
      "pattern": "^[a-zA-Z0-9_-]+$"
    },
    "scope": {
      "type": "string", 
      "description": "Hook trigger scope (only one allowed)",
      "enum": ["git", "github", "fsmonitor", "reference", "email", "patch"]
    },
    "concerns": {
      "type": "array",
      "description": "Required list of Git-native concerns (must match schema)",
      "minItems": 1,
      "uniqueItems": true,
      "items": {
        "type": "string",
        "enum": ["blob", "tree", "commit", "tag", "ref", "note", "attr", "index"]
      }
    },
    "bin": {
      "type": "string",
      "description": "Only one binary per hook, must exist at build time",
      "minLength": 1
    }
  },
  "required": ["name", "scope", "concerns", "bin"]
}
```

## 🔧 Validation System

### Build-Time Validation (`build.rs`)

```rust
fn validate_single_hook(hook_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Validate concerns (Git-native only)
    let concerns = json["concerns"].as_array().ok_or("Missing 'concerns' field")?;
    let valid_concerns = ["blob", "tree", "commit", "tag", "ref", "note", "attr", "index"];
    for concern in concerns {
        let concern_str = concern.as_str().ok_or("Invalid concern format")?;
        if !valid_concerns.contains(&concern_str) {
            return Err(format!("Invalid concern '{}': must be one of {:?}", concern_str, valid_concerns).into());
        }
    }
    // ... rest of validation
}
```

### Runtime Validation

```rust
impl StaticHook {
    pub fn validate(&self) -> Result<()> {
        // Validate concerns are Git-native
        let concern_strings: Vec<String> = self.concerns.iter()
            .map(|c| serde_json::to_string(c).unwrap().trim_matches('"').to_string())
            .collect();
        GitNativeValidator::validate_concerns(&concern_strings)?;
        
        // ... rest of validation
    }
}
```

## 🧪 Testing

### Git Object Analysis

The system includes a Git object validator that uses your exact command:

```bash
git rev-list --all --objects | cut -d' ' -f1 | git cat-file --batch-check='%(objecttype)' | sort | uniq -c
```

Example output:
```
📊 Git Object Analysis:
   Total objects: 34598

   Object types:
     tree: 13373
     blob: 12198
     commit: 9027

🔍 Contract Validation Analysis:

⚠️  Warnings:
   - Large number of blobs (12198) - consider cleanup
   - Large number of trees (13373) - consider cleanup
   - Large number of commits (9027) - consider cleanup

✅ Git object validation completed successfully
```

### Test Coverage

```rust
#[test]
fn test_map_object_type() {
    assert_eq!(GitNativeValidator::map_object_type("blob"), Some(GitObjectType::Blob));
    assert_eq!(GitNativeValidator::map_object_type("tree"), Some(GitObjectType::Tree));
    assert_eq!(GitNativeValidator::map_object_type("commit"), Some(GitObjectType::Commit));
    assert_eq!(GitNativeValidator::map_object_type("tag"), Some(GitObjectType::Tag));
    assert_eq!(GitNativeValidator::map_object_type("invalid"), None);
}

#[test]
fn test_validate_concerns() {
    let valid_concerns = vec!["blob".to_string(), "tree".to_string(), "ref".to_string()];
    assert!(GitNativeValidator::validate_concerns(&valid_concerns).is_ok());
    
    let invalid_concerns = vec!["blob".to_string(), "contract-violation".to_string()];
    assert!(GitNativeValidator::validate_concerns(&invalid_concerns).is_err());
}
```

## 🎯 Key Benefits

1. **Git-Native Only**: All concerns map directly to Git's internal object types
2. **Zero Dynamic Resolution**: No runtime inference or fallbacks
3. **Mandatory Binary Validation**: All binaries must exist at build time
4. **Schema Enforcement**: Strict validation against Git-native types only
5. **Build-Time Safety**: Compile-time validation prevents runtime surprises

## 🚫 Removed Concepts

The following non-Git-native concepts have been removed:
- `ContractViolation` - Not a Git object type
- `SymbolAnalysis` - Not a Git object type
- Any other derived or project-specific concerns

These can be added later in a separate `ValidationConcern` or `ProjectConcern` enum if needed, but the core `HookConcern` enum remains strictly Git-native.

## 🔄 Migration

Existing hook definitions using non-Git-native concerns will fail validation and must be updated to use only the 7 canonical Git-native types: `blob`, `tree`, `commit`, `tag`, `ref`, `note`, `attr`.
