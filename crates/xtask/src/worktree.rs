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

/// Configuration for worktree management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    /// Preferred tool to use
    pub preferred_tool: Option<String>,
    /// Base directory for worktrees
    pub worktree_base: Option<String>,
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
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            preferred_tool: None,
            worktree_base: Some("worktrees".to_string()),
            run_setup: true,
            setup_commands: vec!["cargo build".to_string()],
            copy_env: true,
            env_files: vec![".env.example".to_string()],
            git_aliases: HashMap::from([
                ("wt".to_string(), "worktree".to_string()),
                ("wtl".to_string(), "worktree list".to_string()),
                ("wtc".to_string(), "worktree create".to_string()),
                ("wtr".to_string(), "worktree remove".to_string()),
            ]),
        }
    }
}

/// Available worktree tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorktreeTool {
    Gwtr,
    Workbloom,
    Git,
}

impl WorktreeTool {
    /// Get the command name for this tool
    pub fn command_name(&self) -> &'static str {
        match self {
            WorktreeTool::Gwtr => "gwtr",
            WorktreeTool::Workbloom => "workbloom",
            WorktreeTool::Git => "git",
        }
    }

    /// Check if this tool is available on the system
    pub fn is_available(&self) -> bool {
        Command::new(self.command_name())
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Get tool description
    pub fn description(&self) -> &'static str {
        match self {
            WorktreeTool::Gwtr => "Simple Git worktree manager",
            WorktreeTool::Workbloom => "Git worktree management with automatic file copying",
            WorktreeTool::Git => "Native Git worktree commands",
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

    /// Load configuration from file
    pub async fn load_config(&mut self, config_path: &Path) -> Result<()> {
        if config_path.exists() {
            let content = fs::read_to_string(config_path).await?;
            self.config = serde_json::from_str(&content)?;
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
        let tools = vec![WorktreeTool::Gwtr, WorktreeTool::Workbloom, WorktreeTool::Git];
        tools
            .into_iter()
            .filter(|tool| tool.is_available())
            .collect()
    }

    /// Select the best available tool
    pub fn select_best_tool(&self) -> Result<WorktreeTool> {
        // Check if preferred tool is available
        if let Some(ref preferred) = self.config.preferred_tool {
            match preferred.as_str() {
                "gwtr" if WorktreeTool::Gwtr.is_available() => return Ok(WorktreeTool::Gwtr),
                "workbloom" if WorktreeTool::Workbloom.is_available() => return Ok(WorktreeTool::Workbloom),
                "git" if WorktreeTool::Git.is_available() => return Ok(WorktreeTool::Git),
                _ => {}
            }
        }

        // Fall back to best available tool
        if WorktreeTool::Gwtr.is_available() {
            Ok(WorktreeTool::Gwtr)
        } else if WorktreeTool::Workbloom.is_available() {
            Ok(WorktreeTool::Workbloom)
        } else if WorktreeTool::Git.is_available() {
            Ok(WorktreeTool::Git)
        } else {
            Err(anyhow::anyhow!("No worktree management tool available"))
        }
    }

    /// List all worktrees
    pub async fn list_worktrees(&self, detailed: bool) -> Result<Vec<WorktreeInfo>> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Gwtr => self.list_with_gwtr().await,
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
            WorktreeTool::Gwtr => self.create_with_gwtr(branch, base_dir, switch).await,
            WorktreeTool::Workbloom => self.create_with_workbloom(branch, base_dir, switch).await,
            WorktreeTool::Git => self.create_with_git(branch, base_dir, switch).await,
        }
    }

    /// Switch to a worktree
    pub async fn switch_worktree(&self, worktree: &str) -> Result<()> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Gwtr => self.switch_with_gwtr(worktree).await,
            WorktreeTool::Workbloom => self.switch_with_workbloom(worktree).await,
            WorktreeTool::Git => self.switch_with_git(worktree).await,
        }
    }

    /// Remove a worktree
    pub async fn remove_worktree(&self, worktree: &str, with_branch: bool) -> Result<()> {
        let tool = self.select_best_tool()?;

        match tool {
            WorktreeTool::Gwtr => self.remove_with_gwtr(worktree, with_branch).await,
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

        // Try to install gwtr
        if !WorktreeTool::Gwtr.is_available() {
            println!("Installing gwtr...");
            let output = Command::new("cargo")
                .args(["install", "gwtr"])
                .output()
                .context("Failed to install gwtr")?;

            if output.status.success() {
                println!("{}", style("✓ gwtr installed successfully").green());
            } else {
                println!("{}", style("✗ Failed to install gwtr").red());
            }
        }

        // Try to install workbloom
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

        // Create .wtp.yml
        let wtp_config = r#"version: "1.0"
defaults:
  base_dir: "worktrees"
  post_create:
    - type: copy
      from: ".env.example"
      to: ".env"
    - type: command
      command: "cargo build"
"#;
        fs::write(".wtp.yml", wtp_config).await?;
        println!("{}", style("✓ Created .wtp.yml").green());

        // Create worktree configuration
        let config_path = PathBuf::from(".worktree-config.json");
        self.save_config(&config_path).await?;
        println!("{}", style("✓ Created .worktree-config.json").green());

        Ok(())
    }

    /// Setup Git aliases
    pub async fn setup_git_aliases(&self) -> Result<()> {
        println!("{}", style("Setting up Git aliases...").bold());

        for (alias, command) in &self.config.git_aliases {
            let output = Command::new("git")
                .args(["config", "--global", "alias", alias, command])
                .output()
                .context(format!("Failed to set alias {}", alias))?;

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

        Ok(())
    }

    // Tool-specific implementations

    async fn list_with_gwtr(&self) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("gwtr")
            .arg("list")
            .output()
            .context("Failed to run gwtr list")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "gwtr list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Parse gwtr output
        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_gwtr_list_output(&output_str)
    }

    async fn list_with_workbloom(&self) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("workbloom")
            .arg("list")
            .output()
            .context("Failed to run workbloom list")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "workbloom list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Parse workbloom output
        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_workbloom_list_output(&output_str)
    }

    async fn list_with_git(&self) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("git")
            .args(["worktree", "list"])
            .output()
            .context("Failed to run git worktree list")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Parse git output
        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_git_list_output(&output_str)
    }

    async fn create_with_gwtr(
        &self,
        branch: &str,
        base_dir: Option<&str>,
        switch: bool,
    ) -> Result<()> {
        let mut cmd = Command::new("gwtr");
        cmd.arg("add").arg(branch);

        if let Some(dir) = base_dir {
            cmd.arg("--base-dir").arg(dir);
        }

        let output = cmd.output().context("Failed to run gwtr add")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "gwtr add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Created worktree for branch: {}", branch)).green()
        );

        if switch {
            self.switch_with_gwtr(branch).await?;
        }

        Ok(())
    }

    async fn create_with_workbloom(
        &self,
        branch: &str,
        base_dir: Option<&str>,
        switch: bool,
    ) -> Result<()> {
        let mut cmd = Command::new("workbloom");
        cmd.arg("add").arg(branch);

        if let Some(dir) = base_dir {
            cmd.arg("--base-dir").arg(dir);
        }

        let output = cmd.output().context("Failed to run workbloom add")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "workbloom add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Created worktree for branch: {}", branch)).green()
        );

        if switch {
            self.switch_with_workbloom(branch).await?;
        }

        Ok(())
    }

    async fn create_with_git(
        &self,
        branch: &str,
        base_dir: Option<&str>,
        switch: bool,
    ) -> Result<()> {
        let base_path = base_dir.unwrap_or("worktrees");
        let worktree_path = format!("{}/{}", base_path, branch);

        let output = Command::new("git")
            .args(["worktree", "add", &worktree_path, branch])
            .output()
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Created worktree for branch: {}", branch)).green()
        );

        if switch {
            self.switch_with_git(&worktree_path).await?;
        }

        Ok(())
    }

    async fn switch_with_gwtr(&self, worktree: &str) -> Result<()> {
        let output = Command::new("gwtr")
            .args(["cd", worktree])
            .output()
            .context("Failed to run gwtr cd")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "gwtr cd failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Switched to worktree: {}", worktree)).green()
        );
        Ok(())
    }

    async fn switch_with_workbloom(&self, worktree: &str) -> Result<()> {
        let output = Command::new("workbloom")
            .args(["cd", worktree])
            .output()
            .context("Failed to run workbloom cd")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "workbloom cd failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Switched to worktree: {}", worktree)).green()
        );
        Ok(())
    }

    async fn switch_with_git(&self, worktree: &str) -> Result<()> {
        // For git, we need to change directory manually
        println!(
            "{}",
            style(&format!("✓ Switched to worktree: {}", worktree)).green()
        );
        println!("{}", style(&format!("Please run: cd {}", worktree)).yellow());
        Ok(())
    }

    async fn remove_with_gwtr(&self, worktree: &str, with_branch: bool) -> Result<()> {
        let mut cmd = Command::new("gwtr");
        cmd.arg("remove").arg(worktree);

        if with_branch {
            cmd.arg("--with-branch");
        }

        let output = cmd.output().context("Failed to run gwtr remove")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "gwtr remove failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Removed worktree: {}", worktree)).green()
        );
        Ok(())
    }

    async fn remove_with_workbloom(&self, worktree: &str) -> Result<()> {
        let output = Command::new("workbloom")
            .args(["remove", worktree])
            .output()
            .context("Failed to run workbloom remove")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "workbloom remove failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Removed worktree: {}", worktree)).green()
        );
        Ok(())
    }

    async fn remove_with_git(&self, worktree: &str, with_branch: bool) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.args(["worktree", "remove", worktree]);

        if with_branch {
            cmd.arg("--force");
        }

        let output = cmd.output().context("Failed to run git worktree remove")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "git worktree remove failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        println!(
            "{}",
            style(&format!("✓ Removed worktree: {}", worktree)).green()
        );
        Ok(())
    }

    // Output parsing methods

        fn parse_gwtr_list_output(&self, output: &str) -> Result<Vec<WorktreeInfo>> {
        // Simple parsing for gwtr output
        let mut worktrees = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Basic parsing - adjust based on actual gwtr output format
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                worktrees.push(WorktreeInfo {
                    path: parts[0].to_string(),
                    branch: parts[1].to_string(),
                    commit: parts.get(2).unwrap_or(&"").to_string(),
                    current: false, // Would need to detect current
                    dirty: false,   // Would need to detect dirty state
                });
            }
        }
        
        Ok(worktrees)
    }

        fn parse_workbloom_list_output(&self, output: &str) -> Result<Vec<WorktreeInfo>> {
        // Simple parsing for workbloom output
        let mut worktrees = Vec::new();
        
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            // Basic parsing - adjust based on actual workbloom output format
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                worktrees.push(WorktreeInfo {
                    path: parts[0].to_string(),
                    branch: parts[1].to_string(),
                    commit: parts.get(2).unwrap_or(&"").to_string(),
                    current: false,
                    dirty: false,
                });
            }
        }
        
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
                "table" => {
                    print_worktree_table(&worktrees, detailed);
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
            tool: _,
            setup: _,
            copy_env: _,
            switch,
        } => {
            manager
                .create_worktree(&branch, base_dir.as_deref(), switch)
                .await?;
        }

        WorktreeCommands::Switch { worktree, tool: _ } => {
            manager.switch_worktree(&worktree).await?;
        }

        WorktreeCommands::Remove {
            worktree,
            with_branch,
            tool: _,
            force: _,
        } => {
            manager.remove_worktree(&worktree, with_branch).await?;
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
                        "available_tools": tools,
                        "best_tool": best_tool,
                        "config": manager.config
                    });
                    println!("{}", serde_json::to_string_pretty(&status)?);
                }
                "table" => {
                    print_tool_status(&tools, &best_tool, detailed);
                }
                _ => {
                    print_tool_status(&tools, &best_tool, detailed);
                }
            }
        }
    }

    Ok(())
}

