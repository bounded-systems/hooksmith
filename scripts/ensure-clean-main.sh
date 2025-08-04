#!/bin/bash

# Hooksmith Worktree Management - Comprehensive workflow for managing worktrees
# This script provides commands to clean main, commit, and push worktrees

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Get the command from arguments, default to cleanup
COMMAND="${1:-cleanup}"

echo "🧹 Hooksmith Worktree Management"
echo "================================"

# Run the Rust script with the specified command
cargo run --manifest-path "$REPO_ROOT/scripts/ensure-clean-main/Cargo.toml" "$COMMAND"

echo ""
echo "✅ Workflow completed!" 
