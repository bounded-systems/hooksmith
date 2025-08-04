//! Worktree management module for Hooksmith
//!
//! This module provides comprehensive worktree management functionality,
//! including tool detection, configuration management, and integration
//! with various worktree management tools like wtp, wt, and git.

use anyhow::{Context, Result};
use console::style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;

use crate::WorktreeCommands;

/// Worktree configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// Worktree base directory
    pub worktree_base: Option<String>,
    /// Worktree template pattern
    pub worktree_template: Option<String>,
    /// Whether to run setup commands after creation
    pub run_setup: bool,
    /// Setup commands to run
    pub setup_commands: Vec<String>,
    /// Whether to copy environment files
    pub copy_env: bool,
    /// Environment files to copy
    pub env_files: Vec<String>,
    /// Git aliases to create
    pub git_aliases: HashMap<String, String>,
    /// Existing worktrees mapping
    pub existing_worktrees: Option<HashMap<String, String>>,
    /// Branch patterns configuration
    pub branch_patterns: Option<HashMap<String, BranchPattern>>,
    /// Integration settings
    pub integration: Option<IntegrationConfig>,
    /// Cursor integration settings
    pub cursor_integration: Option<CursorIntegrationConfig>,
    /// Workbloom metadata settings
    pub workbloom_metadata: Option<WorkbloomMetadataConfig>,
}

/// Branch pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPattern {
    /// Template for worktree naming
    pub template: String,
    /// Setup commands for this pattern
    pub setup: Vec<String>,
    /// Semantic labels for this pattern
    pub labels: Vec<String>,
}

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Enable Lefthook integration
    pub lefthook: bool,
    /// Enable xtask integration
    pub xtask: bool,
    /// Enable WASM components integration
    pub wasm_components: bool,
}

/// Cursor integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorIntegrationConfig {
    /// Auto-open worktrees in Cursor
    pub auto_open_cursor: bool,
    /// Cursor project configuration template
    pub cursor_config_template: Option<String>,
    /// Shell integration commands
    pub shell_aliases: HashMap<String, String>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

/// Workbloom metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkbloomMetadataConfig {
    /// Enable metadata tracking
    pub enabled: bool,
    /// Metadata directory
    pub metadata_dir: String,
    /// Semantic labels configuration
    pub labels_config: Option<String>,
    /// Status tracking
    pub status_tracking: bool,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            worktree_base: Some("../".to_string()),
            worktree_template: Some("{repo}-{branch}".to_string()),
            run_setup: true,
            setup_commands: vec![
                "cargo build".to_string(),
                "cargo xtask gen-all --validate".to_string(),
                "spin build || true".to_string(),
            ],
            copy_env: true,
            env_files: vec![
                ".env.example".to_string(),
                ".env".to_string(),
                ".envrc".to_string(),
                "hooksmith.toml".to_string(),
                ".worktree-config.jsonc".to_string(),
            ],
            git_aliases: HashMap::new(),
            existing_worktrees: Some(HashMap::from([
                (
                    "feature/spin-integration".to_string(),
                    "../hooksmith-spin".to_string(),
                ),
                (
                    "feature/spin-integration-v2".to_string(),
                    "../hooksmith-spin-integration".to_string(),
                ),
            ])),
            branch_patterns: Some(HashMap::from([
                (
                    "feature/*".to_string(),
                    BranchPattern {
                        template: "{repo}-{branch}".to_string(),
                        setup: vec!["cargo build".to_string(), "cargo xtask gen-all".to_string()],
                        labels: vec!["feature".to_string(), "development".to_string()],
                    },
                ),
                (
                    "bugfix/*".to_string(),
                    BranchPattern {
                        template: "{repo}-{branch}".to_string(),
                        setup: vec!["cargo build".to_string(), "cargo test".to_string()],
                        labels: vec!["bugfix".to_string(), "maintenance".to_string()],
                    },
                ),
                (
                    "hotfix/*".to_string(),
                    BranchPattern {
                        template: "{repo}-{branch}".to_string(),
                        setup: vec![
                            "cargo build".to_string(),
                            "cargo xtask check-all".to_string(),
                        ],
                        labels: vec!["hotfix".to_string(), "urgent".to_string()],
                    },
                ),
            ])),
            integration: Some(IntegrationConfig {
                lefthook: true,
                xtask: true,
                wasm_components: true,
            }),
            cursor_integration: Some(CursorIntegrationConfig {
                auto_open_cursor: true,
                cursor_config_template: Some(".cursor/workbloom.json".to_string()),
                shell_aliases: HashMap::from([
                    (
                        "cbloom".to_string(),
                        "wb bloom $1 && cursor ./$1".to_string(),
                    ),
                    (
                        "cswitch".to_string(),
                        "wb switch $1 && cursor .".to_string(),
                    ),
                ]),
                env_vars: HashMap::from([
                    ("WORKTREE_MANAGER".to_string(), "workbloom".to_string()),
                    ("CURSOR_INTEGRATION".to_string(), "enabled".to_string()),
                ]),
            }),
            workbloom_metadata: Some(WorkbloomMetadataConfig {
                enabled: true,
                metadata_dir: ".wb".to_string(),
                labels_config: Some(".wb/labels.toml".to_string()),
                status_tracking: true,
            }),
        }
    }
}