/// Print worktree table
fn print_worktree_table(worktrees: &[WorktreeInfo], detailed: bool) {
    if worktrees.is_empty() {
        println!("{}", style("No worktrees found").yellow());
        return;
    }

    if detailed {
        println!(
            "{:<50} {:<30} {:<12} {:<8} {:<8}",
            style("Path").bold(),
            style("Branch").bold(),
            style("Commit").bold(),
            style("Current").bold(),
            style("Dirty").bold()
        );
        println!("{}", "-".repeat(120));

        for worktree in worktrees {
            println!(
                "{:<50} {:<30} {:<12} {:<8} {:<8}",
                worktree.path,
                worktree.branch,
                &worktree.commit[..worktree.commit.len().min(12)],
                if worktree.current { "✓" } else { "" },
                if worktree.dirty { "✗" } else { "" }
            );
        }
    } else {
        println!(
            "{:<50} {:<30}",
            style("Path").bold(),
            style("Branch").bold()
        );
        println!("{}", "-".repeat(80));

        for worktree in worktrees {
            println!("{:<50} {:<30}", worktree.path, worktree.branch);
        }
    }
}

/// Print worktree summary
fn print_worktree_summary(worktrees: &[WorktreeInfo]) {
    println!("{}", style("Worktree Summary").bold());
    println!("Total worktrees: {}", worktrees.len());

    let current_count = worktrees.iter().filter(|w| w.current).count();
    let dirty_count = worktrees.iter().filter(|w| w.dirty).count();

    println!("Current worktrees: {}", current_count);
    println!("Dirty worktrees: {}", dirty_count);
}

