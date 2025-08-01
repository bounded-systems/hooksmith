#!/bin/bash

# Build script for pushd-worktree-cli workspace
# This script builds the CLI binary and all components

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔨 Building pushd-worktree-cli workspace${NC}"

# Build the CLI binary
echo -e "${BLUE}Building CLI binary...${NC}"
cargo build --release --package pushd-worktree-cli

# Build all components
echo -e "${BLUE}Building components...${NC}"

COMPONENTS=(
    "cli-core"
    "worktree-manager"
    "git-validator"
)

for component in "${COMPONENTS[@]}"; do
    echo -e "${BLUE}Building component: ${component}${NC}"
    cargo build --release --package "$component"
    echo -e "${GREEN}✓ Built ${component}${NC}"
done

# Run tests
echo -e "${BLUE}Running tests...${NC}"
cargo test --workspace

echo -e "${GREEN}✅ Build completed successfully!${NC}"

# Show build artifacts
echo -e "${BLUE}Build artifacts:${NC}"
echo "  CLI binary: target/release/pushd-worktree-cli"
for component in "${COMPONENTS[@]}"; do
    echo "  ${component}: target/release/lib${component//-/_}.rlib"
done

# Generate shell completions
echo -e "${BLUE}Generating shell completions...${NC}"
mkdir -p completions
cargo run --package pushd-worktree-cli -- generate --completion bash > completions/pushd-worktree-cli.bash 2>/dev/null || echo "Shell completion generation not implemented yet"

echo -e "${GREEN}✅ Build and test completed!${NC}" 
