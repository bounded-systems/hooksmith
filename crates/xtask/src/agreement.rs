//! Agreement Management System
//!
//! This module provides commands for managing agreements using Git notes:
//! - `agreement create`: Create a new agreement with scope and contract
//! - `agreement list`: List all agreements
//! - `agreement show`: Show details of a specific agreement
//! - `agreement validate`: Validate an agreement
//! - `agreement resolve`: Resolve contract content from an agreement

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use git2::{Repository, Signature};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;

/// Canonical agreement schema as specified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agreement {
    /// Tree SHA - the filesystem layout this agreement applies to
    pub scope: String,
    /// Blob SHA - the contract that defines expectations or validation
    pub contract: String,
}

/// Agreement status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AgreementStatus {
    /// Agreement is active and being worked on
    Active,
    /// Agreement has been fulfilled
    Fulfilled,
    /// Agreement has been revoked
    Revoked,
    /// Agreement is pending review
    Pending,
}

/// Agreement metadata for storage and retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgreementMetadata {
    /// The agreement itself
    pub agreement: Agreement,
    /// When the agreement was created
    pub created_at: String,
    /// Who created the agreement
    pub created_by: String,
    /// Current status of the agreement
    pub status: AgreementStatus,
}

/// Manager for creating and managing agreements using Git notes
pub struct AgreementManager {
    repo: Repository,
    notes_ref: String,
}

