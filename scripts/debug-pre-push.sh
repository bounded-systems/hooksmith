#!/bin/bash
set -euo pipefail

echo "🔍 [DEBUG] Pre-push hook debugging script"
echo "=========================================="

echo ""
echo "🔍 [DEBUG] Running comprehensive validation..."
cargo run -p xtask -- contract-validate validate --range HEAD~1..HEAD --verbose
echo "✅ [DEBUG] Comprehensive validation completed"

echo ""
echo "📋 [DEBUG] Validating generated file headers..."
cargo run -p xtask -- validate-headers --strict --verbose
echo "✅ [DEBUG] Header validation completed"

echo ""
echo "🔒 [DEBUG] Running security audit..."
cargo audit --verbose
echo "✅ [DEBUG] Security audit completed"

echo ""
echo "🚫 [DEBUG] Running security deny check..."
cargo deny check --verbose
echo "✅ [DEBUG] Security deny check completed"

echo ""
echo "✅ [DEBUG] All pre-push checks completed successfully!" 
