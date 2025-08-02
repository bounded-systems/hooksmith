# Git Object Contract System - Discriminated Union with Separate Contracts

## Overview

The Git Object Contract System implements a **Dev Contract** approach using discriminated unions to model different Git object types. This system provides separate, flat contracts for blobs, lines, and chunks, keeping the model clean and extensible while maintaining a structured approach to Git object validation.

## 🎯 Key Concepts

### Discriminated Union Structure

The system uses a discriminated union to represent different Git object types:

```rust
GitObjectContract = {
  type: "blob" | "tree" | "commit" | "tag",  // Discriminator
  // ... object-specific fields
}
```

### Separate Contract Types

1. **Blob Contract** - Represents the entire file as stored in Git
2. **Blob Line Contract** - Each line in the blob
3. **Blob Chunk Contract** - Represents a diff hunk (set of lines that changed together)

## 🏗️ Architecture

### 1. Blob Contract

```rust
BlobContract = {
  id: String,              // SHA-1/SHA-256 hash
  size: usize,             // Size in bytes
  encoding: String,        // Only allow UTF-8 for now
  lines: Vec<String>,      // Split by \n
  valid: bool,             // Validation result
  errors: Vec<String>,     // Validation errors
}
```

**Purpose**: Ensures the raw bytes meet your constraints and validates UTF-8 encoding.

### 2. Blob Line Contract

```rust
BlobLineContract = {
  line_number: usize,      // Line number (1-based)
  text: String,            // Line text content
  valid: bool,             // Whether the line is valid
  errors: Vec<String>,     // Validation errors for this line
  action: LineAction,      // Accept/Reject/Fix decision
}
```

**Purpose**: Validates each line individually according to character rules.

### 3. Blob Chunk Contract

```rust
BlobChunkContract = {
  header: String,          // Diff header (e.g., "@@ -1,3 +1,4 @@")
  old_start: usize,        // Starting line number in old version
  old_lines: usize,        // Number of lines in old version
  new_start: usize,        // Starting line number in new version
  new_lines: usize,        // Number of lines in new version
  lines: Vec<DiffLine>,    // Lines in this chunk
  valid: bool,             // Whether the chunk is valid
  errors: Vec<String>,     // Validation errors
}
```

**Purpose**: Represents a diff hunk (set of lines that changed together).

### 4. Diff Line

```rust
DiffLine = {
  line_type: DiffLineType, // Context | Add | Remove
  content: String,         // Line content
  valid: bool,             // Whether the line is valid
  errors: Vec<String>,     // Validation errors
}
```

## 🚀 Usage Examples

### Basic Blob Contract

```rust
use git_filter::prelude::*;

let validator = GitObjectValidator::default();
let content = b"Hello, World!\nThis is a test file.\n";
let blob = validator.validate_blob("abc123def456", content);

println!("{}", blob.summary());
// Output: ✅ Blob abc123de accepted (25 bytes, 2 lines, encoding: utf-8)
```

### Line Contracts

```rust
let lines = validator.validate_blob_lines(&blob);

for line in &lines {
    println!("{}", line.summary());
    // Output: ✅ Line 1 accepted (13 chars)
    // Output: ✅ Line 2 accepted (18 chars)
}
```

### Chunk Contracts (Diff Hunks)

```rust
let diff_lines = vec![
    (DiffLineType::Context, "Line 1: Unchanged".to_string()),
    (DiffLineType::Remove, "Line 2: This line was removed".to_string()),
    (DiffLineType::Add, "Line 2: This line was added".to_string()),
];

let chunk = validator.create_chunk_contract(
    "@@ -1,2 +1,2 @@",
    1, 2, 1, 2,  // old_start, old_lines, new_start, new_lines
    diff_lines,
);

println!("{}", chunk.summary());
// Output: ✅ Chunk @@ -1,2 +1,2 @@ valid (3 lines, 2 old, 2 new)
```

### Complete Git Object Validation

```rust
let git_object = validator.validate_git_object("abc123", content);

match git_object {
    GitObjectContract::Blob(blob) => {
        println!("Git Object Type: Blob");
        println!("{}", blob.summary());
        
        let lines = validator.validate_blob_lines(&blob);
        let summary = validator.summarize_validation(&blob, &lines);
        println!("{}", summary);
    }
}
```

## 🔧 Configuration

### Git Object Validator Settings

```rust
let validator = GitObjectValidator::new(
    true,   // validate_lines
    true,   // validate_chunks
);
```

### Line Action Types

```rust
enum LineAction {
    Accept,  // Line is valid
    Reject,  // Line is invalid
    Fix,     // Line needs fixing
}
```

### Diff Line Types

```rust
enum DiffLineType {
    Context,  // Unchanged line
    Add,      // Added line
    Remove,   // Removed line
}
```