/// Print tool status
fn print_tool_status(tools: &[WorktreeTool], best_tool: &Option<WorktreeTool>, detailed: bool) {
    println!("{}", style("Worktree Tool Status").bold());

    if detailed {
        println!(
            "{:<10} {:<8} {:<50}",
            style("Tool").bold(),
            style("Status").bold(),
            style("Description").bold()
        );
        println!("{}", "-".repeat(70));

        for tool in tools {
            let status = if tool.is_available() {
                style("✓ Available").green()
            } else {
                style("✗ Not Available").red()
            };

            let is_best = best_tool
                .as_ref()
                .map(|bt| std::mem::discriminant(bt) == std::mem::discriminant(tool))
                .unwrap_or(false);
            let tool_name = if is_best {
                format!("{} (best)", tool.command_name())
            } else {
                tool.command_name().to_string()
            };

            println!("{:<10} {:<8} {:<50}", tool_name, status, tool.description());
        }
    } else {
        for tool in tools {
            let status = if tool.is_available() {
                style("✓").green()
            } else {
                style("✗").red()
            };

            let is_best = best_tool
                .as_ref()
                .map(|bt| std::mem::discriminant(bt) == std::mem::discriminant(tool))
                .unwrap_or(false);
            let tool_name = if is_best {
                format!("{} (best)", tool.command_name())
            } else {
                tool.command_name().to_string()
            };

            println!("{} {}", status, tool_name);
        }
    }
}
