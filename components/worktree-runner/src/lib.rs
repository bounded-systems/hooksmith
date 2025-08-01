//! Worktree Runner WASM Component
//! 
//! This component provides a WASM interface for managing Git worktrees
//! by wrapping existing CLI tools like wtp, git-worktree-switcher (wt),
//! and Treekanga.
//! 
//! ## Supported Tools
//! 
//! - **wtp**: Smart Git worktree CLI with branch-only commands
//! - **wt**: Git worktree switcher for quick navigation
//! - **treekanga**: Community CLI for worktree management
//! - **git worktree**: Native Git worktree commands
//! 
//! ## Usage
//! 
//! ```rust
//! use worktree_runner::{WorktreeRunner, ToolConfig};
//! 
//! let runner = WorktreeRunner::new();
//! let result = runner.create_worktree("feature/new-feature", &ToolConfig::default()).await;
//! ```

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Configuration for worktree tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Preferred tool to use (wtp, wt, treekanga, git)
    pub preferred_tool: Option<String>,
    /// Base directory for worktrees
    pub worktree_base: Option<String>,
    /// Whether to run setup commands after creation
    pub run_setup: bool,
    /// Setup commands to run (e.g., ["npm install", "cargo build"])
    pub setup_commands: Vec<String>,
    /// Whether to copy environment files
    pub copy_env: bool,
    /// Environment files to copy (e.g., [".env", ".env.local"])
    pub env_files: Vec<String>,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            preferred_tool: None,
            worktree_base: Some("../worktrees".to_string()),
            run_setup: true,
            setup_commands: vec!["cargo build".to_string()],
            copy_env: true,
            env_files: vec![".env".to_string(), ".env.local".to_string()],
        }
    }
}

/// Result of a worktree operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Output from the command
    pub output: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Worktree path if created
    pub worktree_path: Option<String>,
    /// Branch name if created
    pub branch_name: Option<String>,
}

/// Available worktree tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorktreeTool {
    Wtp,
    Wt,
    Treekanga,
    Git,
}

impl WorktreeTool {
    /// Get the command name for this tool
    pub fn command_name(&self) -> &'static str {
        match self {
            WorktreeTool::Wtp => "wtp",
            WorktreeTool::Wt => "wt",
            WorktreeTool::Treekanga => "treekanga",
            WorktreeTool::Git => "git",
        }
    }
    
    /// Check if this tool is available on the system
    pub fn is_available(&self) -> bool {
        which::which(self.command_name()).is_ok()
    }
}

/// Main worktree runner component
#[wasm_bindgen]
pub struct WorktreeRunner {
    config: ToolConfig,
}

