//! Command-line interface for lefthook-rs
//!
//! This module provides a CLI interface for the lefthook-rs crate,
//! allowing users to interact with Lefthook through Rust-native commands.

use crate::config::{HookConfig, JobConfig};
use crate::error::Result;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

/// CLI for lefthook-rs
#[derive(Parser)]
#[command(name = "lefthook-rs")]
#[command(about = "Rust wrapper for Lefthook Git hooks manager")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install Lefthook hooks in the current repository
    Install {
        /// Path to the lefthook.yml configuration file
        #[arg(long, default_value = "lefthook.yml")]
        config: PathBuf,
    },
    
    /// Run a specific Lefthook hook
    Run {
        /// Name of the hook to run
        hook_name: String,
        
        /// Path to the lefthook.yml configuration file
        #[arg(long, default_value = "lefthook.yml")]
        config: PathBuf,
    },
    
    /// Generate a lefthook.yml configuration file
    Generate {
        /// Output path for the configuration file
        #[arg(long, default_value = "lefthook.yml")]
        output: PathBuf,
        
        /// Whether to overwrite existing file
        #[arg(long)]
        overwrite: bool,
        
        /// Template to use for generation
        #[arg(long, default_value = "rust")]
        template: String,
    },
    
    /// Validate a lefthook.yml configuration file
    Validate {
        /// Path to the configuration file to validate
        #[arg(long, default_value = "lefthook.yml")]
        config: PathBuf,
    },
    
    /// Check if Lefthook is installed and working
    Check,
    
    /// Get the version of the installed Lefthook binary
    Version,
    
    /// Create a new project with Lefthook configuration
    Init {
        /// Project name
        name: String,
        
        /// Project directory
        #[arg(long)]
        dir: Option<PathBuf>,
        
        /// Template to use
        #[arg(long, default_value = "rust")]
        template: String,
    },
}

/// Run the CLI application
pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Install { config: _ } => {
            println!("Installing Lefthook hooks...");
            crate::install().await?;
            println!("✅ Hooks installed successfully");
        }
        
        Commands::Run { hook_name, config: _ } => {
            println!("Running hook: {}", hook_name);
            crate::run_hook(&hook_name).await?;
            println!("✅ Hook completed successfully");
        }
        
        Commands::Generate { output, overwrite, template } => {
            println!("Generating lefthook.yml configuration...");
            let config = generate_config_from_template(&template)?;
            
            if output.exists() && !overwrite {
                eprintln!("❌ File already exists. Use --overwrite to overwrite.");
                return Ok(());
            }
            
            config.write_to_file(&output).await?;
            println!("✅ Configuration generated at: {:?}", output);
        }
        
        Commands::Validate { config } => {
            println!("Validating configuration...");
            crate::validate_config(&config).await?;
            println!("✅ Configuration is valid");
        }
        
        Commands::Check => {
            println!("Checking Lefthook installation...");
            crate::check_installation().await?;
            println!("✅ Lefthook is installed and working");
        }
        
        Commands::Version => {
            let version = crate::get_version().await?;
            println!("Lefthook version: {}", version);
        }
        
        Commands::Init { name, dir, template } => {
            println!("Initializing new project: {}", name);
            init_project(&name, dir, &template).await?;
            println!("✅ Project initialized successfully");
        }
    }

    Ok(())
}

