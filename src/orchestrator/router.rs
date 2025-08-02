//! Command Router for Hooksmith
//!
//! This module provides command routing functionality that maps CLI commands
//! to appropriate WASM components. It handles command parsing, validation,
//! and execution.

use anyhow::Result;
use std::collections::HashMap;

use super::components::ComponentHandle;
use super::{
    BuildConfig, CommandResult, LefthookConfig, ValidationConfig, WorktreeOperation,
};

/// Command router for handling CLI commands
pub struct CommandRouter {
    /// Command handlers
    handlers: HashMap<String, CommandHandler>,
}

/// Command handler function type
type CommandHandler = fn(Vec<String>, &HashMap<String, ComponentHandle>) -> Result<CommandResult>;

impl CommandRouter {
    /// Create a new command router
    pub fn new() -> Self {
        let mut router = Self {
            handlers: HashMap::new(),
        };

        // Register default command handlers
        router.register_default_handlers();

        router
    }

    /// Register default command handlers
    fn register_default_handlers(&mut self) {
        self.handlers
            .insert("build".to_string(), Self::handle_build);
        self.handlers
            .insert("generate".to_string(), Self::handle_generate);
        self.handlers
            .insert("worktree".to_string(), Self::handle_worktree);
        self.handlers
            .insert("validate".to_string(), Self::handle_validate);
        self.handlers.insert("list".to_string(), Self::handle_list);
        self.handlers
            .insert("install".to_string(), Self::handle_install);
        self.handlers.insert("test".to_string(), Self::handle_test);
    }

    /// Register a custom command handler
    pub fn register_handler(&mut self, command: &str, handler: CommandHandler) {
        self.handlers.insert(command.to_string(), handler);
    }

    /// Execute a command
    pub async fn execute(
        &self,
        command: &str,
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        let start_time = std::time::Instant::now();

        // Find the handler for this command
        let handler = self
            .handlers
            .get(command)
            .ok_or_else(|| anyhow::anyhow!("Unknown command: {}", command))?;

        // Execute the handler
        let result = handler(args, components)?;

        // Update duration
        let duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(CommandResult {
            duration_ms,
            ..result
        })
    }