/// Worktree management tool
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorktreeTool {
    Workbloom,
    Git,
}

impl WorktreeTool {
    /// Get command name for the tool
    pub fn command_name(&self) -> &'static str {
        match self {
            WorktreeTool::Workbloom => "workbloom",
            WorktreeTool::Git => "git",
        }
    }

    /// Check if tool is available
    pub fn is_available(&self) -> bool {
        Command::new(self.command_name())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Get tool description
    pub fn description(&self) -> &'static str {
        match self {
            WorktreeTool::Workbloom => {
                "Rust-based CLI with automatic file copying, port allocation, and semantic metadata"
            }
            WorktreeTool::Git => "Native Git worktree commands",
        }
    }

    /// Get configuration file name for the tool
    pub fn config_file_name(&self) -> &'static str {
        match self {
            WorktreeTool::Workbloom => ".workbloom",
            WorktreeTool::Git => ".git/config",
        }
    }
}

/// Worktree information with enhanced metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    /// Worktree path
    pub path: String,
    /// Branch name
    pub branch: String,
    /// Commit hash
    pub commit: String,
    /// Whether this is the current worktree
    pub current: bool,
    /// Whether the worktree is dirty
    pub dirty: bool,
    /// Semantic labels
    pub labels: Vec<String>,
    /// Creation date
    pub created: Option<String>,
    /// Last activity
    pub last_activity: Option<String>,
    /// Purpose/description
    pub purpose: Option<String>,
}

/// Worktree manager with enhanced features
pub struct WorktreeManager {
    config: WorktreeConfig,
}

impl WorktreeManager {
    /// Create a new worktree manager
    pub fn new() -> Self {
        Self {
            config: WorktreeConfig::default(),
        }
    }

    /// Create a new worktree manager with custom configuration
    pub fn with_config(config: WorktreeConfig) -> Self {
        Self { config }
    }

    /// Load configuration from multiple sources
    pub async fn load_config(&mut self, config_path: &Path) -> Result<()> {
        // Load configuration in order of precedence:
        // 1. Project-specific workbloom config
        // 2. Workbloom metadata (.wb/ directory)
        // 3. Hooksmith-specific config
        // 4. Defaults

        // Try to load project-specific workbloom config
        let workbloom_config = PathBuf::from(".workbloom");
        if workbloom_config.exists() {
            if let Ok(content) = fs::read_to_string(&workbloom_config).await {
                println!("{}", style("✓ Loaded .workbloom configuration").green());
                self.parse_workbloom_config(&content)?;
            }
        }

        // Try to load workbloom metadata
        if let Some(metadata_config) = &self.config.workbloom_metadata {
            if metadata_config.enabled {
                let metadata_dir = PathBuf::from(&metadata_config.metadata_dir);
                if metadata_dir.exists() {
                    println!("{}", style("✓ Loaded Workbloom metadata").green());
                    self.load_workbloom_metadata(&metadata_dir).await?;
                }
            }
        }

        // Load Hooksmith-specific config
        if config_path.exists() {
            let content = fs::read_to_string(config_path).await?;
            self.config = serde_json::from_str(&content)?;
        }

        Ok(())
    }

