# Cursor Integration Example

This example demonstrates how to use the new Cursor integration feature in worktree management.

## Basic Example

Create a new worktree and automatically open it in Cursor:

```bash
# Create a feature branch and open in Cursor
cargo xtask worktree create --branch feat-user-authentication --open-cursor
```

**Expected Output:**
```
✓ Loaded .workbloom configuration
✓ Loaded Workbloom metadata
  - Loaded worktree metadata
  - Loaded semantic labels
  - Loaded worktree status
Creating worktree: feat-user-authentication -> feat-user-authentication
✓ Worktree created successfully with Workbloom
  - Automatic file copying enabled
  - Port allocation configured
Opening worktree in Cursor...
✓ Opened worktree in Cursor
  - Path: ../feat-user-authentication
```

## Advanced Example

Create a worktree with setup commands and open in Cursor:

```bash
# Create worktree with setup and Cursor integration
cargo xtask worktree create \
  --branch feat-api-endpoints \
  --setup \
  --open-cursor
```

**Expected Output:**
```
✓ Loaded .workbloom configuration
✓ Loaded Workbloom metadata
Creating worktree: feat-api-endpoints -> feat-api-endpoints
Running setup commands...
Running: cargo check
✓ Setup command completed
Opening worktree in Cursor...
✓ Opened worktree in Cursor
  - Path: ../feat-api-endpoints
```

## Custom Base Directory Example

Create a worktree in a custom location and open in Cursor:

```bash
# Create worktree in custom location
cargo xtask worktree create \
  --branch feat-database-migration \
  --base-dir ../custom-worktrees \
  --open-cursor
```

**Expected Output:**
```
✓ Loaded .workbloom configuration
Creating worktree: feat-database-migration -> feat-database-migration
✓ Worktree created successfully
Opening worktree in Cursor...
✓ Opened worktree in Cursor
  - Path: ../custom-worktreesfeat-database-migration
```

## Error Handling Example

When Cursor is not available:

```bash
# This will show an error if Cursor is not installed
cargo xtask worktree create --branch feat-error-handling --open-cursor
```

**Expected Output:**
```
✓ Loaded .workbloom configuration
Creating worktree: feat-error-handling -> feat-error-handling
✓ Worktree created successfully with Workbloom
Opening worktree in Cursor...
✗ Cursor not found in PATH
  - Please install Cursor or add it to your PATH
  - You can manually open it with: cursor ../feat-error-handling
```

## Integration with Existing Workflows

### With Git Workflow

```bash
# Create worktree and immediately start development
cargo xtask worktree create --branch feat-new-component --open-cursor

# The worktree is now open in Cursor, ready for development
# You can immediately start coding without manual steps
```

### With Setup Commands

```bash
# Create worktree with automatic setup and Cursor opening
cargo xtask worktree create \
  --branch feat-testing-framework \
  --setup \
  --open-cursor

# The worktree is created, setup commands run, and Cursor opens
# All in one seamless workflow
```

### With Environment Files

```bash
# Create worktree with environment setup and Cursor opening
cargo xtask worktree create \
  --branch feat-environment-config \
  --copy-env \
  --open-cursor

# Environment files are copied and Cursor opens automatically
```

## Troubleshooting Examples

### Cursor Not Found

If you see this error:
```
✗ Cursor not found in PATH
```

**Solution:**
```bash
# Add Cursor to your PATH (macOS)
export PATH="/Applications/Cursor.app/Contents/MacOS:$PATH"

# Or install Cursor if not installed
# Download from: https://cursor.sh/
```

### Cursor Launch Failure

If you see this error:
```
✗ Failed to open Cursor: <error_message>
```

**Solution:**
```bash
# Check Cursor installation
which cursor

# Test manual launch
cursor --version

# Try opening manually
cursor <worktree_path>
```

### Path Issues

If the worktree path is incorrect:
```bash
# Check worktree location
git worktree list

# Verify the path exists
ls -la <worktree_path>

# Open manually if needed
cursor <correct_path>
```

## Best Practices

### 1. Use Descriptive Branch Names

```bash
# Good
cargo xtask worktree create --branch feat-user-dashboard --open-cursor

# Avoid
cargo xtask worktree create --branch temp --open-cursor
```

### 2. Combine with Setup Commands

```bash
# Recommended: Use setup for immediate development readiness
cargo xtask worktree create \
  --branch feat-api-integration \
  --setup \
  --open-cursor
```

### 3. Use Custom Base Directories for Organization

```bash
# Organize worktrees by project
cargo xtask worktree create \
  --branch feat-frontend-redesign \
  --base-dir ../frontend-worktrees \
  --open-cursor
```

### 4. Clean Up After Development

```bash
# Remove worktree when done
git worktree remove <worktree-name>

# Or use the remove command
cargo xtask worktree remove <worktree-name>
```

## Integration with CI/CD

The Cursor integration is designed for local development and won't interfere with CI/CD pipelines:

```bash
# In CI/CD, don't use --open-cursor
cargo xtask worktree create --branch ci-test-branch

# For local development, use --open-cursor
cargo xtask worktree create --branch local-feature --open-cursor
```

## Performance Considerations

- The Cursor opening is asynchronous and won't block the worktree creation
- Error handling is non-blocking - worktree creation succeeds even if Cursor fails
- The feature adds minimal overhead to the worktree creation process

## Security Notes

- The feature only opens Cursor with the worktree path
- No additional permissions or system access is required
- Cursor launch uses the same permissions as manual `cursor` command execution 
