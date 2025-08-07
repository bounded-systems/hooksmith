//! Configuration management for Git proxy

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

use crate::{GitProxyConfig, ProxyAuthConfig, ValidationConfig, ServerConfig, LoggingConfig};

/// Configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    /// Upstream repository URL
    pub upstream_url: String,
    /// Proxy repository path
    pub proxy_repo_path: String,
    /// Authentication settings
    pub auth: AuthConfigFile,
    /// Validation settings
    pub validation: ValidationConfigFile,
    /// Server settings
    pub server: ServerConfigFile,
    /// Logging settings
    pub logging: LoggingConfigFile,
}

/// Authentication configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfigFile {
    /// GitHub personal access token
    pub github_token: Option<String>,
    /// SSH key path
    pub ssh_key_path: Option<String>,
    /// Username
    pub username: Option<String>,
}

/// Validation configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfigFile {
    /// Enable pre-push validation
    pub enable_pre_push: Option<bool>,
    /// Enable commit message validation
    pub enable_commit_validation: Option<bool>,
    /// Enable file size validation
    pub enable_file_size_validation: Option<bool>,
    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
    /// Blocked file patterns
    pub blocked_patterns: Option<Vec<String>>,
    /// Required commit message patterns
    pub required_patterns: Option<Vec<String>>,
}

/// Server configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfigFile {
    /// HTTP server port
    pub http_port: Option<u16>,
    /// HTTP server host
    pub http_host: Option<String>,
    /// SSH server port
    pub ssh_port: Option<u16>,
    /// SSH server host
    pub ssh_host: Option<String>,
    /// Enable HTTP server
    pub enable_http: Option<bool>,
    /// Enable SSH server
    pub enable_ssh: Option<bool>,
}

/// Logging configuration file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfigFile {
    /// Log level
    pub level: Option<String>,
    /// Log file path
    pub file_path: Option<String>,
    /// Enable structured logging
    pub structured: Option<bool>,
}

impl ConfigFile {
    /// Load configuration from a file
    pub fn load_from_file(path: &PathBuf) -> Result<GitProxyConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        
        let config_file: ConfigFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        
        config_file.into_git_proxy_config()
    }
    
    /// Save configuration to a file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config to TOML")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        
        Ok(())
    }
    
    /// Convert to GitProxyConfig
    fn into_git_proxy_config(self) -> Result<GitProxyConfig> {
        Ok(GitProxyConfig {
            upstream_url: self.upstream_url,
            proxy_repo_path: PathBuf::from(self.proxy_repo_path),
            auth: ProxyAuthConfig {
                github_token: self.auth.github_token,
                ssh_key_path: self.auth.ssh_key_path.map(PathBuf::from),
                username: self.auth.username,
            },
            validation: ValidationConfig {
                enable_pre_push: self.validation.enable_pre_push.unwrap_or(true),
                enable_commit_validation: self.validation.enable_commit_validation.unwrap_or(true),
                enable_file_size_validation: self.validation.enable_file_size_validation.unwrap_or(true),
                max_file_size: self.validation.max_file_size,
                blocked_patterns: self.validation.blocked_patterns.unwrap_or_default(),
                required_patterns: self.validation.required_patterns.unwrap_or_default(),
            },
            server: ServerConfig {
                http_port: self.server.http_port.unwrap_or(8080),
                http_host: self.server.http_host.unwrap_or_else(|| "127.0.0.1".to_string()),
                ssh_port: self.server.ssh_port.unwrap_or(2222),
                ssh_host: self.server.ssh_host.unwrap_or_else(|| "127.0.0.1".to_string()),
                enable_http: self.server.enable_http.unwrap_or(true),
                enable_ssh: self.server.enable_ssh.unwrap_or(false),
            },
            logging: LoggingConfig {
                level: self.logging.level.unwrap_or_else(|| "info".to_string()),
                file_path: self.logging.file_path.map(PathBuf::from),
                structured: self.logging.structured.unwrap_or(true),
            },
        })
    }
}