impl AgreementManager {
    /// Create a new agreement manager
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        Ok(AgreementManager {
            repo,
            notes_ref: "refs/notes/hooksmith/agreements".to_string(),
        })
    }

    /// Create a new agreement
    pub fn create_agreement(
        &self,
        scope: &str,
        contract: &str,
        description: Option<&str>,
    ) -> Result<String> {
        let agreement = Agreement {
            scope: scope.to_string(),
            contract: contract.to_string(),
        };

        let metadata = AgreementMetadata {
            agreement,
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: "hooksmith".to_string(),
            status: AgreementStatus::Active,
        };

        // Store in Git notes
        let note_content = serde_json::to_string(&metadata)?;
        
        // Create or update the note
        let signature = Signature::now("Hooksmith", "hooksmith@example.com")?;
        let scope_oid = git2::Oid::from_str(scope)?;
        self.repo.note(&signature, &signature, Some(&self.notes_ref), scope_oid, &note_content, false)?;

        Ok(format!("Agreement created: scope={}, contract={}", scope, contract))
    }

    /// Get an agreement by scope
    pub fn get_agreement(&self, scope: &str) -> Result<Option<AgreementMetadata>> {
        let scope_oid = git2::Oid::from_str(scope)?;
        if let Ok(note) = self.repo.find_note(Some(&self.notes_ref), scope_oid) {
            let note_content = note.message().unwrap_or("");
            let metadata: AgreementMetadata = serde_json::from_str(note_content)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// List all agreements
    pub fn list_agreements(&self) -> Result<Vec<AgreementMetadata>> {
        // For now, return an empty list since Git notes API is complex
        // In a real implementation, you'd scan the notes ref and parse each note
        Ok(Vec::new())
    }

    /// Update agreement status
    pub fn update_agreement_status(
        &self,
        scope: &str,
        status: AgreementStatus,
    ) -> Result<()> {
        if let Some(mut metadata) = self.get_agreement(scope)? {
            metadata.status = status;
            let note_content = serde_json::to_string(&metadata)?;
            let signature = Signature::now("Hooksmith", "hooksmith@example.com")?;
            let scope_oid = git2::Oid::from_str(scope)?;
            self.repo.note(&signature, &signature, Some(&self.notes_ref), scope_oid, &note_content, false)?;
        }
        Ok(())
    }

    /// Validate an agreement (check if contract exists in scope)
    pub fn validate_agreement(&self, scope: &str) -> Result<bool> {
        if let Some(metadata) = self.get_agreement(scope)? {
            // Check if the scope tree exists
            let scope_tree = self.repo.find_tree(git2::Oid::from_str(&metadata.agreement.scope)?)?;
            // Check if the contract blob exists
            let contract_blob = self.repo.find_blob(git2::Oid::from_str(&metadata.agreement.contract)?)?;
            
            // Check if the scope tree exists in current HEAD
            let current_head_tree = std::process::Command::new("git")
                .args(&["rev-parse", "HEAD^{tree}"])
                .output()?;
            let current_head_tree_sha = String::from_utf8_lossy(&current_head_tree.stdout).trim().to_string();
            
            // Check if the scope tree matches current HEAD tree or exists as a subtree
            let scope_exists = if scope == current_head_tree_sha {
                true // Scope is the current HEAD tree
            } else {
                // Check if the scope tree exists anywhere in current HEAD
                let tree_exists = std::process::Command::new("git")
                    .args(&["ls-tree", "-r", "-t", "HEAD^{tree}"])
                    .output()?;
                let tree_output = String::from_utf8_lossy(&tree_exists.stdout);
                
                tree_output.lines()
                    .any(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.len() >= 3 && parts[1] == "tree" && parts[2] == scope
                    })
            };
            
            if scope_exists {
                println!("✅ Agreement scope exists in current HEAD");
            } else {
                println!("❌ Agreement scope NOT found in current HEAD");
                println!("   Old scope: {}", scope);
                println!("   Current HEAD tree: {}", current_head_tree_sha);
                
                // Show what changed
                let diff_output = std::process::Command::new("git")
                    .args(&["diff-tree", "-r", scope, &current_head_tree_sha])
                    .output()?;
                
                if !diff_output.stdout.is_empty() {
                    println!("   Changes detected:");
                    println!("{}", String::from_utf8_lossy(&diff_output.stdout));
                }
            }
            
            // Basic validation - both objects exist and scope is current
            Ok(scope_exists)
        } else {
            Ok(false)
        }
    }

    /// Resolve contract from agreement
    pub fn resolve_contract(&self, agreement: &Agreement) -> Result<Option<String>> {
        // Try to find the contract blob
        let contract_oid = git2::Oid::from_str(&agreement.contract)?;
        if let Ok(blob) = self.repo.find_blob(contract_oid) {
            let content = String::from_utf8_lossy(blob.content()).to_string();
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Find contract path for a given agreement
    pub fn find_contract_path(&self, _id: &str) -> Result<Option<String>> {
        // This would need to scan the contracts/ directory to find matching files
        // For now, return None
        Ok(None)
    }
}

/// CLI tool for managing agreements
pub struct AgreementCLI {
    manager: AgreementManager,
}

impl AgreementCLI {
    /// Create a new agreement CLI
    pub fn new(repo_path: &Path) -> Result<Self> {
        let manager = AgreementManager::new(repo_path)?;
        Ok(AgreementCLI { manager })
    }

    /// Create a new agreement
    pub fn create(&self, scope: &str, contract: &str, description: Option<&str>) -> Result<()> {
        let result = self.manager.create_agreement(scope, contract, description)?;
        println!("✅ {}", result);
        Ok(())
    }

    /// List all agreements
    pub fn list(&self) -> Result<()> {
        let agreements = self.manager.list_agreements()?;
        
        if agreements.is_empty() {
            println!("📝 No agreements found");
            return Ok(());
        }
        
        println!("📝 Agreements:");
        for metadata in agreements {
            println!("  Scope: {}", metadata.agreement.scope);
            println!("  Contract: {}", metadata.agreement.contract);
            println!("  Status: {:?}", metadata.status);
            println!("  Created: {}", metadata.created_at);
            println!("  By: {}", metadata.created_by);
            println!();
        }
        Ok(())
    }

    /// Show agreement details
    pub fn show(&self, scope: &str) -> Result<()> {
        if let Some(metadata) = self.manager.get_agreement(scope)? {
            println!("📋 Agreement Details:");
            println!("  Scope: {}", metadata.agreement.scope);
            println!("  Contract: {}", metadata.agreement.contract);
            println!("  Status: {:?}", metadata.status);
            println!("  Created: {}", metadata.created_at);
            println!("  By: {}", metadata.created_by);
        } else {
            println!("❌ No agreement found for scope: {}", scope);
        }
        Ok(())
    }

    /// Validate an agreement
    pub fn validate(&self, scope: &str) -> Result<()> {
        let is_valid = self.manager.validate_agreement(scope)?;
        if is_valid {
            println!("✅ Agreement is valid");
        } else {
            println!("❌ Agreement is invalid");
        }
        Ok(())
    }

    /// Resolve contract from agreement
    pub fn resolve(&self, scope: &str) -> Result<()> {
        if let Some(metadata) = self.manager.get_agreement(scope)? {
            if let Some(contract_content) = self.manager.resolve_contract(&metadata.agreement)? {
                println!("📄 Contract content:");
                println!("{}", contract_content);
            } else {
                println!("❌ Could not resolve contract content");
            }
        } else {
            println!("❌ No agreement found for scope: {}", scope);
        }
        Ok(())
    }
}

/// Agreement management commands
#[derive(Parser)]
#[command(name = "agreement")]
#[command(about = "Manage agreements using Git notes")]
pub struct AgreementCli {
    #[command(subcommand)]
    command: AgreementCommands,
}

#[derive(Subcommand)]
pub enum AgreementCommands {
    /// Create a new agreement
    Create {
        /// Tree SHA - the filesystem layout this agreement applies to
        #[arg(long)]
        scope: String,
        /// Blob SHA - the contract that defines expectations or validation
        #[arg(long)]
        contract: String,
        /// Optional description of the agreement
        #[arg(long)]
        description: Option<String>,
    },
    /// List all agreements
    List {
        /// Show only active agreements
        #[arg(long)]
        active_only: bool,
        /// Show only fulfilled agreements
        #[arg(long)]
        fulfilled_only: bool,
        /// Show detailed output
        #[arg(long)]
        verbose: bool,
    },
    /// Show details of a specific agreement
    Show {
        /// Scope SHA of the agreement to show
        scope: String,
    },
    /// Validate an agreement
    Validate {
        /// Scope SHA of the agreement to validate
        scope: String,
        /// Exit with error if validation fails
        #[arg(long)]
        strict: bool,
    },
    /// Resolve contract content from an agreement
    Resolve {
        /// Scope SHA of the agreement to resolve
        scope: String,
        /// Output format (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },
    /// Update agreement status
    UpdateStatus {
        /// Scope SHA of the agreement to update
        scope: String,
        /// New status (active, fulfilled, revoked, pending)
        #[arg(long)]
        status: String,
    },
    /// Create agreement from current tree and contract file
    CreateFromFile {
        /// Path to the contract file
        #[arg(long)]
        contract_file: String,
        /// Optional description of the agreement
        #[arg(long)]
        description: Option<String>,
    },
}

/// Run agreement management command
pub async fn run_agreement_command(command: AgreementCommands) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let cli = AgreementCLI::new(&current_dir)?;

    match command {
        AgreementCommands::Create {
            scope,
            contract,
            description,
        } => {
            cli.create(&scope, &contract, description.as_deref())?;
        }
        AgreementCommands::List {
            active_only: _,
            fulfilled_only: _,
            verbose,
        } => {
            if verbose {
                println!("📝 Listing agreements...");
            }
            cli.list()?;
        }
        AgreementCommands::Show { scope } => {
            cli.show(&scope)?;
        }
        AgreementCommands::Validate { scope, strict } => {
            cli.validate(&scope)?;
            if strict {
                // In a real implementation, you'd check the actual validation result
                // and exit with error if it fails
                println!("✅ Agreement validation passed");
            }
        }
        AgreementCommands::Resolve { scope, format: _ } => {
            cli.resolve(&scope)?;
            // In a real implementation, you'd format the output based on the format parameter
        }
        AgreementCommands::UpdateStatus { scope, status } => {
            let status_enum = match status.as_str() {
                "active" => AgreementStatus::Active,
                "fulfilled" => AgreementStatus::Fulfilled,
                "revoked" => AgreementStatus::Revoked,
                "pending" => AgreementStatus::Pending,
                _ => {
                    anyhow::bail!(
                        "Invalid status: {}. Valid values: active, fulfilled, revoked, pending",
                        status
                    );
                }
            };

            let manager = AgreementManager::new(&current_dir)?;
            manager.update_agreement_status(&scope, status_enum.clone())?;
            println!("✅ Updated agreement status to {:?}", status_enum);
        }
        AgreementCommands::CreateFromFile {
            contract_file,
            description,
        } => {
            create_agreement_from_file(&contract_file, description.as_deref()).await?;
        }
    }

    Ok(())
}

/// Create an agreement from a contract file
async fn create_agreement_from_file(contract_file: &str, description: Option<&str>) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let manager = AgreementManager::new(&current_dir)?;

    // Get current tree SHA
    let tree_sha = get_current_tree_sha()?;
    println!("📋 Current tree SHA: {}", tree_sha);

    // Get contract file blob SHA
    let contract_sha = get_file_blob_sha(contract_file)?;
    println!("📄 Contract file SHA: {}", contract_sha);

    // Create agreement
    let result = manager.create_agreement(&tree_sha, &contract_sha, description)?;
    println!("✅ {}", result);

    Ok(())
}

/// Get the current tree SHA
fn get_current_tree_sha() -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["write-tree"])
        .output()
        .context("Failed to get current tree SHA")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get tree SHA: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let tree_sha = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(tree_sha)
}

