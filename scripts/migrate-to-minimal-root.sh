#!/bin/bash

# Migration script to achieve minimal root layout
# This script moves files from root to their proper locations

set -e

echo "🚀 Starting migration to minimal root layout..."

# Create necessary directories
mkdir -p docs/summaries
mkdir -p schemas
mkdir -p scripts

echo "📁 Moving summary and implementation docs to docs/summaries/"
# Move summary and implementation docs
git mv *_SUMMARY.md docs/summaries/ 2>/dev/null || echo "  No *_SUMMARY.md files found"
git mv *_IMPLEMENTATION_*.md docs/summaries/ 2>/dev/null || echo "  No *_IMPLEMENTATION_*.md files found"

echo "📁 Moving schemas and config files"
# Move schema and config files
git mv languages.yml schemas/ 2>/dev/null || echo "  languages.yml not found"
git mv lefthook.yml .github/ 2>/dev/null || echo "  lefthook.yml not found"
git mv agreement.json docs/ 2>/dev/null || echo "  agreement.json not found"

echo "📁 Moving generated and contract files"
# Move generated and contract files
git mv contract_snapshots docs/ 2>/dev/null || echo "  contract_snapshots not found"
git mv contracts docs/ 2>/dev/null || echo "  contracts not found"
git mv generated-sources crates/ 2>/dev/null || echo "  generated-sources not found"

echo "📁 Moving examples and test files"
# Move examples and test files
git mv examples crates/ 2>/dev/null || echo "  examples not found"
git mv test-* docs/test-files/ 2>/dev/null || echo "  No test-* files found"
git mv test_* docs/test-files/ 2>/dev/null || echo "  No test_* files found"

echo "📁 Moving scripts and tools"
# Move scripts and tools
git mv scripts crates/scripts-cli 2>/dev/null || echo "  scripts not found"
mkdir -p scripts
git mv crates/scripts-cli/scripts/* scripts/ 2>/dev/null || echo "  No scripts to move"

echo "📁 Moving hooks"
# Move hooks
git mv hooks crates/lefthook-rs 2>/dev/null || echo "  hooks not found"

echo "📁 Moving other files"
# Move other files
git mv src crates/ 2>/dev/null || echo "  src not found"
git mv wit crates/ 2>/dev/null || echo "  wit not found"
git mv worktree-lifecycle crates/ 2>/dev/null || echo "  worktree-lifecycle not found"

echo "📁 Moving config and schemas"
# Move config and schemas
git mv config crates/config-model 2>/dev/null || echo "  config not found"
git mv schemas crates/schemas 2>/dev/null || echo "  schemas not found"

echo "📁 Moving documentation"
# Move documentation
git mv docs crates/docs 2>/dev/null || echo "  docs not found"

echo "✅ Migration completed!"
echo ""
echo "📋 Next steps:"
echo "1. Review the changes: git status"
echo "2. Test the minimal root contract:"
echo "   cd standalone-auditor && cargo run -- HEAD ../contracts/object-names@root-minimal.json"
echo "3. Commit the changes: git commit -m 'Migrate to minimal root layout'"
echo ""
echo "🎯 Target root layout:"
echo "  - .gitignore"
echo "  - .gitattributes"
echo "  - .github/"
echo "  - Cargo.toml"
echo "  - README.md"
echo "  - LICENSE*"
echo "  - rust-toolchain.toml"
echo "  - deny.toml"
echo "  - clippy.toml"
echo "  - rustfmt.toml"
echo "  - crates/"
echo "  - docs/"
echo "  - scripts/"
echo "  - schemas/"
