#!/bin/bash

# Hooksmith Root Directory Structure Migration
# This script reorganizes the root directory to be cleaner and more organized

set -e

echo "🔄 Starting Hooksmith directory structure migration..."

# 1. Move all summary/implementation/complete docs to docs/summaries/
echo "📁 Moving summary documents to docs/summaries/"

mv *_SUMMARY.md docs/summaries/ 2>/dev/null || true
mv *_COMPLETE.md docs/summaries/ 2>/dev/null || true
mv *_IMPLEMENTATION.md docs/summaries/ 2>/dev/null || true

# 2. Move design and architecture docs to docs/design/
echo "📁 Moving design documents to docs/design/"

mv ARCHITECTURE.md docs/design/ 2>/dev/null || true
mv PROJECT_STRUCTURE.md docs/design/ 2>/dev/null || true
mv WORKSPACE_LAYOUT.md docs/design/ 2>/dev/null || true
mv CONTRACT_WORKFLOW_DESIGN.md docs/design/ 2>/dev/null || true
mv STATUS_SYSTEM_DESIGN.md docs/design/ 2>/dev/null || true
mv SOURCE_BASED_DOCS.md docs/design/ 2>/dev/null || true
mv README_STRUCTURE.md docs/design/ 2>/dev/null || true
mv STRUCTURE.md docs/design/ 2>/dev/null || true

# 3. Move guides to docs/guides/
echo "📁 Moving guides to docs/guides/"

mv CONTRIBUTING.md docs/guides/ 2>/dev/null || true
mv QUICKSTART_OPTIMIZED.md docs/guides/ 2>/dev/null || true
mv SECURITY_GUIDE.md docs/guides/ 2>/dev/null || true
mv XTASK_COMMANDS.md docs/guides/ 2>/dev/null || true
mv CARGO_BEST_PRACTICES.md docs/guides/ 2>/dev/null || true
mv CHANGELOG.md docs/guides/ 2>/dev/null || true

# 4. Move test/demo .rs files to examples/
echo "📁 Moving test/demo files to examples/"

mv test_*.rs examples/ 2>/dev/null || true
mv fix_format.rs examples/ 2>/dev/null || true
mv bootstrap.rs examples/ 2>/dev/null || true

# 5. Move generated artifacts to gen/
echo "📁 Moving generated artifacts to gen/"

mv hooksmith-events.jsonl gen/ 2>/dev/null || true
mv status-badge.json gen/ 2>/dev/null || true
mv status-badge.jsonc gen/ 2>/dev/null || true

# 6. Move config files to config/
echo "📁 Moving config files to config/"

mv lefthook-example.yml config/ 2>/dev/null || true
mv .envrc config/ 2>/dev/null || true
mv .editorconfig config/ 2>/dev/null || true

# 7. Move components and xtask to crates/
echo "📁 Moving crates to crates/"

mv components crates/ 2>/dev/null || true
mv xtask crates/ 2>/dev/null || true
mv lefthook-rs crates/ 2>/dev/null || true

# 8. Move diagrams to docs/
echo "📁 Moving diagrams to docs/"

mv diagrams docs/ 2>/dev/null || true

# 9. Move remaining misc docs to docs/
echo "📁 Moving remaining documentation to docs/"

mv *.md docs/ 2>/dev/null || true

# 10. Clean up any remaining files that should be moved
echo "🧹 Cleaning up remaining files..."

# Move any remaining .md files that weren't caught
find . -maxdepth 1 -name "*.md" -not -path "./README.md" -exec mv {} docs/ \; 2>/dev/null || true

# Move any remaining .yml files
find . -maxdepth 1 -name "*.yml" -not -path "./lefthook.yml" -exec mv {} config/ \; 2>/dev/null || true

# Move any remaining .json/.jsonc files
find . -maxdepth 1 -name "*.json" -not -path "./Cargo.toml" -exec mv {} gen/ \; 2>/dev/null || true
find . -maxdepth 1 -name "*.jsonc" -exec mv {} gen/ \; 2>/dev/null || true

echo "✅ Migration completed!"
echo ""
echo "📂 New structure:"
echo "├── Cargo.toml"
echo "├── Cargo.lock"
echo "├── README.md"
echo "├── CODEOWNERS"
echo "├── lefthook.yml"
echo "├── rust-toolchain.toml"
echo "├── rustfmt.toml"
echo "├── clippy.toml"
echo "├── deny.toml"
echo "├── crates/           # components, xtask, lefthook-rs"
echo "├── config/           # configs & schemas"
echo "├── docs/             # all docs + summaries + diagrams"
echo "├── examples/         # example code"
echo "├── gen/              # generated outputs"
echo "├── scripts/          # dev scripts"
echo "├── src/"
echo "├── tests/            # integration tests"
echo "└── target/"
echo ""
echo "🔍 Next steps:"
echo "1. Review the new structure"
echo "2. Update any hardcoded paths in your code"
echo "3. Update Cargo.toml workspace paths if needed"
echo "4. Test that everything still builds and works" 