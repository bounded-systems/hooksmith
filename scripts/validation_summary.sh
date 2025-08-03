#!/bin/bash
# validation_summary.sh
# Generate comprehensive validation summary from structured logging output

set -e

echo "📊 Hooksmith Validation Summary"
echo "==============================="

# Check if jql is installed
if ! command -v jql &> /dev/null; then
    echo "❌ jql not found. Installing..."
    cargo install jql
fi

# Create temporary file for events
EVENTS_FILE=$(mktemp)
trap "rm -f $EVENTS_FILE" EXIT

echo "🔄 Running structured auto-push and collecting events..."
echo ""

# Run structured auto-push and capture output
cargo run -p xtask -- structured-auto-push > "$EVENTS_FILE" 2>/dev/null || true

echo "📈 Generating summary..."
echo ""

# Basic statistics
TOTAL_EVENTS=$(jql '.' "$EVENTS_FILE" | wc -l)
ERROR_EVENTS=$(jql '.level=="error"' "$EVENTS_FILE" | wc -l)
WARN_EVENTS=$(jql '.level=="warn"' "$EVENTS_FILE" | wc -l)
INFO_EVENTS=$(jql '.level=="info"' "$EVENTS_FILE" | wc -l)
SUCCESS_EVENTS=$(jql '.level=="info" and .action=="completion"' "$EVENTS_FILE" | wc -l)

echo "=== 📊 Event Statistics ==="
echo "Total events:     $TOTAL_EVENTS"
echo "Info events:      $INFO_EVENTS"
echo "Warning events:   $WARN_EVENTS"
echo "Error events:     $ERROR_EVENTS"
echo "Success events:   $SUCCESS_EVENTS"
echo ""

# Tool breakdown
echo "=== 🛠️  Tool Breakdown ==="
jql '.tool' "$EVENTS_FILE" | sort | uniq -c | while read count tool; do
    echo "$tool: $count events"
done
echo ""

# Action breakdown
echo "=== ⚡ Action Breakdown ==="
jql '.action' "$EVENTS_FILE" | sort | uniq -c | while read count action; do
    echo "$action: $count events"
done
echo ""

# Performance metrics
echo "=== ⏱️  Performance Metrics ==="
DURATION=$(jql '.action=="completion" | .details.duration_ms' "$EVENTS_FILE" | head -1)
if [ -n "$DURATION" ]; then
    echo "Total duration: ${DURATION}ms"
else
    echo "Duration: Not available (no completion event found)"
fi
echo ""

# Recent errors
echo "=== ❌ Recent Errors ==="
ERROR_COUNT=$(jql '.level=="error"' "$EVENTS_FILE" | wc -l)
if [ "$ERROR_COUNT" -gt 0 ]; then
    jql '.level=="error" | {timestamp, tool, action, message}' "$EVENTS_FILE" | tail -5
else
    echo "No errors found! 🎉"
fi
echo ""

# Recent warnings
echo "=== ⚠️  Recent Warnings ==="
WARN_COUNT=$(jql '.level=="warn"' "$EVENTS_FILE" | wc -l)
if [ "$WARN_COUNT" -gt 0 ]; then
    jql '.level=="warn" | {timestamp, tool, action, message}' "$EVENTS_FILE" | tail -5
else
    echo "No warnings found! 🎉"
fi
echo ""

# Diagnostic information
echo "=== 🔍 Diagnostic Information ==="
DIAGNOSTIC_COUNT=$(jql 'select(.code)' "$EVENTS_FILE" | wc -l)
if [ "$DIAGNOSTIC_COUNT" -gt 0 ]; then
    echo "Found $DIAGNOSTIC_COUNT diagnostic events:"
    jql 'select(.code) | {file, line, code, message}' "$EVENTS_FILE" | head -3
else
    echo "No diagnostic events found"
fi
echo ""

# Session information
echo "=== 🆔 Session Information ==="
SESSION_COUNT=$(jql '.session_id' "$EVENTS_FILE" | sort | uniq | wc -l)
echo "Total sessions: $SESSION_COUNT"
if [ "$SESSION_COUNT" -gt 0 ]; then
    echo "Recent session IDs:"
    jql '.session_id' "$EVENTS_FILE" | sort | uniq | tail -3
fi
echo ""

# Summary
echo "=== 📋 Summary ==="
if [ "$ERROR_EVENTS" -eq 0 ]; then
    echo "✅ Validation completed successfully!"
    if [ "$WARN_EVENTS" -eq 0 ]; then
        echo "🎉 No errors or warnings found!"
    else
        echo "⚠️  $WARN_EVENTS warnings found (non-critical)"
    fi
else
    echo "❌ Validation failed with $ERROR_EVENTS errors"
    echo "⚠️  $WARN_EVENTS warnings also found"
fi

echo ""
echo "📁 Events saved to: $EVENTS_FILE"
echo "💡 Use 'jless $EVENTS_FILE' to browse events interactively"
echo "💡 Use 'jql \".level==\\\"error\\\"\" $EVENTS_FILE' to see all errors" 
