# Git + Lefthook Integration with Event-Driven State Machine

This document describes the Git + Lefthook integration system that provides structured event-driven workflows with SARIF integration for contract validation.

## Overview

The Git + Lefthook integration transforms raw Git and Lefthook outputs into structured, machine-readable events that can be processed by a finite state machine. This enables:

- **Structured Git Events**: Wrapping git commit/push commands with JSONL events
- **Lefthook Integration**: Capturing and normalizing Lefthook hook outputs
- **State Machine Integration**: Mapping Git/Lefthook events to state transitions
- **SARIF Integration**: Emitting contract violations as SARIF results
- **Event Blocking**: Supporting dependency relationships between validation rules

## Architecture

### State Machine

The integration uses a finite state machine with the following states:

```rust
pub enum GitWorkflowState {
    IDLE,           // Initial state - no Git operations in progress
    COMMITTING,     // Git commit operation started
    HOOK_RUNNING,   // Lefthook hooks running after commit
    COMMITTED,      // Commit completed successfully
    PUSHING,        // Git push operation started
    PUSHED,         // Push completed successfully
    ERROR,          // Error state - operation failed
}
```

### Events

Events that trigger state transitions:

```rust
pub enum GitWorkflowEvent {
    CommitStarted,    // Git commit started
    HookStarted,      // Lefthook hook started
    HookCompleted,    // Lefthook hook completed
    CommitCompleted,  // Commit completed
    PushStarted,      // Git push started
    PushCompleted,    // Git push completed
    OperationFailed,  // Operation failed
}
```

### State Transitions

```
IDLE --CommitStarted--> COMMITTING
COMMITTING --HookStarted--> HOOK_RUNNING
HOOK_RUNNING --HookCompleted--> COMMITTED
COMMITTED --PushStarted--> PUSHING
PUSHING --PushCompleted--> PUSHED
* --OperationFailed--> ERROR
```

## Usage

### CLI Commands

The integration provides several CLI commands through the `cargo xtask git-lefthook` interface:

#### Complete Workflow

Execute a complete Git workflow (commit + hooks + push):

```bash
cargo xtask git-lefthook workflow \
  --message "feat: add contract validation" \
  --hook post-commit \
  --remote origin \
  --branch feature/contract-validation \
  --sarif-output results.sarif
```

#### Individual Operations

Execute Git commit with structured events:

```bash
cargo xtask git-lefthook commit \
  --message "feat: add new feature" \
  --files src/new_feature.rs
```

Execute Lefthook hooks with structured events:

```bash
cargo xtask git-lefthook hooks \
  --hook post-commit \
  --quiet
```

Execute Git push with structured events:

```bash
cargo xtask git-lefthook push \
  --remote origin \
  --branch main \
  --force
```

#### Contract Validation

Add contract validation with SARIF integration:

```bash
cargo xtask git-lefthook validate \
  --contract-id file-extension-policy \
  --file src/old_file.py \
  --rule-id file-extension-only-rs \
  --message "File has .py extension, only .rs files allowed" \
  --severity error \
  --line 1 \
  --column 1
```

Generate SARIF document from validation results:

```bash
cargo xtask git-lefthook generate-sarif \
  --output validation-results.sarif
```

Show current state and validation results:

```bash
cargo xtask git-lefthook status
```

### Programmatic Usage

#### Basic Integration

```rust
use hooksmith::git_lefthook_integration::GitLefthookIntegration;

let mut integration = GitLefthookIntegration::new();

// Execute Git commit
let commit_metadata = integration.execute_git_commit(
    "feat: add contract validation",
    Some(vec!["src/contract.rs".to_string()])
).await?;

// Execute Lefthook hooks
let hook_metadata = integration.execute_lefthook_hooks(
    "post-commit",
    false
).await?;

// Execute Git push
integration.execute_git_push("origin", "main", false).await?;
```

#### Contract Validation

```rust
use hooksmith::git_lefthook_integration::{
    ContractValidationResult, ContractViolation, ViolationSeverity
};

// Create a contract violation
let violation = ContractViolation {
    id: "ext-001".to_string(),
    rule_id: "file-extension-only-rs".to_string(),
    message: "File has .py extension, only .rs files allowed".to_string(),
    severity: ViolationSeverity::Error,
    file: "src/old_file.py".to_string(),
    line: Some(1),
    column: Some(1),
    end_line: Some(1),
    end_column: Some(1),
    details: Some(serde_json::json!({
        "expected_extension": ".rs",
        "actual_extension": ".py"
    })),
    fingerprint: Some("ext-py-file-001".to_string()),
    blocked_by: None,
};

// Create validation result
let validation_result = ContractValidationResult {
    is_valid: false,
    contract_id: "file-extension-policy".to_string(),
    file: "src/old_file.py".to_string(),
    errors: vec![violation],
    warnings: vec![],
    sarif_result: None,
    blocked_by: None,
    timestamp: Utc::now(),
};

// Add to integration
integration.add_validation_result(validation_result)?;
```

