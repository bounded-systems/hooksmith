#!/bin/bash
# Script to show statistics about hooksmith-events.jsonl

LOG_FILE="hooksmith-events.jsonl"

echo "📊 Hooksmith Event Log Statistics"
echo "================================="

if [ ! -f "$LOG_FILE" ]; then
    echo "❌ No log file found at $LOG_FILE"
    exit 1
fi

# Basic file info
file_size=$(du -h "$LOG_FILE" | cut -f1)
file_age=$(echo $(( ($(date +%s) - $(stat -f %m "$LOG_FILE")) / 3600 )) hours ago)
line_count=$(wc -l < "$LOG_FILE")

echo "📁 File: $LOG_FILE"
echo "📏 Size: $file_size"
echo "⏰ Age: $file_age"
echo "📝 Lines: $line_count"

echo ""
echo "📈 Event Statistics:"
echo "-------------------"

# Count by severity
echo "🔴 Errors: $(grep -c '"severity":"Error"' "$LOG_FILE" 2>/dev/null || echo "0")"
echo "🟡 Warnings: $(grep -c '"severity":"Warning"' "$LOG_FILE" 2>/dev/null || echo "0")"
echo "🟢 Info: $(grep -c '"severity":"Info"' "$LOG_FILE" 2>/dev/null || echo "0")"

echo ""
echo "🏷️  Event Types:"
echo "---------------"

# Count by event type
grep -o '"event_type":"[^"]*"' "$LOG_FILE" | sort | uniq -c | sort -nr | while read count type; do
    event_type=$(echo "$type" | sed 's/"event_type":"//' | sed 's/"//')
    printf "  %-20s %s\n" "$event_type" "$count"
done

echo ""
echo "📅 Recent Activity:"
echo "------------------"

# Show recent events (last 5)
echo "Last 5 events:"
tail -5 "$LOG_FILE" | while read line; do
    timestamp=$(echo "$line" | grep -o '"timestamp":"[^"]*"' | sed 's/"timestamp":"//' | sed 's/"//')
    event_type=$(echo "$line" | grep -o '"event_type":"[^"]*"' | sed 's/"event_type":"//' | sed 's/"//')
    severity=$(echo "$line" | grep -o '"severity":"[^"]*"' | sed 's/"severity":"//' | sed 's/"//')
    printf "  %s | %-15s | %s\n" "$timestamp" "$event_type" "$severity"
done

echo ""
echo "💡 Tips:"
echo "  • Run './scripts/cleanup-logs.sh' to rotate and clean up old logs"
echo "  • Run './scripts/check-errors.sh' to see only error events"
echo "  • Run './scripts/monitor-errors.sh' to monitor errors in real-time" 