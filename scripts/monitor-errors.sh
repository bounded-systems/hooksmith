#!/bin/bash

# Monitor hooksmith events for errors in real-time
# Usage: ./scripts/monitor-errors.sh

echo "🔍 Monitoring hooksmith-events.jsonl for errors..."
echo "   Press Ctrl+C to stop"
echo ""

# Monitor the log file for error events
tail -f hooksmith-events.jsonl | while read line; do
    # Check if the line contains an error event
    if echo "$line" | grep -q '"severity":"Error"'; then
        # Extract and format the error information
        timestamp=$(echo "$line" | jq -r '.timestamp' 2>/dev/null)
        event_type=$(echo "$line" | jq -r '.event_type' 2>/dev/null)
        message=$(echo "$line" | jq -r '.message' 2>/dev/null)
        source=$(echo "$line" | jq -r '.source' 2>/dev/null)
        
        echo "🚨 ERROR DETECTED at $timestamp"
        echo "   Type: $event_type"
        echo "   Source: $source"
        echo "   Message: $message"
        echo "   Raw: $line"
        echo ""
    fi
done 