/// Generate a configuration from a template
fn generate_config_from_template(template: &str) -> Result<HookConfig> {
    let mut config = HookConfig::default();

    match template {
        "rust" => {
            // Rust project template
            let mut fmt_job = JobConfig::new("cargo fmt --all -- --check");
            fmt_job.with_files("*.rs");
            config.add_pre_commit_hook("fmt".to_string(), fmt_job);
            
            let mut clippy_job = JobConfig::new("cargo clippy --all-targets --all-features -- -D warnings");
            clippy_job.with_files("*.rs");
            config.add_pre_commit_hook("clippy".to_string(), clippy_job);
            
            let mut test_job = JobConfig::new("cargo test --all-targets --all-features");
            test_job.with_files("*.rs");
            config.add_pre_commit_hook("test".to_string(), test_job);

            config.add_pre_push_hook("audit".to_string(), JobConfig::new("cargo audit"));
            config.add_pre_push_hook("build".to_string(), JobConfig::new("cargo build --release"));

            config.add_commit_msg_hook(
                "conventional-commits".to_string(),
                JobConfig::new(r#"if ! echo "$1" | grep -qE "^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .+"; then
    echo "Commit message must follow conventional commit format:"
    echo "  <type>(<scope>): <description>"
    echo "  Examples:"
    echo "    feat(cli): add new command"
    echo "    fix(wasm): correct parsing bug"
    echo "    docs: update README"
    exit 1
fi"#),
            );
        }
        
        "node" => {
            // Node.js project template
            let mut lint_job = JobConfig::new("npm run lint");
            lint_job.with_files("*.js,*.ts,*.jsx,*.tsx");
            config.add_pre_commit_hook("lint".to_string(), lint_job);
            
            config.add_pre_commit_hook("test".to_string(), JobConfig::new("npm test"));
            config.add_pre_push_hook("build".to_string(), JobConfig::new("npm run build"));
        }
        
        "python" => {
            // Python project template
            let mut black_job = JobConfig::new("black --check .");
            black_job.with_files("*.py");
            config.add_pre_commit_hook("black".to_string(), black_job);
            
            let mut flake8_job = JobConfig::new("flake8 .");
            flake8_job.with_files("*.py");
            config.add_pre_commit_hook("flake8".to_string(), flake8_job);
            
            let mut pytest_job = JobConfig::new("pytest");
            pytest_job.with_files("*.py");
            config.add_pre_commit_hook("pytest".to_string(), pytest_job);
        }
        
        "go" => {
            // Go project template
            let mut fmt_job = JobConfig::new("go fmt ./...");
            fmt_job.with_files("*.go");
            config.add_pre_commit_hook("fmt".to_string(), fmt_job);
            
            let mut vet_job = JobConfig::new("go vet ./...");
            vet_job.with_files("*.go");
            config.add_pre_commit_hook("vet".to_string(), vet_job);
            
            let mut test_job = JobConfig::new("go test ./...");
            test_job.with_files("*.go");
            config.add_pre_commit_hook("test".to_string(), test_job);
        }
        
        _ => {
            return Err(crate::error::LefthookError::Configuration(
                format!("Unknown template: {}", template),
            ));
        }
    }

    Ok(config)
}

/// Initialize a new project with Lefthook configuration
async fn init_project(name: &str, dir: Option<PathBuf>, template: &str) -> Result<()> {
    use std::fs;
    use tokio::fs as tokio_fs;

    let project_dir = dir.unwrap_or_else(|| PathBuf::from(name));
    
    // Create project directory
    fs::create_dir_all(&project_dir)?;
    
    // Generate configuration
    let config = generate_config_from_template(template)?;
    let config_path = project_dir.join("lefthook.yml");
    config.write_to_file(&config_path).await?;
    
    // Create README
    let readme_content = format!(
        r#"# {}

This project uses Lefthook for Git hooks management.

## Setup

1. Install Lefthook:
   ```bash
   npm install -g @evilmartians/lefthook
   # or
   brew install lefthook
   ```

2. Install hooks:
   ```bash
   lefthook install
   ```

## Available Hooks

- **pre-commit**: Runs before each commit
- **pre-push**: Runs before pushing to remote
- **commit-msg**: Validates commit messages

## Configuration

Hooks are configured in `lefthook.yml`. This file was generated using the `{}` template.

## Development

```bash
# Run hooks manually
lefthook run pre-commit
lefthook run pre-push

# Validate configuration
lefthook run commit-msg
```
"#,
        name, template
    );
    
    let readme_path = project_dir.join("README.md");
    tokio_fs::write(readme_path, readme_content).await?;
    
    // Initialize Git repository
    let git_status = std::process::Command::new("git")
        .arg("init")
        .current_dir(&project_dir)
        .status();
    
    if git_status.is_ok() {
        println!("   Git repository initialized");
    }
    
    // Install hooks
    let install_status = std::process::Command::new("lefthook")
        .arg("install")
        .current_dir(&project_dir)
        .status();
    
    if install_status.is_ok() {
        println!("   Lefthook hooks installed");
    }
    
    println!("   Project created at: {:?}", project_dir);
    println!("   Configuration: {:?}", config_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_config_from_template() {
        let config = generate_config_from_template("rust").unwrap();
        assert!(config.pre_commit.is_some());
        assert!(config.pre_push.is_some());
        assert!(config.commit_msg.is_some());
        
        let pre_commit = config.pre_commit.unwrap();
        assert!(pre_commit.jobs.contains_key("fmt"));
        assert!(pre_commit.jobs.contains_key("clippy"));
        assert!(pre_commit.jobs.contains_key("test"));
    }

    #[test]
    fn test_generate_config_from_unknown_template() {
        let result = generate_config_from_template("unknown");
        assert!(result.is_err());
    }
}
