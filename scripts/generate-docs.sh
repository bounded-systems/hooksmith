#!/bin/bash

# Generate Comprehensive Documentation
# This script creates various types of documentation for the repository

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📚 Generating comprehensive documentation...${NC}"

# Create docs directory if it doesn't exist
mkdir -p docs

# 1. Generate API documentation
echo -e "${BLUE}📖 Generating API documentation...${NC}"
cargo doc --no-deps --document-private-items
echo -e "${GREEN}✅ API documentation generated in target/doc/${NC}"

# 2. Generate repository structure
echo -e "${BLUE}📁 Generating repository structure...${NC}"
if [ -f "scripts/generate-structure.sh" ]; then
    ./scripts/generate-structure.sh
    cp STRUCTURE.md docs/STRUCTURE.md
    echo -e "${GREEN}✅ Repository structure saved to docs/STRUCTURE.md${NC}"
else
    echo -e "${YELLOW}⚠️  Structure generation script not found${NC}"
fi

# 3. Generate CLI help documentation
echo -e "${BLUE}🖥️  Generating CLI help documentation...${NC}"
{
    echo "# CLI Help Documentation"
    echo ""
    echo "This document contains the help output for all CLI commands."
    echo ""
    echo "## Main Help"
    echo '```'
    cargo run -- --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "## Command Help"
    echo ""
    echo "### Test Command"
    echo '```'
    cargo run -- test --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### Build Command"
    echo '```'
    cargo run -- build --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### Generate Command"
    echo '```'
    cargo run -- generate --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### Install Command"
    echo '```'
    cargo run -- install --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### List Command"
    echo '```'
    cargo run -- list --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### WASM Commands"
    echo '```'
    cargo run -- wasm --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### WASM Build Command"
    echo '```'
    cargo run -- wasm build --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### WASM Run Command"
    echo '```'
    cargo run -- wasm run --help || echo "Help command failed"
    echo '```'
    echo ""
    echo "### WASM Bindings Command"
    echo '```'
    cargo run -- wasm bindings --help || echo "Help command failed"
    echo '```'
} > docs/CLI_HELP.md
echo -e "${GREEN}✅ CLI help documentation saved to docs/CLI_HELP.md${NC}"

# 4. Generate test summary
echo -e "${BLUE}🧪 Generating test summary...${NC}"
{
    echo "# Test Summary"
    echo ""
    echo "This document contains information about the test suite."
    echo ""
    echo "## Test Results"
    echo '```'
    cargo test --quiet --no-fail-fast 2>&1 | grep -E "(running|test result|passed|failed)" || true
    echo '```'
    echo ""
    echo "## Test Files"
    echo ""
    echo "- \`tests/integration.rs\`: Integration tests for CLI functionality"
    echo "- \`tests/hooks_test.rs\`: Unit tests for hook functionality"
    echo ""
    echo "## Running Tests"
    echo ""
    echo "```bash"
    echo "# Run all tests"
    echo "cargo test"
    echo ""
    echo "# Run specific test file"
    echo "cargo test --test integration"
    echo "cargo test --test hooks_test"
    echo ""
    echo "# Run specific test"
    echo "cargo test test_cli_help"
    echo "```"
} > docs/TEST_SUMMARY.md
echo -e "${GREEN}✅ Test summary saved to docs/TEST_SUMMARY.md${NC}"

# 5. Generate development guide
echo -e "${BLUE}🛠️  Generating development guide...${NC}"
{
    echo "# Development Guide"
    echo ""
    echo "This guide provides information for developers working on Hooksmith."
    echo ""
    echo "## Prerequisites"
    echo ""
    echo "- Rust (latest stable)"
    echo "- Git"
    echo "- Bash (for build scripts)"
    echo ""
    echo "## Development Workflow"
    echo ""
    echo "### 1. Build the Project"
    echo "```bash"
    echo "./build.sh"
    echo "```"
    echo ""
    echo "### 2. Run Tests"
    echo "```bash"
    echo "cargo test"
    echo "```"
    echo ""
    echo "### 3. Generate Documentation"
    echo "```bash"
    echo "./scripts/generate-docs.sh"
    echo "```"
    echo ""
    echo "### 4. Run CLI Commands"
    echo "```bash"
    echo "cargo run -- test"
    echo "cargo run -- --help"
    echo "```"
    echo ""
    echo "## Project Structure"
    echo ""
    echo "- \`src/main.rs\`: Main CLI application"
    echo "- \`src/lib.rs\`: Library exports"
    echo "- \`components/cli-core/\`: Core CLI functionality"
    echo "- \`tests/\`: Test files"
    echo "- \`scripts/\`: Build and documentation scripts"
    echo ""
    echo "## Code Style"
    echo ""
    echo "This project uses Trunk for code quality:"
    echo "```bash"
    echo "trunk check"
    echo "```"
    echo ""
    echo "## Documentation"
    echo ""
    echo "- API docs: \`cargo doc --no-deps --open\`"
    echo "- CLI help: \`cargo run -- --help\`"
    echo "- Project docs: \`docs/\` directory"
} > docs/DEVELOPMENT.md
echo -e "${GREEN}✅ Development guide saved to docs/DEVELOPMENT.md${NC}"

# 6. Create documentation index
echo -e "${BLUE}📋 Creating documentation index...${NC}"
{
    echo "# Documentation Index"
    echo ""
    echo "This is the main documentation index for the Hooksmith project."
    echo ""
    echo "## 📚 Available Documentation"
    echo ""
    echo "### Core Documentation"
    echo "- [README.md](../README.md) - Main project documentation"
    echo "- [STRUCTURE.md](STRUCTURE.md) - Repository structure overview"
    echo "- [CLI_HELP.md](CLI_HELP.md) - CLI command help documentation"
    echo ""
    echo "### Development Documentation"
    echo "- [DEVELOPMENT.md](DEVELOPMENT.md) - Development guide"
    echo "- [TEST_SUMMARY.md](TEST_SUMMARY.md) - Test suite information"
    echo ""
    echo "### API Documentation"
    echo "- [API Docs](../target/doc/hooksmith/index.html) - Generated API documentation"
    echo ""
    echo "## 🚀 Quick Start"
    echo ""
    echo "1. Read the [README.md](../README.md) for project overview"
    echo "2. Check [STRUCTURE.md](STRUCTURE.md) for file organization"
    echo "3. Use [CLI_HELP.md](CLI_HELP.md) for command reference"
    echo "4. Follow [DEVELOPMENT.md](DEVELOPMENT.md) for development setup"
    echo ""
    echo "## 📊 Documentation Status"
    echo ""
    echo "- ✅ **Complete**: CLI structure, API docs, test documentation"
    echo "- ✅ **Complete**: Repository structure, development guide"
    echo "- 🚧 **In Progress**: Implementation examples, tutorials"
    echo ""
    echo "---"
    echo ""
    echo "*Generated on $(date) by generate-docs.sh*"
} > docs/README.md
echo -e "${GREEN}✅ Documentation index saved to docs/README.md${NC}"

echo -e "${GREEN}✅ All documentation generated successfully!${NC}"
echo -e "${BLUE}📁 Documentation files created in docs/ directory${NC}"
echo -e "${BLUE}📖 API documentation available at target/doc/hooksmith/index.html${NC}" 