#### Event Blocking

```rust
// Add blocking dependency
integration.add_blocking_dependency("code-quality", "file-extension-policy");

// This means code-quality validation is blocked until file-extension-policy passes
```

#### SARIF Generation

```rust
// Generate SARIF document
let sarif_document = integration.generate_sarif_document()?;

// Access SARIF results
for result in integration.sarif_results() {
    println!("Rule: {}, Level: {}, Message: {}", 
        result.rule_id, result.level, result.message);
}
```

## Event Structure

### Git Commit Events

```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "type": "git.commit.started",
  "session_id": "uuid-1234-5678",
  "data": {
    "message": "feat: add contract validation",
    "files": ["src/contract.rs", "src/validation.rs"],
    "branch": "feature/contract-validation"
  }
}
```

### Lefthook Hook Events

```json
{
  "timestamp": "2024-01-15T10:30:05Z",
  "type": "lefthook.hook.started",
  "session_id": "uuid-1234-5678",
  "data": {
    "hook": "post-commit",
    "command": "cargo test",
    "files": ["src/contract.rs", "src/validation.rs"]
  }
}
```

### Contract Validation Events

```json
{
  "timestamp": "2024-01-15T10:30:10Z",
  "type": "contract.validation.result",
  "session_id": "uuid-1234-5678",
  "data": {
    "contract_id": "file-extension-policy",
    "file": "src/old_file.py",
    "is_valid": false,
    "errors": [
      {
        "id": "ext-001",
        "rule_id": "file-extension-only-rs",
        "message": "File has .py extension, only .rs files allowed",
        "severity": "error",
        "line": 1,
        "column": 1
      }
    ]
  }
}
```

## SARIF Integration

The integration automatically generates SARIF-compliant results for contract violations:

### SARIF Result Structure

```json
{
  "ruleId": "file-extension-only-rs",
  "level": "error",
  "message": {
    "text": "File has .py extension, only .rs files allowed"
  },
  "locations": [
    {
      "physicalLocation": {
        "artifactLocation": {
          "uri": "src/old_file.py"
        },
        "region": {
          "startLine": 1,
          "startColumn": 1,
          "endLine": 1,
          "endColumn": 1
        }
      }
    }
  ],
  "properties": {
    "contract_id": "file-extension-policy",
    "fingerprint": "ext-py-file-001"
  }
}
```

### SARIF Document Generation

```rust
let sarif_document = integration.generate_sarif_document()?;
// Returns a complete SARIF document with all validation results
```

## Configuration

### Lefthook Output Configuration

To reduce noise and improve parsing, configure Lefthook output settings:

```yaml
# lefthook.yml
output:
  - summary
  - failure
  - execution
```

Or disable output entirely (only errors will print):

```yaml
# lefthook.yml
output: false
```

Environment variables can also be used:

```bash
export LEFTHOOK_OUTPUT=false
export LEFTHOOK_QUIET=true
```

### Event Bus Integration

The integration can be connected to the event bus for real-time processing:

```rust
use hooksmith::event_bus::emit_event;

// Emit events during workflow execution
emit_event(&HooksmithEvent::GitWorkflowStarted {
    session_id: integration.session_id().to_string(),
    timestamp: Utc::now(),
}).await?;
```

## Examples

### Complete Workflow Example

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let mut integration = GitLefthookIntegration::new();
    
    // Execute complete workflow
    let commit_metadata = integration.execute_git_commit(
        "feat: add contract validation system",
        Some(vec!["src/contract.rs".to_string()])
    ).await?;
    
    let hook_metadata = integration.execute_lefthook_hooks(
        "post-commit",
        false
    ).await?;
    
    integration.execute_git_push("origin", "main", false).await?;
    
    // Generate SARIF report
    let sarif_document = integration.generate_sarif_document()?;
    std::fs::write("workflow-results.sarif", sarif_document)?;
    
    Ok(())
}
```

### Contract Validation Example

```rust
// Validate file extensions
let validation_result = ContractValidationResult {
    is_valid: false,
    contract_id: "file-extension-policy".to_string(),
    file: "src/old_file.py".to_string(),
    errors: vec![
        ContractViolation {
            id: "ext-001".to_string(),
            rule_id: "file-extension-only-rs".to_string(),
            message: "File has .py extension, only .rs files allowed".to_string(),
            severity: ViolationSeverity::Error,
            file: "src/old_file.py".to_string(),
            line: Some(1),
            column: Some(1),
            end_line: Some(1),
            end_column: Some(1),
            details: Some(serde_json::json!({
                "expected_extension": ".rs",
                "actual_extension": ".py"
            })),
            fingerprint: Some("ext-py-file-001".to_string()),
            blocked_by: None,
        }
    ],
    warnings: vec![],
    sarif_result: None,
    blocked_by: None,
    timestamp: Utc::now(),
};

