#!/bin/bash
# enforce_structured_logging.sh
# CI script to enforce structured logging and prevent direct console output

set -e

echo "🔍 Enforcing structured logging rules..."

# Check for direct println! usage (excluding emit macros)
echo "📋 Checking for direct println! usage..."

# Find all println! calls that are not in emit macros
VIOLATIONS=$(grep -r "println!" --include="*.rs" . | grep -v "emit_" | grep -v "//.*println!" | grep -v ".*println!.*//" || true)

if [ -n "$VIOLATIONS" ]; then
    echo "❌ Found direct println! usage (not allowed):"
    echo "$VIOLATIONS"
    echo ""
    echo "💡 Use structured logging macros instead:"
    echo "   - emit_info!(\"tool\", \"action\", \"message\")"
    echo "   - emit_warn!(\"tool\", \"action\", \"message\")"
    echo "   - emit_error!(\"tool\", \"action\", \"message\")"
    echo "   - emit_success!(\"tool\", \"action\", \"message\")"
    echo "   - emit_failure!(\"tool\", \"action\", \"message\")"
    echo "   - emit_warning!(\"tool\", \"action\", \"message\")"
    echo ""
    echo "📚 See docs/STRUCTURED_LOGGING_MIGRATION.md for migration guide"
    exit 1
fi

# Check for direct eprintln! usage
echo "📋 Checking for direct eprintln! usage..."

EPRINTL_VIOLATIONS=$(grep -r "eprintln!" --include="*.rs" . | grep -v "emit_" | grep -v "//.*eprintln!" | grep -v ".*eprintln!.*//" || true)

if [ -n "$EPRINTL_VIOLATIONS" ]; then
    echo "❌ Found direct eprintln! usage (not allowed):"
    echo "$EPRINTL_VIOLATIONS"
    echo ""
    echo "💡 Use structured logging macros instead:"
    echo "   - emit_error!(\"tool\", \"action\", \"message\")"
    echo "   - emit_warning!(\"tool\", \"action\", \"message\")"
    echo ""
    echo "📚 See docs/STRUCTURED_LOGGING_MIGRATION.md for migration guide"
    exit 1
fi

# Check for direct print! usage
echo "📋 Checking for direct print! usage..."

PRINT_VIOLATIONS=$(grep -r "print!" --include="*.rs" . | grep -v "emit_" | grep -v "//.*print!" | grep -v ".*print!.*//" || true)

if [ -n "$PRINT_VIOLATIONS" ]; then
    echo "❌ Found direct print! usage (not allowed):"
    echo "$PRINT_VIOLATIONS"
    echo ""
    echo "💡 Use structured logging macros instead:"
    echo "   - emit_info!(\"tool\", \"action\", \"message\")"
    echo ""
    echo "📚 See docs/STRUCTURED_LOGGING_MIGRATION.md for migration guide"
    exit 1
fi

# Check for direct eprint! usage
echo "📋 Checking for direct eprint! usage..."

EPRINT_VIOLATIONS=$(grep -r "eprint!" --include="*.rs" . | grep -v "emit_" | grep -v "//.*eprint!" | grep -v ".*eprint!.*//" || true)

if [ -n "$EPRINT_VIOLATIONS" ]; then
    echo "❌ Found direct eprint! usage (not allowed):"
    echo "$EPRINT_VIOLATIONS"
    echo ""
    echo "💡 Use structured logging macros instead:"
    echo "   - emit_error!(\"tool\", \"action\", \"message\")"
    echo ""
    echo "📚 See docs/STRUCTURED_LOGGING_MIGRATION.md for migration guide"
    exit 1
fi

# Verify schema generation works
echo "📋 Verifying schema generation..."

if ! cargo run -p xtask -- gen-schema > /dev/null 2>&1; then
    echo "❌ Schema generation failed"
    exit 1
fi

# Test schema validation
echo "📋 Testing schema validation..."

# Create a test event
TEST_OUTPUT=$(cargo run -p xtask -- gen-lefthook --output test.yml --validate 2>/dev/null | head -1)

if [ -z "$TEST_OUTPUT" ]; then
    echo "❌ No test output generated"
    exit 1
fi

# Validate the test output
if ! echo "$TEST_OUTPUT" | cargo run -p xtask -- validate-schema --strict > /dev/null 2>&1; then
    echo "❌ Schema validation failed"
    echo "Test output: $TEST_OUTPUT"
    exit 1
fi

echo "✅ All structured logging rules passed!"
echo ""
echo "🎯 Summary:"
echo "   - No direct println! usage found"
echo "   - No direct eprintln! usage found"
echo "   - No direct print! usage found"
echo "   - No direct eprint! usage found"
echo "   - Schema generation working"
echo "   - Schema validation working"
echo ""
echo "🚀 All output is schema-validated JSONL!" 