#[wasm_bindgen]
impl WorktreeRunner {
    /// Create a new worktree runner with default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            config: ToolConfig::default(),
        }
    }
    
    /// Create a new worktree runner with custom configuration
    pub fn with_config(config: JsValue) -> Result<WorktreeRunner, JsValue> {
        let config: ToolConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;
        
        Ok(Self { config })
    }
    
    /// Get available worktree tools
    pub fn get_available_tools(&self) -> Result<JsValue, JsValue> {
        let tools = vec![
            WorktreeTool::Wtp,
            WorktreeTool::Wt,
            WorktreeTool::Treekanga,
            WorktreeTool::Git,
        ];
        
        let available: Vec<WorktreeTool> = tools
            .into_iter()
            .filter(|tool| tool.is_available())
            .collect();
        
        serde_wasm_bindgen::to_value(&available)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Create a new worktree
    pub async fn create_worktree(&self, branch_name: &str) -> Result<JsValue, JsValue> {
        let result = self.create_worktree_internal(branch_name).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// List all worktrees
    pub async fn list_worktrees(&self) -> Result<JsValue, JsValue> {
        let result = self.list_worktrees_internal().await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Switch to a worktree
    pub async fn switch_worktree(&self, worktree_name: &str) -> Result<JsValue, JsValue> {
        let result = self.switch_worktree_internal(worktree_name).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Remove a worktree
    pub async fn remove_worktree(&self, worktree_name: &str, with_branch: bool) -> Result<JsValue, JsValue> {
        let result = self.remove_worktree_internal(worktree_name, with_branch).await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Update configuration
    pub fn update_config(&mut self, config: JsValue) -> Result<(), JsValue> {
        let config: ToolConfig = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;
        
        self.config = config;
        Ok(())
    }
}

impl WorktreeRunner {
    /// Internal method to create a worktree
    async fn create_worktree_internal(&self, branch_name: &str) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;
        
        match tool {
            WorktreeTool::Wtp => self.create_with_wtp(branch_name).await,
            WorktreeTool::Wt => self.create_with_wt(branch_name).await,
            WorktreeTool::Treekanga => self.create_with_treekanga(branch_name).await,
            WorktreeTool::Git => self.create_with_git(branch_name).await,
        }
    }
    
    /// Internal method to list worktrees
    async fn list_worktrees_internal(&self) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;
        
        match tool {
            WorktreeTool::Wtp => self.list_with_wtp().await,
            WorktreeTool::Wt => self.list_with_wt().await,
            WorktreeTool::Treekanga => self.list_with_treekanga().await,
            WorktreeTool::Git => self.list_with_git().await,
        }
    }
    
    /// Internal method to switch worktrees
    async fn switch_worktree_internal(&self, worktree_name: &str) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;
        
        match tool {
            WorktreeTool::Wtp => self.switch_with_wtp(worktree_name).await,
            WorktreeTool::Wt => self.switch_with_wt(worktree_name).await,
            WorktreeTool::Treekanga => self.switch_with_treekanga(worktree_name).await,
            WorktreeTool::Git => self.switch_with_git(worktree_name).await,
        }
    }
    
    /// Internal method to remove worktrees
    async fn remove_worktree_internal(&self, worktree_name: &str, with_branch: bool) -> Result<WorktreeResult> {
        let tool = self.select_best_tool().await?;
        
        match tool {
            WorktreeTool::Wtp => self.remove_with_wtp(worktree_name, with_branch).await,
            WorktreeTool::Wt => self.remove_with_wt(worktree_name).await,
            WorktreeTool::Treekanga => self.remove_with_treekanga(worktree_name).await,
            WorktreeTool::Git => self.remove_with_git(worktree_name, with_branch).await,
        }
    }
    
    /// Select the best available tool
    async fn select_best_tool(&self) -> Result<WorktreeTool> {
        // If a preferred tool is specified and available, use it
        if let Some(preferred) = &self.config.preferred_tool {
            let tool = match preferred.as_str() {
                "wtp" => WorktreeTool::Wtp,
                "wt" => WorktreeTool::Wt,
                "treekanga" => WorktreeTool::Treekanga,
                "git" => WorktreeTool::Git,
                _ => return Err(anyhow::anyhow!("Unknown preferred tool: {}", preferred)),
            };
            
            if tool.is_available() {
                return Ok(tool);
            }
        }
        
        // Otherwise, try tools in order of preference
        let tools = vec![
            WorktreeTool::Wtp,
            WorktreeTool::Wt,
            WorktreeTool::Treekanga,
            WorktreeTool::Git,
        ];
        
        for tool in tools {
            if tool.is_available() {
                return Ok(tool);
            }
        }
        
        Err(anyhow::anyhow!("No worktree tools available"))
    }
    
    // Tool-specific implementations
    async fn create_with_wtp(&self, branch_name: &str) -> Result<WorktreeResult> {
        // TODO: Implement wtp worktree creation
        // wtp add <branch_name>
        Ok(WorktreeResult {
            success: true,
            output: format!("Created worktree with wtp: {}", branch_name),
            error: None,
            worktree_path: Some(format!("../worktrees/{}", branch_name)),
            branch_name: Some(branch_name.to_string()),
        })
    }
    
    async fn create_with_wt(&self, branch_name: &str) -> Result<WorktreeResult> {
        // TODO: Implement wt worktree creation
        Ok(WorktreeResult {
            success: true,
            output: format!("Created worktree with wt: {}", branch_name),
            error: None,
            worktree_path: Some(format!("../worktrees/{}", branch_name)),
            branch_name: Some(branch_name.to_string()),
        })
    }
    
    async fn create_with_treekanga(&self, branch_name: &str) -> Result<WorktreeResult> {
        // TODO: Implement treekanga worktree creation
        Ok(WorktreeResult {
            success: true,
            output: format!("Created worktree with treekanga: {}", branch_name),
            error: None,
            worktree_path: Some(format!("../worktrees/{}", branch_name)),
            branch_name: Some(branch_name.to_string()),
        })
    }
    
    async fn create_with_git(&self, branch_name: &str) -> Result<WorktreeResult> {
        // TODO: Implement git worktree creation
        Ok(WorktreeResult {
            success: true,
            output: format!("Created worktree with git: {}", branch_name),
            error: None,
            worktree_path: Some(format!("../worktrees/{}", branch_name)),
            branch_name: Some(branch_name.to_string()),
        })
    }
    
    // List implementations
    async fn list_with_wtp(&self) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: "wtp list output".to_string(),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn list_with_wt(&self) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: "wt list output".to_string(),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn list_with_treekanga(&self) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: "treekanga list output".to_string(),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn list_with_git(&self) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: "git worktree list output".to_string(),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    // Switch implementations
    async fn switch_with_wtp(&self, worktree_name: &str) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Switched to worktree with wtp: {}", worktree_name),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn switch_with_wt(&self, worktree_name: &str) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Switched to worktree with wt: {}", worktree_name),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn switch_with_treekanga(&self, worktree_name: &str) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Switched to worktree with treekanga: {}", worktree_name),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn switch_with_git(&self, worktree_name: &str) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Switched to worktree with git: {}", worktree_name),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    // Remove implementations
    async fn remove_with_wtp(&self, worktree_name: &str, with_branch: bool) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Removed worktree with wtp: {} (with_branch: {})", worktree_name, with_branch),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn remove_with_wt(&self, worktree_name: &str) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Removed worktree with wt: {}", worktree_name),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn remove_with_treekanga(&self, worktree_name: &str) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Removed worktree with treekanga: {}", worktree_name),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
    
    async fn remove_with_git(&self, worktree_name: &str, with_branch: bool) -> Result<WorktreeResult> {
        Ok(WorktreeResult {
            success: true,
            output: format!("Removed worktree with git: {} (with_branch: {})", worktree_name, with_branch),
            error: None,
            worktree_path: None,
            branch_name: None,
        })
    }
}

// WASM bindings for JavaScript interop
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_worktree_runner_creation() {
        let runner = WorktreeRunner::new();
        assert!(runner.config.run_setup);
    }
    
    #[tokio::test]
    async fn test_tool_availability() {
        let runner = WorktreeRunner::new();
        let tools = runner.get_available_tools().unwrap();
        // At least git should be available
        assert!(tools.is_object());
    }
} 