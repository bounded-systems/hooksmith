#!/usr/bin/env bash
set -euo pipefail

# Cross-compilation script for xtask
# Usage: ./scripts/build_xtask_cross.sh <target-triple> [additional-cargo-args]

if [ $# -lt 1 ]; then
    echo "Usage: $0 <target-triple> [additional-cargo-args]"
    echo "Example: $0 aarch64-apple-darwin --release"
    echo "Example: $0 x86_64-unknown-linux-gnu"
    exit 1
fi

TARGET="$1"
shift  # Remove target from arguments, leaving additional cargo args

echo "🔧 Cross-compiling xtask for target: $TARGET"

# Install target if not already installed
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "📦 Installing target: $TARGET"
    rustup target add "$TARGET"
fi

# Build for the specified target
echo "🏗️  Building xtask for $TARGET..."
cargo build -p xtask --target "$TARGET" "$@"

echo "✅ xtask cross-compilation completed successfully!"
echo "📁 Binary location: target/$TARGET/debug/xtask" 
