#!/bin/bash

# CI Setup Validation Script
# Validates that all optimization features are properly configured

set -euo pipefail

echo "🔍 Validating CI Setup..."
echo "========================="

# Check required files
echo "📁 Checking required files..."
required_files=(
    ".github/workflows/ci.yml"
    "scripts/optimize-build.sh"
    "scripts/ci-build.sh"
    "scripts/dev-cycle.sh"
    ".cargo/config.toml"
)

for file in "${required_files[@]}"; do
    if [[ -f "$file" ]]; then
        echo "  ✅ $file"
    else
        echo "  ❌ $file (missing)"
        exit 1
    fi
done

# Check CI workflow features
echo "🔧 Checking CI workflow features..."
if grep -q "sccache" ".github/workflows/ci.yml"; then
    echo "  ✅ sccache configured"
else
    echo "  ❌ sccache not configured"
    exit 1
fi

if grep -q "cargo-hakari" ".github/workflows/ci.yml"; then
    echo "  ✅ cargo-hakari configured"
else
    echo "  ❌ cargo-hakari not configured"
    exit 1
fi

if grep -q "cargo-nextest" ".github/workflows/ci.yml"; then
    echo "  ✅ cargo-nextest configured"
else
    echo "  ❌ cargo-nextest not configured"
    exit 1
fi

if grep -q "performance" ".github/workflows/ci.yml"; then
    echo "  ✅ performance job configured"
else
    echo "  ❌ performance job not configured"
    exit 1
fi

# Check cache configuration
echo "💾 Checking cache configuration..."
if grep -q "SCCACHE_DIR" ".github/workflows/ci.yml"; then
    echo "  ✅ sccache cache configured"
else
    echo "  ❌ sccache cache not configured"
    exit 1
fi

if grep -q ".cargo/hakari/" ".github/workflows/ci.yml"; then
    echo "  ✅ hakari cache configured"
else
    echo "  ❌ hakari cache not configured"
    exit 1
fi

echo "✅ CI setup validation passed!"
echo "🚀 Ready for optimized CI builds"
