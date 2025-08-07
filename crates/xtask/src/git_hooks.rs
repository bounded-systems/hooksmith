use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git hook entry with metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHook {
    /// Hook name (e.g., "pre-commit", "commit-msg")
    pub name: String,
    /// Rust binary name (e.g., "hook-pre-commit")
    pub binary_name: String,
    /// Rust source file path
    pub source_path: String,
    /// Git alias name
    pub alias_name: String,
    /// Whether this hook is enabled
    pub enabled: bool,
    /// Hook description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Hook dependencies (other hooks that must run first)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,
    /// Hook configuration
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub config: HashMap<String, String>,
}

/// Complete Git hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHooksConfig {
    /// All hook definitions
    pub hooks: Vec<GitHook>,
    /// Hooks directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks_dir: Option<String>,
    /// Binaries directory path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin_dir: Option<String>,
    /// Whether to auto-install hooks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_install: Option<bool>,
    /// Global configuration
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub global_config: HashMap<String, String>,
}

/// Standard Git hooks that can be implemented
pub const STANDARD_HOOKS: &[&str] = &[
    "applypatch-msg",
    "pre-applypatch", 
    "post-applypatch",
    "pre-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "post-receive",
    "post-update",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
];

impl GitHooksConfig {
    /// Create a new hooks configuration
    pub fn new() -> Self {
        Self {
            hooks: Vec::new(),
            hooks_dir: Some(".git/hooks".to_string()),
            bin_dir: Some(".git/hooks/bin".to_string()),
            auto_install: Some(true),
            global_config: HashMap::new(),
        }
    }

    /// Generate standard hooks configuration
    pub fn generate_standard_hooks() -> Self {
        let mut config = Self::new();
        
        for hook_name in STANDARD_HOOKS {
            let binary_name = format!("hook-{}", hook_name.replace('-', "_"));
            let source_path = format!("hooks/{}.rs", hook_name.replace('-', "_"));
            let alias_name = format!("hook-{}", hook_name);
            
            config.hooks.push(GitHook {
                name: hook_name.to_string(),
                binary_name,
                source_path,
                alias_name,
                enabled: true,
                description: Some(format!("Standard {} hook", hook_name)),
                dependencies: Vec::new(),
                config: HashMap::new(),
            });
        }
        
        config
    }

    /// Add a custom hook
    pub fn add_hook(&mut self, hook: GitHook) {
        self.hooks.push(hook);
    }

    /// Find a hook by name
    pub fn find_hook(&self, name: &str) -> Option<&GitHook> {
        self.hooks.iter().find(|h| h.name == name)
    }

    /// Get enabled hooks
    pub fn enabled_hooks(&self) -> Vec<&GitHook> {
        self.hooks.iter().filter(|h| h.enabled).collect()
    }

    /// Validate hooks configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for hook in &self.hooks {
            // Check for duplicate names
            let count = self.hooks.iter().filter(|h| h.name == hook.name).count();
            if count > 1 {
                errors.push(format!("Duplicate hook name: {}", hook.name));
            }
            
            // Check for valid binary names
            if hook.binary_name.is_empty() {
                errors.push(format!("Empty binary name for hook: {}", hook.name));
            }
            
            // Check for valid source paths
            if hook.source_path.is_empty() {
                errors.push(format!("Empty source path for hook: {}", hook.name));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Convert to JSONC format
    pub fn to_jsonc(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut jsonc = serde_json::to_string_pretty(&self)?;
        
        // Add schema and documentation
        let schema = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Git Hooks Configuration",
  "description": "Configuration for Git hooks with Rust binaries and Git aliases",
  "type": "object",
  "properties": {
    "hooks": {
      "type": "array",
      "description": "Git hook definitions",
      "items": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "description": "Hook name (e.g., pre-commit)"
          },
          "binary_name": {
            "type": "string",
            "description": "Rust binary name (e.g., hook-pre-commit)"
          },
          "source_path": {
            "type": "string",
            "description": "Rust source file path"
          },
          "alias_name": {
            "type": "string",
            "description": "Git alias name"
          },
          "enabled": {
            "type": "boolean",
            "description": "Whether this hook is enabled"
          },
          "description": {
            "type": "string",
            "description": "Hook description"
          },
          "dependencies": {
            "type": "array",
            "description": "Hook dependencies",
            "items": {
              "type": "string"
            }
          },
          "config": {
            "type": "object",
            "description": "Hook configuration"
          }
        }
      }
    },
    "hooks_dir": {
      "type": "string",
      "description": "Hooks directory path"
    },
    "bin_dir": {
      "type": "string",
      "description": "Binaries directory path"
    },
    "auto_install": {
      "type": "boolean",
      "description": "Whether to auto-install hooks"
    },
    "global_config": {
      "type": "object",
      "description": "Global configuration"
    }
  }
}"#;

        // Replace the opening brace with schema and documentation
        jsonc = jsonc.replacen("{", &format!("{}{{", schema), 1);
        
        Ok(jsonc)
    }
}

