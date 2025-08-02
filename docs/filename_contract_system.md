# Filename Contract System - Flat Unscoped Contract for Filename Validation

## Overview

The Filename Contract System implements a **flat unscoped contract** for filename validation that is completely independent of Git object types. This system focuses solely on filename validation rules, making it reusable across different contexts and file systems.

## 🎯 Key Concepts

### Flat Unscoped Contract

The `FilenameContract` is designed as a **flat unscoped contract** that:

1. **Focuses on a single concern**: Filename validation only
2. **Independent of context**: Not tied to Git objects, file systems, or specific use cases
3. **Reusable**: Can be used anywhere filename validation is needed
4. **Configurable**: Supports both strict and non-strict validation modes

### Validation Modes

```rust
// Non-strict mode (allows subdirectories)
FilenameContract::new("src/main.rs") // ✅ Valid

// Strict mode (blocks path separators)
FilenameContract::new_strict("src/main.rs") // ❌ Invalid
```

## 🏗️ Architecture

### 1. Filename Contract

```rust
FilenameContract = {
  filename: String,         // The filename to validate
  valid: bool,             // Overall validation result
  errors: Vec<String>,     // Validation errors
}
```

**Purpose**: Validates a single filename according to configurable rules.

### 2. Filename Validator

```rust
FilenameValidator = {
  strict: bool,            // Whether to block path separators
}
```

**Purpose**: Provides batch validation and utility methods for filename processing.

## 🚀 Usage Examples

### Basic Filename Validation

```rust
use git_filter::prelude::*;

// Validate a simple filename
let contract = FilenameContract::new("README.md".to_string());
println!("{}", contract.summary());
// Output: ✅ Filename 'README.md' valid

// Validate an empty filename
let contract = FilenameContract::new("".to_string());
println!("{}", contract.summary());
// Output: ❌ Filename '' invalid: Filename must not be empty
```

### Strict vs Non-Strict Validation

```rust
// Non-strict validation (allows subdirectories)
let contract = FilenameContract::new("src/main.rs".to_string());
println!("{}", contract.summary());
// Output: ✅ Filename 'src/main.rs' valid

// Strict validation (blocks path separators)
let contract = FilenameContract::new_strict("src/main.rs".to_string());
println!("{}", contract.summary());
// Output: ❌ Filename 'src/main.rs' invalid: Filename must not contain '/'
```

### Batch Filename Validation

```rust
let validator = FilenameValidator::new(false); // Non-strict
let filenames = vec![
    "README.md".to_string(),
    "src/main.rs".to_string(),
    "".to_string(),
];

let contracts = validator.validate_filenames(filenames);
let summary = validator.summarize_validation(&contracts);

println!("{}", summary);
// Output: Filenames: 3 total (2 valid, 1 invalid)
```

### Integration with Other Contracts

```rust
// Use with TreeEntryContract
let tree_entry = TreeEntryContract::new(
    "100644",
    "src/main.rs".to_string(),
    "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
);

// Validate filename separately
let filename_contract = FilenameContract::new("src/main.rs".to_string());

println!("Tree entry: {}", tree_entry.summary());
println!("Filename: {}", filename_contract.summary());
```

## 🔧 Configuration

### Validation Rules

1. **Non-empty**: Filename must not be empty
2. **Path separators** (strict mode only): Filename must not contain `/`

### Validation Modes

```rust
// Non-strict mode (default)
let validator = FilenameValidator::new(false);
// Allows: README.md, src/main.rs, scripts/build.sh
// Blocks: "" (empty)

// Strict mode
let validator = FilenameValidator::new(true);
// Allows: README.md, file.txt, script.sh
// Blocks: "", src/main.rs, scripts/build.sh
```

## 🧪 Testing

Run the filename contract demo:

```bash
cargo run --example filename_contract_demo
```

This demonstrates:
- Basic filename validation
- Strict vs non-strict validation
- Batch filename validation
- Integration with tree entries
- Unscoped contract characteristics

## 📊 Contract Results

### Example Output

```
📄 Example 1: Basic Filename Validation
  ✅ Filename 'README.md' valid
  ✅ Filename 'src/main.rs' valid
  ❌ Filename '' invalid: Filename must not be empty

🚫 Example 2: Strict Filename Validation (No Path Separators)
  ✅ Filename 'README.md' valid
  ❌ Filename 'src/main.rs' invalid: Filename must not contain '/'

📂 Example 3: Non-Strict Filename Validation (Allows Subdirectories)
  ✅ Filename 'src/main.rs' valid
  ✅ Filename 'scripts/build.sh' valid

📋 Example 4: Batch Filename Validation
  Filenames: 7 total (6 valid, 1 invalid)
```

## 🛠️ Integration with Other Systems

### With Tree Entry Contracts

