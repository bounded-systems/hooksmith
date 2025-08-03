# Structured Logging System

The Hooksmith project now includes a comprehensive structured logging system that provides JSONL (JSON Lines) output for all validation tools and integrates with the existing event bus.

## Overview

The structured logging system provides:

- **JSONL Output**: Machine-readable JSON events for all operations
- **Event Bus Integration**: Seamless integration with the existing event bus
- **Tool Support**: Structured output for Cargo, Git, and custom tools
- **Session Tracking**: Group related events with session IDs
- **Diagnostic Parsing**: Automatic parsing of Cargo/rustc JSON diagnostics

## Quick Start

### Basic Usage

```bash
# Run structured auto-push with JSONL output
cargo run -p xtask -- structured-auto-push

# Run with custom commit message
cargo run -p xtask -- structured-auto-push -m "feat: implement structured logging"

# Run in watchdog mode
cargo run -p xtask -- structured-auto-push --watchdog --interval 30

# Disable JSONL output (for TUI mode)
cargo run -p xtask -- structured-auto-push --no-jsonl

# Disable event bus integration
cargo run -p xtask -- structured-auto-push --no-event-bus
```

### Example Output

```jsonl
{"timestamp":"2025-08-03T18:11:44Z","level":"info","tool":"hooksmith","action":"start","message":"Starting structured auto-push workflow","details":null}
{"timestamp":"2025-08-03T18:11:50Z","level":"info","tool":"cargo","action":"check","message":"Running cargo check","details":null}
{"timestamp":"2025-08-03T18:11:54Z","level":"warn","tool":"cargo","action":"diagnostic","message":"variables can be used directly in the `format!` string","code":"clippy::uninlined_format_args","file":"xtask/src/main.rs","line":5609,"column":9}
{"timestamp":"2025-08-03T18:12:23Z","level":"info","tool":"git","action":"commit","message":"Committed changes: abc123def456","details":{"commit_hash":"abc123def456","commit_message":"feat: implement structured logging"}}
{"timestamp":"2025-08-03T18:12:31Z","level":"info","tool":"git","action":"push","message":"Successfully pushed changes","details":{"force":false,"output":"To github.com:bdelanghe/hooksmith.git"}}
```

## Architecture

### Core Components

1. **StructuredEvent**: Standard event structure for all operations
2. **StructuredLogger**: Main logging manager with JSONL and event bus integration
3. **StructuredAutoPush**: Auto-push workflow with structured logging

### Event Structure

```rust
pub struct StructuredEvent {
    pub timestamp: String,        // RFC3339 timestamp
    pub level: String,           // "info", "warn", "error"
    pub tool: String,            // "cargo", "git", "hooksmith"
    pub action: String,          // "validation", "commit", "push", etc.
    pub message: String,         // Human-readable message
    pub details: Option<Value>,  // Optional JSON details
    pub code: Option<String>,    // Error code (for diagnostics)
    pub file: Option<String>,    // File path (for diagnostics)
    pub line: Option<u32>,       // Line number (for diagnostics)
    pub column: Option<u32>,     // Column number (for diagnostics)
    pub session_id: Option<String>, // Session ID for grouping
}
```

## Tool Integration

### Cargo Commands

The system automatically adds JSON output format to supported Cargo commands:

```rust
// Automatically adds --message-format=json
logger.run_cargo_command("clippy", &["--workspace", "--all-targets"]).await?;

// Automatically adds --message-format=json
logger.run_cargo_command("check", &[]).await?;

// Automatically adds -- -Z unstable-options --format json
logger.run_cargo_command("test", &[]).await?;
```

### Git Commands

Git commands are enhanced with JSON output where available:

```rust
// Uses git status --json (Git ≥ 2.36)
let status = logger.git_status().await?;

// Uses git log --format=json (Git ≥ 2.41)
let commits = logger.git_log(Some(10)).await?;

// Standard git command with JSON output
let output = logger.run_git_command("push", &[]).await?;
```