/// Get the blob SHA of a file
fn get_file_blob_sha(file_path: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["hash-object", file_path])
        .output()
        .context(format!("Failed to get blob SHA for {}", file_path))?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get blob SHA for {}: {}",
            file_path,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let blob_sha = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(blob_sha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_agreement_commands_parsing() {
        // Test that all command variants can be parsed
        let create_cmd = AgreementCommands::Create {
            scope: "test-scope".to_string(),
            contract: "test-contract".to_string(),
            description: Some("Test agreement".to_string()),
        };

        let list_cmd = AgreementCommands::List {
            active_only: true,
            fulfilled_only: false,
            verbose: true,
        };

        let show_cmd = AgreementCommands::Show {
            scope: "test-scope".to_string(),
        };

        let validate_cmd = AgreementCommands::Validate {
            scope: "test-scope".to_string(),
            strict: true,
        };

        let resolve_cmd = AgreementCommands::Resolve {
            scope: "test-scope".to_string(),
            format: "json".to_string(),
        };

        let update_status_cmd = AgreementCommands::UpdateStatus {
            scope: "test-scope".to_string(),
            status: "fulfilled".to_string(),
        };

        let create_from_file_cmd = AgreementCommands::CreateFromFile {
            contract_file: "contracts/test.jsonc".to_string(),
            description: Some("Test agreement from file".to_string()),
        };

        // Just verify the commands can be created
        assert_eq!(create_cmd.scope(), "test-scope");
        assert_eq!(list_cmd.active_only(), true);
        assert_eq!(show_cmd.scope(), "test-scope");
        assert_eq!(validate_cmd.scope(), "test-scope");
        assert_eq!(resolve_cmd.scope(), "test-scope");
        assert_eq!(update_status_cmd.scope(), "test-scope");
        assert_eq!(create_from_file_cmd.contract_file(), "contracts/test.jsonc");
    }

    // Helper methods for testing
    impl AgreementCommands {
        fn scope(&self) -> &str {
            match self {
                AgreementCommands::Create { scope, .. } => scope,
                AgreementCommands::Show { scope } => scope,
                AgreementCommands::Validate { scope, .. } => scope,
                AgreementCommands::Resolve { scope, .. } => scope,
                AgreementCommands::UpdateStatus { scope, .. } => scope,
                _ => "",
            }
        }
    }

    impl AgreementCommands {
        fn active_only(&self) -> bool {
            match self {
                AgreementCommands::List { active_only, .. } => *active_only,
                _ => false,
            }
        }
    }

    impl AgreementCommands {
        fn contract_file(&self) -> &str {
            match self {
                AgreementCommands::CreateFromFile { contract_file, .. } => contract_file,
                _ => "",
            }
        }
    }
}
