//! Git Hooks Demo
//! 
//! This example demonstrates how to set up Git hooks using Rust binaries
//! and Git aliases, eliminating the need for shell scripts.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Example Git hooks configuration
fn create_hooks_config() -> git_hooks::GitHooksConfig {
    let mut config = git_hooks::GitHooksConfig::new();
    
    // Add some common hooks
    config.add_hook(git_hooks::GitHook {
        name: "pre-commit".to_string(),
        binary_name: "hook-pre-commit".to_string(),
        source_path: "hooks/pre_commit.rs".to_string(),
        alias_name: "hook-pre-commit".to_string(),
        enabled: true,
        description: Some("Run linting and tests before commit".to_string()),
        dependencies: Vec::new(),
        config: HashMap::new(),
    });
    
    config.add_hook(git_hooks::GitHook {
        name: "commit-msg".to_string(),
        binary_name: "hook-commit-msg".to_string(),
        source_path: "hooks/commit_msg.rs".to_string(),
        alias_name: "hook-commit-msg".to_string(),
        enabled: true,
        description: Some("Validate commit message format".to_string()),
        dependencies: Vec::new(),
        config: HashMap::new(),
    });
    
    config.add_hook(git_hooks::GitHook {
        name: "pre-push".to_string(),
        binary_name: "hook-pre-push".to_string(),
        source_path: "hooks/pre_push.rs".to_string(),
        alias_name: "hook-pre-push".to_string(),
        enabled: true,
        description: Some("Run security checks before push".to_string()),
        dependencies: Vec::new(),
        config: HashMap::new(),
    });
    
    config
}

/// Example hook implementation for pre-commit
fn generate_pre_commit_hook() -> String {
    r#"//! Pre-commit Hook
//! 
//! This hook runs before each commit to ensure code quality.

use std::env;
use std::process;
use std::process::Command;

fn main() {
    println!("🔍 Running pre-commit checks...");
    
    // Get hook arguments
    let args: Vec<String> = env::args().collect();
    
    // Run cargo check
    println!("  📦 Running cargo check...");
    let check_status = Command::new("cargo")
        .args(["check"])
        .status();
    
    if let Ok(status) = check_status {
        if !status.success() {
            eprintln!("❌ Cargo check failed");
            process::exit(1);
        }
    } else {
        eprintln!("❌ Failed to run cargo check");
        process::exit(1);
    }
    
    // Run clippy
    println!("  🔍 Running clippy...");
    let clippy_status = Command::new("cargo")
        .args(["clippy", "--", "-D", "warnings"])
        .status();
    
    if let Ok(status) = clippy_status {
        if !status.success() {
            eprintln!("❌ Clippy found issues");
            process::exit(1);
        }
    } else {
        eprintln!("❌ Failed to run clippy");
        process::exit(1);
    }
    
    // Run tests
    println!("  🧪 Running tests...");
    let test_status = Command::new("cargo")
        .args(["test"])
        .status();
    
    if let Ok(status) = test_status {
        if !status.success() {
            eprintln!("❌ Tests failed");
            process::exit(1);
        }
    } else {
        eprintln!("❌ Failed to run tests");
        process::exit(1);
    }
    
    println!("✅ Pre-commit checks passed");
    process::exit(0);
}
"#.to_string()
}

/// Example hook implementation for commit-msg
fn generate_commit_msg_hook() -> String {
    r#"//! Commit Message Hook
//! 
//! This hook validates commit message format.

use std::env;
use std::fs;
use std::process;

fn main() {
    println!("📝 Validating commit message...");
    
    // Get commit message file path
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("❌ No commit message file specified");
        process::exit(1);
    }
    
    let msg_file = &args[1];
    
    // Read commit message
    let msg_content = match fs::read_to_string(msg_file) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("❌ Failed to read commit message file");
            process::exit(1);
        }
    };
    
    // Validate message format
    if !validate_commit_message(&msg_content) {
        eprintln!("❌ Invalid commit message format");
        eprintln!("   Expected: <type>(<scope>): <description>");
        eprintln!("   Example: feat(api): add user authentication");
        process::exit(1);
    }
    
    println!("✅ Commit message is valid");
    process::exit(0);
}

fn validate_commit_message(message: &str) -> bool {
    let lines: Vec<&str> = message.lines().collect();
    if lines.is_empty() {
        return false;
    }
    
    let first_line = lines[0].trim();
    
    // Check for conventional commit format
    // Pattern: <type>(<scope>): <description>
    let parts: Vec<&str> = first_line.split(':').collect();
    if parts.len() != 2 {
        return false;
    }
    
    let type_scope = parts[0].trim();
    let description = parts[1].trim();
    
    // Validate type and scope
    if !type_scope.contains('(') || !type_scope.contains(')') {
        return false;
    }
    
    // Check for valid types
    let valid_types = ["feat", "fix", "docs", "style", "refactor", "test", "chore"];
    let type_part = type_scope.split('(').next().unwrap_or("");
    
    if !valid_types.contains(&type_part) {
        return false;
    }
    
    // Check description length
    if description.len() < 10 {
        return false;
    }
    
    true
}
"#.to_string()
}