integration.add_validation_result(validation_result)?;
```

## Best Practices

### 1. Event Structure

- Always include session IDs for grouping related events
- Use consistent event types and data structures
- Include timestamps for all events
- Provide meaningful context in event data

### 2. State Management

- Validate state transitions before executing operations
- Handle error states gracefully
- Provide clear error messages for invalid transitions
- Log state changes for debugging

### 3. Contract Validation

- Use descriptive rule IDs and messages
- Include line and column information when possible
- Provide actionable details in violation messages
- Use fingerprints for deduplication

### 4. SARIF Integration

- Generate SARIF results for all contract violations
- Include physical location information
- Use appropriate result levels (error, warning, note)
- Provide partial fingerprints for deduplication

### 5. Event Blocking

- Use blocking dependencies sparingly
- Ensure blocking relationships are acyclic
- Provide clear error messages for blocked operations
- Consider timeout mechanisms for blocked operations

## Troubleshooting

### Common Issues

1. **State Transition Errors**
   - Ensure events are emitted in the correct order
   - Check that the current state allows the requested transition
   - Verify event data is valid

2. **Lefthook Output Parsing**
   - Configure Lefthook output settings to reduce noise
   - Use environment variables for quiet operation
   - Handle different Lefthook versions gracefully

3. **SARIF Generation**
   - Ensure all required fields are provided
   - Validate SARIF document structure
   - Check file URIs are properly formatted

4. **Event Blocking**
   - Verify blocking dependencies are acyclic
   - Check that blocked contracts exist
   - Ensure blocking relationships are properly configured

### Debugging

Enable debug logging:

```rust
use tracing::Level;

tracing_subscriber::fmt()
    .with_max_level(Level::DEBUG)
    .init();
```

Check state machine transitions:

```rust
println!("Current state: {:?}", integration.current_state());
println!("Session ID: {}", integration.session_id());
```

Validate SARIF document:

```bash
# Use SARIF validator if available
sarif-validator workflow-results.sarif
```

## Integration with Existing Systems

### CI/CD Integration

The integration can be used in CI/CD pipelines:

```yaml
# .github/workflows/contract-validation.yml
name: Contract Validation
on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run contract validation
        run: |
          cargo xtask git-lefthook workflow \
            --message "CI validation" \
            --sarif-output ci-results.sarif
      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: ci-results.sarif
```

### IDE Integration

SARIF results can be displayed in IDEs that support SARIF:

- VS Code with SARIF Viewer extension
- IntelliJ IDEA with SARIF plugin
- Emacs with SARIF mode

### Monitoring Integration

Events can be sent to monitoring systems:

```rust
// Send to monitoring system
let event = HooksmithEvent::ContractValidationFailed {
    contract_id: "file-extension-policy".to_string(),
    file: "src/old_file.py".to_string(),
    violations: vec![violation.clone()],
    timestamp: Utc::now(),
};

monitoring_client.send_event(event).await?;
```

## Future Enhancements

### Planned Features

1. **Advanced State Machine**
   - Support for parallel operations
   - Timeout handling
   - Retry mechanisms

2. **Enhanced SARIF Support**
   - SARIF 2.1.0 compliance
   - Advanced location information
   - Custom rule definitions

3. **Event Streaming**
   - Real-time event streaming
   - Event replay capabilities
   - Event persistence

4. **Contract Templates**
   - Predefined contract templates
   - Contract composition
   - Contract inheritance

5. **Integration APIs**
   - REST API for external tools
   - WebSocket API for real-time updates
   - GraphQL API for complex queries

### Contributing

To contribute to the Git + Lefthook integration:

1. Follow the existing code style and patterns
2. Add tests for new functionality
3. Update documentation for new features
4. Ensure SARIF compliance for validation results
5. Consider backward compatibility

## Conclusion

The Git + Lefthook integration provides a powerful foundation for structured Git workflows with contract validation and SARIF integration. By transforming raw Git and Lefthook outputs into structured events, it enables sophisticated automation and validation workflows while maintaining compatibility with existing tools and systems.

The integration is designed to be extensible and can be adapted to various use cases, from simple file validation to complex multi-stage deployment pipelines with comprehensive contract enforcement. 