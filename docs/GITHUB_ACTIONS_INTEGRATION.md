# GitHub Actions Integration with Hooksmith

This document describes the integration between GitHub Actions events and Hooksmith's Git hook validation system.

## Overview

Hooksmith provides a comprehensive set of stub binaries that map GitHub Actions events to Git hooks, enabling server-side validation that mirrors your local Git hook validation. This creates a 1:1 parity between client-side and server-side validation.

## Architecture

### Event-to-Hook Mapping

| GitHub Event | Git Hooks | Description |
|--------------|-----------|-------------|
| `push` | `pre-commit`, `commit-msg`, `post-commit` | Validates commits and changes pushed to the repository |
| `pull_request` | `pre-commit`, `commit-msg`, `prepare-commit-msg` | Validates pull request changes and commit messages |
| `pull_request_target` | `pre-receive`, `update`, `post-receive` | Validates pull request changes from the target branch context |
| `issues` | `commit-msg` | Validates issue creation and updates |
| `issue_comment` | `commit-msg` | Validates issue comments |
| `release` | `pre-receive`, `post-receive` | Validates release creation and updates |
| `create` | `pre-receive`, `post-receive` | Validates branch and tag creation |
| `delete` | `pre-receive`, `post-receive` | Validates branch and tag deletion |
| `workflow_dispatch` | `pre-commit`, `post-commit` | Manual workflow trigger for validation |
| `workflow_run` | `pre-commit`, `post-commit` | Validates workflow runs |
| `branch_protection_rule` | `pre-receive` | Validates branch protection rule changes |
| `check_run` | `pre-commit`, `post-commit` | Validates check run status |
| `check_suite` | `pre-commit`, `post-commit` | Validates check suite completion |

## Available Stub Binaries

### Core Event Handlers

- `github-push` - Handles push events
- `github-pull-request` - Handles pull request events
- `github-pull-request-target` - Handles pull request target events
- `github-issues` - Handles issue events
- `github-issue-comment` - Handles issue comment events
- `github-release` - Handles release events
- `github-create` - Handles create events (branches/tags)
- `github-delete` - Handles delete events (branches/tags)

### Additional Event Handlers

- `github-branch-protection-rule` - Handles branch protection rule events
- `github-check-run` - Handles check run events
- `github-check-suite` - Handles check suite events
- `github-deployment` - Handles deployment events
- `github-deployment-status` - Handles deployment status events
- `github-discussion` - Handles discussion events
- `github-discussion-comment` - Handles discussion comment events
- `github-fork` - Handles fork events
- `github-gollum` - Handles wiki page events
- `github-label` - Handles label events
- `github-milestone` - Handles milestone events
- `github-page-build` - Handles page build events
- `github-public` - Handles public repository events
- `github-pull-request-review` - Handles pull request review events
- `github-pull-request-review-comment` - Handles pull request review comment events
- `github-registry-package` - Handles package registry events
- `github-repository-dispatch` - Handles custom repository dispatch events
- `github-schedule` - Handles scheduled events
- `github-status` - Handles status events
- `github-watch` - Handles watch/star events
- `github-workflow-dispatch` - Handles manual workflow dispatch events
- `github-workflow-run` - Handles workflow run events

## Usage

### Basic Workflow Example

```yaml
name: Hooksmith Validation

on:
  push:
    branches: [ main, develop ]
  pull_request:
    types: [ opened, synchronize, reopened ]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Build hooksmith
        run: cargo build --release
      
      - name: Run event-specific validation
        run: |
          case ${{ github.event_name }} in
            push)
              ./target/release/github-push
              ;;
            pull_request)
              ./target/release/github-pull-request
              ;;
          esac
        env:
          GITHUB_EVENT_PATH: ${{ github.event_path }}
          GITHUB_EVENT_NAME: ${{ github.event_name }}
          GITHUB_REPOSITORY: ${{ github.repository }}
          GITHUB_REF: ${{ github.ref }}
          GITHUB_SHA: ${{ github.sha }}
      
      - name: Run Git hook validations
        run: |
          # Run pre-commit validation
          ./target/release/pre-commit
          
          # Run commit-msg validation if applicable
          if [ "${{ github.event_name }}" = "push" ] || [ "${{ github.event_name }}" = "pull_request" ]; then
            ./target/release/commit-msg
          fi
          
          # Run post-commit validation
          ./target/release/post-commit
```

### Comprehensive Workflow Example

See `.github/workflows/hooksmith-example.yml` for a complete example that handles multiple event types and includes proper error handling, job summaries, and workflow outputs.

## Workflow Generator

Use the workflow generator to create GitHub Actions workflows automatically:

```bash
# Generate all workflows
cargo run --bin github-workflow-generator -- --all

# Generate a specific event workflow
cargo run --bin github-workflow-generator -- --event push

# Generate workflows with activity type filters
cargo run --bin github-workflow-generator -- --all --with-types
```

## Features

### Workflow Commands Integration

All stub binaries use GitHub Actions workflow commands for better integration:

- `::group::` and `::endgroup::` for collapsible sections
- `::notice::` for informational messages
- `::info::` for detailed information
- `::warning::` for warnings
- `::error::` for errors

### Output Variables

Stub binaries set output variables that can be used in subsequent workflow steps:

```bash
# Example outputs from github-push
push_ref=refs/heads/main
commit_count=3

# Example outputs from github-pull-request
pr_title=Add new feature
pr_body_length=150
files_changed=5
pr_action=opened
```

### Job Summaries

Workflows can generate rich job summaries with Markdown content:

```yaml
- name: Generate validation summary
  run: |
    echo "## Validation Summary" >> $GITHUB_STEP_SUMMARY
    echo "- **Event Type:** ${{ steps.event_info.outputs.type }}" >> $GITHUB_STEP_SUMMARY
    echo "- **Repository:** ${{ github.repository }}" >> $GITHUB_STEP_SUMMARY
    echo "- **Validation Status:** ${{ steps.validation.outputs.passed }}" >> $GITHUB_STEP_SUMMARY
```

## Integration with Hooksmith Schema

The stub binaries are designed to integrate with Hooksmith's validation schema:

1. **Event Payload Parsing** - Extract relevant information from GitHub event payloads
2. **Hook Mapping** - Map GitHub events to appropriate Git hooks
3. **Validation Execution** - Execute the corresponding Git hook validations
4. **Result Reporting** - Report validation results using workflow commands

## Security Considerations

- All stub binaries are no-op by default and require integration with your validation logic
- Use `pull_request_target` carefully as it runs in the context of the base branch
- Consider using `GITHUB_TOKEN` with minimal permissions
- Validate all inputs and sanitize data before processing

## Future Enhancements

- **Schema Integration** - Direct integration with Hooksmith's validation schema
- **Custom Validators** - Support for custom validation rules
- **Artifact Generation** - Generate validation reports as artifacts
- **Notification Integration** - Send validation results to external systems
- **Caching** - Cache validation results for improved performance

## References

- [GitHub Actions Events](https://docs.github.com/en/actions/reference/events-that-trigger-workflows)
- [Workflow Commands](https://docs.github.com/en/actions/reference/workflow-commands-for-github-actions)
- [Contexts and Expression Syntax](https://docs.github.com/en/actions/reference/context-and-expression-syntax-for-github-actions)
