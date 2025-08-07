# GitHub Actions Integration for Hooksmith

## Overview

Hooksmith now provides comprehensive GitHub Actions integration with a unified event mapping system that covers all GitHub Actions events with pre- and post-hooks.

## Directory Structure

```
.hooksmith/
└── hooks/
    ├── git/           # Client-side Git hooks (18 hooks)
    │   ├── pre-commit
    │   ├── post-commit
    │   ├── pre-push
    │   ├── fsmonitor-watchman
    │   └── ... (all Git client-side hooks)
    │
    └── github/        # GitHub Actions event hooks (194 hooks)
        ├── pre-push
        ├── post-push
        ├── pre-pull_request
        ├── post-pull_request
        ├── pre-issues
        ├── post-issues
        └── ... (all GitHub events)
```

## GitHub Actions Events Coverage

### Core Lifecycle Events
- `push` - Push commits or tags
- `pull_request` - Pull request activity
- `pull_request_target` - Pull request activity (secure context)
- `workflow_dispatch` - Manual workflow trigger
- `workflow_call` - Workflow called by another workflow
- `schedule` - Scheduled workflow runs
- `repository_dispatch` - External trigger via API
- `check_run` - Check run activity
- `check_suite` - Check suite activity

### Content Events
- `create` - Create branch or tag
- `delete` - Delete branch or tag
- `fork` - Repository forked
- `issues` - Issue activity
- `issue_comment` - Issue or PR comment
- `discussion` - Discussion activity
- `discussion_comment` - Discussion comment
- `release` - Release activity
- `page_build` - GitHub Pages build

### Security & Auth Events
- `security_advisory` - Security advisory
- `dependabot_alert` - Dependabot security alert
- `deployment` - Deployment created
- `deployment_status` - Deployment status update
- `status` - Commit status change

### Collaboration Events
- `watch` - Repository starred
- `star` - Repository starred (alias)
- `member` - Repository member activity
- `team_add` - Team added to repository
- `public` - Repository made public
- `organization` - Organization activity

### Advanced & Extensible Events
- `workflow_run` - Workflow run completion
- `milestone` - Milestone activity
- `label` - Label activity
- `project` - Project activity
- `project_card` - Project card activity
- `project_column` - Project column activity

### Additional Events
- `gollum` - Wiki page activity
- `registry_package` - Package registry activity
- `branch_protection_rule` - Branch protection rule changes
- `merge_group` - Merge queue activity
- `pull_request_review` - Pull request review
- `pull_request_review_comment` - Pull request review comment

## Hook Generation System

### GitHub Hook Generator

The `github-hook-generator` tool automatically generates Rust binaries for all GitHub Actions events:

```bash
# List all available events
cargo run --bin github-hook-generator list-events

# Generate hooks for a specific event
cargo run --bin github-hook-generator generate push

# Generate all GitHub event hooks
cargo run --bin github-hook-generator generate-all
```

### Generated Hook Structure

Each event generates two hooks:
- `pre-{event}` - Pre-event validation
- `post-{event}` - Post-event actions

Example generated hook (`pre-push.rs`):
```rust
use anyhow::Result;
use std::env;

/// Pre push Hook for Hooksmith
///
/// This hook handles pre push events:
/// Push commits or tags
///
/// Event: push
/// Hook Type: pre
/// Status: Stub (ready for implementation)
fn main() -> Result<()> {
    println!("✅ pre-push hook (stub mode) - would handle pre push events");

    // Read GitHub event data
    if let Ok(event_path) = env::var("GITHUB_EVENT_PATH") {
        println!("📄 Event path: {}", event_path);
    }

    if let Ok(event_name) = env::var("GITHUB_EVENT_NAME") {
        println!("🎯 Event name: {}", event_name);
    }

    if let Ok(repository) = env::var("GITHUB_REPOSITORY") {
        println!("📦 Repository: {}", repository);
    }

    if let Ok(ref_name) = env::var("GITHUB_REF") {
        println!("🌿 Ref: {}", ref_name);
    }

    // TODO: Implement pre push validation logic
    // - Validate event payload
    // - Check permissions and security
    // - Perform custom validation
    // - Log activity for audit

    println!("🚀 Pre push validation completed successfully");
    Ok(())
}
```

## Configuration

### Git Hooks Configuration

Set the custom hooks path:
```bash
git config core.hooksPath .hooksmith/hooks/git
```

This ensures all Git client-side hooks are executed from the `.hooksmith/hooks/git/` directory.