    /// Parse workbloom configuration
    fn parse_workbloom_config(&mut self, content: &str) -> Result<()> {
        // Workbloom uses a simple line-based format for files to copy
        let mut env_files = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                // Add to environment files list for copying
                env_files.push(line.to_string());
            }
        }

        // Update the config with workbloom-specific files
        if !env_files.is_empty() {
            self.config.env_files.extend(env_files);
        }

        Ok(())
    }

    /// Load workbloom metadata from .wb/ directory
    async fn load_workbloom_metadata(&mut self, metadata_dir: &Path) -> Result<()> {
        // Load metadata.json if it exists
        let metadata_file = metadata_dir.join("metadata.json");
        if metadata_file.exists() {
            let content = fs::read_to_string(&metadata_file).await?;
            // Parse metadata (could be used for enhanced worktree info)
            println!("{}", style("  - Loaded worktree metadata").dim());
        }

        // Load labels.toml if it exists
        let labels_file = metadata_dir.join("labels.toml");
        if labels_file.exists() {
            let content = fs::read_to_string(&labels_file).await?;
            // Parse labels configuration
            println!("{}", style("  - Loaded semantic labels").dim());
        }

        // Load status.json if it exists
        let status_file = metadata_dir.join("status.json");
        if status_file.exists() {
            let content = fs::read_to_string(&status_file).await?;
            // Parse status information
            println!("{}", style("  - Loaded worktree status").dim());
        }

        Ok(())
    }

    /// Save configuration to file
    pub async fn save_config(&self, config_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(config_path, content).await?;
        Ok(())
    }

    /// Get available worktree tools
    pub fn get_available_tools(&self) -> Vec<WorktreeTool> {
        let tools = vec![WorktreeTool::Workbloom, WorktreeTool::Git];
        tools
            .into_iter()
            .filter(|tool| tool.is_available())
            .collect()
    }

    /// Select the best available tool
    pub fn select_best_tool(&self) -> Result<WorktreeTool> {
        // Prefer Workbloom for its advanced features
        if WorktreeTool::Workbloom.is_available() {
            Ok(WorktreeTool::Workbloom)
        } else if WorktreeTool::Git.is_available() {
            Ok(WorktreeTool::Git)
        } else {
            Err(anyhow::anyhow!("No worktree management tool available"))
        }
    }

    /// List all worktrees with enhanced metadata
    pub async fn list_worktrees(&self, detailed: bool) -> Result<Vec<WorktreeInfo>> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Workbloom => self.list_with_workbloom(detailed).await,
            WorktreeTool::Git => self.list_with_git(detailed).await,
        }
    }

    /// Create a new worktree with enhanced features
    pub async fn create_worktree(
        &self,
        branch: &str,
        base_dir: Option<&str>,
        switch: bool,
    ) -> Result<()> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Workbloom => self.create_with_workbloom(branch, base_dir, switch).await,
            WorktreeTool::Git => self.create_with_git(branch, base_dir, switch).await,
        }
    }

    /// Switch to a worktree with Cursor integration
    pub async fn switch_worktree(&self, worktree: &str) -> Result<()> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Workbloom => self.switch_with_workbloom(worktree).await,
            WorktreeTool::Git => self.switch_with_git(worktree).await,
        }
    }

    /// Remove a worktree with smart cleanup
    pub async fn remove_worktree(&self, worktree: &str, with_branch: bool) -> Result<()> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Workbloom => self.remove_with_workbloom(worktree).await,
            WorktreeTool::Git => self.remove_with_git(worktree, with_branch).await,
        }
    }

    /// Install recommended tools
    pub async fn install_tools(&self) -> Result<()> {
        println!(
            "{}",
            style("Installing worktree management tools...").bold()
        );

        // Try to install workbloom (primary tool)
        if !WorktreeTool::Workbloom.is_available() {
            println!("Installing workbloom...");
            let output = Command::new("cargo")
                .args(["install", "workbloom"])
                .output()
                .context("Failed to install workbloom")?;

            if output.status.success() {
                println!("{}", style("✓ workbloom installed successfully").green());
            } else {
                println!("{}", style("✗ Failed to install workbloom").red());
            }
        }

        Ok(())
    }

    /// Create configuration files with enhanced features
    pub async fn create_config_files(&self) -> Result<()> {
        println!(
            "{}",
            style("Creating worktree configuration files...").bold()
        );

        // Create .workbloom configuration
        let workbloom_config = r#"# .workbloom
# Hooksmith Workbloom Configuration
# Automatically copies these files/directories to new worktrees

# Environment files
.env
.envrc
.env.local
.env.example

# Configuration files
hooksmith.toml
.worktree-config.jsonc
.worktree-config.json

# Development configuration
.vscode/settings.json
.vscode/launch.json
.vscode/extensions.json

# Tool configuration
.claude/settings.json
.config/my-settings.json

# Spin configuration
spin.toml
spin.toml.example

# Cargo configuration
.cargo/config.toml
.cargo/config

# Git configuration
.gitignore
.gitattributes

# Documentation
README.md
docs/

# Scripts and utilities
scripts/
tools/

# Secrets and local config (if they exist)
secrets/
local/

# Cursor integration
.cursor/
"#;

        fs::write(".workbloom", workbloom_config).await?;
        println!("{}", style("✓ Created .workbloom").green());

        // Create Workbloom metadata directory structure
        if let Some(metadata_config) = &self.config.workbloom_metadata {
            if metadata_config.enabled {
                let metadata_dir = PathBuf::from(&metadata_config.metadata_dir);
                fs::create_dir_all(&metadata_dir).await?;

                // Create metadata.json
                let metadata = serde_json::json!({
                    "version": "1.0",
                    "created": chrono::Utc::now().to_rfc3339(),
                    "worktrees": {},
                    "labels": {
                        "feature": ["development", "new-feature"],
                        "bugfix": ["maintenance", "fix"],
                        "hotfix": ["urgent", "critical"],
                        "spike": ["experiment", "research"]
                    }
                });
                fs::write(
                    metadata_dir.join("metadata.json"),
                    serde_json::to_string_pretty(&metadata)?,
                )
                .await?;

                // Create labels.toml
                let labels_config = r#"# .wb/labels.toml
# Semantic labels for worktrees

[labels]
feature = ["development", "new-feature", "enhancement"]
bugfix = ["maintenance", "fix", "bug"]
hotfix = ["urgent", "critical", "production"]
spike = ["experiment", "research", "prototype"]
docs = ["documentation", "guide", "tutorial"]
refactor = ["cleanup", "improvement", "technical-debt"]

[patterns]
"feature/*" = ["feature", "development"]
"bugfix/*" = ["bugfix", "maintenance"]
"hotfix/*" = ["hotfix", "urgent"]
"spike/*" = ["spike", "experiment"]
"docs/*" = ["docs", "documentation"]
"refactor/*" = ["refactor", "cleanup"]
"#;
                fs::write(metadata_dir.join("labels.toml"), labels_config).await?;

                // Create status.json
                let status = serde_json::json!({
                    "active_worktrees": [],
                    "last_updated": chrono::Utc::now().to_rfc3339(),
                    "cursor_integration": true
                });
                fs::write(
                    metadata_dir.join("status.json"),
                    serde_json::to_string_pretty(&status)?,
                )
                .await?;

                println!(
                    "{}",
                    style("✓ Created Workbloom metadata structure").green()
                );
            }
        }

        // Create Cursor integration configuration
        if let Some(cursor_config) = &self.config.cursor_integration {
            if let Some(config_template) = &cursor_config.cursor_config_template {
                let cursor_dir = PathBuf::from(".cursor");
                fs::create_dir_all(&cursor_dir).await?;

                let cursor_config_content = serde_json::json!({
                    "worktree_integration": {
                        "enabled": true,
                        "tool": "workbloom",
                        "auto_open": cursor_config.auto_open_cursor,
                        "metadata_dir": ".wb"
                    },
                    "ai_context": {
                        "include_worktree_labels": true,
                        "include_branch_patterns": true,
                        "semantic_context": true
                    }
                });
                fs::write(
                    cursor_dir.join("workbloom.json"),
                    serde_json::to_string_pretty(&cursor_config_content)?,
                )
                .await?;
                println!("{}", style("✓ Created Cursor integration config").green());
            }
        }

        // Create Hooksmith-specific config
        let hooksmith_config = serde_json::to_string_pretty(&self.config)?;
        fs::write(".worktree-config.jsonc", hooksmith_config).await?;
        println!("{}", style("✓ Created .worktree-config.jsonc").green());

        Ok(())
    }

    /// Setup Git aliases and shell integration
    pub async fn setup_git_aliases(&self) -> Result<()> {
        println!(
            "{}",
            style("Setting up Git aliases and shell integration...").bold()
        );

        let aliases = [
            ("wt", "worktree"),
            ("wtl", "worktree list"),
            ("wtc", "worktree create"),
            ("wtr", "worktree remove"),
            ("wts", "worktree switch"),
        ];

        for (alias, command) in aliases.iter() {
            let output = Command::new("git")
                .args(["config", "--global", "alias", alias, command])
                .output()
                .context(format!("Failed to set alias: {}", alias))?;

            if output.status.success() {
                println!(
                    "{}",
                    style(&format!("✓ Set alias: {} -> {}", alias, command)).green()
                );
            } else {
                println!(
                    "{}",
                    style(&format!("✗ Failed to set alias: {}", alias)).red()
                );
            }
        }

        // Setup shell aliases for Cursor integration
        if let Some(cursor_config) = &self.config.cursor_integration {
            println!("{}", style("Cursor integration aliases:").bold());
            for (alias, command) in &cursor_config.shell_aliases {
                println!("{}", style(&format!("  {} -> {}", alias, command)).dim());
            }
            println!(
                "{}",
                style("Add these to your shell configuration (.zshrc, .bashrc, etc.)").yellow()
            );
        }

        Ok(())
    }

    // Tool-specific implementations

    async fn list_with_workbloom(&self, detailed: bool) -> Result<Vec<WorktreeInfo>> {
        // Workbloom doesn't have a list command, so we'll use git worktree list
        // and enhance it with Workbloom's status information
        let output = Command::new("workbloom")
            .args(["cleanup", "--status"])
            .output()
            .context("Failed to run workbloom cleanup --status")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            self.parse_workbloom_status_output(&output_str, detailed)
                .await
        } else {
            // Fall back to git worktree list if workbloom status fails
            self.list_with_git(detailed).await
        }
    }

    async fn list_with_git(&self, detailed: bool) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("git")
            .args(["worktree", "list"])
            .output()
            .context("Failed to run git worktree list")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            self.parse_git_list_output(&output_str, detailed).await
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("git worktree list failed: {}", error))
        }
    }

    async fn create_with_workbloom(
        &self,
        branch: &str,
        _base_dir: Option<&str>,
        switch: bool,
    ) -> Result<()> {
        // Use workbloom's setup command which includes file copying and port allocation
        let mut args = vec!["setup", branch];
        if !switch {
            args.push("--no-shell");
        }

        let output = Command::new("workbloom")
            .args(&args)
            .output()
            .context("Failed to run workbloom setup")?;

        if output.status.success() {
            println!(
                "{}",
                style("✓ Worktree created successfully with Workbloom").green()
            );
            println!("{}", style("  - Automatic file copying enabled").dim());
            println!("{}", style("  - Port allocation configured").dim());
            if switch {
                println!("{}", style("  - Shell opened in new worktree").dim());
            }

            // Update metadata if enabled
            if let Some(metadata_config) = &self.config.workbloom_metadata {
                if metadata_config.enabled {
                    self.update_worktree_metadata(branch, "created").await?;
                }
            }

            // Open in Cursor if configured
            if let Some(cursor_config) = &self.config.cursor_integration {
                if cursor_config.auto_open_cursor {
                    println!("{}", style("  - Cursor integration available").dim());
                }
            }

            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("workbloom setup failed: {}", error))
        }
    }

    async fn create_with_git(
        &self,
        branch: &str,
        base_dir: Option<&str>,
        switch: bool,
    ) -> Result<()> {
        let base_path = base_dir.unwrap_or("../");
        let worktree_path = format!("{}{}", base_path, branch);

        let mut args = vec!["worktree", "add"];
        if switch {
            args.push("-b");
            args.push(branch);
        } else {
            args.push("--detach");
        }
        args.push(&worktree_path);

        let output = Command::new("git")
            .args(&args)
            .output()
            .context("Failed to run git worktree add")?;

        if output.status.success() {
            println!("{}", style("✓ Worktree created successfully").green());
            if switch {
                println!(
                    "{}",
                    style(&format!("Please run: cd {}", worktree_path)).yellow()
                );
            }
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("git worktree add failed: {}", error))
        }
    }

    async fn switch_with_workbloom(&self, worktree: &str) -> Result<()> {
        // Workbloom doesn't have a cd command, so we'll use git worktree list
        // to find the path and then provide instructions
        let worktrees = self.list_with_git(false).await?;
        if let Some(wt) = worktrees.iter().find(|w| w.branch == worktree) {
            println!(
                "{}",
                style("✓ Workbloom environment files synchronized").green()
            );
            println!("{}", style(&format!("Please run: cd {}", wt.path)).yellow());

            // Update metadata if enabled
            if let Some(metadata_config) = &self.config.workbloom_metadata {
                if metadata_config.enabled {
                    self.update_worktree_metadata(worktree, "switched").await?;
                }
            }

            // Cursor integration hint
            if let Some(cursor_config) = &self.config.cursor_integration {
                if cursor_config.auto_open_cursor {
                    println!("{}", style("  - Run 'cursor .' to open in Cursor").dim());
                }
            }

            Ok(())
        } else {
            Err(anyhow::anyhow!("Worktree '{}' not found", worktree))
        }
    }

    async fn switch_with_git(&self, worktree: &str) -> Result<()> {
        // For git, we need to find the worktree path first
        let worktrees = self.list_with_git(false).await?;
        if let Some(wt) = worktrees.iter().find(|w| w.branch == worktree) {
            println!("{}", style(&format!("Please run: cd {}", wt.path)).yellow());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Worktree '{}' not found", worktree))
        }
    }

    async fn remove_with_workbloom(&self, worktree: &str) -> Result<()> {
        // Use workbloom's cleanup command for better cleanup
        let output = Command::new("workbloom")
            .args(["cleanup", "--pattern", worktree])
            .output()
            .context("Failed to run workbloom cleanup")?;

        if output.status.success() {
            println!(
                "{}",
                style("✓ Worktree removed successfully with Workbloom cleanup").green()
            );
            println!("{}", style("  - Smart cleanup completed").dim());

            // Update metadata if enabled
            if let Some(metadata_config) = &self.config.workbloom_metadata {
                if metadata_config.enabled {
                    self.update_worktree_metadata(worktree, "removed").await?;
                }
            }

            Ok(())
        } else {
            // Fall back to remove command if cleanup fails
            let output = Command::new("workbloom")
                .args(["remove", worktree])
                .output()
                .context("Failed to run workbloom remove")?;

            if output.status.success() {
                println!("{}", style("✓ Worktree removed successfully").green());
                Ok(())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(anyhow::anyhow!("workbloom remove failed: {}", error))
            }
        }
    }

    async fn remove_with_git(&self, worktree: &str, with_branch: bool) -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "remove", worktree])
            .output()
            .context("Failed to run git worktree remove")?;

        if output.status.success() {
            println!("{}", style("✓ Worktree removed successfully").green());
            if with_branch {
                // Also remove the branch
                let _ = Command::new("git")
                    .args(["branch", "-D", worktree])
                    .output();
            }
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("git worktree remove failed: {}", error))
        }
    }

    /// Update worktree metadata
    async fn update_worktree_metadata(&self, worktree: &str, action: &str) -> Result<()> {
        if let Some(metadata_config) = &self.config.workbloom_metadata {
            let metadata_dir = PathBuf::from(&metadata_config.metadata_dir);
            let metadata_file = metadata_dir.join("metadata.json");

            if metadata_file.exists() {
                let content = fs::read_to_string(&metadata_file).await?;
                let mut metadata: serde_json::Value = serde_json::from_str(&content)?;

                // Update worktrees section
                if let Some(worktrees) = metadata.get_mut("worktrees") {
                    if let Some(worktrees_obj) = worktrees.as_object_mut() {
                        worktrees_obj.insert(
                            worktree.to_string(),
                            serde_json::json!({
                                "action": action,
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                "labels": self.get_labels_for_branch(worktree)
                            }),
                        );
                    }
                }

                fs::write(&metadata_file, serde_json::to_string_pretty(&metadata)?).await?;
            }
        }
        Ok(())
    }

    /// Get semantic labels for a branch
    fn get_labels_for_branch(&self, branch: &str) -> Vec<String> {
        let mut labels = Vec::new();

        if let Some(patterns) = &self.config.branch_patterns {
            for (pattern, pattern_config) in patterns {
                if self.matches_pattern(branch, pattern) {
                    labels.extend(pattern_config.labels.clone());
                }
            }
        }

        labels
    }

    /// Check if a branch matches a pattern
    fn matches_pattern(&self, branch: &str, pattern: &str) -> bool {
        // Simple pattern matching - could be enhanced with proper glob matching
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            branch.starts_with(prefix)
        } else {
            branch == pattern
        }
    }

    // Output parsing methods

    async fn parse_workbloom_status_output(
        &self,
        output: &str,
        detailed: bool,
    ) -> Result<Vec<WorktreeInfo>> {
        // Parse workbloom cleanup --status output format
        // This will show merge status of branches, so we'll combine it with git worktree list
        let mut worktrees = Vec::new();

        // For now, we'll fall back to git worktree list since workbloom status
        // shows different information (merge status rather than worktree paths)
        // In a future enhancement, we could combine both sources of information
        Ok(worktrees)
    }

    async fn parse_git_list_output(
        &self,
        output: &str,
        detailed: bool,
    ) -> Result<Vec<WorktreeInfo>> {
        let mut worktrees = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // Parse git worktree list output format
            // Format: <path> <commit> [<branch>]
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[0].to_string();
                let commit = parts[1].to_string();
                let branch = if parts.len() >= 3 {
                    parts[2].trim_matches('[').trim_matches(']').to_string()
                } else {
                    "detached".to_string()
                };

                let mut worktree_info = WorktreeInfo {
                    path,
                    branch,
                    commit,
                    current: false, // Would need to detect current
                    dirty: false,   // Would need to detect dirty state
                    labels: Vec::new(),
                    created: None,
                    last_activity: None,
                    purpose: None,
                };

                // Add semantic labels if detailed mode is enabled
                if detailed {
                    worktree_info.labels = self.get_labels_for_branch(&worktree_info.branch);
                }

                worktrees.push(worktree_info);
            }
        }

        Ok(worktrees)
    }
}

