# Git Proxy Component

A comprehensive Git proxy server that can intercept, validate, and forward Git operations to upstream repositories like GitHub. It acts as a middleware layer between Git clients and the upstream repository, providing validation, auditing, and control capabilities.

## 🎯 Features

### Core Functionality
- **Git Protocol Support**: HTTP Smart Protocol and SSH protocol support
- **Upstream Forwarding**: Seamlessly forwards operations to GitHub or other Git hosts
- **Validation Engine**: Comprehensive validation rules for commits, files, and operations
- **Periodic Sync**: Monitors upstream for changes and maintains consistency
- **Audit Logging**: Detailed logging of all operations and validation results

### Validation Capabilities
- **Commit Message Validation**: Enforce commit message patterns (e.g., conventional commits)
- **File Size Limits**: Prevent large files from being pushed
- **Blocked File Patterns**: Block sensitive files (keys, certificates, etc.)
- **Protected Branch Rules**: Prevent direct pushes to main/master branches
- **Force Push Detection**: Monitor and control force push operations

### Security Features
- **Authentication**: GitHub token and SSH key support
- **Access Control**: Configurable access rules and restrictions
- **Audit Trail**: Complete audit logging of all operations
- **Drift Detection**: Monitor for unauthorized upstream changes

## 🏗️ Architecture

```
[Git Client] → [Git Proxy Server] → [GitHub/Upstream]
                    ↓
            [Validation Engine]
                    ↓
            [Sync Manager]
                    ↓
            [Event Bus]
```

### Components

1. **HTTP/SSH Server**: Handles incoming Git protocol requests
2. **Protocol Handlers**: Manage Git protocol operations (HTTP/SSH)
3. **Validation Engine**: Enforces validation rules and policies
4. **Sync Manager**: Periodically syncs with upstream repository
5. **Configuration Manager**: Manages server configuration and settings

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ and Cargo
- Git installed
- GitHub personal access token (for HTTPS) or SSH key (for SSH)

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd hooksmith

# Build the git-proxy component
cargo build -p git-proxy

# Run the server
cargo run -p git-proxy --bin server
```

### Basic Configuration

Create a `git-proxy.toml` configuration file:

```toml
# Upstream repository URL
upstream_url = "https://github.com/user/repo.git"

# Local proxy repository path
proxy_repo_path = "./proxy.git"

[auth]
# GitHub personal access token
github_token = "ghp_your_token_here"
# Or SSH key path
# ssh_key_path = "~/.ssh/id_rsa"

[validation]
# Enable validation features
enable_pre_push = true
enable_commit_validation = true
enable_file_size_validation = true

# Maximum file size (10MB)
max_file_size = 10485760

# Blocked file patterns
blocked_patterns = [
    "*.key",
    "*.pem",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_ed25519"
]

# Required commit message patterns
required_patterns = [
    "feat:",
    "fix:",
    "docs:",
    "style:",
    "refactor:",
    "test:",
    "chore:"
]

[server]
# HTTP server configuration
http_port = 8080
http_host = "127.0.0.1"
enable_http = true

# SSH server configuration
ssh_port = 2222
ssh_host = "127.0.0.1"
enable_ssh = false

[logging]
level = "info"
structured = true
```

### Running the Server

```bash
# Basic run with default configuration
cargo run -p git-proxy --bin server

# With custom configuration
cargo run -p git-proxy --bin server --config my-config.toml

# With command line overrides
cargo run -p git-proxy --bin server \
    --upstream-url "https://github.com/user/repo.git" \
    --github-token "ghp_your_token_here" \
    --http-port 8080 \
    --enable-http

# Dry run mode (no actual operations)
cargo run -p git-proxy --bin server --dry-run
```

## 📋 Usage Examples

### 1. Basic Git Proxy Setup

```bash
# Start the proxy server
cargo run -p git-proxy --bin server \
    --upstream-url "https://github.com/user/repo.git" \
    --github-token "ghp_your_token_here"

# Configure your local repository to use the proxy
git remote set-url origin http://127.0.0.1:8080/repo.git
```

### 2. Validation Rules

The proxy enforces various validation rules:

```toml
[validation]
# Conventional commit messages
required_patterns = ["feat:", "fix:", "docs:", "style:", "refactor:", "test:", "chore:"]

# File size limits
max_file_size = 10485760  # 10MB

# Blocked file types
blocked_patterns = ["*.key", "*.pem", "*.p12", "*.pfx", "id_rsa", "id_ed25519"]

