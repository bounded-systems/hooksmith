# Structured Logging Tools

This document covers Rust-based CLI tools for working with the JSONL output from Hooksmith's structured logging system.

## Overview

The structured logging system produces JSONL (JSON Lines) output that can be processed by various Rust-native tools for filtering, querying, browsing, and analyzing events.

## Recommended Tools

### 🦀 Core Tools

#### jql — JSON Query Language
Fast, jq-like CLI written in Rust for querying and filtering JSON data.

```bash
# Install
cargo install jql

# Basic usage with Hooksmith events
cargo run -p xtask -- structured-auto-push > events.jsonl
jql '.level=="error"' events.jsonl
jql '.tool=="cargo" and .action=="diagnostic"' events.jsonl
jql '.message | contains("validation")' events.jsonl
```

#### jless — Command-Line JSON Viewer
Interactive, syntax-highlighted pager for browsing JSON data.

```bash
# Install
cargo install jless

# Browse Hooksmith events
cargo run -p xtask -- structured-auto-push > events.jsonl
jless events.jsonl
```

#### fblog — JSON Log Tailer with Filters
Filters JSON lines with Lua expressions, perfect for real-time monitoring.

```bash
# Install
cargo install fblog

# Monitor errors in real-time
cargo run -p xtask -- structured-auto-push | fblog -f 'level == "error"'

# Filter by tool and action
cargo run -p xtask -- structured-auto-push | fblog -f 'tool == "cargo" and action == "clippy"'
```

#### lap — Log Analyzer Pro
Live log analyzer with TUI interface for high-frequency JSON streams.

```bash
# Install
cargo install lap

# Live analysis of Hooksmith events
cargo run -p xtask -- structured-auto-push | lap --follow
```

## Integration Examples

### Real-Time Event Monitoring

```bash
# Monitor validation errors in real-time
cargo run -p xtask -- structured-auto-push | \
  jql '.level=="error" or (.level=="warn" and .tool=="cargo")'

# Filter by session ID
cargo run -p xtask -- structured-auto-push | \
  jql '.session_id=="session-123"'

# Extract diagnostic information
cargo run -p xtask -- structured-auto-push | \
  jql 'select(.code) | {file, line, code, message}'
```

### Event Analysis

```bash
# Count events by tool
cargo run -p xtask -- structured-auto-push > events.jsonl
jql '.tool' events.jsonl | sort | uniq -c

# Count events by level
jql '.level' events.jsonl | sort | uniq -c

# Find all validation failures
jql '.action=="validation" and .level=="error"' events.jsonl

# Extract commit information
jql '.action=="commit" | {hash: .details.commit_hash, message: .details.commit_message}' events.jsonl
```

### Performance Analysis

```bash
# Analyze workflow duration
jql '.action=="completion" | .details.duration_ms' events.jsonl

# Find slow operations
jql 'select(.details.duration_ms > 5000)' events.jsonl

# Track validation times
jql '.action=="validation" | {start: .timestamp, duration: .details.duration_ms}' events.jsonl
```

### Interactive Browsing

```bash
# Browse all events
jless events.jsonl

# Browse only errors
jql '.level=="error"' events.jsonl | jless

# Browse by session
jql '.session_id=="session-123"' events.jsonl | jless
```

## Advanced Filtering with fblog

```bash
# Complex filtering with Lua expressions
cargo run -p xtask -- structured-auto-push | \
  fblog -f 'level == "error" and tool == "cargo"'

# Filter by file path
cargo run -p xtask -- structured-auto-push | \
  fblog -f 'file and file:find("main.rs")'

# Filter by time range (if timestamp is recent)
cargo run -p xtask -- structured-auto-push | \
  fblog -f 'timestamp:sub(1, 10) == "2025-08-03"'

# Custom output format
cargo run -p xtask -- structured-auto-push | \
  fblog -f 'level == "error"' -o '{timestamp} {tool}:{action} - {message}'
```

## Live Dashboard with lap

```bash
# Start live monitoring
cargo run -p xtask -- structured-auto-push | lap --follow

# With custom filters
cargo run -p xtask -- structured-auto-push | \
  lap --follow --filter 'error' --filter 'cargo:clippy'
```

## Shell Scripts and Automation

### Error Monitoring Script

```bash
#!/bin/bash
# monitor_errors.sh

echo "Monitoring Hooksmith events for errors..."

cargo run -p xtask -- structured-auto-push | \
  jql '.level=="error"' | \
  while read -r line; do
    echo "[$(date)] ERROR: $line"
    # Send notification, log to file, etc.
  done
```

### Validation Summary Script

```bash
#!/bin/bash
# validation_summary.sh

echo "Generating validation summary..."

cargo run -p xtask -- structured-auto-push > events.jsonl

echo "=== Validation Summary ==="
echo "Total events: $(jql '.' events.jsonl | wc -l)"
echo "Errors: $(jql '.level=="error"' events.jsonl | wc -l)"
echo "Warnings: $(jql '.level=="warn"' events.jsonl | wc -l)"
echo "Success: $(jql '.level=="info" and .action=="completion"' events.jsonl | wc -l)"

echo ""
echo "=== Tool Breakdown ==="
jql '.tool' events.jsonl | sort | uniq -c

echo ""
echo "=== Recent Errors ==="
jql '.level=="error" | {timestamp, tool, action, message}' events.jsonl | tail -5
```

