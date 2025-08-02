# Line Contract System - Line-Level Dev Contracts for Git Blobs

## Overview

The Line Contract System implements a **Dev Contract** approach for validating individual lines within Git blobs. This system treats each line as a separate contract that must be validated according to strict rules, providing granular validation at the line level while maintaining a flat contract structure.

## 🎯 Key Concepts

### Blob Line Contract (Flat)

Each line in a Git blob is validated according to a contract schema:

```rust
BlobLineContract = {
  oid: String,                    // Blob SHA-1/SHA-256
  line_number: usize,             // Line number (1-based index)
  byte_offset: usize,             // Offset in blob where line starts
  length: usize,                  // Line length in bytes
  valid_utf8: bool,               // True if line is valid UTF-8
  normalized_eol: bool,           // True if ends in LF only
  has_forbidden_byte: bool,       // True if line has any forbidden bytes
  action: LineAction,             // Accept/Reject/Fix decision
}
```

### Line Actions

| Action | Meaning | Description |
|--------|---------|-------------|
| `Accept` | Line is valid | Line passes all validation checks |
| `Reject` | Line is invalid | Line has critical issues (forbidden bytes, invalid UTF-8) |
| `Fix` | Line needs fixing | Line has fixable issues (EOL normalization) |

### Line Validation Rules

- **UTF-8 validation**: Each line must be valid UTF-8
- **Forbidden byte detection**: Lines with forbidden characters are rejected
- **EOL normalization**: Lines must end with LF only (CRLF → LF conversion)
- **Line positioning**: Accurate byte offset and line number tracking

## 🏗️ Architecture

### Core Components

1. **BlobLineContract** (`line_contract.rs`)
   - Represents the validation contract for a single line
   - Contains line metadata and validation results
   - Provides accept/reject/fix decisions

2. **LineValidator** (`line_contract.rs`)
   - Validates individual lines within Git blobs
   - Handles UTF-8 validation and line ending normalization
   - Processes lines sequentially with accurate positioning

3. **CombinedContractFilter** (`filter.rs`)
   - Combines blob-level and line-level validation
   - Provides comprehensive validation at multiple levels
   - Integrates with the existing git-filter system

## 🚀 Usage Examples

### Basic Line Validation

```rust
use git_filter::prelude::*;

let validator = LineValidator::default();
let line_content = b"Hello, World!\n";
let oid = "abc123def456";

let (contract, processed) = validator.validate_line(oid, 1, 0, line_content);

if contract.is_accepted() {
    println!("✅ Line accepted: {}", contract.summary());
} else if contract.needs_fixing() {
    println!("🔧 Line needs fixing: {}", contract.summary());
} else {
    println!("❌ Line rejected: {}", contract.summary());
}
```

### Validate All Lines in a Blob

```rust
let content = b"Line 1: Valid\nLine 2: Has\x00NUL\nLine 3: CRLF\r\n";
let (line_contracts, processed) = validator.validate_blob_lines(oid, content);

let summary = validator.summarize_line_contracts(&line_contracts);
println!("Line validation: {}", summary);

for contract in &line_contracts {
    println!("  {}", contract.summary());
}
```

### Line-by-Line Analysis

```rust
let content = b"Line 1: Normal\nLine 2: Has\x01control\nLine 3: CRLF\r\n";
let (contracts, processed) = validator.validate_blob_lines(oid, content);

for contract in &contracts {
    let status = match contract.action {
        LineAction::Accept => "✅",
        LineAction::Reject => "❌",
        LineAction::Fix => "🔧",
    };
    println!("{} Line {}: {} (offset: {}, length: {})", 
        status, 
        contract.line_number, 
        contract.summary(),
        contract.byte_offset,
        contract.length
    );
}
```

### Combined Blob and Line Validation

```rust
let filter = CombinedContractFilter::new(
    true,   // normalize_line_endings
    true,   // apply_binary_heuristic
    30.0,   // binary_threshold
    false,  // allow_mixed_eol
    true,   // generate_line_contracts
);

let content = b"Line 1: Valid\nLine 2: Has\x00NUL\nLine 3: CRLF\r\n";
let file_state = FileState::default();
let operation = GitOperation::Add;

match filter.process(content, &file_state, &operation) {
    Ok(processed) => {
        println!("✅ Processing successful");
    }
    Err(e) => {
        println!("❌ Processing failed: {}", e);
    }
}
```

## 🔧 Configuration

### Line Validator Settings

```rust
let validator = LineValidator::new(
    true,   // normalize_line_endings
    false,  // allow_mixed_eol (if false, marks as Fix)
    false,  // generate_byte_analysis
);
```

### Combined Filter Settings

```rust
let filter = CombinedContractFilter::new(
    true,   // normalize_line_endings
    true,   // apply_binary_heuristic
    30.0,   // binary_threshold
    false,  // allow_mixed_eol
    true,   // generate_line_contracts
);
```

