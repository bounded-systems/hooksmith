#!/usr/bin/env bash
set -euo pipefail

# Auto-detect platform using rustc
TARGET="$(rustc -vV | grep host | awk '{print $2}')"

echo "🔧 Building xtask for platform: $TARGET"

# Use conditional target based on platform
if [[ "$TARGET" == "aarch64-apple-darwin" ]]; then
    echo "📱 Detected Apple Silicon Mac - using native target"
    cargo build -p xtask --target aarch64-apple-darwin "$@"
elif [[ "$TARGET" == "x86_64-apple-darwin" ]]; then
    echo "🖥️  Detected Intel Mac - using native target"
    cargo build -p xtask --target x86_64-apple-darwin "$@"
elif [[ "$TARGET" == "x86_64-unknown-linux-gnu" ]]; then
    echo "🐧 Detected Linux x86_64 - using native target"
    cargo build -p xtask "$@"
elif [[ "$TARGET" == "aarch64-unknown-linux-gnu" ]]; then
    echo "🐧 Detected Linux ARM64 - using native target"
    cargo build -p xtask "$@"
else
    echo "🖥️  Using default target for platform: $TARGET"
    cargo build -p xtask "$@"
fi

echo "✅ xtask build completed successfully!" 
