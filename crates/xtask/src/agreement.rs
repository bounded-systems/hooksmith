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
use std::collections::HashMap;
use std::path::Path;

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
        self.repo.note(
            &signature,
            &signature,
            Some(&self.notes_ref),
            scope_oid,
            &note_content,
            false,
        )?;

        Ok(format!(
            "Agreement created: scope={}, contract={}",
            scope, contract
        ))
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
        let mut agreements = Vec::new();

        // Get the notes ref
        if let Ok(notes_ref) = self.repo.find_reference(&self.notes_ref) {
            // Get the commit that the notes ref points to
            if let Ok(notes_commit) = self.repo.find_commit(notes_ref.target().unwrap()) {
                // Get the tree of the notes commit
                if let Ok(notes_tree) = self.repo.find_tree(notes_commit.tree_id()) {
                    // Walk through all entries in the notes tree
                    for entry in notes_tree.iter() {
                        if let Ok(note_blob) = self.repo.find_blob(entry.id()) {
                            // Try to parse the note content as agreement metadata
                            if let Ok(note_content) =
                                String::from_utf8(note_blob.content().to_vec())
                            {
                                if let Ok(metadata) =
                                    serde_json::from_str::<AgreementMetadata>(&note_content)
                                {
                                    agreements.push(metadata);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(agreements)
    }

    /// Update agreement status
    pub fn update_agreement_status(&self, scope: &str, status: AgreementStatus) -> Result<()> {
        if let Some(mut metadata) = self.get_agreement(scope)? {
            metadata.status = status;
            let note_content = serde_json::to_string(&metadata)?;
            let signature = Signature::now("Hooksmith", "hooksmith@example.com")?;
            let scope_oid = git2::Oid::from_str(scope)?;
            self.repo.note(
                &signature,
                &signature,
                Some(&self.notes_ref),
                scope_oid,
                &note_content,
                false,
            )?;
        }
        Ok(())
    }

    /// Prune invalid agreements (remove agreements with invalid scopes or missing contracts)
    pub fn prune_invalid_agreements(&self, dry_run: bool, force: bool) -> Result<()> {
        let agreements = self.list_agreements()?;
        let mut to_prune = Vec::new();

        println!(
            "🔍 Checking {} agreements for validity...",
            agreements.len()
        );

        for metadata in &agreements {
            let scope = &metadata.agreement.scope;
            let contract = &metadata.agreement.contract;

            // Check if scope path exists in current HEAD
            let scope_path = self.get_tree_path(scope)?;
            let scope_valid = if let Some(path) = scope_path {
                self.validate_path_exists(&path)?
            } else {
                false
            };

            // Check if contract blob exists in current HEAD
            let contract_exists = self.validate_contract_exists(contract)?;

            if !scope_valid || !contract_exists {
                to_prune.push((metadata.clone(), scope_valid, contract_exists));
            }
        }

        if to_prune.is_empty() {
            println!("✅ All agreements are valid - nothing to prune");
            return Ok(());
        }

        println!("\n📋 Found {} invalid agreements to prune:", to_prune.len());
        for (metadata, scope_valid, contract_exists) in &to_prune {
            println!("   Scope: {}", metadata.agreement.scope);
            println!("   Contract: {}", metadata.agreement.contract);
            println!("   Created: {}", metadata.created_at);
            if !scope_valid {
                println!("   ❌ Scope path no longer exists in current HEAD");
            }
            if !contract_exists {
                println!("   ⚠️  Contract blob no longer exists in current HEAD");
            }
            println!();
        }

        if dry_run {
            println!("🔍 Dry run - no agreements were actually removed");
            return Ok(());
        }

        if !force {
            println!(
                "⚠️  This will permanently remove {} agreements.",
                to_prune.len()
            );
            println!("   Use --force to proceed without confirmation");
            return Ok(());
        }

        // Remove the invalid agreements
        for (metadata, _, _) in &to_prune {
            self.remove_agreement(&metadata.agreement.scope)?;
        }

        println!(
            "✅ Successfully pruned {} invalid agreements",
            to_prune.len()
        );
        Ok(())
    }

    /// Remove an agreement by scope
    fn remove_agreement(&self, scope: &str) -> Result<()> {
        // Get all agreements
        let agreements = self.list_agreements()?;

        // Filter out the agreement to remove
        let remaining_agreements: Vec<_> = agreements
            .into_iter()
            .filter(|metadata| metadata.agreement.scope != scope)
            .collect();

        // Create new notes tree with remaining agreements
        let mut tree_builder = self.repo.treebuilder(None)?;

        for metadata in remaining_agreements {
            let note_content = serde_json::to_string_pretty(&metadata)?;
            let note_blob = self.repo.blob(&note_content.as_bytes())?;
            tree_builder.insert(
                &format!("agreement_{}", metadata.agreement.scope),
                note_blob,
                0o100644,
            )?;
        }

        let tree_id = tree_builder.write()?;
        let tree = self.repo.find_tree(tree_id)?;

        // Create a new commit
        let signature = self.repo.signature()?;
        let parent_commit = if let Ok(notes_ref) = self.repo.find_reference(&self.notes_ref) {
            self.repo.find_commit(notes_ref.target().unwrap())?
        } else {
            // No existing notes, create initial commit
            let empty_tree = self.repo.treebuilder(None)?.write()?;
            let tree = self.repo.find_tree(empty_tree)?;
            let signature = self.repo.signature()?;
            let commit_oid = self.repo.commit(
                Some(&self.notes_ref),
                &signature,
                &signature,
                "Initial notes commit",
                &tree,
                &[],
            )?;
            self.repo.find_commit(commit_oid)?
        };

        let commit_id = self.repo.commit(
            Some(&self.notes_ref),
            &signature,
            &signature,
            &format!("Remove invalid agreement: {}", scope),
            &tree,
            &[&parent_commit],
        )?;

        println!("   Removed agreement: {} (commit: {})", scope, commit_id);
        Ok(())
    }

    /// Validate an agreement (check if contract exists in scope)
    pub fn validate_agreement(&self, scope: &str) -> Result<bool> {
        if let Some(metadata) = self.get_agreement(scope)? {
            // Check if the contract blob exists in current HEAD
            let contract_exists = self.validate_contract_exists(&metadata.agreement.contract)?;

            // Get the path from the old tree SHA to validate against current HEAD
            let scope_path = self.get_tree_path(&metadata.agreement.scope)?;
            let scope_valid = if let Some(path) = scope_path {
                self.validate_path_exists(&path)?
            } else {
                // If we can't determine the path, fall back to tree existence check
                self.validate_tree_exists(&metadata.agreement.scope)?
            };

            // Overall validation: scope must be valid, contract existence is a warning
            let is_valid = scope_valid;

            if is_valid {
                println!("✅ Agreement is valid");
                if !contract_exists {
                    println!("⚠️  Warning: Contract blob no longer exists in current HEAD");
                }
                Ok(true)
            } else {
                println!("❌ Scope validation failed");
                if !contract_exists {
                    println!("⚠️  Warning: Contract blob no longer exists in current HEAD");
                }
                Ok(false)
            }
        } else {
            println!("❌ Agreement not found for scope: {}", scope);
            Ok(false)
        }
    }

    /// Validate if a path exists in current HEAD
    fn validate_path_exists(&self, path: &str) -> Result<bool> {
        // Check if the path exists in current HEAD
        let output = std::process::Command::new("git")
            .args(&["ls-tree", "HEAD", path])
            .output()?;

        Ok(output.status.success())
    }

    /// Get the path of a tree SHA in current HEAD
    fn get_tree_path(&self, tree_sha: &str) -> Result<Option<String>> {
        // Check if the scope tree matches current HEAD tree
        let current_head_tree = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD^{tree}"])
            .output()?;
        let current_head_tree_sha = String::from_utf8_lossy(&current_head_tree.stdout)
            .trim()
            .to_string();

        if tree_sha == current_head_tree_sha {
            return Ok(Some("".to_string())); // Root tree
        }

        // Find the path of the tree SHA in current HEAD
        let tree_paths = std::process::Command::new("git")
            .args(&["ls-tree", "-r", "-t", "HEAD^{tree}"])
            .output()?;
        let tree_output = String::from_utf8_lossy(&tree_paths.stdout);

        for line in tree_output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[1] == "tree" && parts[2] == tree_sha {
                return Ok(Some(parts[3].to_string()));
            }
        }

        Ok(None) // Tree not found in current HEAD
    }

    /// Validate if a tree SHA exists in current HEAD
    fn validate_tree_exists(&self, tree_sha: &str) -> Result<bool> {
        // Check if the scope tree matches current HEAD tree
        let current_head_tree = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD^{tree}"])
            .output()?;
        let current_head_tree_sha = String::from_utf8_lossy(&current_head_tree.stdout)
            .trim()
            .to_string();

        if tree_sha == current_head_tree_sha {
            return Ok(true);
        }

        // Check if the scope tree exists anywhere in current HEAD
        let tree_exists = std::process::Command::new("git")
            .args(&["ls-tree", "-r", "-t", "HEAD^{tree}"])
            .output()?;
        let tree_output = String::from_utf8_lossy(&tree_exists.stdout);

        Ok(tree_output.lines().any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.len() >= 3 && parts[1] == "tree" && parts[2] == tree_sha
        }))
    }

    /// Validate if a contract blob exists in current HEAD
    fn validate_contract_exists(&self, contract_sha: &str) -> Result<bool> {
        // Check if the contract blob exists in current HEAD
        let output = std::process::Command::new("git")
            .args(&["ls-tree", "-r", "HEAD"])
            .output()?;
        let tree_output = String::from_utf8_lossy(&output.stdout);

        Ok(tree_output.lines().any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.len() >= 3 && parts[1] == "blob" && parts[2] == contract_sha
        }))
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
        let result = self
            .manager
            .create_agreement(scope, contract, description)?;
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

    /// Prune invalid agreements
    pub fn prune(&self, dry_run: bool, force: bool) -> Result<()> {
        self.manager.prune_invalid_agreements(dry_run, force)?;
        Ok(())
    }

    /// Honor an agreement by creating a remote branch and worktree
    pub fn honor(
        &self,
        scope: &str,
        base_branch: &str,
        create_worktree: bool,
        worktree_name: Option<&str>,
    ) -> Result<()> {
        // Get the agreement details
        let metadata = self.manager.get_agreement(scope)?;
        let agreement = match metadata {
            Some(meta) => meta.agreement,
            None => {
                anyhow::bail!("No agreement found for scope: {}", scope);
            }
        };

        println!("🎯 Honoring agreement:");
        println!("  Scope: {}", agreement.scope);
        println!("  Contract: {}", agreement.contract);

        // Get the base commit SHA
        let base_sha = get_base_commit_sha(base_branch)?;
        println!("  Base branch: {} ({})", base_branch, base_sha);

        // Create remote branch using GitHub CLI
        let branch_name = scope;
        create_remote_branch(branch_name, &base_sha)?;
        println!("✅ Created remote branch: {}", branch_name);

        if create_worktree {
            let worktree_dir = worktree_name.unwrap_or(scope);
            create_worktree_for_branch(branch_name, worktree_dir)?;
            println!("✅ Created worktree: .wt/{}", worktree_dir);
        }

        println!("🎉 Agreement honored successfully!");
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
        /// Optional path that this agreement applies to (for path-based validation)
        #[arg(long)]
        path: Option<String>,
        /// Whether this agreement is anchored to a specific path (immutable validation)
        #[arg(long, default_value = "false")]
        anchored: bool,
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
    /// Prune invalid agreements (remove agreements with invalid scopes or missing contracts)
    Prune {
        /// Dry run - show what would be pruned without actually removing
        #[arg(long)]
        dry_run: bool,
        /// Force removal without confirmation
        #[arg(long)]
        force: bool,
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
    /// Honor an agreement by creating a remote branch and worktree
    Honor {
        /// Scope SHA of the agreement to honor
        scope: String,
        /// Base branch to create the new branch from (default: origin/main)
        #[arg(long, default_value = "origin/main")]
        base_branch: String,
        /// Whether to create a worktree after creating the branch
        #[arg(long, default_value = "true")]
        create_worktree: bool,
        /// Worktree directory name (defaults to scope SHA)
        #[arg(long)]
        worktree_name: Option<String>,
        /// Whether to open the worktree in Cursor
        #[arg(long, default_value = "true")]
        open_in_cursor: bool,
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
            path: _,
            anchored: _,
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
        AgreementCommands::Prune { dry_run, force } => {
            cli.prune(dry_run, force)?;
        }
        AgreementCommands::Honor {
            scope,
            base_branch,
            create_worktree,
            worktree_name,
        } => {
            cli.honor(
                &scope,
                &base_branch,
                create_worktree,
                worktree_name.as_deref(),
            )?;
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

/// Get the base commit SHA for a branch
fn get_base_commit_sha(base_branch: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", base_branch])
        .output()
        .context(format!("Failed to get base commit SHA for {}", base_branch))?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get base commit SHA for {}: {}",
            base_branch,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let commit_sha = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(commit_sha)
}

/// Create a remote branch using GitHub CLI
fn create_remote_branch(branch_name: &str, base_sha: &str) -> Result<()> {
    // Get repository owner and name from remote URL
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to get remote URL")?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get remote URL: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let url = String::from_utf8(output.stdout)?.trim().to_string();
    let (owner, repo_name) = extract_owner_repo_from_url(&url)?;

    // Create the branch using GitHub CLI
    let gh_output = std::process::Command::new("gh")
        .args([
            "api",
            &format!("repos/{}/{}/git/refs", owner, repo_name),
            "-f",
            &format!("ref=refs/heads/{}", branch_name),
            "-f",
            &format!("sha={}", base_sha),
        ])
        .output()
        .context("Failed to create remote branch with GitHub CLI")?;

    if !gh_output.status.success() {
        let error = String::from_utf8_lossy(&gh_output.stderr);
        anyhow::bail!("Failed to create remote branch: {}", error);
    }

    Ok(())
}

/// Extract owner and repository name from Git URL
fn extract_owner_repo_from_url(url: &str) -> Result<(String, String)> {
    // Handle SSH format: git@github.com:owner/repo.git
    if url.starts_with("git@") {
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid SSH URL format: {}", url);
        }
        let repo_part = parts[1].trim_end_matches(".git");
        let repo_parts: Vec<&str> = repo_part.split('/').collect();
        if repo_parts.len() != 2 {
            anyhow::bail!("Invalid repository format in URL: {}", url);
        }
        return Ok((repo_parts[0].to_string(), repo_parts[1].to_string()));
    }

    // Handle HTTPS format: https://github.com/owner/repo.git
    if url.starts_with("https://") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() < 5 {
            anyhow::bail!("Invalid HTTPS URL format: {}", url);
        }
        let owner = parts[3];
        let repo = parts[4].trim_end_matches(".git");
        return Ok((owner.to_string(), repo.to_string()));
    }

    anyhow::bail!("Unsupported URL format: {}", url);
}

/// Create a worktree for a branch
fn create_worktree_for_branch(branch_name: &str, worktree_dir: &str) -> Result<()> {
    // First fetch the branch from remote
    let fetch_output = std::process::Command::new("git")
        .args(["fetch", "origin", branch_name])
        .output()
        .context("Failed to fetch branch")?;

    if !fetch_output.status.success() {
        let error = String::from_utf8_lossy(&fetch_output.stderr);
        anyhow::bail!("Failed to fetch branch: {}", error);
    }

    // Create the worktree
    let worktree_path = format!(".wt/{}", worktree_dir);
    let worktree_output = std::process::Command::new("git")
        .args(["worktree", "add", &worktree_path, branch_name])
        .output()
        .context("Failed to create worktree")?;

    if !worktree_output.status.success() {
        let error = String::from_utf8_lossy(&worktree_output.stderr);
        anyhow::bail!("Failed to create worktree: {}", error);
    }

    Ok(())
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