## 🧪 Testing

Run the Git object contract demo:

```bash
cargo run --example git_object_contract_demo
```

This demonstrates:
- Basic blob contract validation
- Blob with invalid UTF-8
- Line contracts with character validation
- Chunk contracts (diff hunks)
- Complete Git object validation
- Diff modeling

## 📊 Contract Results

### Example Output

```
📝 Example 1: Basic Blob Contract
  ✅ Blob abc123de accepted (56 bytes, 3 lines, encoding: utf-8)

📄 Example 3: Line Contracts
  ✅ Line 1 accepted (19 chars)
  ❌ Line 2 rejected: Forbidden character at position 11: '\0'

🔀 Example 4: Chunk Contracts (Diff Hunks)
  ✅ Chunk @@ -1,3 +1,4 @@ valid (6 lines, 3 old, 4 new)
```

## 🛠️ Integration with Git Filter System

The Git object contract system integrates seamlessly with the existing git-filter system:

```rust
// Create a validator
let validator = GitObjectValidator::default();

// Validate Git objects
let git_object = validator.validate_git_object(oid, content);

// Process validation results
match git_object {
    GitObjectContract::Blob(blob) => {
        if blob.is_valid() {
            // Process valid blob
        } else {
            // Handle validation errors
        }
    }
}
```

## 🎯 Benefits

### 1. **Clean Separation of Concerns**
- Blobs, lines, and chunks are separate contracts
- Each contract has a single responsibility
- Easy to understand and extend

### 2. **Discriminated Union Pattern**
- Type-safe representation of different Git objects
- Extensible for future object types (Tree, Commit, Tag)
- Clear object type identification

### 3. **Flat Contract Structure**
- No nested contracts within objects
- References use simple string identifiers
- Easy to serialize and deserialize

### 4. **Diff Modeling**
- Diffs can be modeled as: Pair of Blob Contracts + Array of BlobChunkContracts
- Clean separation between blob validation and diff validation
- Support for complex diff scenarios

### 5. **Git-Native Integration**
- Works with Git's object model
- Supports all Git object types
- Integrates with Git's filter system

## 🔮 Advanced Features

### Custom Validation Rules

```rust
// Implement custom validation logic
impl GitObjectValidator {
    pub fn validate_with_custom_rules(&self, id: &str, content: &[u8], rules: &CustomRules) -> GitObjectContract {
        // Custom validation implementation
    }
}
```

### Diff Processing

```rust
// Process diffs with chunk validation
impl GitObjectValidator {
    pub fn process_diff(&self, old_blob: &BlobContract, new_blob: &BlobContract, chunks: &[BlobChunkContract]) -> DiffResult {
        // Diff processing implementation
    }
}
```

### Tree and Commit Contracts

```rust
// Future extensions
pub enum GitObjectContract {
    Blob(BlobContract),
    Tree(TreeContract),      // Future
    Commit(CommitContract),  // Future
    Tag(TagContract),        // Future
}
```

## 📖 API Reference

### Core Types

- `GitObjectContract`: Discriminated union for Git objects
- `BlobContract`: Contract for Git blobs
- `BlobLineContract`: Contract for individual lines
- `BlobChunkContract`: Contract for diff chunks
- `DiffLine`: Individual diff line
- `GitObjectValidator`: Main validation engine

### Key Methods

- `GitObjectValidator::validate_blob()`: Validate a Git blob
- `GitObjectValidator::validate_blob_lines()`: Validate all lines in a blob
- `GitObjectValidator::create_chunk_contract()`: Create a chunk contract
- `GitObjectValidator::validate_git_object()`: Validate a complete Git object
- `GitObjectValidator::summarize_validation()`: Get validation summary

## 🚨 Error Handling

The system provides comprehensive error handling:

```rust
match validator.validate_git_object(oid, content) {
    GitObjectContract::Blob(blob) => {
        if blob.is_valid() {
            // Process valid blob
            let lines = validator.validate_blob_lines(&blob);
            for line in &lines {
                if !line.is_valid() {
                    eprintln!("Line {} invalid: {}", line.line_number, line.errors.join(", "));
                }
            }
        } else {
            // Handle blob validation errors
            eprintln!("Blob invalid: {}", blob.errors.join(", "));
        }
    }
}
```

## 🔄 Flow

1. **Validate Git Object**: Create appropriate contract based on object type
2. **Validate Blob**: Check UTF-8 encoding and extract lines
3. **Validate Lines**: Check each line for forbidden characters and EOL issues
4. **Validate Chunks**: If processing diffs, validate chunk structure and content
5. **Emit Results**: Return structured validation results

## 📄 License

This component is part of the hooksmith project and is licensed under the MIT License. 
