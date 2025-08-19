# Worktree Cursor Integration

This document describes the Cursor integration feature for worktree management in Hooksmith.

## Overview

The worktree management system now includes automatic Cursor integration, allowing you to open newly created worktrees directly in Cursor as a new window. This feature streamlines the development workflow by eliminating the need to manually open worktrees in your editor.

## Features

### Automatic Cursor Opening

When creating a new worktree with the `--open-cursor` flag, the system will:

1. Create the worktree as usual
2. Check if Cursor is available in your PATH
3. Automatically open the worktree in a new Cursor window
4. Provide helpful feedback and fallback instructions

### Smart Path Detection

The integration automatically detects the correct worktree path based on:
- Custom base directory (if specified with `--base-dir`)
- Default worktree location (relative to the main repository)

### Error Handling

The feature includes robust error handling:
- Checks if Cursor is installed and available
- Provides clear error messages if Cursor is not found
- Offers manual fallback commands
- Gracefully handles Cursor launch failures

## Usage

### Basic Usage

```bash
# Create a worktree and open it in Cursor
cargo xtask worktree create --branch feature-name --open-cursor
```

### With Custom Base Directory

```bash
# Create worktree in custom location and open in Cursor
cargo xtask worktree create --branch feature-name --base-dir ../custom-location --open-cursor
```

### With Additional Options

```bash
# Create worktree with setup commands and open in Cursor
cargo xtask worktree create --branch feature-name --setup --open-cursor
```

## Command Line Options

### `--open-cursor`

- **Type**: Boolean flag
- **Default**: `false`
- **Description**: Automatically open the created worktree in Cursor

**Example:**
```bash
cargo xtask worktree create --branch my-feature --open-cursor
```

## Implementation Details

### Path Resolution

The worktree path is determined using the following logic:

```rust
let worktree_path = if let Some(base_dir) = base_dir {
    format!("{}{}", base_dir, branch)
} else {
    format!("../{}", branch)
};
```

### Cursor Detection

The system checks for Cursor availability using:

```bash
which cursor
```

### Cursor Launch

If Cursor is available, it launches with:

```bash
cursor <worktree_path>
```

## Error Scenarios

### Cursor Not Found

If Cursor is not installed or not in PATH:

```
✗ Cursor not found in PATH
  - Please install Cursor or add it to your PATH
  - You can manually open it with: cursor <worktree_path>
```

### Cursor Launch Failure

If Cursor fails to launch:

```
✗ Failed to open Cursor: <error_message>
  - You can manually open it with: cursor .
```

### Detection Failure

If the system cannot check for Cursor:

```
✗ Could not check for Cursor installation
  - You can manually open it with: cursor <worktree_path>
```

## Integration with Existing Features

### Workbloom Integration

The Cursor integration works seamlessly with Workbloom:
- Automatic file copying is preserved
- Port allocation configuration is maintained
- Metadata tracking continues to work

### Setup Commands

When used with `--setup`, the Cursor opening happens after:
- Worktree creation
- Setup command execution
- Environment file copying

### Tool Selection

The feature works with both:
- **Workbloom** (recommended)
- **Git worktree** (fallback)

## Configuration

### Environment Requirements

- Cursor must be installed and available in PATH
- The `cursor` command must be accessible from the terminal

### PATH Setup

Ensure Cursor is in your PATH by adding to your shell profile:

```bash
# For macOS
export PATH="/Applications/Cursor.app/Contents/MacOS:$PATH"

# For Linux
export PATH="/opt/Cursor:$PATH"
```

## Examples

### Feature Development

```bash
# Create a feature branch and open in Cursor
cargo xtask worktree create --branch feat-new-component --open-cursor
```

### Bug Fix Development

```bash
# Create a bug fix branch and open in Cursor
cargo xtask worktree create --branch fix-cursor-integration --open-cursor
```

### Hotfix Development

```bash
# Create a hotfix branch and open in Cursor
cargo xtask worktree create --branch hotfix-critical-bug --open-cursor
```

## Troubleshooting

### Cursor Not Opening

1. **Check Cursor installation:**
   ```bash
   which cursor
   ```

2. **Verify PATH setup:**
   ```bash
   echo $PATH | grep -i cursor
   ```

3. **Test manual launch:**
   ```bash
   cursor --version
   ```

### Worktree Path Issues

1. **Check worktree location:**
   ```bash
   git worktree list
   ```

2. **Verify path exists:**
   ```bash
   ls -la <worktree_path>
   ```

### Permission Issues

1. **Check Cursor permissions:**
   ```bash
   ls -la $(which cursor)
   ```

2. **Verify directory permissions:**
   ```bash
   ls -la <worktree_path>
   ```

## Future Enhancements

### Planned Features

- **Workspace Configuration**: Auto-generate Cursor workspace files
- **Extension Management**: Automatically enable recommended extensions
- **Project Templates**: Apply project-specific Cursor configurations
- **Multi-Window Support**: Open multiple worktrees in separate windows

### Configuration Options

Future versions may include:
- Custom Cursor executable path
- Workspace file templates
- Extension auto-installation
- Project-specific settings

## Contributing

To contribute to the Cursor integration:

1. **Report Issues**: Use the issue tracker for bugs and feature requests
2. **Submit PRs**: Follow the contribution guidelines
3. **Test Changes**: Ensure compatibility with different platforms
4. **Update Docs**: Keep documentation current with changes

## Related Documentation

- [Worktree Management](./worktree-management.md)
- [Workbloom Integration](./workbloom-integration.md)
- [Development Workflow](./development-workflow.md)
- [Command Reference](./command-reference.md) 