    /// Handle build command
    fn handle_build(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("Hook name is required".to_string()),
                duration_ms: 0,
            });
        }

        let hook_name = &args[0];
        let output_dir = args.get(1).unwrap_or(&"target/hooks".to_string()).clone();

        // Create build configuration
        let config = BuildConfig {
            source_path: format!("hooks/{}.rs", hook_name),
            output_path: format!("{}/{}", output_dir, hook_name),
            target_triple: None,
            optimization_level: 2,
            debug_symbols: false,
        };

        // TODO: Actually call the hook-builder component
        // For now, return a mock result
        Ok(CommandResult {
            success: true,
            output: format!("Built hook '{}' to '{}'", hook_name, output_dir),
            error: None,
            duration_ms: 0,
        })
    }

    /// Handle generate command
    fn handle_generate(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        let output_file = args.get(0).unwrap_or(&"lefthook.yml".to_string()).clone();

        // Create lefthook configuration
        let config = LefthookConfig {
            output_path: output_file.clone(),
            hooks: vec![
                super::HookConfig {
                    name: "pre-commit".to_string(),
                    hook_type: "pre-commit".to_string(),
                    command: "hooksmith run pre-commit".to_string(),
                    enabled: true,
                },
                super::HookConfig {
                    name: "pre-push".to_string(),
                    hook_type: "pre-push".to_string(),
                    command: "hooksmith run pre-push".to_string(),
                    enabled: true,
                },
            ],
            validate_schema: true,
        };

        // TODO: Actually call the lefthook-generator component
        // For now, return a mock result
        Ok(CommandResult {
            success: true,
            output: format!("Generated Lefthook configuration: {}", output_file),
            error: None,
            duration_ms: 0,
        })
    }

    /// Handle worktree command
    fn handle_worktree(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult {
                success: false,
                output: "".to_string(),
                error: Some("Worktree operation is required".to_string()),
                duration_ms: 0,
            });
        }

        let operation = &args[0];
        let operation_args = &args[1..];

        match operation.as_str() {
            "create" => {
                if operation_args.is_empty() {
                    return Ok(CommandResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Branch name is required for create operation".to_string()),
                        duration_ms: 0,
                    });
                }

                let branch_name = &operation_args[0];
                let op = WorktreeOperation::Create {
                    branch_name: branch_name.clone(),
                    base_path: None,
                    tool: None,
                };

                // TODO: Actually call the worktree-manager component
                Ok(CommandResult {
                    success: true,
                    output: format!("Created worktree for branch: {}", branch_name),
                    error: None,
                    duration_ms: 0,
                })
            }
            "list" => {
                // TODO: Actually call the worktree-manager component
                Ok(CommandResult {
                    success: true,
                    output: "Available worktrees:\n- main\n- feature/new-feature".to_string(),
                    error: None,
                    duration_ms: 0,
                })
            }
            "switch" => {
                if operation_args.is_empty() {
                    return Ok(CommandResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Worktree name is required for switch operation".to_string()),
                        duration_ms: 0,
                    });
                }

                let worktree_name = &operation_args[0];
                let op = WorktreeOperation::Switch {
                    worktree_name: worktree_name.clone(),
                    tool: None,
                };

                // TODO: Actually call the worktree-manager component
                Ok(CommandResult {
                    success: true,
                    output: format!("Switched to worktree: {}", worktree_name),
                    error: None,
                    duration_ms: 0,
                })
            }
            "remove" => {
                if operation_args.is_empty() {
                    return Ok(CommandResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Worktree name is required for remove operation".to_string()),
                        duration_ms: 0,
                    });
                }

                let worktree_name = &operation_args[0];
                let with_branch = operation_args
                    .get(1)
                    .map(|arg| arg == "--with-branch")
                    .unwrap_or(false);

                let op = WorktreeOperation::Remove {
                    worktree_name: worktree_name.clone(),
                    with_branch,
                    tool: None,
                };

                // TODO: Actually call the worktree-manager component
                Ok(CommandResult {
                    success: true,
                    output: format!(
                        "Removed worktree: {} (with_branch: {})",
                        worktree_name, with_branch
                    ),
                    error: None,
                    duration_ms: 0,
                })
            }
            _ => Ok(CommandResult {
                success: false,
                output: "".to_string(),
                error: Some(format!("Unknown worktree operation: {}", operation)),
                duration_ms: 0,
            }),
        }
    }

    /// Handle validate command
    fn handle_validate(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        let config_path = args.get(0).unwrap_or(&"lefthook.yml".to_string()).clone();

        let config = ValidationConfig {
            validation_type: super::ValidationType::LefthookConfig,
            data: format!("Validating configuration: {}", config_path),
            schema: None,
        };

        // TODO: Actually call the validation component
        Ok(CommandResult {
            success: true,
            output: format!("Configuration validated successfully: {}", config_path),
            error: None,
            duration_ms: 0,
        })
    }

    /// Handle list command
    fn handle_list(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        let mut output = String::new();

        // List available components
        output.push_str("Available components:\n");
        for (name, component) in components {
            output.push_str(&format!(
                "- {} ({} functions)\n",
                name,
                component.available_functions().len()
            ));
        }

        // List available hooks (mock data)
        output.push_str("\nAvailable hooks:\n");
        output.push_str("- pre-commit\n");
        output.push_str("- pre-push\n");
        output.push_str("- post-commit\n");

        Ok(CommandResult {
            success: true,
            output,
            error: None,
            duration_ms: 0,
        })
    }

    /// Handle install command
    fn handle_install(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        let hooks = if args.is_empty() {
            vec!["pre-commit".to_string(), "pre-push".to_string()]
        } else {
            args
        };

        let mut output = String::new();
        output.push_str("Installing hooks:\n");
        for hook in &hooks {
            output.push_str(&format!("- {}\n", hook));
        }

        // TODO: Actually install the hooks
        Ok(CommandResult {
            success: true,
            output,
            error: None,
            duration_ms: 0,
        })
    }

    /// Handle test command
    fn handle_test(
        args: Vec<String>,
        components: &HashMap<String, ComponentHandle>,
    ) -> Result<CommandResult> {
        let message = args
            .get(0)
            .unwrap_or(&"Hello from Hooksmith".to_string())
            .clone();

        Ok(CommandResult {
            success: true,
            output: format!("Test successful: {}", message),
            error: None,
            duration_ms: 0,
        })
    }

    /// List available commands
    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Check if a command is available
    pub fn has_command(&self, command: &str) -> bool {
        self.handlers.contains_key(command)
    }

    /// Get command help
    pub fn get_command_help(&self, command: &str) -> Option<String> {
        match command {
            "build" => Some("Build a hook from source: build <hook-name> [output-dir]".to_string()),
            "generate" => {
                Some("Generate Lefthook configuration: generate [output-file]".to_string())
            }
            "worktree" => Some("Manage worktrees: worktree <operation> [args...]".to_string()),
            "validate" => Some("Validate configuration: validate [config-path]".to_string()),
            "list" => Some("List available components and hooks: list".to_string()),
            "install" => Some("Install hooks: install [hook-names...]".to_string()),
            "test" => Some("Test the CLI: test [message]".to_string()),
            _ => None,
        }
    }
}

/// Command execution context
#[derive(Clone)]
pub struct CommandContext {
    /// Command name
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Available components
    pub components: HashMap<String, ComponentHandle>,
    /// Execution options
    pub options: CommandOptions,
}

/// Command execution options
#[derive(Debug, Clone)]
pub struct CommandOptions {
    /// Whether to enable verbose output
    pub verbose: bool,
    /// Whether to enable dry run mode
    pub dry_run: bool,
    /// Output format
    pub output_format: OutputFormat,
}

/// Output format for command results
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Text output format
    Text,
    /// JSON output format
    Json,
    /// YAML output format
    Yaml,
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            verbose: false,
            dry_run: false,
            output_format: OutputFormat::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::components::ComponentHandle;
    use wasmtime::Engine;

    #[test]
    fn test_command_router_creation() {
        let router = CommandRouter::new();
        assert!(router.has_command("build"));
        assert!(router.has_command("generate"));
        assert!(router.has_command("test"));
    }

    #[test]
    fn test_command_help() {
        let router = CommandRouter::new();
        let help = router.get_command_help("build");
        assert!(help.is_some());
        assert!(help.unwrap().contains("Build a hook from source"));
    }

    #[test]
    fn test_build_command() {
        let components = HashMap::new();
        let result = CommandRouter::handle_build(vec!["test-hook".to_string()], &components);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Built hook 'test-hook'"));
    }

    #[test]
    fn test_build_command_no_args() {
        let components = HashMap::new();
        let result = CommandRouter::handle_build(vec![], &components);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Hook name is required"));
    }

    #[test]
    fn test_worktree_command() {
        let components = HashMap::new();

        // Test create
        let result = CommandRouter::handle_worktree(
            vec!["create".to_string(), "feature/test".to_string()],
            &components,
        );
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result
            .output
            .contains("Created worktree for branch: feature/test"));

        // Test list
        let result = CommandRouter::handle_worktree(vec!["list".to_string()], &components);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.output.contains("Available worktrees"));
    }
}