impl GitProxyConfig {
    /// Create a default configuration
    pub fn default() -> Self {
        Self {
            upstream_url: "https://github.com/user/repo.git".to_string(),
            proxy_repo_path: PathBuf::from("./proxy.git"),
            auth: ProxyAuthConfig {
                github_token: None,
                ssh_key_path: None,
                username: None,
            },
            validation: ValidationConfig {
                enable_pre_push: true,
                enable_commit_validation: true,
                enable_file_size_validation: true,
                max_file_size: Some(10 * 1024 * 1024), // 10MB
                blocked_patterns: vec![
                    "*.key".to_string(),
                    "*.pem".to_string(),
                    "*.p12".to_string(),
                    "*.pfx".to_string(),
                    "id_rsa".to_string(),
                    "id_ed25519".to_string(),
                ],
                required_patterns: vec![
                    "feat:".to_string(),
                    "fix:".to_string(),
                    "docs:".to_string(),
                    "style:".to_string(),
                    "refactor:".to_string(),
                    "test:".to_string(),
                    "chore:".to_string(),
                ],
            },
            server: ServerConfig {
                http_port: 8080,
                http_host: "127.0.0.1".to_string(),
                ssh_port: 2222,
                ssh_host: "127.0.0.1".to_string(),
                enable_http: true,
                enable_ssh: false,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                structured: true,
            },
        }
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate upstream URL
        if self.upstream_url.is_empty() {
            anyhow::bail!("Upstream URL cannot be empty");
        }
        
        // Validate proxy repo path
        if self.proxy_repo_path.to_string_lossy().is_empty() {
            anyhow::bail!("Proxy repository path cannot be empty");
        }
        
        // Validate server configuration
        if !self.server.enable_http && !self.server.enable_ssh {
            anyhow::bail!("At least one server protocol must be enabled");
        }
        
        // Validate authentication
        if self.auth.github_token.is_none() && self.auth.ssh_key_path.is_none() {
            anyhow::bail!("Either GitHub token or SSH key must be provided");
        }
        
        // Validate validation configuration
        if self.validation.max_file_size == Some(0) {
            anyhow::bail!("Maximum file size cannot be zero");
        }
        
        Ok(())
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();
        
        // Override with environment variables
        if let Ok(url) = std::env::var("GIT_PROXY_UPSTREAM_URL") {
            config.upstream_url = url;
        }
        
        if let Ok(path) = std::env::var("GIT_PROXY_REPO_PATH") {
            config.proxy_repo_path = PathBuf::from(path);
        }
        
        if let Ok(token) = std::env::var("GIT_PROXY_GITHUB_TOKEN") {
            config.auth.github_token = Some(token);
        }
        
        if let Ok(key_path) = std::env::var("GIT_PROXY_SSH_KEY_PATH") {
            config.auth.ssh_key_path = Some(PathBuf::from(key_path));
        }
        
        if let Ok(username) = std::env::var("GIT_PROXY_USERNAME") {
            config.auth.username = Some(username);
        }
        
        if let Ok(port) = std::env::var("GIT_PROXY_HTTP_PORT") {
            config.server.http_port = port.parse()?;
        }
        
        if let Ok(host) = std::env::var("GIT_PROXY_HTTP_HOST") {
            config.server.http_host = host;
        }
        
        if let Ok(level) = std::env::var("GIT_PROXY_LOG_LEVEL") {
            config.logging.level = level;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to a file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let config_file = ConfigFile {
            upstream_url: self.upstream_url.clone(),
            proxy_repo_path: self.proxy_repo_path.to_string_lossy().to_string(),
            auth: AuthConfigFile {
                github_token: self.auth.github_token.clone(),
                ssh_key_path: self.auth.ssh_key_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                username: self.auth.username.clone(),
            },
            validation: ValidationConfigFile {
                enable_pre_push: Some(self.validation.enable_pre_push),
                enable_commit_validation: Some(self.validation.enable_commit_validation),
                enable_file_size_validation: Some(self.validation.enable_file_size_validation),
                max_file_size: self.validation.max_file_size,
                blocked_patterns: Some(self.validation.blocked_patterns.clone()),
                required_patterns: Some(self.validation.required_patterns.clone()),
            },
            server: ServerConfigFile {
                http_port: Some(self.server.http_port),
                http_host: Some(self.server.http_host.clone()),
                ssh_port: Some(self.server.ssh_port),
                ssh_host: Some(self.server.ssh_host.clone()),
                enable_http: Some(self.server.enable_http),
                enable_ssh: Some(self.server.enable_ssh),
            },
            logging: LoggingConfigFile {
                level: Some(self.logging.level.clone()),
                file_path: self.logging.file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
                structured: Some(self.logging.structured),
            },
        };
        
        config_file.save_to_file(path)
    }
}

/// Configuration manager
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }
    
    /// Load configuration
    pub fn load(&self) -> Result<GitProxyConfig> {
        if self.config_path.exists() {
            ConfigFile::load_from_file(&self.config_path)
        } else {
            // Try environment variables
            GitProxyConfig::from_env()
        }
    }
    
    /// Save configuration
    pub fn save(&self, config: &GitProxyConfig) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        config.save_to_file(&self.config_path)
    }
    
    /// Create default configuration
    pub fn create_default(&self) -> Result<GitProxyConfig> {
        let config = GitProxyConfig::default();
        self.save(&config)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_default_config() {
        let config = GitProxyConfig::default();
        config.validate().unwrap();
        
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.server.http_host, "127.0.0.1");
        assert!(config.validation.enable_pre_push);
        assert!(config.validation.enable_commit_validation);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = GitProxyConfig::default();
        config.validate().unwrap();
        
        // Test invalid upstream URL
        config.upstream_url = "".to_string();
        assert!(config.validate().is_err());
        
        // Test invalid server config
        config.upstream_url = "https://github.com/user/repo.git".to_string();
        config.server.enable_http = false;
        config.server.enable_ssh = false;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_file_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config = GitProxyConfig::default();
        config.save_to_file(&config_path).unwrap();
        
        let loaded_config = ConfigFile::load_from_file(&config_path).unwrap();
        assert_eq!(config.upstream_url, loaded_config.upstream_url);
        assert_eq!(config.server.http_port, loaded_config.server.http_port);
    }
}