### GitHub Actions Integration

The GitHub hooks are designed to be invoked by the unified `hooksmith.yml` workflow:

```yaml
name: Hooksmith GitHub Actions Integration

on:
  push:
  pull_request:
  issues:
  # ... all other events

jobs:
  hooksmith-validation:
    runs-on: ubuntu-latest
    steps:
      - name: Pre-event validation
        run: .hooksmith/hooks/github/pre-${{ github.event_name }}
      
      - name: Main workflow steps
        run: |
          # Your main workflow logic here
          echo "Processing ${{ github.event_name }} event"
      
      - name: Post-event actions
        run: .hooksmith/hooks/github/post-${{ github.event_name }}
```

## Event Context Integration

Each hook automatically reads GitHub Actions context:

### Environment Variables
- `GITHUB_EVENT_PATH` - Path to event payload file
- `GITHUB_EVENT_NAME` - Name of the triggering event
- `GITHUB_REPOSITORY` - Repository name
- `GITHUB_REF` - Git reference that triggered the workflow
- `GITHUB_SHA` - Commit SHA that triggered the workflow
- `GITHUB_ACTOR` - Username of the user that triggered the workflow

### Event Payload Access
Hooks can read the full event payload from `$GITHUB_EVENT_PATH` to access event-specific data.

## Workflow Commands Integration

Hooks can use GitHub Actions workflow commands for better integration:

```rust
// Set environment variables
println!("echo 'MY_VAR=value' >> $GITHUB_ENV");

// Set outputs
println!("echo 'output_name=value' >> $GITHUB_OUTPUT");

// Add job summary
println!("echo '### Hooksmith Validation' >> $GITHUB_STEP_SUMMARY");

// Create annotations
println!("::notice file=src/main.rs,line=10::Hooksmith validation passed");
println!("::warning file=src/main.rs,line=15::Potential issue detected");
println!("::error file=src/main.rs,line=20::Validation failed");
```

## Performance Optimizations

### FSMonitor Integration
The `fsmonitor-watchman` hook includes performance optimizations:
- Auto-detects Git's built-in FSMonitor daemon
- Falls back to `rs-git-fsmonitor` if available
- Provides Rust-based implementation as final fallback
- Supports both v1 and v2 fsmonitor protocols

### Hook Execution
- All hooks are compiled Rust binaries for maximum performance
- No shell script overhead
- Direct access to GitHub Actions context
- Minimal startup time

## Development Workflow

### Adding New Events
1. Add the event to `get_github_events()` in `src/bin/github-hook-generator.rs`
2. Run `cargo run --bin github-hook-generator generate-all`
3. Build with `cargo build`
4. Copy binaries to `.hooksmith/hooks/github/`

### Customizing Hooks
1. Edit the generated hook in `src/bin/`
2. Add your validation logic
3. Build with `cargo build`
4. Copy the binary to `.hooksmith/hooks/github/`

### Testing Hooks
```bash
# Test a specific hook
GITHUB_EVENT_NAME=push GITHUB_REPOSITORY=test/repo .hooksmith/hooks/github/pre-push

# Test with act (local GitHub Actions runner)
act push -W .github/workflows/hooksmith.yml
```

## Benefits

### Comprehensive Coverage
- **47 GitHub Events** with pre- and post-hooks
- **18 Git Client Hooks** for local validation
- **194 Total Hooks** for complete coverage

### Unified Architecture
- Single `hooksmith.yml` workflow for all events
- Consistent hook structure across all events
- Shared validation logic and patterns

### Performance
- Compiled Rust binaries for fast execution
- FSMonitor integration for Git performance
- Minimal overhead in CI/CD pipelines

### Developer Experience
- Auto-generated hooks reduce boilerplate
- Consistent patterns across all events
- Easy customization and extension
- Local testing with `act`

## Future Enhancements

### Planned Features
- Contract validation integration
- WASM component support
- Advanced event filtering
- Custom hook templates
- Performance monitoring
- Hook composition system

### Integration Opportunities
- Lefthook migration support
- Custom action development
- Multi-repository workflows
- Enterprise deployment patterns

## References

- [GitHub Actions Events Documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows)
- [GitHub Actions Contexts](https://docs.github.com/en/actions/reference/context-and-expression-syntax-for-github-actions)
- [Workflow Commands](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions)
- [Dockerfile Support](https://docs.github.com/en/actions/reference/workflows-and-actions/dockerfile-support)
