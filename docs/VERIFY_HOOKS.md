# Hooksmith Verify-Hooks Command

The `verify-hooks` command allows you to check if Hooksmith is properly registered in all Git hooks in your repository.

## Overview

This command enumerates all hooks in `.git/hooks/` and checks if each hook script contains Hooksmith invocation. It provides immediate feedback about the registration status of Hooksmith in your Git workflow.

## Usage

### Basic Usage

```bash
# Verify hooks in the current repository
hooksmith verify-hooks

# Verify hooks in a specific repository
hooksmith verify-hooks --repo-path /path/to/repo

# Show detailed information about each hook
hooksmith verify-hooks --verbose
```

### Command Options

- `--repo-path <PATH>`: Git repository root directory (default: current directory)
- `--verbose`: Show detailed information about each hook including content preview
- `-h, --help`: Print help information

## Example Output

```
🔍 Verifying Git hooks registration: .
✔ pre-commit ✅ Hooksmith registered
✖ pre-push ❌ Missing Hooksmith
✔ commit-msg ✅ Hooksmith registered
✅ Hook verification completed
📊 2 / 3 hooks have Hooksmith registered
```

### Verbose Output

With `--verbose` flag, you get additional details:

```
🔍 Verifying Git hooks registration: .
✔ pre-commit ✅ Hooksmith registered
   Content preview:
   #!/bin/sh
   # Pre-commit hook with Hooksmith integration
   
   echo "Running pre-commit checks..."
   
   ...

✖ pre-push ❌ Missing Hooksmith
   Content preview:
   #!/bin/sh
   # Pre-push hook for security checks
   
   echo "Running pre-push checks..."
   
   ...

✅ Hook verification completed
📊 1 / 2 hooks have Hooksmith registered
```

## Integration Options

### 1. Manual Verification

Run the command manually to check your hooks:

```bash
hooksmith verify-hooks
```

### 2. Post-Push Hook Integration

Add a post-push hook to automatically verify after each push:

```bash
# Create post-push hook
cat > .git/hooks/post-push << 'EOF'
#!/bin/sh
# Post-push hook to verify Hooksmith registration

echo "🔍 Verifying Hooksmith registration after push..."

# Run Hooksmith verify-hooks command
cargo run --bin hooksmith -- verify-hooks

echo "✅ Post-push verification completed"
EOF

chmod +x .git/hooks/post-push
```

### 3. Lefthook Integration

Add to your `lefthook.yml`:

```yaml
# Post-push hook to verify Hooksmith registration
post-push:
  commands:
    verify-hooksmith:
      run: hooksmith verify-hooks
      description: "Verify Hooksmith is properly registered in all Git hooks"
```

### 4. CI/CD Integration

#### GitHub Actions

```yaml
name: Verify Hooksmith Registration

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  verify-hooksmith:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout code
      uses: actions/checkout@v4
      
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Verify Hooksmith Registration
      run: |
        cargo build --bin hooksmith
        cargo run --bin hooksmith -- verify-hooks
        
    - name: Check for missing hooks
      run: |
        output=$(cargo run --bin hooksmith -- verify-hooks 2>&1)
        echo "$output"
        
        if echo "$output" | grep -q "❌ Missing Hooksmith"; then
          echo "❌ Some Git hooks are missing Hooksmith registration"
          exit 1
        fi
```

#### GitLab CI

```yaml
verify-hooksmith:
  stage: test
  script:
    - cargo build --bin hooksmith
    - cargo run --bin hooksmith -- verify-hooks
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

## Hook Detection Logic

The command checks for the presence of these strings in hook files:
- `hooksmith` (case-insensitive)
- `Hooksmith` 
- `HOOKSMITH`

### Supported Hook Types

The command checks for these common Git hooks:

**Client-side hooks:**
- `pre-commit`
- `pre-push`
- `commit-msg`
- `post-commit`
- `pre-rebase`
- `post-merge`
- `post-checkout`
- `prepare-commit-msg`
- `pre-merge-commit`
- `post-rewrite`
- `pre-auto-gc`
- `fsmonitor-watchman`
- `p4-changelist`
- `p4-prepare-changelist`
- `p4-post-changelist`
- `p4-pre-submit`
- `post-index-change`

**Server-side hooks:**
- `pre-receive`
- `update`
- `proc-receive`
- `post-receive`
- `post-update`
- `reference-transaction`
- `push-to-checkout`
- `applypatch-msg`
- `pre-applypatch`
- `post-applypatch`
- `sendemail-validate`

## Best Practices

1. **Regular Verification**: Run `verify-hooks` regularly to ensure all hooks are properly configured
2. **CI Integration**: Include verification in your CI pipeline to catch missing registrations early
3. **Post-Push Automation**: Use post-push hooks to get immediate feedback after pushing
4. **Verbose Debugging**: Use `--verbose` flag when troubleshooting hook issues

## Troubleshooting

### Common Issues

1. **"Git hooks directory not found"**: Ensure you're running the command from a Git repository
2. **"No Git hooks found"**: The repository doesn't have any hooks configured
3. **False positives**: Check if hook content contains "hooksmith" in comments or strings

### Debugging

Use the verbose flag to see hook contents:

```bash
hooksmith verify-hooks --verbose
```

This will show you the first 5 lines of each hook file, helping you understand why a hook is or isn't being detected as having Hooksmith registered.

## Exit Codes

- `0`: Success (verification completed)
- `1`: Error (verification failed, hooks directory not found, etc.)

The command will exit with code 1 if there are any errors during verification, making it suitable for use in CI/CD pipelines. 
