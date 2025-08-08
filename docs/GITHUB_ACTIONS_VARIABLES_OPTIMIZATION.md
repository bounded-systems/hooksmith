# GitHub Actions Variables Optimization

This document explains the optimizations made to the Hooksmith GitHub Actions workflow based on the official GitHub Actions variables reference.

## Overview

The `hooksmith.yml` workflow has been optimized to properly utilize GitHub Actions context variables, environment variables, and workflow commands according to best practices.

## Key Optimizations

### 1. Proper Context Variable Usage

#### Before (Limited Context)
```yaml
- name: Get event information
  run: |
    echo "type=${{ github.event_name }}" >> $GITHUB_OUTPUT
    echo "repository=${{ github.repository }}" >> $GITHUB_OUTPUT
    echo "ref=${{ github.ref }}" >> $GITHUB_OUTPUT
    echo "sha=${{ github.sha }}" >> $GITHUB_OUTPUT
```

#### After (Comprehensive Context)
```yaml
- name: Get event information
  run: |
    # Use proper GitHub context variables
    echo "type=${{ github.event_name }}" >> $GITHUB_OUTPUT
    echo "repository=${{ github.repository }}" >> $GITHUB_OUTPUT
    echo "repository_owner=${{ github.repository_owner }}" >> $GITHUB_OUTPUT
    echo "repository_id=${{ github.repository_id }}" >> $GITHUB_OUTPUT
    echo "ref=${{ github.ref }}" >> $GITHUB_OUTPUT
    echo "ref_name=${{ github.ref_name }}" >> $GITHUB_OUTPUT
    echo "ref_type=${{ github.ref_type }}" >> $GITHUB_OUTPUT
    echo "sha=${{ github.sha }}" >> $GITHUB_OUTPUT
    echo "actor=${{ github.actor }}" >> $GITHUB_OUTPUT
    echo "actor_id=${{ github.actor_id }}" >> $GITHUB_OUTPUT
    echo "triggering_actor=${{ github.triggering_actor }}" >> $GITHUB_OUTPUT
    echo "run_id=${{ github.run_id }}" >> $GITHUB_OUTPUT
    echo "run_number=${{ github.run_number }}" >> $GITHUB_OUTPUT
    echo "run_attempt=${{ github.run_attempt }}" >> $GITHUB_OUTPUT
    echo "workflow=${{ github.workflow }}" >> $GITHUB_OUTPUT
    echo "workflow_ref=${{ github.workflow_ref }}" >> $GITHUB_OUTPUT
    echo "workflow_sha=${{ github.workflow_sha }}" >> $GITHUB_OUTPUT
    
    # Construct workflow run URL using proper variables
    echo "workflow_run_url=${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}" >> $GITHUB_OUTPUT
    
    # Set runner information
    echo "runner_os=${{ runner.os }}" >> $GITHUB_OUTPUT
    echo "runner_arch=${{ runner.arch }}" >> $GITHUB_OUTPUT
    echo "runner_name=${{ runner.name }}" >> $GITHUB_OUTPUT
    echo "runner_environment=${{ runner.environment }}" >> $GITHUB_OUTPUT
```

### 2. Environment Variable Export

#### Before (Limited Environment Variables)
```yaml
env:
  GITHUB_EVENT_PATH: ${{ github.event_path }}
  GITHUB_EVENT_NAME: ${{ github.event_name }}
  GITHUB_REPOSITORY: ${{ github.repository }}
  GITHUB_REF: ${{ github.ref }}
  GITHUB_SHA: ${{ github.sha }}
```

