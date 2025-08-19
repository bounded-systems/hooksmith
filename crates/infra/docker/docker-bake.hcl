// Hooksmith Docker Bake Configuration
// Optimized for GitHub Actions and act with BuildKit

group "default" {
  targets = ["hooksmith"]
}

// Base Rust image with common dependencies
target "rust-base" {
  dockerfile = "Dockerfile"
  target = "rust-base"
  platforms = ["linux/amd64", "linux/arm64"]
  cache-from = ["type=gha"]
  cache-to = ["type=gha,mode=max"]
}

// Development target with all tools
target "dev" {
  inherits = ["rust-base"]
  target = "dev"
  tags = ["hooksmith:dev"]
  cache-from = ["type=gha"]
  cache-to = ["type=gha,mode=max"]
}

// Production target - minimal runtime
target "hooksmith" {
  inherits = ["rust-base"]
  target = "hooksmith"
  tags = ["hooksmith:latest"]
  cache-from = ["type=gha"]
  cache-to = ["type=gha,mode=max"]
}

// Test target for CI/CD
target "test" {
  inherits = ["rust-base"]
  target = "test"
  tags = ["hooksmith:test"]
  cache-from = ["type=gha"]
  cache-to = ["type=gha,mode=max"]
}

// All targets for comprehensive builds
target "all" {
  inherits = ["hooksmith", "dev", "test"]
  platforms = ["linux/amd64", "linux/arm64"]
}
