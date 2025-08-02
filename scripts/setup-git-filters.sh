#!/bin/bash
# Setup Git filters and diffs for contract validation

set -e

echo "🔧 Setting up Git filters and diffs for contract validation..."

# Set up the contract validation filter
echo "   Setting up contract_validate filter..."
git config filter.contract_validate.clean "./xtask.sh contract-validate clean"
git config filter.contract_validate.smudge "./xtask.sh contract-validate smudge"
git config filter.contract_validate.required true

# Set up the contract diff
echo "   Setting up contract_diff..."
git config diff.contract_diff.textconv "./xtask.sh contract-validate diff"
git config diff.contract_diff.cachetextconv true

echo "✅ Git filters and diffs configured successfully!"
echo ""
echo "📋 Configuration summary:"
echo "   Filter: contract_validate"
echo "   Diff: contract_diff"
echo ""
echo "🔍 To verify the configuration, run:"
echo "   git config --list | grep contract" 
