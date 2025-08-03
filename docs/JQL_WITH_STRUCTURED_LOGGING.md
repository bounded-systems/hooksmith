# Using jql with Structured Logging

## Overview

`jql` is a powerful command-line tool for querying and manipulating JSON data. When combined with our schema-validated structured logging system, it provides an incredibly powerful way to analyze, filter, and transform our JSONL output.

## Key Features

- **Token-based queries** - Simple, intuitive syntax
- **JSONL support** - Native support for streaming JSON (`-s` flag)
- **Array operations** - Select, slice, and transform arrays
- **Object operations** - Extract keys, filter objects
- **Parallel processing** - Apply queries to array elements (`|>` operator)
- **Flattening** - Expand nested structures (`..` operator)

## Installation

```bash
# Install jql
cargo install jql

# Verify installation
jql --version
```

## Basic Usage with Structured Logging

### 1. Extract Specific Fields

```bash
# Extract just the log levels
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"level"'

# Output:
"warn"
"info"
"info"

# Extract tool and action
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"tool","action"'

# Output:
[
  "hooksmith",
  "lefthook"
]
[
  "hooksmith",
  "lefthook"
]
[
  "hooksmith",
  "lefthook"
]
```

### 2. Extract Messages

```bash
# Get all log messages
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"message"'

# Output:
"⚠️ Lefthook generation disabled - lefthook_rs dependency missing"
"Output: test.yml"
"Validate: true"
```

### 3. Extract Timestamps

```bash
# Get timestamps
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"timestamp"'

# Output:
1754249000
1754249000
1754249000
```

## Advanced Querying

### 1. Multiple Field Selection

```bash
# Extract level, tool, and message in one query
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"level","tool","message"'

# Output:
[
  "warn",
  "hooksmith",
  "⚠️ Lefthook generation disabled - lefthook_rs dependency missing"
]
[
  "info",
  "hooksmith",
  "Output: test.yml"
]
[
  "info",
  "hooksmith",
  "Validate: true"
]
```

### 2. Filtering by Level

```bash
# Get only warning messages
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"level"=="warn"'

# Get only info messages
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"level"=="info"'
```

### 3. Session Correlation

```bash
# Extract session ID for correlation
cargo run -p xtask -- gen-lefthook --output test.yml --validate | jql -s '"sessionId"'

# Output:
"7c8ab370-2b7b-4504-858e-e7cf34cec40b"
"7c8ab370-2b7b-4504-858e-e7cf34cec40b"
"7c8ab370-2b7b-4504-858e-e7cf34cec40b"
```

## Real-World Examples

### 1. Error Analysis

```bash
# Filter for error-level events
cargo run -p xtask -- structured-auto-push | jql -s '"level"=="error"'

# Get error messages with details
cargo run -p xtask -- structured-auto-push | jql -s '"level"=="error" | "message","details"'
```

### 2. Performance Monitoring

```bash
# Extract timing information from events with details
cargo run -p xtask -- structured-auto-push | jql -s '"details"."duration_ms"'

# Get build times
cargo run -p xtask -- structured-auto-push | jql -s '"action"=="build" | "details"."duration_ms"'
```

### 3. Tool Usage Statistics

```bash
# Count events by tool
cargo run -p xtask -- structured-auto-push | jql -s '"tool"' | sort | uniq -c

# Count events by action
cargo run -p xtask -- structured-auto-push | jql -s '"action"' | sort | uniq -c
```

### 4. Diagnostic Information

```bash
# Extract file paths from diagnostic events
cargo run -p xtask -- structured-auto-push | jql -s '"file"'

# Get line numbers for errors
cargo run -p xtask -- structured-auto-push | jql -s '"line"'

# Get error codes
cargo run -p xtask -- structured-auto-push | jql -s '"code"'
```

## Advanced jql Features

### 1. Array Operations

```bash
# If we had an array of events in a single JSON object
echo '{"events":[{"level":"info","message":"test"}]}' | jql '"events"[0]."message"'

# Slice arrays
echo '{"events":[{"level":"info"},{"level":"warn"},{"level":"error"}]}' | jql '"events"[0:2]."level"'
```

### 2. Flattening Nested Structures

```bash
# Flatten nested objects (useful for complex details)
echo '{"details":{"nested":{"value":42}}}' | jql '.."value"'
```

### 3. Parallel Processing

```bash
# Apply query to each array element
echo '[{"level":"info"},{"level":"warn"}]' | jql '|>"level"'
```

### 4. Object Keys

```bash
# Get all keys from an object
echo '{"level":"info","tool":"hooksmith","action":"test"}' | jql '@'
```

## Integration with Other Tools

### 1. Combine with grep for filtering

```bash
# Filter for specific messages
cargo run -p xtask -- structured-auto-push | jql -s '"message"' | grep "error"

# Filter for specific tools
cargo run -p xtask -- structured-auto-push | jql -s '"tool"' | grep "cargo"
```

### 2. Combine with wc for counting

```bash
# Count total events
cargo run -p xtask -- structured-auto-push | jql -s '"level"' | wc -l

# Count error events
cargo run -p xtask -- structured-auto-push | jql -s '"level"=="error"' | wc -l
```

### 3. Combine with sort and uniq

```bash
# Get unique tools used
cargo run -p xtask -- structured-auto-push | jql -s '"tool"' | sort | uniq

# Get unique actions performed
cargo run -p xtask -- structured-auto-push | jql -s '"action"' | sort | uniq
```

## Schema-Aware Queries

Since our structured logging uses a consistent schema, we can create reusable queries:

### 1. Event Summary

```bash
# Create a summary of all events
cargo run -p xtask -- structured-auto-push | jql -s '"timestamp","tool","action","level","message"'
```

### 2. Error Report

```bash
# Generate an error report
cargo run -p xtask -- structured-auto-push | jql -s '"level"=="error" | "timestamp","tool","action","message","file","line"'
```

### 3. Performance Report

```bash
# Extract performance data
cargo run -p xtask -- structured-auto-push | jql -s '"details"."duration_ms"' | grep -v null
```

## Best Practices

### 1. Use the `-s` flag for JSONL

Always use `-s` when processing our structured logging output since it's JSONL format.

### 2. Quote field names

Always quote field names in jql queries: `'"field_name"'`

### 3. Use arrays for multiple fields

When selecting multiple fields, use array syntax: `'"field1","field2"'`

### 4. Combine with other tools

jql works great with standard Unix tools like `grep`, `sort`, `uniq`, `wc`, etc.

### 5. Create reusable queries

Save common queries in files and use `-q <FILE>` option:

```bash
# error_report.jql
'"level"=="error" | "timestamp","tool","action","message"'

# Usage
cargo run -p xtask -- structured-auto-push | jql -s -q error_report.jql
```

## Comparison with jq

jql provides a simpler, more intuitive syntax compared to jq:

| jq | jql | Description |
|----|-----|-------------|
| `.level` | `'"level"'` | Extract field |
| `.tool, .action` | `'"tool","action"'` | Multiple fields |
| `.[0:2]` | `'[0:2]'` | Array slice |
| `map(.level)` | `'|>"level"'` | Parallel processing |
| `keys` | `'@'` | Get object keys |

## Conclusion

jql is an excellent tool for working with our structured logging output. Its simple syntax, JSONL support, and powerful querying capabilities make it perfect for:

- **Real-time analysis** of structured logging output
- **Error monitoring** and alerting
- **Performance analysis** and optimization
- **Debugging** and troubleshooting
- **Reporting** and metrics collection

The combination of schema-validated JSONL output and jql's querying capabilities provides a powerful foundation for observability and analysis in our development workflow. 
