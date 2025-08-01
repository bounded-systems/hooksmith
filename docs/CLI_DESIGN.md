# CLI Design

This document outlines the design decisions and patterns used in the pushd-cli prototype.

## Design Principles

### 1. Unified Interface
- **Single Entry Point**: All operations go through one CLI binary
- **Consistent Commands**: Similar operations use similar command patterns
- **Progressive Disclosure**: Simple commands for common tasks, advanced options for complex workflows

### 2. Developer Experience First
- **Fast Startup**: Rust binary with minimal overhead
- **Rich Feedback**: Colored output, progress bars, and clear messaging
- **Error Recovery**: Helpful error messages with suggestions
- **Interactive Mode**: Fuzzy search and interactive prompts for complex operations

### 3. Safety and Validation
- **Built-in Safety**: Automatic safety checks for dangerous operations
- **Dry Run Mode**: Test operations without making changes
- **Validation**: Validate inputs and state before execution
- **Confirmation**: Require confirmation for destructive operations

### 4. Extensibility
- **Plugin Architecture**: Support for custom commands and workflows
- **Wasm Components**: Language-agnostic functionality through WebAssembly
- **Configuration**: Flexible configuration system
- **Hooks**: Support for pre/post operation hooks

## Command Structure

### Command Hierarchy

```
pushd-cli
├── worktree
│   ├── create
│   ├── list
│   ├── switch
│   ├── remove
│   └── sync
├── daemon
│   ├── run
│   ├── list
│   ├── test
│   ├── validate
│   └── logs
├── docker
│   ├── build
│   ├── up
│   ├── down
│   ├── logs
│   └── db
│       ├── reset
│       ├── migrate
│       └── seed
├── git
│   ├── check-safety
│   ├── validate-commit
│   ├── pre-push-check
│   ├── sync
│   ├── create-branch
│   └── delete-branch
├── setup
│   ├── init
│   ├── env
│   ├── docker
│   └── hooks
├── config
│   ├── show
│   ├── set
│   └── reset
└── component
    ├── list
    ├── build
    ├── test
    └── info
```

### Command Patterns

#### Resource Management Commands

```bash
# Create resource
pushd-cli <resource> create [options]

# List resources
pushd-cli <resource> list [options]

# Show resource details
pushd-cli <resource> show <name>

# Remove resource
pushd-cli <resource> remove <name> [options]
```

#### Action Commands

```bash
# Execute action
pushd-cli <resource> <action> [options]

# Examples
pushd-cli daemon run <name> [options]
pushd-cli docker build [options]
pushd-cli git sync [options]
```

#### Configuration Commands

```bash
# Show configuration
pushd-cli config show

# Set configuration
pushd-cli config set <key> <value>

# Reset configuration
pushd-cli config reset
```

## Error Handling

### Error Types

1. **Validation Errors**: Invalid input or state
2. **Execution Errors**: Failed operations
3. **Configuration Errors**: Missing or invalid configuration
4. **System Errors**: External system failures

### Error Response Pattern

```rust
#[derive(Debug)]
pub struct CliError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug)]
pub enum ErrorKind {
    Validation,
    Execution,
    Configuration,
    System,
}
```

### Error Display

```bash
❌ Error: Failed to create worktree
   Context: Worktree 'fix-netsuite-daemon' already exists
   Suggestion: Use --force to overwrite or choose a different name
```

## Configuration System

### Configuration Hierarchy

1. **Default Values**: Built-in sensible defaults
2. **Global Config**: `~/.config/pushd-cli/config.json`
3. **Project Config**: `.pushd-cli.json` in project root
4. **Environment Variables**: `PUSHD_CLI_*` variables
5. **Command Line**: `--config` flags

### Configuration Schema

```json
{
  "default_environment": "staging",
  "aws_profile": "pushd",
  "docker_compose_file": "docker-compose.yml",
  "worktree_directory": "..",
  "auto_setup": true,
  "safety_checks": true,
  "log_level": "info",
  "timeout": 300,
  "retry_attempts": 3
}
```

## Output Formatting

### Success Messages

```bash
✅ Worktree created successfully!
✅ Daemon started in background
✅ Configuration saved
```

### Warning Messages