### Custom Tools

You can create structured events for custom tools:

```rust
let event = StructuredEvent::new("info", "custom", "validation", "Running custom validation")
    .with_details(json!({
        "tool_version": "1.0.0",
        "config_file": "config.yaml"
    }));

logger.log(event)?;
```

## Event Bus Integration

The structured logging system integrates with the existing event bus:

```rust
// Events are automatically emitted to the event bus
let logger = StructuredLogger::new()
    .with_session_id("session-123");

// This will emit both JSONL and event bus events
logger.info("cargo", "check", "Running cargo check")?;
```

### Event Bus Events

Structured events are converted to HooksmithEvent objects:

```rust
// StructuredEvent -> HooksmithEvent conversion
HooksmithEvent::new(
    event.tool.clone(),           // actor
    event.action.clone(),         // event
    serde_json::to_value(&event)? // context
)
.with_state(event.action.clone())
.with_session_id(event.session_id.clone().unwrap_or_default())
```

## Advanced Usage

### Custom Logger Configuration

```rust
use hooksmith::structured_logging::{StructuredLogger, StructuredEvent};

let logger = StructuredLogger::new()
    .with_session_id("my-session")
    .without_jsonl()           // Disable JSONL output
    .without_event_bus();      // Disable event bus integration

// Log custom events
logger.info("my-tool", "custom-action", "Custom message")?;
```

### Diagnostic Parsing

The system automatically parses Cargo/rustc JSON diagnostics:

```rust
// Raw cargo clippy JSON output
let diagnostic = json!({
    "reason": "compiler-message",
    "message": {
        "level": "warning",
        "code": {"code": "clippy::uninlined_format_args"},
        "spans": [{"file_name": "src/main.rs", "line_start": 10}]
    }
});

// Automatically parsed into structured event
logger.diagnostic(&diagnostic)?;
```

### Session Management

Group related events with session IDs:

```rust
let session_id = uuid::Uuid::new_v4().to_string();
let logger = StructuredLogger::new().with_session_id(&session_id);

// All events will have the same session_id
logger.info("hooksmith", "start", "Starting workflow")?;
logger.info("cargo", "check", "Running validation")?;
logger.info("hooksmith", "completion", "Workflow completed")?;
```

## CLI Commands

### Structured Auto-Push

```bash
# Basic usage
cargo run -p xtask -- structured-auto-push

# With options
cargo run -p xtask -- structured-auto-push \
    --message "feat: implement new feature" \
    --watchdog \
    --interval 60 \
    --force \
    --verbose

# Disable features
cargo run -p xtask -- structured-auto-push \
    --no-jsonl \
    --no-event-bus
```

### Available Options

- `-m, --message`: Commit message
- `--allow-empty-message`: Allow empty commit messages (Trunk-style)
- `--watchdog`: Run in continuous monitoring mode
- `--interval`: Watchdog interval in seconds (default: 30)
- `--force`: Force push
- `--verbose`: Enable verbose output
- `--no-jsonl`: Disable JSONL output
- `--no-event-bus`: Disable event bus integration

## Integration Examples

### Dashboard Integration

```rust
// Subscribe to structured events
let mut rx = event_bus.subscribe();
while let Some(event) = rx.recv().await {
    if event.actor == "hooksmith" && event.event == "completion" {
        // Handle completion event
        let details: serde_json::Value = serde_json::from_value(event.context)?;
        println!("Workflow completed in {}ms", details["duration_ms"]);
    }
}
```

### CI/CD Integration

```bash
# Run structured auto-push and capture JSONL output
cargo run -p xtask -- structured-auto-push > events.jsonl

# Parse events for CI reporting
jq -r 'select(.level == "error") | .message' events.jsonl

# Count events by type
jq -r '.action' events.jsonl | sort | uniq -c
```

### Log Analysis

