#!/bin/bash
# install_logging_tools.sh
# Install Rust-based tools for working with structured logging

set -e

echo "🦀 Installing Rust-based Logging Tools"
echo "======================================"
echo ""

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust first:"
    echo "   https://rustup.rs/"
    exit 1
fi

echo "📦 Installing tools..."
echo ""

# Install core tools
echo "Installing jql (JSON Query Language)..."
cargo install jql
echo "✅ jql installed"

echo "Installing jless (JSON Viewer)..."
cargo install jless
echo "✅ jless installed"

echo "Installing fblog (JSON Log Tailer)..."
cargo install fblog
echo "✅ fblog installed"

echo "Installing lap (Log Analyzer Pro)..."
cargo install lap
echo "✅ lap installed"

echo ""
echo "🎉 All tools installed successfully!"
echo ""

# Verify installations
echo "🔍 Verifying installations..."
echo ""

TOOLS=("jql" "jless" "fblog" "lap")
for tool in "${TOOLS[@]}"; do
    if command -v "$tool" &> /dev/null; then
        echo "✅ $tool: $(which $tool)"
    else
        echo "❌ $tool: Not found"
    fi
done

echo ""
echo "📚 Usage Examples:"
echo "=================="
echo ""
echo "🔍 Basic filtering:"
echo "  jql '.level==\"error\"' events.jsonl"
echo "  jql '.tool==\"cargo\" and .action==\"clippy\"' events.jsonl"
echo ""
echo "👀 Interactive browsing:"
echo "  jless events.jsonl"
echo "  jql '.level==\"error\"' events.jsonl | jless"
echo ""
echo "📺 Real-time monitoring:"
echo "  cargo run -p xtask -- structured-auto-push | fblog -f 'level == \"error\"'"
echo "  cargo run -p xtask -- structured-auto-push | lap --follow"
echo ""
echo "🔄 Watchdog mode with monitoring:"
echo "  cargo run -p xtask -- structured-auto-push --watchdog | fblog -f 'level == \"error\"'"
echo ""
echo "📊 Generate summary:"
echo "  ./scripts/validation_summary.sh"
echo ""
echo "🔍 Monitor errors:"
echo "  ./scripts/monitor_errors.sh"
echo ""
echo "💡 Quick test:"
echo "  cargo run -p xtask -- structured-auto-push | head -5 | jql '.'"
echo ""
echo "📖 For more information, see:"
echo "  docs/STRUCTURED_LOGGING_TOOLS.md"
echo ""
echo "🚀 Happy logging!" 
