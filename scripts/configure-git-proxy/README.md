# Configure Git Proxy

This script configures your local Git installation to use the Git proxy server for all HTTPS Git operations.

## Usage

```bash
# Run the configuration script
cargo run --bin configure-git-proxy
```

## What it does

1. **Checks if the proxy is running** on port 8080
2. **Configures Git** to use the proxy for:
   - GitHub repositories (`https://github.com/`)
   - GitLab repositories (`https://gitlab.com/`)
   - Any HTTPS Git server (`https://`)
3. **Tests the configuration** with sample repositories
4. **Provides instructions** for testing and disabling the proxy

## Configuration

The script sets up Git URL rewriting:

```bash
# GitHub repositories
git config --global url."http://127.0.0.1:8080/".insteadOf "https://github.com/"

# GitLab repositories  
git config --global url."http://127.0.0.1:8080/".insteadOf "https://gitlab.com/"

# Any HTTPS Git server
git config --global url."http://127.0.0.1:8080/".insteadOf "https://"
```

## Testing

After configuration, all Git operations will go through the proxy:

```bash
# This will go through your proxy
git clone https://github.com/username/repo.git

# This will also go through your proxy
git push origin main
```

## Disabling the proxy

To disable the proxy configuration:

```bash
git config --global --unset url.http://127.0.0.1:8080/.insteadOf
```

## Requirements

- Git proxy server running on `http://127.0.0.1:8080`
- `curl` command available
- Git installed and configured

## Benefits

- **Validation**: All Git operations are validated by the proxy
- **Hooks**: Server-side hooks are executed for every operation
- **Security**: File size limits, blocked patterns, and protected branches are enforced
- **Monitoring**: All Git operations are logged and monitored