```rust
// Create tree entries
let tree_entries = vec![
    TreeEntryContract::new("100644", "README.md".to_string(), "abc123...".to_string()),
    TreeEntryContract::new("100644", "src/main.rs".to_string(), "def456...".to_string()),
];

// Validate filenames separately
let filename_validator = FilenameValidator::new(false);
let filenames: Vec<String> = tree_entries.iter().map(|e| e.filename.clone()).collect();
let filename_contracts = filename_validator.validate_filenames(filenames);

// Show combined validation
for (tree_entry, filename_contract) in tree_entries.iter().zip(filename_contracts.iter()) {
    println!("Tree: {} | Filename: {}", tree_entry.summary(), filename_contract.summary());
}
```

### With File System Operations

```rust
// Validate filenames before file operations
let filenames = vec!["file1.txt", "file2.txt", "subdir/file3.txt"];

let validator = FilenameValidator::new(true); // Strict mode
let contracts = validator.validate_filenames(filenames.into_iter().map(|s| s.to_string()).collect());

// Only process valid filenames
for contract in &contracts {
    if contract.is_valid() {
        // Process file
        println!("Processing: {}", contract.filename);
    } else {
        println!("Skipping invalid filename: {}", contract.filename);
    }
}
```

## 🎯 Benefits

### 1. **Unscoped Design**
- Independent of Git object types
- Not tied to specific file systems
- Reusable across different contexts
- Focuses on a single validation concern

### 2. **Flat Contract Structure**
- Simple, explicit validation rules
- No nested objects or complex relationships
- Easy to understand and serialize
- Clear validation results

### 3. **Configurable Validation**
- Two validation modes (strict/non-strict)
- Flexible for different use cases
- Easy to extend with additional rules
- Clear error reporting

### 4. **Batch Processing**
- Efficient validation of multiple filenames
- Summary statistics and reporting
- Filtering of valid/invalid filenames
- Integration with other validation systems

### 5. **Reusability**
- Can be used with any file system
- Works with Git objects, regular files, or any path
- Independent validation logic
- Easy to test and maintain

## 🔮 Advanced Features

### Custom Validation Rules

```rust
// Extend with custom validation rules
impl FilenameContract {
    pub fn new_with_custom_rules(filename: String, rules: &CustomRules) -> Self {
        let mut errors = Vec::new();
        
        // Apply custom rules
        if rules.block_spaces && filename.contains(' ') {
            errors.push("Filename must not contain spaces".to_string());
        }
        
        if rules.max_length > 0 && filename.len() > rules.max_length {
            errors.push(format!("Filename too long (max: {})", rules.max_length));
        }
        
        let valid = errors.is_empty();
        Self { filename, valid, errors }
    }
}
```

### Filename Statistics

```rust
// Get detailed filename statistics
impl FilenameValidator {
    pub fn get_filename_statistics(&self, contracts: &[FilenameContract]) -> FilenameStatistics {
        let total = contracts.len();
        let valid = contracts.iter().filter(|c| c.is_valid()).count();
        let with_paths = contracts.iter().filter(|c| c.contains_path_separators()).count();
        let empty = contracts.iter().filter(|c| c.is_empty()).count();
        
        FilenameStatistics {
            total,
            valid,
            invalid: total - valid,
            with_paths,
            empty,
        }
    }
}
```

### Integration with Path Validation

```rust
// Combine with path validation
impl FilenameContract {
    pub fn validate_as_path(&self) -> PathValidationResult {
        // Validate as a file system path
        // Check for invalid characters, length limits, etc.
    }
}
```

## 📖 API Reference

### Core Types

- `FilenameContract`: Contract for individual filename validation
- `FilenameValidator`: Validator for batch filename processing

### Key Methods

- `FilenameContract::new()`: Create contract (non-strict)
- `FilenameContract::new_strict()`: Create contract (strict)
- `FilenameContract::is_valid()`: Check if filename is valid
- `FilenameContract::summary()`: Get validation summary
- `FilenameValidator::validate_filename()`: Validate single filename
- `FilenameValidator::validate_filenames()`: Validate multiple filenames
- `FilenameValidator::summarize_validation()`: Get batch validation summary

## 🚨 Error Handling

The system provides clear error reporting:

```rust
let contract = FilenameContract::new_strict("src/main.rs".to_string());

if !contract.is_valid() {
    for error in &contract.errors {
        eprintln!("Filename validation error: {}", error);
    }
}

// Output: Filename validation error: Filename must not contain '/'
```

## 🔄 Flow

1. **Create Contract**: Choose validation mode (strict/non-strict)
2. **Validate Filename**: Apply validation rules
3. **Check Results**: Examine validation status and errors
4. **Process Accordingly**: Handle valid/invalid filenames as needed

## 📄 License

This component is part of the hooksmith project and is licensed under the MIT License. 
