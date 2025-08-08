use anyhow::Result;
use clap::{Parser, Subcommand};
use dircheck_core::{validate_files_index, validate_tree_commit, FileRuleSet, TreeRuleSet};
use std::process::{exit, Command};

#[derive(Parser)]
#[command(name = "git-hooks")]
#[command(about = "Rust-based Git hooks for Hooksmith")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Pre-commit hook: validate directory structure
    PreCommit,

    /// Pre-push hook: validate both directory and file structure
    PrePush,

    /// Run directory structure validation
    Tree,

    /// Run file structure validation
    Files,

    /// Run both validations
    All,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PreCommit => {
            println!("🔍 Running pre-commit validation...");
            if let Err(e) = run_tree_validation() {
                eprintln!("❌ Directory structure validation failed: {}", e);
                exit(1);
            }
            println!("✅ Pre-commit validation passed");
        }

        Commands::PrePush => {
            println!("🔍 Running pre-push validation...");

            // Run tree validation
            if let Err(e) = run_tree_validation() {
                eprintln!("❌ Directory structure validation failed: {}", e);
                exit(1);
            }

            // Run file validation
            if let Err(e) = run_file_validation() {
                eprintln!("❌ File structure validation failed: {}", e);
                exit(1);
            }

            println!("✅ Pre-push validation passed");
        }

        Commands::Tree => {
            if let Err(e) = run_tree_validation() {
                eprintln!("❌ Directory structure validation failed: {}", e);
                exit(1);
            }
            println!("✅ Directory structure validation passed");
        }

        Commands::Files => {
            if let Err(e) = run_file_validation() {
                eprintln!("❌ File structure validation failed: {}", e);
                exit(1);
            }
            println!("✅ File structure validation passed");
        }

        Commands::All => {
            println!("🔍 Running comprehensive validation...");

            if let Err(e) = run_tree_validation() {
                eprintln!("❌ Directory structure validation failed: {}", e);
                exit(1);
            }

            if let Err(e) = run_file_validation() {
                eprintln!("❌ File structure validation failed: {}", e);
                exit(1);
            }

            println!("✅ All validations passed");
        }
    }

    Ok(())
}

fn run_tree_validation() -> Result<()> {
    println!("📋 Validating directory structure...");

    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git ls-tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout.lines().map(str::to_string).collect();

    let rules = TreeRuleSet::default();
    let violations = validate_tree_commit(&paths, &rules);

    if !violations.is_empty() {
        eprintln!(
            "❌ Found {} directory structure violations:",
            violations.len()
        );
        for violation in violations {
            eprintln!("  - {}: {}", violation.path, violation.message);
        }
        return Err(anyhow::anyhow!("Directory structure validation failed"));
    }

    Ok(())
}

fn run_file_validation() -> Result<()> {
    println!("📋 Validating file structure...");

    let output = Command::new("git").args(["ls-files"]).output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout.lines().map(str::to_string).collect();

    let rules = FileRuleSet::default();
    let violations = validate_files_index(&paths, &rules);

    if !violations.is_empty() {
        eprintln!("❌ Found {} file structure violations:", violations.len());
        for violation in violations {
            eprintln!("  - {}: {}", violation.path, violation.message);
        }
        return Err(anyhow::anyhow!("File structure validation failed"));
    }

    Ok(())
}