```bash
# Filter events by tool
jq -r 'select(.tool == "cargo")' events.jsonl

# Filter events by level
jq -r 'select(.level == "error")' events.jsonl

# Group events by session
jq -r 'group_by(.session_id) | .[] | {session: .[0].session_id, count: length}' events.jsonl

# Extract diagnostic information
jq -r 'select(.code) | {file, line, code, message}' events.jsonl
```

## Best Practices

### 1. Consistent Event Structure

Always use the standard event structure:

```rust
// Good
logger.info("cargo", "check", "Running cargo check")?;

// Avoid custom event structures
logger.log(CustomEvent { ... })?;
```

### 2. Meaningful Action Names

Use descriptive action names:

```rust
// Good
logger.info("git", "commit", "Committed changes")?;
logger.info("cargo", "clippy", "Running clippy checks")?;

// Avoid
logger.info("tool", "action", "message")?;
```

### 3. Proper Error Handling

Include relevant details in error events:

```rust
match result {
    Ok(_) => logger.info("cargo", "build", "Build successful")?,
    Err(e) => {
        logger.error("cargo", "build", &format!("Build failed: {}", e))?;
        // Include additional context if available
    }
}
```

### 4. Session Management

Use session IDs to group related events:

```rust
let session_id = uuid::Uuid::new_v4().to_string();
let logger = StructuredLogger::new().with_session_id(&session_id);

// All events in this workflow will have the same session_id
```

### 5. Diagnostic Details

Include file, line, and column information for diagnostics:

```rust
let event = StructuredEvent::new("warn", "cargo", "clippy", message)
    .with_diagnostic(code, file, line, column);
logger.log(event)?;
```

## Migration Guide

### From Plain Text Logging

Replace `println!` statements with structured logging:

```rust
// Before
println!("Running cargo check...");

// After
logger.info("cargo", "check", "Running cargo check")?;
```

### From Custom Event Systems

Convert custom events to structured events:

```rust
// Before
emit_custom_event(CustomEvent { tool: "cargo", action: "check" });

// After
logger.info("cargo", "check", "Running cargo check")?;
```

### From Event Bus Only

Add JSONL output to existing event bus usage:

```rust
// Before
event_bus.emit(HooksmithEvent::new("cargo", "check", context))?;

// After
logger.info("cargo", "check", "Running cargo check")?;
// Automatically emits to both JSONL and event bus
```

## Troubleshooting

### Common Issues

1. **JSONL not appearing**: Check if `--no-jsonl` flag is set
2. **Event bus not working**: Check if `--no-event-bus` flag is set
3. **Git JSON not working**: Ensure Git version ≥ 2.36 for `git status --json`
4. **Cargo JSON not working**: Ensure using supported Cargo commands

### Debug Mode

Enable verbose output for debugging:

```bash
cargo run -p xtask -- structured-auto-push --verbose
```

### Event Validation

Validate JSONL output:

```bash
# Check JSONL syntax
jq -r '.' events.jsonl

# Validate event structure
jq -r 'select(.timestamp and .level and .tool and .action and .message)' events.jsonl
```

## Future Enhancements

- **WebSocket Streaming**: Real-time event streaming
- **Event Filtering**: Filter events by tool, level, or action
- **Event Aggregation**: Aggregate events for reporting
- **Custom Formatters**: Support for different output formats
- **Event Replay**: Replay events from JSONL files
- **Metrics Integration**: Integration with metrics systems

## Contributing

When adding new tools or commands:

1. Use the structured logging system for all output
2. Follow the standard event structure
3. Include relevant diagnostic information
4. Add appropriate session management
5. Update this documentation

## See Also

- [Event Bus Documentation](EVENT_BUS.md)
- [Auto-Push Workflow](AUTO_PUSH.md)
- [CLI Reference](CLI_HELP.md)
- [Examples](../examples/structured_logging_demo.rs) 