#### After (Comprehensive Environment Variables)
```yaml
run: |
  # Use proper environment variables for event context
  export GITHUB_EVENT_PATH="${{ github.event_path }}"
  export GITHUB_EVENT_NAME="${{ github.event_name }}"
  export GITHUB_REPOSITORY="${{ github.repository }}"
  export GITHUB_REPOSITORY_OWNER="${{ github.repository_owner }}"
  export GITHUB_REPOSITORY_ID="${{ github.repository_id }}"
  export GITHUB_REF="${{ github.ref }}"
  export GITHUB_REF_NAME="${{ github.ref_name }}"
  export GITHUB_REF_TYPE="${{ github.ref_type }}"
  export GITHUB_SHA="${{ github.sha }}"
  export GITHUB_ACTOR="${{ github.actor }}"
  export GITHUB_ACTOR_ID="${{ github.actor_id }}"
  export GITHUB_TRIGGERING_ACTOR="${{ github.triggering_actor }}"
  export GITHUB_RUN_ID="${{ github.run_id }}"
  export GITHUB_RUN_NUMBER="${{ github.run_number }}"
  export GITHUB_WORKFLOW="${{ github.workflow }}"
  export GITHUB_WORKSPACE="${{ github.workspace }}"
```

### 3. Enhanced Logging and Debugging

#### Before (Basic Logging)
```yaml
- name: Check if validation is enabled
  run: |
    if [ "${{ env.ENABLE_HOOKSMITH_VALIDATION }}" = "true" ]; then
      echo "enabled=true" >> $GITHUB_OUTPUT
      echo "::notice::✅ Hooksmith validation is enabled"
    else
      echo "enabled=false" >> $GITHUB_OUTPUT
      echo "::notice::🔇 Hooksmith validation is disabled"
    fi
```

#### After (Enhanced Logging)
```yaml
- name: Check if validation is enabled
  run: |
    # Use proper environment variable handling
    if [ "${{ env.ENABLE_HOOKSMITH_VALIDATION }}" = "true" ]; then
      echo "enabled=true" >> $GITHUB_OUTPUT
      echo "::notice::✅ Hooksmith validation is enabled"
      echo "::info::Repository: ${{ github.repository }}"
      echo "::info::Event: ${{ github.event_name }}"
      echo "::info::Actor: ${{ github.actor }}"
    else
      echo "enabled=false" >> $GITHUB_OUTPUT
      echo "::notice::🔇 Hooksmith validation is disabled"
      echo "::info::Repository: ${{ github.repository }}"
      echo "::info::Event: ${{ github.event_name }}"
      echo "::info::Actor: ${{ github.actor }}"
    fi
```

## GitHub Actions Variables Reference

### Default Environment Variables Used

| Variable | Description | Usage in Hooksmith |
|----------|-------------|-------------------|
| `GITHUB_ACTIONS` | Always set to true when GitHub Actions is running | Used to differentiate local vs CI execution |
| `GITHUB_ACTOR` | Name of person/app that initiated the workflow | Logging and audit trails |
| `GITHUB_ACTOR_ID` | Account ID of the person/app that triggered the workflow | Enhanced security and audit |
| `GITHUB_TRIGGERING_ACTOR` | Username of user that initiated the workflow run | Distinguish between original and re-run actors |
| `GITHUB_EVENT_NAME` | Name of the event that triggered the workflow | Event-specific validation logic |
| `GITHUB_EVENT_PATH` | Path to the file containing the full event webhook payload | Pass event data to Rust binaries |
| `GITHUB_REPOSITORY` | Owner and repository name | Repository context for validation |
| `GITHUB_REPOSITORY_OWNER` | Repository owner's name | Enhanced repository context |
| `GITHUB_REPOSITORY_ID` | ID of the repository | Unique repository identification |
| `GITHUB_REF` | Fully-formed ref that triggered the workflow | Branch/tag context |
| `GITHUB_REF_NAME` | Short ref name of the branch or tag | Simplified branch name |
| `GITHUB_REF_TYPE` | Type of ref that triggered the workflow (branch/tag) | Ref type validation |
| `GITHUB_SHA` | Commit SHA that triggered the workflow | Commit-specific validation |
| `GITHUB_RUN_ID` | Unique number for each workflow run | Run identification |
| `GITHUB_RUN_NUMBER` | Unique number for each run of a particular workflow | Run numbering |
| `GITHUB_RUN_ATTEMPT` | Unique number for each attempt of a particular workflow run | Re-run tracking |
| `GITHUB_WORKFLOW` | Name of the workflow | Workflow identification |
| `GITHUB_WORKFLOW_REF` | Ref path to the workflow | Workflow source tracking |
| `GITHUB_WORKFLOW_SHA` | Commit SHA for the workflow file | Workflow version tracking |
| `GITHUB_WORKSPACE` | Default working directory on the runner | File system operations |
| `GITHUB_SERVER_URL` | URL of the GitHub server | Constructing URLs |
| `RUNNER_OS` | Operating system of the runner | Platform-specific logic |
| `RUNNER_ARCH` | Architecture of the runner | Architecture-specific logic |
| `RUNNER_NAME` | Name of the runner executing the job | Runner identification |
| `RUNNER_ENVIRONMENT` | Environment of the runner (github-hosted/self-hosted) | Environment-specific logic |