/// Git hooks manager for installation and management
pub struct GitHooksManager;

impl GitHooksManager {
    /// Install all hooks from configuration
    pub fn install_hooks(config: &GitHooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        // Validate configuration
        config.validate()?;
        
        // Create directories
        let hooks_dir = Path::new(&config.hooks_dir.as_ref().unwrap_or(&".git/hooks".to_string()));
        let bin_dir = Path::new(&config.bin_dir.as_ref().unwrap_or(&".git/hooks/bin".to_string()));
        
        fs::create_dir_all(hooks_dir)?;
        fs::create_dir_all(bin_dir)?;
        
        // Build and install binaries
        for hook in &config.hooks {
            if hook.enabled {
                Self::install_hook(hook, hooks_dir, bin_dir)?;
            }
        }
        
        // Register Git aliases
        Self::register_git_aliases(config)?;
        
        println!("✅ Git hooks installed successfully");
        Ok(())
    }

    /// Install a single hook
    fn install_hook(hook: &GitHook, hooks_dir: &Path, bin_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Build the Rust binary
        let binary_path = bin_dir.join(&hook.binary_name);
        
        // For now, we'll create a simple stub binary
        // In a real implementation, you'd compile the actual Rust code
        let stub_content = format!(
            r#"#!/usr/bin/env rustc
fn main() {{
    println!("Running {} hook...");
    // TODO: Implement actual hook logic
    std::process::exit(0);
}}"#,
            hook.name
        );
        
        fs::write(&binary_path, stub_content)?;
        
        // Make binary executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }
        
        // Create hook stub
        let hook_stub_path = hooks_dir.join(&hook.name);
        let stub_content = format!(
            "exec git {}\"$@\"\n",
            hook.alias_name
        );
        
        fs::write(&hook_stub_path, stub_content)?;
        
        // Make hook stub executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_stub_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_stub_path, perms)?;
        }
        