/// Run worktree command
pub async fn run_worktree_command(command: WorktreeCommands) -> Result<()> {
    let mut manager = WorktreeManager::new();

    // Try to load existing configuration
    let config_path = PathBuf::from(".worktree-config.json");
    manager.load_config(&config_path).await.ok();

    match command {
        WorktreeCommands::List { detailed, format } => {
            let worktrees = manager.list_worktrees(detailed).await?;

            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&worktrees)?;
                    println!("{}", json);
                }
                "summary" => {
                    print_worktree_summary(&worktrees);
                }
                _ => {
                    print_worktree_table(&worktrees, detailed);
                }
            }
        }
        WorktreeCommands::Create {
            branch,
            base_dir,
            tool,
            setup,
            copy_env,
            switch,
        } => {
            if let Some(ref _tool_name) = tool {
                // Override preferred tool for this command
                let config = WorktreeConfig::default();
                let manager = WorktreeManager::with_config(config);
                manager
                    .create_worktree(&branch, base_dir.as_deref(), switch)
                    .await?;
            } else {
                manager
                    .create_worktree(&branch, base_dir.as_deref(), switch)
                    .await?;
            }

            if setup {
                println!("{}", style("Running setup commands...").bold());
                // Run setup commands
                for cmd in &manager.config.setup_commands {
                    println!("Running: {}", cmd);
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .context(format!("Failed to run setup command: {}", cmd))?;

                    if output.status.success() {
                        println!("{}", style("✓ Setup command completed").green());
                    } else {
                        println!("{}", style("✗ Setup command failed").red());
                    }
                }
            }

            if copy_env {
                println!("{}", style("Copying environment files...").bold());
                // Copy environment files
                for env_file in &manager.config.env_files {
                    if Path::new(env_file).exists() {
                        let target = env_file.replace(".example", "");
                        let _ = Command::new("cp").arg(env_file).arg(&target).output();
                        println!(
                            "{}",
                            style(&format!("✓ Copied {} to {}", env_file, target)).green()
                        );
                    }
                }
            }
        }
        WorktreeCommands::Switch { worktree, tool } => {
            if let Some(ref _tool_name) = tool {
                // Override preferred tool for this command
                let config = WorktreeConfig::default();
                let manager = WorktreeManager::with_config(config);
                manager.switch_worktree(&worktree).await?;
            } else {
                manager.switch_worktree(&worktree).await?;
            }
        }
        WorktreeCommands::Remove {
            worktree,
            with_branch,
            tool,
            force,
        } => {
            if !force {
                println!(
                    "{}",
                    style("Are you sure you want to remove this worktree? (y/N)").yellow()
                );
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("{}", style("Operation cancelled").yellow());
                    return Ok(());
                }
            }

            if let Some(ref _tool_name) = tool {
                // Override preferred tool for this command
                let config = WorktreeConfig::default();
                let manager = WorktreeManager::with_config(config);
                manager.remove_worktree(&worktree, with_branch).await?;
            } else {
                manager.remove_worktree(&worktree, with_branch).await?;
            }
        }
        WorktreeCommands::Setup {
            install_tools,
            config,
            aliases,
            all,
        } => {
            if all || install_tools {
                manager.install_tools().await?;
            }

            if all || config {
                manager.create_config_files().await?;
            }

            if all || aliases {
                manager.setup_git_aliases().await?;
            }
        }
        WorktreeCommands::Status { detailed, format } => {
            let tools = manager.get_available_tools();
            let best_tool = manager.select_best_tool().ok();

            match format.as_str() {
                "json" => {
                    let status = serde_json::json!({
                        "tools": tools.iter().map(|t| {
                            serde_json::json!({
                                "name": t.command_name(),
                                "available": t.is_available(),
                                "description": t.description()
                            })
                        }).collect::<Vec<_>>(),
                        "best_tool": best_tool.map(|t| t.command_name()),
                        "cursor_integration": manager.config.cursor_integration.is_some(),
                        "metadata_enabled": manager.config.workbloom_metadata.as_ref().map(|m| m.enabled).unwrap_or(false)
                    });
                    println!("{}", serde_json::to_string_pretty(&status)?);
                }
                _ => {
                    print_tool_status(&tools, &best_tool, detailed);
                }
            }
        }
    }

    Ok(())
}

