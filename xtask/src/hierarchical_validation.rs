//! Xtask Pipeline for Hierarchical Contract Validation
//!
//! This module provides the CLI interface and Git hook integration for
//! hierarchical contract validation using the bottom-up validation pipeline.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;

use hooksmith::modules::hierarchical_validation::{
    ChangeScope, HierarchicalValidator, ValidationResult, ValidationScope,
};

/// CLI for hierarchical contract validation
#[derive(Parser)]
#[command(name = "xtask-contract-validate")]
#[command(about = "Hierarchical contract validation pipeline")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clean operation for Git filter
    Clean {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Smudge operation for Git filter
    Smudge {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Diff operation for Git diff
    Diff {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Validate changes in a commit range
    Validate {
        /// Commit range (e.g., HEAD~1..HEAD)
        #[arg(long, default_value = "HEAD~1..HEAD")]
        range: String,
        /// Repository path
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Verify validation chain integrity
    Verify {
        /// Commit hash to verify
        #[arg(value_name = "COMMIT")]
        commit: String,
        /// Repository path
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Show validation notes for a commit
    Show {
        /// Commit hash
        #[arg(value_name = "COMMIT")]
        commit: String,
        /// Repository path
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Pre-commit hook for validation
    PreCommit {
        /// Repository path
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
    /// Post-commit hook for validation
    PostCommit {
        /// Repository path
        #[arg(long, default_value = ".")]
        repo: PathBuf,
    },
}

/// Run a hierarchical validation command
pub async fn run_command(command: Commands) -> Result<()> {
    match command {
        Commands::Clean { file } => {
            clean_operation(&file).await?;
        }
        Commands::Smudge { file } => {
            smudge_operation(&file).await?;
        }
        Commands::Diff { file } => {
            diff_operation(&file).await?;
        }
        Commands::Validate { range, repo } => {
            validate_changes(&range, &repo).await?;
        }
        Commands::Verify { commit, repo } => {
            verify_validation_chain(&commit, &repo).await?;
        }
        Commands::Show { commit, repo } => {
            show_validation_notes(&commit, &repo).await?;
        }
        Commands::PreCommit { repo } => {
            pre_commit_hook(&repo).await?;
        }
        Commands::PostCommit { repo } => {
            post_commit_hook(&repo).await?;
        }
    }

    Ok(())
}

/// Git filter clean operation
async fn clean_operation(file: &PathBuf) -> Result<()> {
    // Read the file content
    let content = fs::read_to_string(file)
        .await
        .context("Failed to read file")?;

    // For clean operation, we just output the content as-is
    // In a real implementation, you might want to validate the content
    print!("{}", content);

    Ok(())
}

/// Git filter smudge operation
async fn smudge_operation(_file: &PathBuf) -> Result<()> {
    // Read the file content from stdin (Git filter input)
    let mut content = String::new();
    std::io::Read::read_to_string(&mut std::io::stdin(), &mut content)
        .context("Failed to read stdin")?;

    // For smudge operation, we just output the content as-is
    // In a real implementation, you might want to validate the content
    print!("{}", content);

    Ok(())
}

/// Git diff operation
async fn diff_operation(file: &PathBuf) -> Result<()> {
    // Read the file content
    let content = fs::read_to_string(file)
        .await
        .context("Failed to read file")?;

    // For diff operation, we output the content for text conversion
    print!("{}", content);

    Ok(())
}

/// Validate changes in a commit range
async fn validate_changes(range: &str, repo: &PathBuf) -> Result<()> {
    println!("🔍 Detecting changes in range: {}", range);

    let validator = HierarchicalValidator::new(repo.clone());

    // Detect changes
    let changes = validator
        .detect_changes(Some(range))
        .await
        .context("Failed to detect changes")?;

    if changes.is_empty() {
        println!("✅ No changes detected in range: {}", range);
        return Ok(());
    }

    println!("📝 Found {} change scopes:", changes.len());
    for change in &changes {
        println!("  - {}: {:?} scope", change.file.display(), change.scope);
    }

    // Run hierarchical validation
    println!("🔧 Running hierarchical validation...");
    let results = validator
        .validate_hierarchically(changes)
        .await
        .context("Failed to validate changes")?;

    // Report results
    let mut total_validated = 0;
    let mut total_failed = 0;

    for result in &results {
        if result.validated {
            total_validated += 1;
            println!(
                "✅ {:?} scope validated successfully ({}ms)",
                result.scope, result.duration_ms
            );
        } else {
            total_failed += 1;
            println!(
                "❌ {:?} scope validation failed ({}ms)",
                result.scope, result.duration_ms
            );
            for error in &result.errors {
                println!("    - {}: {}", error.severity, error.message);
            }
        }
    }

    println!("\n📊 Validation Summary:");
    println!("  - Total scopes: {}", results.len());
    println!("  - Validated: {}", total_validated);
    println!("  - Failed: {}", total_failed);

    if total_failed > 0 {
        anyhow::bail!("Validation failed for {} scopes", total_failed);
    }

    println!("✅ All validations passed successfully!");
    Ok(())
}

/// Verify validation chain integrity
async fn verify_validation_chain(commit: &str, repo: &PathBuf) -> Result<()> {
    println!("🔍 Verifying validation chain for commit: {}", commit);

    let validator = HierarchicalValidator::new(repo.clone());

    let is_valid = validator
        .verify_validation_chain(commit)
        .await
        .context("Failed to verify validation chain")?;

    if is_valid {
        println!("✅ Validation chain integrity verified successfully!");
    } else {
        println!("❌ Validation chain integrity check failed!");
        anyhow::bail!("Validation chain integrity check failed");
    }

    Ok(())
}

/// Show validation notes for a commit
async fn show_validation_notes(commit: &str, repo: &PathBuf) -> Result<()> {
    println!("📝 Validation notes for commit: {}", commit);

    let validator = HierarchicalValidator::new(repo.clone());

    // Get validation notes
    let notes = validator
        .get_validation_notes(commit)
        .await
        .context("Failed to get validation notes")?;

    if notes.is_empty() {
        println!("ℹ️  No validation notes found for commit: {}", commit);
        return Ok(());
    }

    println!("Found {} validation notes:", notes.len());
    println!();

    for note in &notes {
        println!("📋 Scope: {}", note.scope);
        println!("   File: {}", note.file);
        if let Some(range) = &note.range {
            println!("   Range: {}:{}", range.start_line, range.end_line);
        }
        println!("   Hash: {}", note.hash);
        println!("   Validated: {}", note.validated);
        println!("   Contract: {}", note.contract_type);
        println!("   Duration: {}ms", note.validation_duration_ms);
        println!("   Timestamp: {}", note.timestamp);

        if !note.validation_errors.is_empty() {
            println!("   Errors:");
            for error in &note.validation_errors {
                println!("     - {}: {}", error.severity, error.message);
            }
        }

        if !note.child_scopes.is_empty() {
            println!("   Child scopes:");
            for child in &note.child_scopes {
                println!("     - {}: {}", child.scope, child.hash);
            }
        }

        println!();
    }

    Ok(())
}

/// Pre-commit hook for validation
async fn pre_commit_hook(repo: &PathBuf) -> Result<()> {
    println!("🔧 Running pre-commit validation hook...");

    // Get staged changes
    let output = Command::new("git")
        .args(&["diff", "--cached", "--name-only"])
        .current_dir(repo)
        .output()
        .context("Failed to get staged changes")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get staged changes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let staged_files: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();

    if staged_files.is_empty() {
        println!("✅ No staged changes to validate");
        return Ok(());
    }

    println!("📝 Found {} staged files:", staged_files.len());
    for file in &staged_files {
        println!("  - {}", file);
    }

    // For pre-commit, we'll validate the current working directory state
    // against the staged changes
    let validator = HierarchicalValidator::new(repo.clone());

    // Create a temporary commit to validate against
    let temp_commit = create_temp_commit(repo).await?;

    // Validate the changes
    let changes = validator
        .detect_changes(Some(&format!("{}~1..{}", temp_commit, temp_commit)))
        .await
        .context("Failed to detect changes")?;

    if !changes.is_empty() {
        let results = validator
            .validate_hierarchically(changes)
            .await
            .context("Failed to validate changes")?;

        let failed_count = results.iter().filter(|r| !r.validated).count();
        if failed_count > 0 {
            // Clean up temp commit
            cleanup_temp_commit(repo, &temp_commit).await?;
            anyhow::bail!("Pre-commit validation failed for {} scopes", failed_count);
        }
    }

    // Clean up temp commit
    cleanup_temp_commit(repo, &temp_commit).await?;

    println!("✅ Pre-commit validation passed successfully!");
    Ok(())
}

/// Post-commit hook for validation
async fn post_commit_hook(repo: &PathBuf) -> Result<()> {
    println!("🔧 Running post-commit validation hook...");

    // Get the current commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .context("Failed to get current commit hash")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get current commit hash: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let commit_hash = String::from_utf8(output.stdout)?.trim().to_string();
    println!("📝 Validating commit: {}", commit_hash);

    // Validate the changes in this commit
    let validator = HierarchicalValidator::new(repo.clone());

    let changes = validator
        .detect_changes(Some(&format!("{}~1..{}", commit_hash, commit_hash)))
        .await
        .context("Failed to detect changes")?;

    if changes.is_empty() {
        println!("✅ No changes to validate in this commit");
        return Ok(());
    }

    let results = validator
        .validate_hierarchically(changes)
        .await
        .context("Failed to validate changes")?;

    let failed_count = results.iter().filter(|r| !r.validated).count();
    if failed_count > 0 {
        println!(
            "⚠️  Post-commit validation failed for {} scopes",
            failed_count
        );
        println!("   Validation notes have been stored in Git notes");
        println!(
            "   Use 'xtask-contract-validate show {}' to view details",
            commit_hash
        );
    } else {
        println!("✅ Post-commit validation passed successfully!");
    }

    Ok(())
}

/// Create a temporary commit for validation
async fn create_temp_commit(repo: &PathBuf) -> Result<String> {
    // Create a temporary commit with staged changes
    let output = Command::new("git")
        .args(&["commit", "--no-verify", "-m", "temp: validation commit"])
        .current_dir(repo)
        .output()
        .context("Failed to create temp commit")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to create temp commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Get the commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .context("Failed to get temp commit hash")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get temp commit hash: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

/// Clean up temporary commit
async fn cleanup_temp_commit(repo: &PathBuf, commit_hash: &str) -> Result<()> {
    // Reset to the previous commit
    let output = Command::new("git")
        .args(&["reset", "--soft", &format!("{}~1", commit_hash)])
        .current_dir(repo)
        .output()
        .context("Failed to reset temp commit")?;

    if !output.status.success() {
        eprintln!(
            "Warning: Failed to reset temp commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}