/// Example hook implementation for pre-push
fn generate_pre_push_hook() -> String {
    r#"//! Pre-push Hook
//! 
//! This hook runs security checks before pushing.

use std::env;
use std::process;
use std::process::Command;

fn main() {
    println!("🔒 Running pre-push security checks...");
    
    // Get hook arguments
    let args: Vec<String> = env::args().collect();
    
    // Check for secrets in code
    println!("  🔍 Scanning for secrets...");
    let secrets_check = check_for_secrets();
    if !secrets_check {
        eprintln!("❌ Potential secrets found in code");
        process::exit(1);
    }
    
    // Run security audit
    println!("  🛡️ Running security audit...");
    let audit_status = Command::new("cargo")
        .args(["audit"])
        .status();
    
    if let Ok(status) = audit_status {
        if !status.success() {
            eprintln!("❌ Security audit failed");
            process::exit(1);
        }
    } else {
        eprintln!("⚠️ Security audit not available");
    }
    
    // Check for large files
    println!("  📏 Checking file sizes...");
    let size_check = check_file_sizes();
    if !size_check {
        eprintln!("❌ Large files detected");
        process::exit(1);
    }
    
    println!("✅ Pre-push checks passed");
    process::exit(0);
}

fn check_for_secrets() -> bool {
    // Simple check for common secret patterns
    let secret_patterns = [
        "password",
        "secret",
        "api_key",
        "token",
        "private_key",
    ];
    
    // This is a simplified check - in practice you'd scan actual files
    true // Always pass for demo
}

fn check_file_sizes() -> bool {
    // Check for files larger than 10MB
    // This is a simplified check - in practice you'd scan actual files
    true // Always pass for demo
}
"#.to_string()
}

/// Generate Cargo.toml binary entries
fn generate_cargo_binaries() -> String {
    r#"# Git Hook Binaries
# Generated by git-hooks-manager

[[bin]]
name = "hook-pre-commit"
path = "hooks/pre_commit.rs"

[[bin]]
name = "hook-commit-msg"
path = "hooks/commit_msg.rs"

[[bin]]
name = "hook-pre-push"
path = "hooks/pre_push.rs"
"#.to_string()
}

/// Demo function to show the complete workflow
fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Git Hooks Demo");
    println!("==================");
    
    // Create hooks configuration
    let config = create_hooks_config();
    
    // Generate hook source files
    println!("\n📝 Generating hook source files...");
    fs::create_dir_all("hooks")?;
    fs::write("hooks/pre_commit.rs", generate_pre_commit_hook())?;
    fs::write("hooks/commit_msg.rs", generate_commit_msg_hook())?;
    fs::write("hooks/pre_push.rs", generate_pre_push_hook())?;
    
    // Generate Cargo.toml binary entries
    println!("📦 Generating Cargo.toml binary entries...");
    let cargo_binaries = generate_cargo_binaries();
    fs::write("cargo-binaries.toml", cargo_binaries)?;
    
    // Generate hooks configuration
    println!("⚙️ Generating hooks configuration...");
    let jsonc = config.to_jsonc()?;
    fs::write("git-hooks.jsonc", jsonc)?;
    
    // Show what would be installed
    println!("\n📋 Hook Configuration Summary:");
    println!("==============================");
    
    for hook in &config.hooks {
        println!("• {}: {}", hook.name, hook.description.as_ref().unwrap_or(&"No description".to_string()));
        println!("  Binary: {}", hook.binary_name);
        println!("  Source: {}", hook.source_path);
        println!("  Alias: {}", hook.alias_name);
        println!();
    }
    
    // Show Git alias commands
    println!("🔗 Git Aliases that would be created:");
    println!("=====================================");
    for hook in &config.hooks {
        println!("git config --local alias.{} '!./.git/hooks/bin/{}'", 
            hook.alias_name, hook.binary_name);
    }
    println!();
    
    // Show hook stub examples
    println!("📄 Hook Stub Examples:");
    println!("======================");
    for hook in &config.hooks {
        println!("# .git/hooks/{}", hook.name);
        println!("exec git {}\"$@\"", hook.alias_name);
        println!();
    }
    
    println!("✅ Demo completed successfully!");
    println!("\nTo install these hooks, run:");
    println!("cargo xtask git-hooks install --config git-hooks.jsonc");
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_demo()
}
