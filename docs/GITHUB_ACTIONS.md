# GitHub Actions Integration

Hooksmith provides a single comprehensive GitHub Actions workflow that handles all GitHub events through Rust stub binaries.

## 🚀 Quick Start

### 1. Build the Stub Binaries

```bash
# Build all GitHub stub binaries
cargo build --release

# Verify all binaries work
cargo run --bin verify-stubs
```

### 2. Enable Validation (Optional)

Edit `.github/workflows/hooksmith.yml` and change:

```yaml
env:
  ENABLE_HOOKSMITH_VALIDATION: true  # Change from false to true
```

## 📋 Available Stub Binaries

All these binaries are **no-op by default** and ready for your validation logic:

- `github-push` - Push events
- `github-pull-request` - Pull request events  
- `github-issues` - Issue events
- `github-release` - Release events
- `github-create` - Branch/tag creation
- `github-delete` - Branch/tag deletion
- `github-branch-protection-rule` - Branch protection events
- `github-check-run` - Check run events

## 🎯 No-Op Behavior

All stub binaries:

1. **Parse GitHub event payloads** - Extract relevant information
2. **Use workflow commands** - `::group::`, `::notice::`, `::info::`
3. **Set output variables** - For workflow consumption
4. **Exit successfully** - No validation failures by default
5. **Ready for integration** - Call your validation logic

## 📊 Event-to-Hook Mapping

| GitHub Event | Git Hooks | Description |
|--------------|-----------|-------------|
| `push` | `pre-commit`, `commit-msg`, `post-commit` | Validates commits |
| `pull_request` | `pre-commit`, `commit-msg`, `prepare-commit-msg` | Validates PR changes |
| `issues` | `commit-msg` | Validates issue content |
| `release` | `pre-receive`, `post-receive` | Validates releases |
| `create/delete` | `pre-receive`, `post-receive` | Validates ref operations |

## 🔧 Testing

```bash
# Test individual stubs
cargo run --bin github-push
cargo run --bin github-pull-request

# Test all stubs
cargo run --bin verify-stubs
```

## 🛡️ Security

- All stubs are **no-op by default** - No validation failures
- Set minimal `GITHUB_TOKEN` permissions
- Validate all inputs before processing

## 🚀 Next Steps

1. **Customize validation logic** - Replace TODO comments in stub binaries
2. **Add schema integration** - Connect to your Hooksmith validation schema
3. **Configure event filters** - Add branch/path filters to workflows
4. **Create custom validators** - Add project-specific validation rules
