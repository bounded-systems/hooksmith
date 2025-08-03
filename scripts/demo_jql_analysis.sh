#!/bin/bash
# demo_jql_analysis.sh
# Demonstrate jql analysis with structured logging output

set -e

echo "🔍 jql Analysis Demo with Structured Logging"
echo "============================================="
echo ""

# Ensure jql is available
if ! command -v jql &> /dev/null; then
    echo "❌ jql not found. Installing..."
    cargo install jql
fi

# Ensure PATH includes cargo bin
export PATH="$HOME/.cargo/bin:$PATH"

echo "📊 Generating sample structured logging output..."
echo ""

# Generate some structured logging output
echo "🎯 Sample 1: Basic Event Analysis"
echo "--------------------------------"
cargo run -p xtask -- gen-lefthook --output test.yml --validate | head -3 | jql -s '"level","tool","action"'
echo ""

echo "🎯 Sample 2: Message Extraction"
echo "-------------------------------"
cargo run -p xtask -- gen-lefthook --output test.yml --validate | head -3 | jql -s '"message"'
echo ""

echo "🎯 Sample 3: Session Correlation"
echo "--------------------------------"
cargo run -p xtask -- gen-lefthook --output test.yml --validate | head -3 | jql -s '"sessionId"' | head -1
echo ""

echo "🎯 Sample 4: Error Report Query"
echo "-------------------------------"
echo "Using: scripts/jql_queries/error_report.jql"
echo "Query: $(cat scripts/jql_queries/error_report.jql)"
echo ""

echo "🎯 Sample 5: Event Summary Query"
echo "--------------------------------"
echo "Using: scripts/jql_queries/event_summary.jql"
echo "Query: $(cat scripts/jql_queries/event_summary.jql)"
echo ""

echo "🎯 Sample 6: Performance Data Query"
echo "-----------------------------------"
echo "Using: scripts/jql_queries/performance_report.jql"
echo "Query: $(cat scripts/jql_queries/performance_report.jql)"
echo ""

echo "🎯 Sample 7: Warning Report Query"
echo "---------------------------------"
echo "Using: scripts/jql_queries/warning_report.jql"
echo "Query: $(cat scripts/jql_queries/warning_report.jql)"
echo ""

echo "🔧 Practical Usage Examples"
echo "=========================="
echo ""

echo "📈 Count events by level:"
echo "cargo run -p xtask -- gen-lefthook | jql -s '\"level\"' | sort | uniq -c"
echo ""

echo "📊 Count events by tool:"
echo "cargo run -p xtask -- gen-lefthook | jql -s '\"tool\"' | sort | uniq -c"
echo ""

echo "🔍 Filter for specific messages:"
echo "cargo run -p xtask -- gen-lefthook | jql -s '\"message\"' | grep 'error'"
echo ""

echo "⏱️ Extract timing data:"
echo "cargo run -p xtask -- structured-auto-push | jql -s '\"details\".\"duration_ms\"' | grep -v null"
echo ""

echo "🎯 Real-time monitoring:"
echo "cargo run -p xtask -- structured-auto-push | jql -s '\"level\"==\"error\" | \"message\"'"
echo ""

echo "✅ jql Analysis Demo Complete!"
echo ""
echo "💡 Key Benefits:"
echo "   - Simple, intuitive query syntax"
echo "   - Native JSONL support"
echo "   - Powerful filtering and transformation"
echo "   - Easy integration with Unix tools"
echo "   - Perfect for real-time analysis"
echo ""
echo "📚 For more examples, see: docs/JQL_WITH_STRUCTURED_LOGGING.md" 