## 🧪 Testing

Run the combined contract demo:

```bash
cargo run --example combined_contract_demo
```

This demonstrates:
- Basic combined validation
- Blob with mixed line issues
- Line-by-line analysis
- EOL normalization at line level
- UTF-8 validation per line
- Combined filter usage

## 📊 Contract Results

### BlobLineContract

```rust
BlobLineContract {
    oid: String,                    // Blob SHA
    line_number: usize,             // Line number (1-based)
    byte_offset: usize,             // Byte offset in blob
    length: usize,                  // Line length in bytes
    valid_utf8: bool,               // UTF-8 validation result
    normalized_eol: bool,           // Line ending normalization status
    has_forbidden_byte: bool,       // Whether line has forbidden bytes
    action: LineAction,             // Accept/Reject/Fix decision
}
```

### Example Output

```
✅ Line 1 accepted (14 bytes, UTF-8: valid, EOL: normalized)
❌ Line 2 rejected: forbidden bytes
🔧 Line 3 needs fixing: EOL normalization
```

## 🛠️ Integration with Git Filter System

The line contract system integrates seamlessly with the existing git-filter system:

```rust
// Add the combined contract filter to a MultiFilter
let mut filter = MultiFilter::new();
filter.add_driver("combined-contract", Box::new(CombinedContractFilter::default()));

// Use with file state and Git operations
let file_state = FileState::from_attributes(&attributes);
let operation = GitOperation::Add;

let processed = filter.process_file(content, &file_state, &operation)?;
```

## 🎯 Benefits

### 1. **Granular Validation**
- Line-level validation provides precise control
- Individual line contracts enable targeted fixes
- Accurate positioning with byte offsets

### 2. **Flat Contract Structure**
- One contract per line, no nesting
- Clear accept/reject/fix decisions
- Explicit validation metadata

### 3. **Combined Validation**
- Blob-level and line-level validation together
- Comprehensive validation at multiple levels
- Detailed error reporting

### 4. **Git-Native Integration**
- Works with Git's filter system
- Integrates with .gitattributes
- Supports clean/smudge operations

### 5. **Flexible Actions**
- Accept: Line is valid
- Reject: Line has critical issues
- Fix: Line has fixable issues

## 🔮 Advanced Features

### Custom Line Validation Rules

```rust
// Implement custom line validation logic
impl LineValidator {
    pub fn validate_line_with_custom_rules(&self, oid: &str, line_number: usize, content: &[u8], rules: &CustomRules) -> BlobLineContract {
        // Custom validation implementation
    }
}
```

### Line Processing Pipelines

```rust
// Process lines in a pipeline
impl LineValidator {
    pub fn process_lines_pipeline(&self, oid: &str, content: &[u8]) -> Vec<BlobLineContract> {
        // Pipeline processing implementation
    }
}
```

### Line Statistics and Analysis

```rust
// Get detailed line statistics
impl LineValidator {
    pub fn analyze_line_statistics(&self, contracts: &[BlobLineContract]) -> LineStatistics {
        // Statistics analysis implementation
    }
}
```

## 📖 API Reference

### Core Types

- `BlobLineContract`: Validation contract for a single line
- `LineAction`: Action to take (Accept/Reject/Fix)
- `LineValidator`: Main line validation engine
- `CombinedContractFilter`: Combined blob and line filter driver

### Key Methods

- `BlobLineContract::new()`: Create contract for a line
- `BlobLineContract::is_accepted()`: Check if line should be accepted
- `BlobLineContract::needs_fixing()`: Check if line needs fixing
- `LineValidator::validate_line()`: Validate a single line
- `LineValidator::validate_blob_lines()`: Validate all lines in a blob
- `LineValidator::summarize_line_contracts()`: Get summary of line validation

## 🚨 Error Handling

The system provides comprehensive error handling:

```rust
match validator.validate_blob_lines(oid, content) {
    Ok((contracts, processed)) => {
        // Check for rejected lines
        let rejected_lines: Vec<_> = contracts.iter()
            .filter(|c| c.is_rejected())
            .collect();
        
        if !rejected_lines.is_empty() {
            for contract in &rejected_lines {
                eprintln!("Line {} rejected: {}", contract.line_number, contract.summary());
            }
        }
    }
    Err(e) => {
        // Handle processing errors
        eprintln!("Processing error: {}", e);
    }
}
```

## 🔄 Flow

1. **Split blob into lines** (using newline as delimiter)
2. **For each line**:
   - Validate UTF-8
   - Check for forbidden bytes
   - Check EOL normalization
   - Generate line contract
3. **Emit**:
   - BlobLineContract per line
   - Summary statistics
   - Processed content

## 📄 License

This component is part of the hooksmith project and is licensed under the MIT License. 
