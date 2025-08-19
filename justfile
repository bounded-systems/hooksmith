# Justfile for Hooksmith development
# Run with: just <recipe-name>

# Default recipe - show available commands
default:
    @echo "🔨 Hooksmith Development Commands"
    @echo "================================"
    @echo ""
    @echo "🚀 Getting Started:"
    @echo "  just bootstrap         - Setup development environment"
    @echo "  just info              - Show environment information"
    @echo ""
    @echo "🦀 Rust Development:"
    @echo "  just build             - Build workspace (cargo)"
    @echo "  just test              - Run tests"
    @echo "  just check             - Quick check without building"
    @echo "  just clippy            - Run clippy lints"
    @echo "  just fmt               - Format all code"
    @echo "  just lint              - Lint all code"
    @echo ""
    @echo "🧹 Code Quality:"
    @echo "  just pre-commit        - Run all pre-commit checks"
    @echo "  just fmt-all           - Format Rust, Nix, and other files"
    @echo "  just lint-all          - Lint Rust, Nix, and shell files"
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
    @echo "💡 Quick Start:"
    @echo "  just bootstrap && just analyze-size"

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

# Bootstrap development environment
bootstrap:
    @echo "🚀 Bootstrapping Hooksmith development environment..."
    @echo "1. Verifying toolchain..."
    @rustc --version
    @cargo --version
    @git --version
    @echo "2. Setting up git hooks..."
    @# Clear any existing hooks path that might conflict
    @git config --unset-all core.hooksPath || true
    @# Pre-commit hooks are managed by Nix in this environment
    @echo "✅ Git hooks managed by Nix (via flake.nix pre-commit configuration)"
    @echo "3. Verifying workspace can build..."
    @cargo check --workspace || echo "⚠️  Some workspace members may need attention"
    @echo "4. Running initial analysis to verify tools..."
    @cargo run --bin repository_size_auditor > /dev/null 2>&1 || echo "⚠️  Some analysis tools may need building first"
    @echo "✅ Bootstrap complete! Environment ready for development."
    @echo "💡 Next steps: just info, just analyze-size, or just build"

# Run all pre-commit checks
pre-commit:
    @echo "🧹 Running pre-commit checks..."
    @if command -v pre-commit >/dev/null 2>&1; then \
        pre-commit run --all-files; \
    else \
        echo "⚠️  pre-commit not available, running individual checks..."; \
        just fmt && just clippy && just nix-fmt; \
    fi

# Format all code (Rust + Nix + others)
fmt-all:
    @echo "🎨 Formatting all code..."
    @cargo fmt --all
    @if command -v alejandra >/dev/null 2>&1; then \
        alejandra .; \
    else \
        echo "⚠️  alejandra not available, skipping Nix formatting"; \
    fi
    @if command -v prettier >/dev/null 2>&1; then \
        prettier --write "**/*.{json,yaml,yml,md}" || true; \
    else \
        echo "⚠️  prettier not available, skipping JS/JSON/YAML/MD formatting"; \
    fi

# Lint all code (Rust + Nix + Shell)
lint-all:
    @echo "🔍 Linting all code..."
    @cargo clippy --all-targets --all-features -- -D warnings
    @if command -v statix >/dev/null 2>&1; then \
        statix check .; \
    else \
        echo "⚠️  statix not available, skipping Nix linting"; \
    fi
    @if command -v deadnix >/dev/null 2>&1; then \
        deadnix --check .; \
    else \
        echo "⚠️  deadnix not available, skipping Nix dead code check"; \
    fi
    @if command -v shellcheck >/dev/null 2>&1; then \
        find . -name "*.sh" -exec shellcheck {} \; || true; \
    else \
        echo "⚠️  shellcheck not available, skipping shell script linting"; \
    fi

# Enhanced lint command (alias for lint-all)
lint: lint-all

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
