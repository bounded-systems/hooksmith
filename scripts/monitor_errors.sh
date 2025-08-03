#!/bin/bash
# monitor_errors.sh
# Real-time error monitoring for Hooksmith structured logging

set -e

echo "🔍 Hooksmith Error Monitor"
echo "=========================="
echo "Monitoring structured logging output for errors..."
echo "Press Ctrl+C to stop"
echo ""

# Check if jql is installed
if ! command -v jql &> /dev/null; then
    echo "❌ jql not found. Installing..."
    cargo install jql
fi

# Check if fblog is installed
if ! command -v fblog &> /dev/null; then
    echo "❌ fblog not found. Installing..."
    cargo install fblog
fi

echo "✅ Tools ready. Starting monitoring..."
echo ""

# Monitor errors in real-time with fblog
# Filter for errors and warnings, with custom output format
cargo run -p xtask -- structured-auto-push | \
  fblog -f 'level == "error" or level == "warn"' \
        -o '[{timestamp}] {level:upper} {tool}:{action} - {message}' \
        -d 