```bash
⚠️  Uncommitted changes detected
⚠️  Branch is behind origin/main
⚠️  This is a prototype - actual execution would run the daemon
```

### Info Messages

```bash
ℹ️  Building Docker images...
ℹ️  Running validation checks...
ℹ️  Found 3 worktrees
```

### Progress Indicators

```bash
⠋ Creating worktree...
⠙ Building Docker images...
⠹ Running tests...
✅ Operation completed!
```

## Interactive Features

### Fuzzy Search

```bash
# Interactive worktree selection
pushd-cli worktree switch
? Select worktree: 
  ❯ fix-netsuite-daemon
    hotfix-critical-bug
    feature-new-ui
```

### Confirmation Prompts

```bash
pushd-cli worktree remove fix-netsuite-daemon
? Remove worktree 'fix-netsuite-daemon'? (y/N)
```

### Multi-select

```bash
pushd-cli daemon run
? Select daemons to run: 
  ◯ netsuite_transactions_daemon
  ◯ booking_report_to_slack
  ◯ inbound_container_netsuite_sync_daemon
```

## Component Architecture

### Component Integration

```rust
// Main CLI with optional component support
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Use Wasm components (experimental)
    #[arg(long)]
    use_components: bool,
}

// Component-aware command execution
match cli.command {
    Commands::Daemon { command } => {
        if let Some(ref mut manager) = component_manager {
            daemon::execute_with_components(command, &config, manager).await?;
        } else {
            daemon::execute(command, &config).await?;
        }
    }
}
```

### Component Discovery

```rust
pub struct ComponentManager {
    components_path: PathBuf,
    components: HashMap<String, ComponentInfo>,
}

impl ComponentManager {
    pub async fn discover_components(&mut self) -> Result<()> {
        // Scan components directory
        // Load component metadata
        // Validate component interfaces
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worktree_creation() {
        // Test worktree creation logic
    }

    #[tokio::test]
    async fn test_daemon_execution() {
        // Test daemon execution logic
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        // Test complete workflow from start to finish
    }
}
```

### Component Tests

```rust
#[cfg(test)]
mod component_tests {
    use super::*;

    #[tokio::test]
    async fn test_component_loading() {
        // Test component discovery and loading
    }
}
```

## Performance Considerations

### Startup Time

- **Minimal Dependencies**: Only essential dependencies
- **Lazy Loading**: Load components only when needed
- **Caching**: Cache configuration and component metadata

### Memory Usage

- **Streaming**: Process large outputs in streams
- **Resource Cleanup**: Proper cleanup of resources
- **Component Isolation**: Isolated memory spaces for components

### Execution Speed

- **Async Operations**: Non-blocking I/O operations
- **Parallel Execution**: Parallel execution where possible
- **Optimized Algorithms**: Efficient algorithms for common operations

## Security Considerations

### Sandboxing

- **Component Isolation**: Components run in isolated environments
- **Resource Limits**: Limit resource usage per component
- **Permission Model**: Explicit permissions for component operations

### Input Validation

- **Sanitization**: Sanitize all user inputs
- **Type Safety**: Strong typing for all interfaces
- **Boundary Checks**: Validate all boundaries and limits

### Error Handling

- **No Information Leakage**: Don't expose sensitive information in errors
- **Secure Logging**: Log sensitive operations securely
- **Audit Trail**: Maintain audit trail for security-sensitive operations

## Future Enhancements

### Planned Features

1. **Plugin System**: Runtime plugin loading
2. **Remote Components**: Load components from registries
3. **Hot Reloading**: Update components without restart
4. **Performance Monitoring**: Track component performance
5. **Multi-language Support**: Components in TypeScript, Go, C

### Architecture Evolution

1. **Microservices**: Decompose into microservices
2. **Event-driven**: Event-driven architecture for scalability
3. **Distributed**: Support for distributed execution
4. **Cloud Integration**: Native cloud platform integration

## Resources

- [Rust CLI Guidelines](https://rust-cli.github.io/book/)
- [Clap Documentation](https://docs.rs/clap/)
- [Wasm Component Model](https://component-model.bytecodealliance.org/)
- [CLI Design Patterns](https://clig.dev/) 
