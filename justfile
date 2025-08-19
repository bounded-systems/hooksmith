# Justfile for Hooksmith development
# Run with: just <recipe-name>

# Default recipe - show available commands
default:
    @echo "🔨 Hooksmith Development Commands"
    @echo "================================"
    @echo ""
    @echo "🦀 Rust Development:"
    @echo "  just build             - Build workspace (cargo)"
    @echo "  just test              - Run tests"
    @echo "  just check             - Quick check without building"
    @echo "  just clippy            - Run clippy lints"
    @echo "  just fmt               - Format code"
    @echo ""
    @echo "🔧 Xtask Commands:"
    @echo "  just xtask-build       - Build via xtask"
    @echo "  just xtask-docs        - Generate documentation"
    @echo "  just xtask-check       - Full xtask validation"
    @echo ""
    @echo "📦 Nix Commands:"
    @echo "  just nix-build         - Build all packages with Nix"
    @echo "  just nix-analyze       - Run quick analysis suite"
    @echo "  just nix-dev           - Enter Nix dev shell"
    @echo "  just nix-fmt           - Format Nix files"
    @echo ""
    @echo "🔍 Analysis Tools:"
    @echo "  just analyze-size      - Repository size analysis"
    @echo "  just analyze-blobs     - Rust blob analysis"
    @echo "  just analyze-delta     - Git delta analysis"
    @echo "  just analyze-churn     - File churn analysis"
    @echo "  just analyze-all       - Run all analysis tools"
    @echo ""
    @echo "🚀 Quick Start:"
    @echo "  just build && just analyze-size"

# Build the workspace with cargo
build:
    @echo "🦀 Building Hooksmith workspace..."
    cargo build --release

# Run tests
test:
    @echo "🧪 Running tests..."
    cargo test --all-targets --all-features

# Quick check without building binaries
check:
    @echo "✅ Running cargo check..."
    cargo check --all-targets --all-features

# Run clippy lints
clippy:
    @echo "📎 Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt --all

# Build via xtask
xtask-build:
    @echo "🔧 Building via xtask..."
    cargo run -p xtask -- build --target all --release

# Generate documentation via xtask
xtask-docs:
    @echo "📚 Generating documentation..."
    cargo run -p xtask -- gen-docs-comprehensive --all --validate

# Full xtask validation
xtask-check:
    @echo "✅ Running xtask validation..."
    cargo run -p xtask -- check-all --strict

# Build all packages with Nix
nix-build:
    @echo "📦 Building Hooksmith with Nix..."
    nix build .#hooksmith-suite

# Build analysis tools specifically
nix-build-analysis:
    @echo "🔍 Building analysis tools with Nix..."
    nix build .#analysis-tools

# Run quick analysis suite with Nix
nix-analyze:
    @echo "🔍 Running Nix-based analysis..."
    nix run .#analyze

# Enter Nix development shell
nix-dev:
    @echo "🚀 Entering Nix development shell..."
    nix develop

# Format Nix files
nix-fmt:
    @echo "🎨 Formatting Nix files..."
    nix fmt

# Repository size analysis
analyze-size:
    @echo "📏 Running repository size analysis..."
    cargo run --bin repository_size_auditor

# Rust blob analysis
analyze-blobs:
    @echo "🦀 Running Rust blob analysis..."
    cargo run --bin rust_blob_analyzer

# Git delta compression analysis
analyze-delta:
    @echo "🔗 Running Git delta analysis..."
    cargo run --bin git_delta_analyzer

# File churn analysis
analyze-churn:
    @echo "📈 Running file churn analysis..."
    cargo run --bin file_churn_analyzer "6 months ago"

# Tree object stability analysis
analyze-stability:
    @echo "🌳 Running tree stability analysis..."
    cargo run --bin tree_object_stability_auditor "6 months ago"

# Git history cleanliness analysis
analyze-history:
    @echo "🧼 Running Git history analysis..."
    cargo run --bin git_history_cleanliness_analyzer

# LFS auto-tracker analysis
analyze-lfs:
    @echo "📦 Running Git LFS analysis..."
    cargo run --bin git_lfs_auto_tracker

# Run all key analysis tools
analyze-all: analyze-size analyze-blobs analyze-delta analyze-churn analyze-stability analyze-history analyze-lfs
    @echo "✅ All analysis tools completed!"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Full development cycle: format, check, test, build
dev-cycle: fmt clippy test build
    @echo "✅ Development cycle complete!"

# Quick setup for new contributors
setup:
    @echo "🚀 Setting up Hooksmith development environment..."
    @echo "1. Installing Rust components..."
    rustup component add clippy rustfmt
    @echo "2. Building workspace..."
    cargo build
    @echo "3. Running a quick test..."
    just analyze-size
    @echo "✅ Setup complete! See WARP.md for detailed workflows."

# Watch for changes and rebuild
watch:
    @echo "👀 Watching for changes..."
    cargo watch -x build

# Release build with all optimizations
release:
    @echo "🚀 Building optimized release..."
    cargo build --release --all-targets
    @echo "📦 Release binaries available in target/release/"

# Generate and open documentation
docs:
    @echo "📚 Generating and opening documentation..."
    cargo doc --open --all-features

# Run benchmarks if any exist
bench:
    @echo "⚡ Running benchmarks..."
    cargo bench

# Audit dependencies for security issues
audit:
    @echo "🔒 Auditing dependencies..."
    cargo audit

# Update dependencies
update:
    @echo "⬆️  Updating dependencies..."
    cargo update

# Show workspace information
info:
    @echo "ℹ️  Hooksmith Workspace Information"
    @echo "=================================="
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"
    @echo "Git version: $(git --version)"
    @echo ""
    @echo "Workspace members:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.workspace_members[]' | sed 's/^/  - /'
    @echo ""
    @echo "Binary targets:"
    @cargo metadata --no-deps --format-version 1 | jq -r '.packages[].targets[] | select(.kind[] == "bin") | .name' | sort | sed 's/^/  - /'
