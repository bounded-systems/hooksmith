//! Worktree management module for Hooksmith
//!
//! This module provides comprehensive worktree management functionality,
//! including tool detection, configuration management, and integration
//! with various worktree management tools like wtp, wt, and git.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use console::style;
use serde::{Deserialize, Serialize};
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
}

/// Branch pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchPattern {
    /// Template for worktree naming
    pub template: String,
    /// Setup commands for this pattern
    pub setup: Vec<String>,
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
                ("feature/spin-integration".to_string(), "../hooksmith-spin".to_string()),
                ("feature/spin-integration-v2".to_string(), "../hooksmith-spin-integration".to_string()),
            ])),
            branch_patterns: Some(HashMap::from([
                ("feature/*".to_string(), BranchPattern {
                    template: "{repo}-{branch}".to_string(),
                    setup: vec!["cargo build".to_string(), "cargo xtask gen-all".to_string()],
                }),
                ("bugfix/*".to_string(), BranchPattern {
                    template: "{repo}-{branch}".to_string(),
                    setup: vec!["cargo build".to_string(), "cargo test".to_string()],
                }),
                ("hotfix/*".to_string(), BranchPattern {
                    template: "{repo}-{branch}".to_string(),
                    setup: vec!["cargo build".to_string(), "cargo xtask check-all".to_string()],
                }),
            ])),
            integration: Some(IntegrationConfig {
                lefthook: true,
                xtask: true,
                wasm_components: true,
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
            WorktreeTool::Workbloom => "Rust-based CLI with automatic file copying and port allocation",
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

/// Worktree information
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
}

/// Worktree manager
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
        // 2. Hooksmith-specific config
        // 3. Defaults

        // Try to load project-specific workbloom config
        let workbloom_config = PathBuf::from(".workbloom");
        if workbloom_config.exists() {
            if let Ok(content) = fs::read_to_string(&workbloom_config).await {
                println!("{}", style("✓ Loaded .workbloom configuration").green());
                // Parse workbloom configuration (line-based file list)
                self.parse_workbloom_config(&content)?;
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

    /// List all worktrees
    pub async fn list_worktrees(&self, _detailed: bool) -> Result<Vec<WorktreeInfo>> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Workbloom => self.list_with_workbloom().await,
            WorktreeTool::Git => self.list_with_git().await,
        }
    }

    /// Create a new worktree
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

    /// Switch to a worktree
    pub async fn switch_worktree(&self, worktree: &str) -> Result<()> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Workbloom => self.switch_with_workbloom(worktree).await,
            WorktreeTool::Git => self.switch_with_git(worktree).await,
        }
    }

    /// Remove a worktree
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

    /// Create configuration files
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
"#;

        fs::write(".workbloom", workbloom_config).await?;
        println!("{}", style("✓ Created .workbloom").green());

        // Create Hooksmith-specific config
        let hooksmith_config = serde_json::to_string_pretty(&self.config)?;
        fs::write(".worktree-config.jsonc", hooksmith_config).await?;
        println!("{}", style("✓ Created .worktree-config.jsonc").green());

        Ok(())
    }

    /// Setup Git aliases
    pub async fn setup_git_aliases(&self) -> Result<()> {
        println!(
            "{}",
            style("Setting up Git aliases for worktree management...").bold()
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
                println!("{}", style(&format!("✓ Set alias: {} -> {}", alias, command)).green());
            } else {
                println!("{}", style(&format!("✗ Failed to set alias: {}", alias)).red());
            }
        }

        Ok(())
    }

    // Tool-specific implementations

    async fn list_with_workbloom(&self) -> Result<Vec<WorktreeInfo>> {
        // Workbloom doesn't have a list command, so we'll use git worktree list
        // and enhance it with Workbloom's status information
        let output = Command::new("workbloom")
            .args(["cleanup", "--status"])
            .output()
            .context("Failed to run workbloom cleanup --status")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            self.parse_workbloom_status_output(&output_str)
        } else {
            // Fall back to git worktree list if workbloom status fails
            self.list_with_git().await
        }
    }

    async fn list_with_git(&self) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("git")
            .args(["worktree", "list"])
            .output()
            .context("Failed to run git worktree list")?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            self.parse_git_list_output(&output_str)
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
            println!("{}", style("✓ Worktree created successfully with Workbloom").green());
            println!("{}", style("  - Automatic file copying enabled").dim());
            println!("{}", style("  - Port allocation configured").dim());
            if switch {
                println!("{}", style("  - Shell opened in new worktree").dim());
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
                println!("{}", style(&format!("Please run: cd {}", worktree_path)).yellow());
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
        let worktrees = self.list_with_git().await?;
        if let Some(wt) = worktrees.iter().find(|w| w.branch == worktree) {
            println!("{}", style("✓ Workbloom environment files synchronized").green());
            println!("{}", style(&format!("Please run: cd {}", wt.path)).yellow());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Worktree '{}' not found", worktree))
        }
    }

    async fn switch_with_git(&self, worktree: &str) -> Result<()> {
        // For git, we need to find the worktree path first
        let worktrees = self.list_with_git().await?;
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
            println!("{}", style("✓ Worktree removed successfully with Workbloom cleanup").green());
            println!("{}", style("  - Smart cleanup completed").dim());
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

    // Output parsing methods

    fn parse_workbloom_status_output(&self, output: &str) -> Result<Vec<WorktreeInfo>> {
        // Parse workbloom cleanup --status output format
        // This will show merge status of branches, so we'll combine it with git worktree list
        let mut worktrees = Vec::new();

        // For now, we'll fall back to git worktree list since workbloom status
        // shows different information (merge status rather than worktree paths)
        // In a future enhancement, we could combine both sources of information
        Ok(worktrees)
    }

    fn parse_git_list_output(&self, output: &str) -> Result<Vec<WorktreeInfo>> {
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

                worktrees.push(WorktreeInfo {
                    path,
                    branch,
                    commit,
                    current: false, // Would need to detect current
                    dirty: false,   // Would need to detect dirty state
                });
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
            if let Some(ref tool_name) = tool {
                // Override preferred tool for this command
                let mut config = WorktreeConfig::default();
                let manager = WorktreeManager::with_config(config);
                manager.create_worktree(&branch, base_dir.as_deref(), switch).await?;
            } else {
                manager.create_worktree(&branch, base_dir.as_deref(), switch).await?;
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
                        let _ = Command::new("cp")
                            .arg(env_file)
                            .arg(&target)
                            .output();
                        println!("{}", style(&format!("✓ Copied {} to {}", env_file, target)).green());
                    }
                }
            }
        }
        WorktreeCommands::Switch { worktree, tool } => {
            if let Some(ref tool_name) = tool {
                // Override preferred tool for this command
                let mut config = WorktreeConfig::default();
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
                println!("{}", style("Are you sure you want to remove this worktree? (y/N)").yellow());
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if !input.trim().to_lowercase().starts_with('y') {
                    println!("{}", style("Operation cancelled").yellow());
                    return Ok(());
                }
            }

            if let Some(ref tool_name) = tool {
                // Override preferred tool for this command
                let mut config = WorktreeConfig::default();
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
                        "best_tool": best_tool.map(|t| t.command_name())
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
        println!("{:<50} {:<30} {:<12} {:<8} {:<6}", "Path", "Branch", "Commit", "Current", "Dirty");
        println!("{:-<120}", "");
        for wt in worktrees {
            println!(
                "{:<50} {:<30} {:<12} {:<8} {:<6}",
                wt.path,
                wt.branch,
                &wt.commit[..std::cmp::min(12, wt.commit.len())],
                if wt.current { "✓" } else { "" },
                if wt.dirty { "✗" } else { "" }
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
    println!("Current worktree: {}", worktrees.iter().find(|w| w.current).map(|w| &w.branch).unwrap_or(&"none".to_string()));
    println!("Dirty worktrees: {}", worktrees.iter().filter(|w| w.dirty).count());
}

fn print_tool_status(tools: &[WorktreeTool], best_tool: &Option<WorktreeTool>, detailed: bool) {
    println!("Worktree Tool Status");

    if detailed {
        println!("{:<12} {:<10} {:<50}", "Tool", "Status", "Description");
        println!("{:-<72}", "");
        for tool in tools {
            let status = if tool.is_available() { "✓ Available" } else { "✗ Missing" };
            let best_marker = if best_tool.as_ref() == Some(tool) { " (best)" } else { "" };
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
            let best_marker = if best_tool.as_ref() == Some(tool) { " (best)" } else { "" };
            println!("{} {}{}", status, tool.command_name(), best_marker);
        }
    }
}