### CI/CD Integration

```bash
#!/bin/bash
# ci_validation.sh

echo "Running structured validation..."

# Run validation and capture output
cargo run -p xtask -- structured-auto-push > validation_events.jsonl

# Check for errors
error_count=$(jql '.level=="error"' validation_events.jsonl | wc -l)

if [ "$error_count" -gt 0 ]; then
    echo "❌ Validation failed with $error_count errors:"
    jql '.level=="error" | {tool, action, message}' validation_events.jsonl
    exit 1
else
    echo "✅ Validation passed successfully"
    # Extract performance metrics
    duration=$(jql '.action=="completion" | .details.duration_ms' validation_events.jsonl)
    echo "⏱️  Total duration: ${duration}ms"
    exit 0
fi
```

## Tool Comparison

| Tool | Best For | Use Case |
|------|----------|----------|
| **jql** | Scripting, automation | Filtering, querying, data extraction |
| **jless** | Interactive browsing | Manual exploration of log files |
| **fblog** | Real-time monitoring | Live filtering with complex expressions |
| **lap** | TUI dashboards | Multi-panel live monitoring |
| **jq** | Universal compatibility | Cross-platform scripting |

## Installation Script

```bash
#!/bin/bash
# install_logging_tools.sh

echo "Installing Rust-based logging tools..."

# Core tools
cargo install jql
cargo install jless
cargo install fblog
cargo install lap

echo "✅ All tools installed successfully!"
echo ""
echo "Usage examples:"
echo "  jql '.level==\"error\"' events.jsonl"
echo "  jless events.jsonl"
echo "  fblog -f 'level == \"error\"' events.jsonl"
echo "  lap --follow events.jsonl"
```

## Integration with Hooksmith Workflows

### Watchdog Mode with Live Monitoring

```bash
# Start structured auto-push in watchdog mode
cargo run -p xtask -- structured-auto-push --watchdog --interval 30 | \
  fblog -f 'level == "error" or level == "warn"'
```

### Dashboard Mode

```bash
# Run with TUI dashboard
cargo run -p xtask -- structured-auto-push --no-jsonl | \
  lap --follow --title "Hooksmith Events"
```

### Batch Analysis

```bash
# Collect events for analysis
cargo run -p xtask -- structured-auto-push > batch_events.jsonl

# Analyze performance
jql '.action=="completion" | .details.duration_ms' batch_events.jsonl | \
  awk '{sum+=$1; count++} END {print "Average duration:", sum/count, "ms"}'

# Find common issues
jql '.level=="error" | .code' batch_events.jsonl | sort | uniq -c | sort -nr
```

## Best Practices

### 1. Use Appropriate Tools for the Task

- **jql** for scripting and automation
- **jless** for manual exploration
- **fblog** for real-time monitoring
- **lap** for TUI dashboards

### 2. Optimize Queries

```bash
# Good: Specific filtering
jql '.tool=="cargo" and .action=="clippy" and .level=="error"'

# Avoid: Overly broad queries
jql '.level=="error"'  # Too broad, may return too many results
```

### 3. Use Session IDs for Correlation

```bash
# Track events by session
jql '.session_id=="session-123" | {timestamp, action, message}'
```

### 4. Extract Meaningful Metrics

```bash
# Performance metrics
jql '.action=="completion" | {duration: .details.duration_ms, success: .level=="info"}'

# Error patterns
jql '.level=="error" | {tool, action, code, file}'
```

### 5. Combine Tools for Complex Workflows

```bash
# Real-time error monitoring with notification
cargo run -p xtask -- structured-auto-push | \
  jql '.level=="error"' | \
  fblog -f 'true' -o '{timestamp} ERROR: {message}' | \
  while read -r line; do
    echo "$line" | notify-send "Hooksmith Error"
  done
```

## Troubleshooting

### Common Issues

1. **jql not found**: Install with `cargo install jql`
2. **Permission errors**: Ensure tools are in your PATH
3. **Empty output**: Check if events are being generated
4. **Performance issues**: Use more specific filters

### Debug Commands

```bash
# Check if events are being generated
cargo run -p xtask -- structured-auto-push | head -5

# Validate JSONL format
cargo run -p xtask -- structured-auto-push | jql '.' | head -1

# Test specific queries
cargo run -p xtask -- structured-auto-push | jql '.level=="info"' | wc -l
```

## Future Enhancements

- **WebSocket streaming** for real-time dashboards
- **Custom aggregations** for specific metrics
- **Integration with external monitoring systems**
- **Advanced filtering with regex support**
- **Export to various formats (CSV, XML, etc.)**

## See Also

- [Structured Logging Documentation](STRUCTURED_LOGGING.md)
- [CLI Reference](CLI_HELP.md)
- [Event Bus Documentation](EVENT_BUS.md)
- [Examples](../examples/structured_logging_demo.rs) 