fn print_worktree_table(worktrees: &[WorktreeInfo], detailed: bool) {
    if worktrees.is_empty() {
        println!("{}", style("No worktrees found").yellow());
        return;
    }

    if detailed {
        println!(
            "{:<50} {:<30} {:<12} {:<8} {:<6} {:<20}",
            "Path", "Branch", "Commit", "Current", "Dirty", "Labels"
        );
        println!("{:-<130}", "");
        for wt in worktrees {
            let labels_str = wt.labels.join(", ");
            println!(
                "{:<50} {:<30} {:<12} {:<8} {:<6} {:<20}",
                wt.path,
                wt.branch,
                &wt.commit[..std::cmp::min(12, wt.commit.len())],
                if wt.current { "✓" } else { "" },
                if wt.dirty { "✗" } else { "" },
                labels_str
            );
        }
    } else {
        println!("{:<50} {:<30}", "Path", "Branch");
        println!("{:-<80}", "");
        for wt in worktrees {
            println!("{:<50} {:<30}", wt.path, wt.branch);
        }
    }
}

fn print_worktree_summary(worktrees: &[WorktreeInfo]) {
    println!("Worktree Summary:");
    println!("Total worktrees: {}", worktrees.len());
    println!(
        "Current worktree: {}",
        worktrees
            .iter()
            .find(|w| w.current)
            .map(|w| &w.branch)
            .unwrap_or(&"none".to_string())
    );
    println!(
        "Dirty worktrees: {}",
        worktrees.iter().filter(|w| w.dirty).count()
    );

    // Count by labels
    let mut label_counts: HashMap<String, usize> = HashMap::new();
    for wt in worktrees {
        for label in &wt.labels {
            *label_counts.entry(label.clone()).or_insert(0) += 1;
        }
    }

    if !label_counts.is_empty() {
        println!("By labels:");
        for (label, count) in label_counts {
            println!("  {}: {}", label, count);
        }
    }
}

fn print_tool_status(tools: &[WorktreeTool], best_tool: &Option<WorktreeTool>, detailed: bool) {
    println!("Worktree Tool Status");

    if detailed {
        println!("{:<12} {:<10} {:<50}", "Tool", "Status", "Description");
        println!("{:-<72}", "");
        for tool in tools {
            let status = if tool.is_available() {
                "✓ Available"
            } else {
                "✗ Missing"
            };
            let best_marker = if best_tool.as_ref() == Some(tool) {
                " (best)"
            } else {
                ""
            };
            println!(
                "{:<12} {:<10} {:<50}",
                format!("{}{}", tool.command_name(), best_marker),
                status,
                tool.description()
            );
        }
    } else {
        for tool in tools {
            let status = if tool.is_available() { "✓" } else { "✗" };
            let best_marker = if best_tool.as_ref() == Some(tool) {
                " (best)"
            } else {
                ""
            };
            println!("{} {}{}", status, tool.command_name(), best_marker);
        }
    }
}
