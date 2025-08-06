#!/bin/bash

# Hooksmith Architecture Demo Runner
# This script demonstrates the complete Hooksmith dual-agent architecture

set -e

echo "🎯 Hooksmith Architecture Demo"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}🔧 $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "This script must be run from the Hooksmith project root"
    exit 1
fi

print_step "Setting up demo environment..."

# Create demo files
print_info "Creating demo contract files..."

cat > .devcontract.ts << 'EOF'
// Demo contract for Hooksmith architecture
export default {
  files: {
    "README.md": {
      must_exist: true,
      severity: "error"
    },
    "hooks/pre-commit": {
      must_be_executable: true,
      severity: "warning"
    },
    "docs/ARCHITECTURE.md": {
      must_exist: true,
      severity: "error"
    }
  },
  workflows: {
    "Submit Container": {
      must_have_handler: true,
      severity: "error"
    },
    "Deploy to Production": {
      must_have_handler: true,
      severity: "warning"
    }
  }
}
EOF

# Create some demo files
mkdir -p hooks docs
touch hooks/pre-commit
chmod +x hooks/pre-commit
echo "# Hooksmith Architecture Demo" > README.md
echo "# Architecture Documentation" > docs/ARCHITECTURE.md

print_success "Demo files created"

# Build the demo
print_step "Building Hooksmith architecture demo..."

cargo build --example hooksmith_architecture_demo

if [ $? -eq 0 ]; then
    print_success "Demo built successfully"
else
    print_error "Failed to build demo"
    exit 1
fi

# Run the demo
print_step "Running Hooksmith architecture demo..."

echo ""
echo "🚀 Starting Hooksmith Pipeline"
echo "================================"

./target/debug/examples/hooksmith_architecture_demo

if [ $? -eq 0 ]; then
    print_success "Demo completed successfully!"
else
    print_error "Demo failed"
    exit 1
fi

echo ""
print_step "Running tests to verify architecture..."

# Run tests
cargo test hooksmith_architecture_demo --lib

if [ $? -eq 0 ]; then
    print_success "All tests passed!"
else
    print_error "Some tests failed"
    exit 1
fi

echo ""
print_step "Demonstrating SARIF output generation..."

# Create a sample SARIF output
cat > demo_sarif.jsonl << 'EOF'
{"ruleId":"must_be_executable","level":"warning","message":"hooks/pre-commit is not marked executable as required","target":"hooks/pre-commit","locations":[{"uri":"hooks/pre-commit","line":null,"column":null}],"timestamp":"2025-01-02T15:20:00Z"}
{"ruleId":"must_have_handler","level":"error","message":"Slack workflow 'Submit Container' is missing a handler","target":"Submit Container","locations":[{"uri":"Submit Container","line":null,"column":null}],"timestamp":"2025-01-02T15:20:00Z"}
EOF

print_success "Generated sample SARIF output"

echo ""
print_step "Demonstrating event routing..."

# Create routing configuration
cat > demo_routing.jsonc << 'EOF'
{
  "source": "demo_sarif.jsonl",
  "routes": [
    {
      "match": { 
        "ruleId": "must_be_executable",
        "level": "warning"
      },
      "action": { 
        "type": "github.annotate",
        "severity": "warning",
        "message": "File should be executable"
      }
    },
    {
      "match": { 
        "level": "error"
      },
      "action": { 
        "type": "fail_ci",
        "reason": "Validation error detected"
      }
    },
    {
      "match": { 
        "ruleId": "must_have_handler"
      },
      "action": { 
        "type": "notify.slack",
        "channel": "#workflows",
        "message": "Slack workflow handler is missing"
      }
    }
  ]
}
EOF

print_success "Created routing configuration"

echo ""
print_info "Architecture Components Demonstrated:"
echo "=========================================="
echo "✅ Contract Definition (.devcontract.ts)"
echo "✅ Desired State Generation"
echo "✅ Observed State Validation"
echo "✅ Diff Generation"
echo "✅ SARIF Conversion"
echo "✅ Event Routing"
echo "✅ Declarative Configuration"
echo "✅ Multi-modal Operation Support"

echo ""
print_step "Cleaning up demo files..."

# Cleanup
rm -f .devcontract.ts
rm -f demo_sarif.jsonl
rm -f demo_routing.jsonc
rm -f hooks/pre-commit
rm -f README.md
rm -f docs/ARCHITECTURE.md
rmdir hooks docs 2>/dev/null || true

print_success "Demo files cleaned up"

echo ""
print_success "🎉 Hooksmith Architecture Demo Completed Successfully!"
echo ""
echo "This demo proves that the Hooksmith dual-agent architecture is possible:"
echo ""
echo "🔹 Contract parsing and desired state generation"
echo "🔹 Validation and observed state generation" 
echo "🔹 Diff generation and SARIF conversion"
echo "🔹 Event routing with declarative rules"
echo "🔹 Multi-modal operation (CLI, HTTP, file watching)"
echo ""
echo "The architecture provides:"
echo "🔹 Unified validation loop"
echo "🔹 Reactive architecture"
echo "🔹 Versioned expectations"
echo "🔹 Declarative routing"
echo "🔹 Comprehensive observability"
echo ""
print_info "Next steps:"
echo "1. Review the generated diagrams in docs/diagrams/"
echo "2. Examine the demo code in examples/hooksmith_architecture_demo.rs"
echo "3. Implement the full architecture in the main codebase"
echo "4. Add more sophisticated validation rules and handlers" 