        Ok(())
    }

    /// Register Git aliases for all hooks
    fn register_git_aliases(config: &GitHooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        for hook in &config.hooks {
            if hook.enabled {
                let bin_dir = config.bin_dir.as_ref().unwrap_or(&".git/hooks/bin".to_string());
                let alias_value = format!("!./{}/{}", bin_dir, hook.binary_name);
                
                // Set Git alias
                let status = Command::new("git")
                    .args(["config", "--local", &format!("alias.{}", hook.alias_name), &alias_value])
                    .status()?;
                
                if !status.success() {
                    eprintln!("Warning: Failed to register alias for hook: {}", hook.name);
                }
            }
        }
        
        Ok(())
    }

    /// Uninstall all hooks
    pub fn uninstall_hooks(config: &GitHooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        let hooks_dir = Path::new(&config.hooks_dir.as_ref().unwrap_or(&".git/hooks".to_string()));
        let bin_dir = Path::new(&config.bin_dir.as_ref().unwrap_or(&".git/hooks/bin".to_string()));
        
        // Remove hook stubs
        for hook in &config.hooks {
            let hook_path = hooks_dir.join(&hook.name);
            if hook_path.exists() {
                fs::remove_file(&hook_path)?;
            }
        }
        
        // Remove binaries
        if bin_dir.exists() {
            fs::remove_dir_all(bin_dir)?;
        }
        
        // Remove Git aliases
        for hook in &config.hooks {
            let status = Command::new("git")
                .args(["config", "--local", "--unset", &format!("alias.{}", hook.alias_name)])
                .status();
            
            // Ignore errors for unset (alias might not exist)
            if let Ok(status) = status {
                if !status.success() {
                    eprintln!("Warning: Failed to unset alias for hook: {}", hook.name);
                }
            }
        }
        
        println!("✅ Git hooks uninstalled successfully");
        Ok(())
    }

    /// List installed hooks
    pub fn list_hooks(config: &GitHooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        let hooks_dir = Path::new(&config.hooks_dir.as_ref().unwrap_or(&".git/hooks".to_string()));
        let bin_dir = Path::new(&config.bin_dir.as_ref().unwrap_or(&".git/hooks/bin".to_string()));
        
        println!("## Installed Git Hooks\n");
        
        for hook in &config.hooks {
            let hook_path = hooks_dir.join(&hook.name);
            let binary_path = bin_dir.join(&hook.binary_name);
            
            let hook_status = if hook_path.exists() { "✅" } else { "❌" };
            let binary_status = if binary_path.exists() { "✅" } else { "❌" };
            
            println!("{} {}: {} (binary: {})", 
                hook_status, 
                hook.name, 
                binary_status,
                hook.binary_name
            );
            
            if let Some(desc) = &hook.description {
                println!("  Description: {}", desc);
            }
        }
        
        Ok(())
    }

    /// Generate Cargo.toml binary entries
    pub fn generate_cargo_binaries(config: &GitHooksConfig) -> String {
        let mut output = String::new();
        output.push_str("# Git Hook Binaries\n");
        output.push_str("# Generated by git-hooks-manager\n\n");
        
        for hook in &config.hooks {
            if hook.enabled {
                output.push_str(&format!(
                    "[[bin]]\nname = \"{}\"\npath = \"{}\"\n\n",
                    hook.binary_name, hook.source_path
                ));
            }
        }
        
        output
    }

    /// Generate hook source templates
    pub fn generate_hook_templates(config: &GitHooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        for hook in &config.hooks {
            if hook.enabled {
                let template = Self::generate_hook_template(hook);
                let source_path = Path::new(&hook.source_path);
                
                // Create parent directory if it doesn't exist
                if let Some(parent) = source_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::write(source_path, template)?;
            }
        }
        
        println!("✅ Hook source templates generated successfully");
        Ok(())
    }

    /// Generate a hook source template
    fn generate_hook_template(hook: &GitHook) -> String {
        format!(
            r#"//! {} Hook
//! 
//! This hook is automatically called by Git when {} occurs.
//! 
//! Exit codes:
//! - 0: Success, allow the operation to proceed
//! - 1: Failure, abort the operation

use std::env;
use std::process;

fn main() {{
    println!("Running {} hook...");
    
    // TODO: Implement hook logic here
    // 
    // Examples:
    // - Run linting: cargo clippy
    // - Run tests: cargo test
    // - Validate commit message format
    // - Check for sensitive data
    // - Run security scans
    
    // Get hook arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {{
        println!("Hook arguments: {{:?}}", &args[1..]);
    }}
    
    // Example: Always succeed for now
    // Replace with actual validation logic
    println!("✅ {} hook completed successfully");
    process::exit(0);
    
    // Example: Fail on certain conditions
    // if some_condition {{
    //     eprintln!("❌ {} hook failed: reason");
    //     process::exit(1);
    // }}
}}"#,
            hook.name.to_title_case(),
            hook.name,
            hook.name,
            hook.name,
            hook.name
        )
    }

    /// Test a specific hook
    pub fn test_hook(hook_name: &str, config: &GitHooksConfig) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(hook) = config.find_hook(hook_name) {
            let bin_dir = config.bin_dir.as_ref().unwrap_or(&".git/hooks/bin".to_string());
            let binary_path = format!("{}/{}", bin_dir, hook.binary_name);
            
            println!("Testing hook: {} ({})", hook.name, hook.binary_name);
            
            let output = Command::new(&binary_path)
                .args(["test"])
                .output()?;
            
            println!("Exit code: {}", output.status);
            println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
            
            Ok(())
        } else {
            Err(format!("Hook '{}' not found in configuration", hook_name).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_standard_hooks() {
        let config = GitHooksConfig::generate_standard_hooks();
        assert_eq!(config.hooks.len(), STANDARD_HOOKS.len());
        
        let pre_commit = config.find_hook("pre-commit").unwrap();
        assert_eq!(pre_commit.binary_name, "hook-pre_commit");
        assert_eq!(pre_commit.alias_name, "hook-pre-commit");
    }

    #[test]
    fn test_validate_hooks() {
        let mut config = GitHooksConfig::new();
        config.add_hook(GitHook {
            name: "test".to_string(),
            binary_name: "hook-test".to_string(),
            source_path: "hooks/test.rs".to_string(),
            alias_name: "hook-test".to_string(),
            enabled: true,
            description: None,
            dependencies: Vec::new(),
            config: HashMap::new(),
        });
        
        assert!(config.validate().is_ok());
    }
}
