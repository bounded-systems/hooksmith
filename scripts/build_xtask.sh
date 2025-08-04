#!/usr/bin/env bash
set -euo pipefail

# Auto-detect platform
TARGET="$(rustc -vV | grep host | awk '{print $2}')"

echo "🔧 Building xtask for platform: $TARGET"

if [[ "$TARGET" == "aarch64-apple-darwin" ]]; then
    echo "📱 Detected Apple Silicon Mac - using native target"
    cargo build -p xtask --target aarch64-apple-darwin "$@"
else
    echo "🖥️  Using default target for platform: $TARGET"
    cargo build -p xtask "$@"
fi

echo "✅ xtask build completed successfully!" 