# Protected branches
protected_branches = ["main", "master", "develop", "production"]
```

### 3. Periodic Sync Configuration

```toml
[sync]
# Sync every 5 minutes
interval_seconds = 300

# Enable automatic sync
auto_sync = true

# Detect force pushes and deletions
detect_force_pushes = true
detect_deletions = true

# Track PR branches
track_pr_branches = true
```

### 4. HTTP Server Endpoints

The proxy provides several HTTP endpoints:

- `GET /health` - Health check
- `GET /status` - Server status
- `GET /info/refs` - Git info/refs endpoint
- `POST /git-upload-pack` - Git upload-pack endpoint
- `POST /git-receive-pack` - Git receive-pack endpoint

## 🔧 Configuration

### Environment Variables

You can also configure the proxy using environment variables:

```bash
export GIT_PROXY_UPSTREAM_URL="https://github.com/user/repo.git"
export GIT_PROXY_GITHUB_TOKEN="ghp_your_token_here"
export GIT_PROXY_HTTP_PORT="8080"
export GIT_PROXY_LOG_LEVEL="info"
```

### Advanced Configuration

```toml
# Advanced validation rules
[validation]
enable_pre_push = true
enable_commit_validation = true
enable_file_size_validation = true
max_file_size = 10485760

# Custom validation patterns
blocked_patterns = [
    "*.key",
    "*.pem",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_ed25519",
    "secrets.json",
    ".env"
]

required_patterns = [
    "feat:",
    "fix:",
    "docs:",
    "style:",
    "refactor:",
    "test:",
    "chore:",
    "BREAKING CHANGE:"
]

# Server configuration
[server]
http_port = 8080
http_host = "127.0.0.1"
ssh_port = 2222
ssh_host = "127.0.0.1"
enable_http = true
enable_ssh = false

# Logging configuration
[logging]
level = "info"
file_path = "logs/git-proxy.log"
structured = true
```

## 🔍 Monitoring and Debugging

### Log Levels

- `error` - Only error messages
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debug information
- `trace` - Very detailed trace information

### Health Checks

```bash
# Check server health
curl http://127.0.0.1:8080/health

# Get server status
curl http://127.0.0.1:8080/status
```

### Validation Monitoring

The proxy logs all validation results:

```bash
# View validation logs
tail -f logs/git-proxy.log | grep "validation"
```

## 🛠️ Development

### Building

```bash
# Build the component
cargo build -p git-proxy

# Build with optimizations
cargo build -p git-proxy --release

# Run tests
cargo test -p git-proxy
```

### Testing

```bash
# Run all tests
cargo test -p git-proxy

# Run specific test
cargo test -p git-proxy test_validation_engine_creation

# Run with verbose output
cargo test -p git-proxy -- --nocapture
```

### Integration Testing

```bash
# Start the server for testing
cargo run -p git-proxy --bin server --dry-run

# Test with a real repository
git clone http://127.0.0.1:8080/repo.git test-repo
cd test-repo
echo "test" > test.txt
git add test.txt
git commit -m "feat: add test file"
git push origin main
```

## 🔒 Security Considerations

### Authentication

- Use GitHub personal access tokens for HTTPS
- Use SSH keys for SSH connections
- Store credentials securely
- Rotate tokens regularly

### Network Security

- Run the proxy on a secure network
- Use HTTPS for external access
- Configure firewall rules appropriately
- Monitor for unauthorized access

### Validation Rules

- Regularly review and update validation rules
- Monitor validation failures
- Adjust rules based on team needs
- Document rule changes

## 📊 Performance

### Optimization Tips

1. **Repository Size**: Keep proxy repositories reasonably sized
2. **Sync Interval**: Adjust sync frequency based on activity
3. **File Size Limits**: Set appropriate file size limits
4. **Logging**: Use appropriate log levels for production

### Monitoring

- Monitor server resource usage
- Track validation performance
- Monitor sync operations
- Watch for validation failures

## 🤝 Contributing

### Development Setup

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Style

- Follow Rust coding conventions
- Add comprehensive tests
- Update documentation
- Use meaningful commit messages

## 📄 License

This component is part of the Hooksmith project and is licensed under the MIT License.

## 🆘 Support

For issues and questions:

1. Check the documentation
2. Review existing issues
3. Create a new issue with details
4. Provide logs and configuration

## 🔄 Changelog

### v0.1.0
- Initial implementation
- HTTP and SSH protocol support
- Validation engine
- Sync manager
- Configuration management
- Basic server functionality