### Custom Environment Variables

| Variable | Description | Usage |
|----------|-------------|-------|
| `ENABLE_HOOKSMITH_VALIDATION` | Controls whether validation is enabled | Feature flag for validation |

## Best Practices Implemented

### 1. Context Variable Usage

- **Use `github` context** for workflow run information
- **Use `runner` context** for runner information
- **Use `env` context** for custom environment variables
- **Avoid printing sensitive context** (like `github` context) to logs

### 2. Environment Variable Handling

- **Export variables explicitly** in shell scripts
- **Use proper variable expansion** with quotes
- **Handle missing variables gracefully**
- **Follow naming conventions** (no `GITHUB_` prefix for custom vars)

### 3. Workflow Commands

- **Use `$GITHUB_OUTPUT`** for step outputs
- **Use `$GITHUB_STEP_SUMMARY`** for job summaries
- **Use `$GITHUB_ENV`** for setting environment variables
- **Use `$GITHUB_PATH`** for modifying PATH

### 4. Error Handling

- **Use proper exit codes** in shell scripts
- **Handle missing variables** with fallbacks
- **Log errors appropriately** using workflow commands
- **Provide meaningful error messages**

## Benefits of Optimization

### 1. Enhanced Debugging
- More comprehensive logging
- Better error messages
- Detailed context information

### 2. Improved Security
- Proper actor tracking
- Enhanced audit trails
- Secure variable handling

### 3. Better Performance
- Efficient variable usage
- Reduced redundant calls
- Optimized context access

### 4. Enhanced Reliability
- Proper error handling
- Graceful fallbacks
- Robust variable validation

## Testing the Optimized Workflow

### Local Testing with act
```bash
# Validate workflow syntax
./target/release/hooksmith-tasks act-validate

# Test specific events
act push
act pull_request
act issues
```

### GitHub Actions Testing
```bash
# Enable validation for testing
# Set ENABLE_HOOKSMITH_VALIDATION=true in workflow
```

## Future Enhancements

### 1. Additional Context Variables
- Add support for more event-specific variables
- Implement conditional variable usage
- Add validation for required variables

### 2. Enhanced Logging
- Implement structured logging
- Add performance metrics
- Include timing information

### 3. Security Improvements
- Add variable validation
- Implement secure variable handling
- Add audit logging

### 4. Performance Optimizations
- Cache frequently used variables
- Optimize variable access patterns
- Reduce redundant context calls

## Conclusion

The optimized workflow now properly utilizes GitHub Actions variables according to best practices, providing:

- **Comprehensive context information** for better debugging
- **Enhanced security** with proper actor tracking
- **Improved reliability** with robust error handling
- **Better performance** through efficient variable usage

This optimization ensures the Hooksmith workflow follows GitHub Actions best practices and provides a solid foundation for future enhancements.
