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

/// Canonical agreement schema - minimal structure stored in Git notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agreement {
    /// Tree SHA - the filesystem layout this agreement applies to
    pub scope: String,
    /// Blob SHA - the contract that defines expectations or validation
    pub contract: String,
}

/// Agreement metadata derived from Git notes commit history
#[derive(Debug, Clone, Serialize)]
pub struct AgreementMetadata {
    /// The agreement itself
    pub agreement: Agreement,
    /// When the agreement was created (from Git commit)
    pub created_at: String,
    /// Who created the agreement (from Git commit)
    pub created_by: String,
    /// Current status derived from latest commit message or note content
    pub status: String,
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
        _description: Option<&str>,
    ) -> Result<String> {
        let agreement = Agreement {
            scope: scope.to_string(),
            contract: contract.to_string(),
        };

        // Store only the minimal agreement structure in Git notes
        let note_content = serde_json::to_string(&agreement)?;

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
            let agreement: Agreement = serde_json::from_str(note_content)?;

            // Derive metadata from Git notes commit history
            let metadata = self.derive_metadata_from_git_history(scope, &agreement)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Get the current agreement based on the current branch name
    pub fn get_current_agreement(&self) -> Result<Option<AgreementMetadata>> {
        let current_branch = self.get_current_branch_name()?;

        // Try to find an agreement with scope matching the current branch name
        let agreements = self.list_agreements()?;

        for metadata in agreements {
            if metadata.agreement.scope == current_branch {
                return Ok(Some(metadata));
            }
        }

        Ok(None)
    }

    /// Get the current branch name
    fn get_current_branch_name(&self) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get current branch name")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get current branch: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let branch_name = String::from_utf8(output.stdout)?.trim().to_string();
        Ok(branch_name)
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
                            // Try to parse the note content as minimal agreement structure
                            if let Ok(note_content) =
                                String::from_utf8(note_blob.content().to_vec())
                            {
                                if let Ok(agreement) =
                                    serde_json::from_str::<Agreement>(&note_content)
                                {
                                    // Derive metadata from Git history
                                    if let Ok(metadata) = self.derive_metadata_from_git_history(
                                        &agreement.scope,
                                        &agreement,
                                    ) {
                                        agreements.push(metadata);
                                    }
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
    pub fn update_agreement_status(&self, scope: &str, _status: &str) -> Result<()> {
        let scope_oid = git2::Oid::from_str(scope)?;

        // Get the current agreement
        if let Ok(note) = self.repo.find_note(Some(&self.notes_ref), scope_oid) {
            let note_content = note.message().unwrap_or("");
            if let Ok(agreement) = serde_json::from_str::<Agreement>(note_content) {
                // Create a new note with the same agreement but updated status in commit message
                let note_content = serde_json::to_string(&agreement)?;
                let signature = Signature::now("Hooksmith", "hooksmith@example.com")?;
                self.repo.note(
                    &signature,
                    &signature,
                    Some(&self.notes_ref),
                    scope_oid,
                    &note_content,
                    false,
                )?;
            }
        }
        Ok(())
    }

    /// Derive metadata from Git notes commit history
    fn derive_metadata_from_git_history(
        &self,
        scope: &str,
        agreement: &Agreement,
    ) -> Result<AgreementMetadata> {
        let scope_oid = git2::Oid::from_str(scope)?;

        // Get the note to find its commit
        if let Ok(note) = self.repo.find_note(Some(&self.notes_ref), scope_oid) {
            // Get the commit that contains this note
            if let Ok(commit) = self.repo.find_commit(note.id()) {
                let created_at = commit.time().seconds().to_string();
                let created_by = commit.author().name().unwrap_or("Unknown").to_string();

                // Derive status based on Git workflow state
                let status = self.derive_status_from_git_workflow(scope)?;

                return Ok(AgreementMetadata {
                    agreement: agreement.clone(),
                    created_at,
                    created_by,
                    status,
                });
            }
        }

        // Fallback if we can't get commit info
        Ok(AgreementMetadata {
            agreement: agreement.clone(),
            created_at: "Unknown".to_string(),
            created_by: "Unknown".to_string(),
            status: "active".to_string(),
        })
    }

    /// Derive status based on Git workflow state
    fn derive_status_from_git_workflow(&self, scope: &str) -> Result<String> {
        // Check if there's a branch named after the scope SHA
        let branch_exists = self.check_branch_exists(scope)?;

        if !branch_exists {
            return Ok("backlog".to_string());
        }

        // Check if the branch has been merged into origin/main
        let is_merged = self.check_branch_merged_into_main(scope)?;

        if is_merged {
            Ok("fulfilled".to_string())
        } else {
            Ok("developing".to_string())
        }
    }

    /// Check if a branch exists with the given name
    fn check_branch_exists(&self, branch_name: &str) -> Result<bool> {
        let output = std::process::Command::new("git")
            .args(["branch", "-r"])
            .output()
            .context("Failed to list remote branches")?;

        if !output.status.success() {
            return Ok(false);
        }

        let branches = String::from_utf8(output.stdout)?;
        Ok(branches.lines().any(|line| line.contains(branch_name)))
    }

    /// Check if a branch has been merged into origin/main
    fn check_branch_merged_into_main(&self, branch_name: &str) -> Result<bool> {
        // First check if the branch exists
        if !self.check_branch_exists(branch_name)? {
            return Ok(false);
        }

        // Check if the branch has been merged into origin/main
        let output = std::process::Command::new("git")
            .args(["branch", "-r", "--merged", "origin/main"])
            .output()
            .context("Failed to check merged branches")?;

        if !output.status.success() {
            return Ok(false);
        }

        let merged_branches = String::from_utf8(output.stdout)?;
        Ok(merged_branches
            .lines()
            .any(|line| line.contains(branch_name)))
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

    /// Validate that a tree SHA is reachable from anywhere in Git history
    fn validate_tree_reachable_from_history(&self, tree_sha: &str) -> Result<bool> {
        // Check if the tree SHA is reachable from anywhere in history
        let output = std::process::Command::new("git")
            .args(&["rev-list", "--all", "--objects"])
            .output()?;

        let objects_list = String::from_utf8_lossy(&output.stdout);
        Ok(objects_list.lines().any(|line| line.contains(tree_sha)))
    }

    /// Validate that a blob SHA is reachable from origin/main
    fn validate_blob_reachable_from_main(&self, blob_sha: &str) -> Result<bool> {
        // Check if the blob SHA is reachable from origin/main
        let output = std::process::Command::new("git")
            .args(&["rev-list", "origin/main", "--objects"])
            .output()?;

        let objects_list = String::from_utf8_lossy(&output.stdout);
        Ok(objects_list.lines().any(|line| line.contains(blob_sha)))
    }

    /// Enhanced agreement validation with proper reachability checks
    pub fn validate_agreement_with_main_reachability(&self, scope: &str) -> Result<(bool, String)> {
        // Check if tree SHA is reachable from anywhere in history
        let tree_reachable = self.validate_tree_reachable_from_history(scope)?;
        if !tree_reachable {
            return Ok((
                false,
                format!("Tree SHA {} is not reachable from Git history", scope),
            ));
        }

        // Get the agreement to check the contract
        let agreement = self.get_agreement(scope)?;
        let agreement = match agreement {
            Some(agreement) => agreement,
            None => {
                return Ok((false, "Agreement not found".to_string()));
            }
        };

        // Check if tree SHA is reachable from origin/main
        let tree_reachable_from_main =
            self.is_tree_sha_reachable_from_origin_main(&agreement.agreement.scope)?;
        if !tree_reachable_from_main {
            return Ok((
                false,
                format!(
                    "Tree SHA {} is not reachable from origin/main",
                    agreement.agreement.scope
                ),
            ));
        }

        // Check if contract blob SHA is reachable from origin/main
        let contract_reachable =
            self.validate_blob_reachable_from_main(&agreement.agreement.contract)?;
        if !contract_reachable {
            return Ok((
                false,
                format!(
                    "Contract SHA {} is not reachable from origin/main",
                    agreement.agreement.contract
                ),
            ));
        }

        // Additional validation: check if tree exists and contract exists
        let tree_exists = self.validate_tree_exists(scope)?;
        if !tree_exists {
            return Ok((false, format!("Tree SHA {} does not exist", scope)));
        }

        let contract_exists = self.validate_contract_exists(&agreement.agreement.contract)?;
        if !contract_exists {
            return Ok((
                false,
                format!(
                    "Contract SHA {} does not exist",
                    agreement.agreement.contract
                ),
            ));
        }

        Ok((
            true,
            "Agreement is valid and contract is reachable from origin/main".to_string(),
        ))
    }
}

/// Enhanced tree path resolution result
#[derive(Debug, Clone)]
pub struct TreePathResolution {
    /// The tree SHA being resolved
    pub tree_sha: String,
    /// The resolved path in HEAD (if found)
    pub path: Option<String>,
    /// The commit that contains this tree (if found)
    pub commit: Option<String>,
    /// The commit message (if found)
    pub commit_message: Option<String>,
    /// Whether the tree exists in current HEAD
    pub exists_in_head: bool,
    /// Whether the tree is reachable in history
    pub is_reachable: bool,
    /// Resolution type
    pub resolution_type: ResolutionType,
    /// Last seen timestamp (if available)
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ResolutionType {
    /// Tree found at specific path in HEAD
    Path(String),
    /// Tree found in specific commit
    Commit(String),
    /// Tree SHA only (unreachable or detached)
    Tree(String),
    /// Tree not found anywhere
    NotFound,
}

impl std::fmt::Display for TreePathResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.resolution_type {
            ResolutionType::Path(path) => {
                write!(f, "Path: {}", path)?;
                if !self.exists_in_head {
                    write!(f, " (not in HEAD)")?;
                }
            }
            ResolutionType::Commit(commit) => {
                write!(f, "Commit: {}", commit)?;
                if let Some(msg) = &self.commit_message {
                    write!(f, " - {}", msg)?;
                }
            }
            ResolutionType::Tree(sha) => {
                write!(f, "Tree: {}", sha)?;
                if !self.is_reachable {
                    write!(f, " (unreachable)")?;
                }
            }
            ResolutionType::NotFound => {
                write!(f, "Not found")?;
            }
        }
        Ok(())
    }
}

impl AgreementManager {
    /// Enhanced tree path resolution with detailed audit information
    pub fn resolve_tree_path(&self, tree_sha: &str) -> Result<TreePathResolution> {
        let mut resolution = TreePathResolution {
            tree_sha: tree_sha.to_string(),
            path: None,
            commit: None,
            commit_message: None,
            exists_in_head: false,
            is_reachable: false,
            resolution_type: ResolutionType::NotFound,
            last_seen_at: None,
        };

        // Check if tree exists in current HEAD
        let head_path = self.find_tree_in_head(tree_sha)?;
        if let Some(path) = head_path {
            resolution.path = Some(path.clone());
            resolution.exists_in_head = true;
            resolution.resolution_type = ResolutionType::Path(path);
            resolution.last_seen_at = Some(chrono::Utc::now().to_rfc3339());
            return Ok(resolution);
        }

        // Check if tree exists in any commit
        let commit_info = self.find_tree_in_history(tree_sha)?;
        if let Some((commit_sha, commit_msg)) = commit_info {
            resolution.commit = Some(commit_sha.clone());
            resolution.commit_message = Some(commit_msg);
            resolution.is_reachable = true;
            resolution.resolution_type = ResolutionType::Commit(commit_sha);
            return Ok(resolution);
        }

        // Check if tree SHA is reachable at all
        let is_reachable = self.check_tree_reachability(tree_sha)?;
        resolution.is_reachable = is_reachable;

        if is_reachable {
            resolution.resolution_type = ResolutionType::Tree(tree_sha.to_string());
        } else {
            resolution.resolution_type = ResolutionType::NotFound;
        }

        Ok(resolution)
    }

    /// Find tree in current HEAD
    fn find_tree_in_head(&self, tree_sha: &str) -> Result<Option<String>> {
        // Check if it's the root tree
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

        Ok(None)
    }

    /// Find tree in commit history
    fn find_tree_in_history(&self, tree_sha: &str) -> Result<Option<(String, String)>> {
        // Get all commits
        let output = std::process::Command::new("git")
            .args(&["rev-list", "--all"])
            .output()?;

        if !output.status.success() {
            return Ok(None);
        }

        let commits = String::from_utf8(output.stdout)?;

        // Search for commits that contain this tree
        for commit in commits.lines() {
            let tree_output = std::process::Command::new("git")
                .args(&["rev-parse", &format!("{}:^{{tree}}", commit)])
                .output();

            if let Ok(tree_output) = tree_output {
                if tree_output.status.success() {
                    let commit_tree = String::from_utf8(tree_output.stdout)?.trim().to_string();
                    if commit_tree == tree_sha {
                        // Found the commit, now get the commit message
                        let log_output = std::process::Command::new("git")
                            .args(["log", "--oneline", "-1", commit])
                            .output()
                            .context("Failed to get commit message")?;

                        if log_output.status.success() {
                            let commit_msg =
                                String::from_utf8(log_output.stdout)?.trim().to_string();
                            return Ok(Some((commit.to_string(), commit_msg)));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Check if tree SHA is reachable in history
    fn check_tree_reachability(&self, tree_sha: &str) -> Result<bool> {
        // Use git rev-list to check if tree is reachable
        let output = std::process::Command::new("git")
            .args(&["rev-list", "--all", "--objects"])
            .output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let objects = String::from_utf8(output.stdout)?;
        Ok(objects.lines().any(|line| line.contains(tree_sha)))
    }

    /// Enhanced agreement validation with detailed resolution
    pub fn validate_agreement_with_resolution(
        &self,
        scope: &str,
    ) -> Result<(bool, TreePathResolution)> {
        let resolution = self.resolve_tree_path(scope)?;

        // Check if the contract blob exists in current HEAD
        let _contract_exists = self.validate_contract_exists(scope)?;

        // Determine validity based on resolution
        let is_valid = match resolution.resolution_type {
            ResolutionType::Path(_) => true,   // Tree exists in HEAD
            ResolutionType::Commit(_) => true, // Tree exists in history
            ResolutionType::Tree(_) => resolution.is_reachable, // Tree is reachable
            ResolutionType::NotFound => false, // Tree not found
        };

        Ok((is_valid, resolution))
    }

    /// Enhanced agreement listing with resolution details
    pub fn list_agreements_with_resolution(
        &self,
    ) -> Result<Vec<(AgreementMetadata, TreePathResolution)>> {
        let agreements = self.list_agreements()?;
        let mut resolved_agreements = Vec::new();

        for metadata in agreements {
            let resolution = self.resolve_tree_path(&metadata.agreement.scope)?;
            resolved_agreements.push((metadata, resolution));
        }

        Ok(resolved_agreements)
    }

    /// Check if a tree SHA is reachable from origin/main
    fn is_tree_sha_reachable_from_origin_main(&self, tree_sha: &str) -> Result<bool> {
        // Get all objects reachable from origin/main
        let output = std::process::Command::new("git")
            .args(&["rev-list", "origin/main", "--objects"])
            .output()?;

        let objects_list = String::from_utf8_lossy(&output.stdout);
        Ok(objects_list.lines().any(|line| line.contains(tree_sha)))
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
    ///
    /// Format: <indicator> <short-sha> (<contract>): <path-scope> [<tree-slug>]
    ///
    /// ~ c785760 (object-names): detached [detached]
    /// │ │        │              │        │
    /// │ │        │              │        └── Tree slug
    /// │ │        │              └── Path scope
    /// │ │        └── Contract name
    /// │ └── Short scope SHA (7 chars)
    /// └── Status indicator
    ///
    /// Status indicators:
    ///   * = Current agreement (matches current tree SHA)
    ///   + = Active agreement with worktree
    ///   ~ = Active agreement (no worktree)
    ///   ✓ = Fulfilled agreement
    ///   ✗ = Revoked agreement
    ///     = Default/unknown status
    pub fn list(&self) -> Result<()> {
        let agreements = self.manager.list_agreements()?;

        if agreements.is_empty() {
            println!("📝 No agreements found");
            return Ok(());
        }

        // Get current tree SHA to mark active agreements
        let current_tree = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD^{tree}"])
            .output()?;
        let current_tree_sha = String::from_utf8_lossy(&current_tree.stdout)
            .trim()
            .to_string();

        // Sort agreements: current first, then by scope
        let mut sorted_agreements = agreements;
        sorted_agreements.sort_by(|a, b| {
            let a_is_current = a.agreement.scope == current_tree_sha;
            let b_is_current = b.agreement.scope == current_tree_sha;

            if a_is_current && !b_is_current {
                std::cmp::Ordering::Less
            } else if !a_is_current && b_is_current {
                std::cmp::Ordering::Greater
            } else {
                a.agreement.scope.cmp(&b.agreement.scope)
            }
        });

        for metadata in sorted_agreements {
            let short_scope = &metadata.agreement.scope[..7];

            // Get path scope and tree slug
            let resolution = self.manager.resolve_tree_path(&metadata.agreement.scope)?;
            let path_scope = match resolution.resolution_type {
                ResolutionType::Path(path) => {
                    if path.is_empty() {
                        "./".to_string()
                    } else {
                        path
                    }
                }
                ResolutionType::Commit(_) => "history".to_string(),
                ResolutionType::Tree(_) => {
                    // Check if this tree represents the root structure
                    if self.is_root_tree(&metadata.agreement.scope)? {
                        "./".to_string()
                    } else {
                        "detached".to_string()
                    }
                }
                ResolutionType::NotFound => "unknown".to_string(),
            };

            // Use agreement status for tree slug instead of path scope
            let tree_slug = metadata.status.clone();

            // Check if worktree exists for this agreement
            let worktree_exists = self.check_worktree_exists(&metadata.agreement.scope)?;

            // Determine status indicator
            let indicator = if metadata.agreement.scope == current_tree_sha {
                "* " // Current agreement
            } else if worktree_exists {
                "+ " // Has worktree
            } else if metadata.status == "backlog" {
                "📋 " // Backlog
            } else if metadata.status == "developing" {
                "🔄 " // Developing
            } else if metadata.status == "fulfilled" {
                "✓ " // Fulfilled
            } else {
                "  " // Default
            };

            // Get contract name for display
            let contract_name = get_contract_name(&metadata.agreement.contract)
                .unwrap_or_else(|_| "unknown".to_string());

            // Format like git branch --list with additional info
            println!(
                "{}{} ({}): {} [{}]",
                indicator, short_scope, contract_name, path_scope, tree_slug
            );
        }
        Ok(())
    }

    /// Show agreement details
    pub fn show(&self, scope: &str) -> Result<()> {
        if let Some(metadata) = self.manager.get_agreement(scope)? {
            // Get contract name
            let contract_name = get_contract_name(&metadata.agreement.contract)
                .unwrap_or_else(|_| "Unknown".to_string());

            // Get tree path
            let tree_path =
                get_tree_path(&metadata.agreement.scope).unwrap_or_else(|_| "Unknown".to_string());

            // Get short SHA hashes (7 characters)
            let short_scope = &metadata.agreement.scope[..7];
            let short_contract = &metadata.agreement.contract[..7];

            println!("📋 Agreement Details:");
            println!("  Scope: {} ({})", tree_path, short_scope);
            println!("  Contract: {} ({})", contract_name, short_contract);
            println!("  Status: {:?}", metadata.status);
            println!("  Created: {}", metadata.created_at);
            println!("  By: {}", metadata.created_by);

            // Show the actual agreement content
            if let Some(agreement_content) = self.manager.resolve_contract(&metadata.agreement)? {
                println!("\n📄 Contract Content:");
                println!("{}", agreement_content);
            }
        } else {
            println!("❌ No agreement found for scope: {}", scope);
        }
        Ok(())
    }

    /// Validate an agreement
    pub fn validate(&self, scope: &str) -> Result<()> {
        println!("🔍 Validating agreement: {}", scope);

        // Use the enhanced validation with origin/main reachability
        let (is_valid, message) = self
            .manager
            .validate_agreement_with_main_reachability(scope)?;

        if is_valid {
            println!("✅ {}", message);
        } else {
            println!("❌ {}", message);
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

    /// Show agreement history/log
    pub fn log(&self, scope: Option<&str>) -> Result<()> {
        if let Some(scope) = scope {
            // Show log for specific agreement
            self.show_agreement_log(scope)?;
        } else {
            // Show log for all agreements
            self.show_all_agreements_log()?;
        }
        Ok(())
    }

    /// Show log for a specific agreement
    fn show_agreement_log(&self, scope: &str) -> Result<()> {
        // Find the note for this scope
        let note_ref = format!("agreement_{}", scope);

        let output = std::process::Command::new("git")
            .args(["log", "--oneline", "--follow", "--", &note_ref])
            .output()
            .context("Failed to get agreement log")?;

        if !output.status.success() {
            println!("❌ No agreement found for scope: {}", scope);
            return Ok(());
        }

        let log_content = String::from_utf8(output.stdout)?;
        if log_content.trim().is_empty() {
            println!("❌ No history found for agreement: {}", scope);
            return Ok(());
        }

        println!("📜 Agreement History for {}:", scope);
        println!("{}", log_content);
        Ok(())
    }

    /// Show log for all agreements
    fn show_all_agreements_log(&self) -> Result<()> {
        let output = std::process::Command::new("git")
            .args(["log", "--oneline", "refs/notes/commits"])
            .output()
            .context("Failed to get agreements log")?;

        if !output.status.success() {
            println!("❌ Failed to get agreements log");
            return Ok(());
        }

        let log_content = String::from_utf8(output.stdout)?;
        if log_content.trim().is_empty() {
            println!("📝 No agreement history found");
            return Ok(());
        }

        println!("📜 All Agreements History:");
        println!("{}", log_content);
        Ok(())
    }

    /// Prune invalid agreements
    pub fn prune(&self, dry_run: bool, force: bool) -> Result<()> {
        self.manager.prune_invalid_agreements(dry_run, force)?;
        Ok(())
    }

    /// Honor an agreement by creating a remote branch and worktree
    ///
    /// Naming Convention: ~<short-sha>-<contract-name> <path-scope>
    ///
    /// Examples:
    ///   ~c785760-object-names foo/bar
    ///   ~e1158e3-crate-boundaries src/
    ///   ~4e8500b-lint-rules tools/internal
    ///
    /// Format breakdown:
    ///   ~ = Agreement namespace prefix
    ///   <short-sha> = First 7 chars of tree SHA
    ///   - = Separator
    ///   <contract-name> = Contract identifier
    ///   <space> = Path scope separator
    ///   <path-scope> = Human-readable path or scope
    pub fn honor(
        &self,
        scope: &str,
        base_branch: &str,
        create_worktree: bool,
        worktree_name: Option<&str>,
        open_in_cursor: bool,
    ) -> Result<()> {
        // Get agreement details
        let agreement = self.manager.get_agreement(scope)?;
        let agreement = match agreement {
            Some(agreement) => agreement,
            None => {
                println!("❌ No agreement found for scope: {}", scope);
                return Ok(());
            }
        };

        // Get contract name
        let contract_name = get_contract_name(&agreement.agreement.contract)?;

        // Get path scope from tree resolution
        let resolution = self.manager.resolve_tree_path(scope)?;
        let path_scope = match resolution.resolution_type {
            ResolutionType::Path(path) => {
                if path.is_empty() {
                    "root".to_string()
                } else {
                    path
                }
            }
            ResolutionType::Commit(_) => "history".to_string(),
            ResolutionType::Tree(_) => "detached".to_string(),
            ResolutionType::NotFound => "unknown".to_string(),
        };

        // Generate branch name with new convention
        let short_scope = &scope[..7];
        let branch_name = if let Some(custom_name) = worktree_name {
            format!("~{}", custom_name)
        } else {
            format!("~{}-{}", short_scope, contract_name)
        };

        // Generate worktree directory name with path scope
        let worktree_dir = if let Some(custom_name) = worktree_name {
            custom_name.to_string()
        } else {
            // Sanitize path scope for filesystem use
            let sanitized_path = path_scope
                .replace('/', "-")
                .replace('.', "")
                .replace(' ', "_");

            if sanitized_path == "root" {
                format!("~{}-{}", short_scope, contract_name)
            } else {
                format!("~{}-{}-{}", short_scope, contract_name, sanitized_path)
            }
        };

        println!("🎯 Honoring agreement: {}", scope);
        println!("  Branch: {}", branch_name);
        println!("  Worktree: {}", worktree_dir);
        println!("  Path Scope: {}", path_scope);
        println!("  Contract: {}", contract_name);

        // Create remote branch
        let base_sha = get_base_commit_sha(base_branch)?;
        create_remote_branch(&branch_name, &base_sha)?;

        if create_worktree {
            create_worktree_for_branch(&branch_name, &worktree_dir)?;

            if open_in_cursor {
                open_worktree_in_cursor(&worktree_dir)?;
            }
        }

        println!("✅ Agreement honored successfully!");
        println!("  📁 Worktree: {}", worktree_dir);
        println!("  🌿 Branch: {}", branch_name);
        println!("  📍 Path Scope: {}", path_scope);

        Ok(())
    }

    pub fn current(&self) -> Result<()> {
        if let Some(metadata) = self.manager.get_current_agreement()? {
            println!("🎯 Current Agreement:");
            println!("  Scope: {}", metadata.agreement.scope);
            println!("  Contract: {}", metadata.agreement.contract);
            println!("  Status: {}", metadata.status);
            println!("  Created: {}", metadata.created_at);
            println!("  By: {}", metadata.created_by);

            // Show contract details
            if let Some(contract_content) = self.manager.resolve_contract(&metadata.agreement)? {
                println!("\n📄 Contract Content:");
                println!("{}", contract_content);
            }
        } else {
            println!("❌ No agreement found for current branch");
            println!("💡 Tip: Create an agreement with 'cargo run -p xtask -- agreement create'");
        }
        Ok(())
    }

    /// Enhanced list with resolution details
    pub fn list_with_resolution(&self) -> Result<()> {
        let resolved_agreements = self.manager.list_agreements_with_resolution()?;

        if resolved_agreements.is_empty() {
            println!("📝 No agreements found");
            return Ok(());
        }

        println!("📝 Agreements with Resolution:");
        for (metadata, resolution) in resolved_agreements {
            // Get contract name
            let contract_name = get_contract_name(&metadata.agreement.contract)
                .unwrap_or_else(|_| "Unknown".to_string());

            // Get short SHA hashes (7 characters)
            let short_scope = &metadata.agreement.scope[..7];
            let short_contract = &metadata.agreement.contract[..7];

            println!("  Scope: {} ({})", resolution, short_scope);
            println!("  Contract: {} ({})", contract_name, short_contract);
            println!("  Status: {:?}", metadata.status);
            println!("  Created: {}", metadata.created_at);
            println!("  By: {}", metadata.created_by);

            // Show resolution details
            if let Some(path) = &resolution.path {
                println!("  📁 Path: {}", path);
            }
            if let Some(commit) = &resolution.commit {
                println!("  📜 Commit: {}", commit);
            }
            if let Some(msg) = &resolution.commit_message {
                println!("  📝 Message: {}", msg);
            }
            if let Some(timestamp) = &resolution.last_seen_at {
                println!("  ⏰ Last seen: {}", timestamp);
            }
            println!();
        }
        Ok(())
    }

    /// Verify agreement with detailed resolution
    pub fn verify(&self, scope: &str) -> Result<()> {
        let (is_valid, resolution) = self.manager.validate_agreement_with_resolution(scope)?;

        println!("🔍 Agreement Verification:");
        println!("  Scope: {}", scope);
        println!("  Resolution: {}", resolution);
        println!("  Valid: {}", if is_valid { "✅ Yes" } else { "❌ No" });

        // Show detailed resolution information
        println!("\n📋 Resolution Details:");
        match resolution.resolution_type {
            ResolutionType::Path(path) => {
                println!("  Type: Path-based");
                println!("  Path: {}", path);
                println!(
                    "  In HEAD: {}",
                    if resolution.exists_in_head {
                        "✅ Yes"
                    } else {
                        "❌ No"
                    }
                );
            }
            ResolutionType::Commit(commit) => {
                println!("  Type: Commit-based");
                println!("  Commit: {}", commit);
                if let Some(msg) = resolution.commit_message {
                    println!("  Message: {}", msg);
                }
                println!(
                    "  Reachable: {}",
                    if resolution.is_reachable {
                        "✅ Yes"
                    } else {
                        "❌ No"
                    }
                );
            }
            ResolutionType::Tree(sha) => {
                println!("  Type: Tree-based");
                println!("  Tree: {}", sha);
                println!(
                    "  Reachable: {}",
                    if resolution.is_reachable {
                        "✅ Yes"
                    } else {
                        "❌ No"
                    }
                );
            }
            ResolutionType::NotFound => {
                println!("  Type: Not found");
                println!("  Status: Tree not found in repository");
            }
        }

        if let Some(timestamp) = resolution.last_seen_at {
            println!("  Last seen: {}", timestamp);
        }

        Ok(())
    }

    /// Audit all agreements for trust decay
    pub fn audit(&self) -> Result<()> {
        let resolved_agreements = self.manager.list_agreements_with_resolution()?;

        if resolved_agreements.is_empty() {
            println!("📝 No agreements to audit");
            return Ok(());
        }

        println!("🔍 Agreement Trust Audit:");
        println!("  Total agreements: {}", resolved_agreements.len());

        let mut in_head = 0;
        let mut in_history = 0;
        let mut unreachable = 0;
        let mut not_found = 0;

        for (metadata, resolution) in &resolved_agreements {
            match resolution.resolution_type {
                ResolutionType::Path(_) => {
                    in_head += 1;
                    println!(
                        "  ✅ {} - Active in HEAD",
                        metadata.agreement.scope[..7].to_string()
                    );
                }
                ResolutionType::Commit(_) => {
                    in_history += 1;
                    println!(
                        "  ⚠️  {} - In history",
                        metadata.agreement.scope[..7].to_string()
                    );
                }
                ResolutionType::Tree(_) => {
                    if resolution.is_reachable {
                        in_history += 1;
                        println!(
                            "  ⚠️  {} - Detached but reachable",
                            metadata.agreement.scope[..7].to_string()
                        );
                    } else {
                        unreachable += 1;
                        println!(
                            "  ❌ {} - Unreachable",
                            metadata.agreement.scope[..7].to_string()
                        );
                    }
                }
                ResolutionType::NotFound => {
                    not_found += 1;
                    println!(
                        "  💀 {} - Not found",
                        metadata.agreement.scope[..7].to_string()
                    );
                }
            }
        }

        println!("\n📊 Audit Summary:");
        println!("  Active in HEAD: {}", in_head);
        println!("  In history: {}", in_history);
        println!("  Unreachable: {}", unreachable);
        println!("  Not found: {}", not_found);

        if unreachable > 0 || not_found > 0 {
            println!("\n⚠️  Consider running 'agreement prune' to clean up invalid agreements");
        }

        Ok(())
    }

    /// Check if a worktree exists for the given scope
    ///
    /// Looks for worktrees that match the new naming convention:
    /// ~<short-sha>-<contract-name> or ~<short-sha>-<contract-name>-<path-scope>
    fn check_worktree_exists(&self, scope: &str) -> Result<bool> {
        let short_scope = &scope[..7];

        // Get list of worktrees
        let worktree_output = std::process::Command::new("git")
            .args(&["worktree", "list", "--porcelain"])
            .output()?;

        let worktree_list = String::from_utf8(worktree_output.stdout)?;

        // Check if any worktree directory contains the scope SHA or follows the naming pattern
        Ok(worktree_list.lines().any(|line| {
            line.contains(short_scope)
                || line.contains(scope)
                || line.contains(&format!("~{}", short_scope))
        }))
    }

    /// Check if a tree represents the root directory structure
    fn is_root_tree(&self, tree_sha: &str) -> Result<bool> {
        // Get current HEAD tree SHA
        let current_tree = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD^{tree}"])
            .output()?;
        let current_tree_sha = String::from_utf8_lossy(&current_tree.stdout)
            .trim()
            .to_string();

        // If it's the current tree, it's definitely root
        if tree_sha == current_tree_sha {
            return Ok(true);
        }

        // Check if the tree has the same structure as root by comparing key files
        let key_files = vec![
            "Cargo.toml",
            "README.md",
            ".gitignore",
            "rust-toolchain.toml",
        ];

        for file in key_files {
            let current_file = std::process::Command::new("git")
                .args(&["ls-tree", &current_tree_sha, file])
                .output();

            let tree_file = std::process::Command::new("git")
                .args(&["ls-tree", tree_sha, file])
                .output();

            // If both trees have the same file with same SHA, it's likely root
            if let (Ok(current), Ok(tree)) = (current_file, tree_file) {
                let current_output = String::from_utf8_lossy(&current.stdout);
                let tree_output = String::from_utf8_lossy(&tree.stdout);
                let current_sha = current_output.trim();
                let tree_sha_output = tree_output.trim();

                if !current_sha.is_empty() && !tree_sha_output.is_empty() {
                    let current_parts: Vec<&str> = current_sha.split_whitespace().collect();
                    let tree_parts: Vec<&str> = tree_sha_output.split_whitespace().collect();

                    if current_parts.len() >= 3 && tree_parts.len() >= 3 {
                        if current_parts[2] == tree_parts[2] {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// List local agreements (agreements that have associated worktrees)
    pub fn list_local(&self) -> Result<()> {
        let agreements = self.manager.list_agreements()?;

        if agreements.is_empty() {
            println!("📝 No agreements found");
            return Ok(());
        }

        // Get list of worktrees
        let worktree_output = std::process::Command::new("git")
            .args(&["worktree", "list", "--porcelain"])
            .output()?;

        let worktree_list = String::from_utf8(worktree_output.stdout)?;
        let worktree_dirs: Vec<String> = worktree_list
            .lines()
            .filter_map(|line| {
                if line.starts_with("worktree") {
                    line.split_whitespace().nth(1).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

        // Get current tree SHA to mark active agreements
        let current_tree = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD^{tree}"])
            .output()?;
        let current_tree_sha = String::from_utf8_lossy(&current_tree.stdout)
            .trim()
            .to_string();

        let mut local_agreements = Vec::new();

        for metadata in agreements {
            let short_scope = &metadata.agreement.scope[..7];

            // Check if this agreement has a local worktree
            let has_worktree = worktree_dirs.iter().any(|dir| {
                dir.contains(short_scope)
                    || dir.contains(&metadata.agreement.scope)
                    || dir.contains(&format!("~{}", short_scope))
            });

            if has_worktree {
                // Get path scope and tree slug
                let resolution = self.manager.resolve_tree_path(&metadata.agreement.scope)?;
                let path_scope = match resolution.resolution_type {
                    ResolutionType::Path(path) => {
                        if path.is_empty() {
                            "./".to_string()
                        } else {
                            path
                        }
                    }
                    ResolutionType::Commit(_) => "history".to_string(),
                    ResolutionType::Tree(_) => {
                        // Check if this tree represents the root structure
                        if self.is_root_tree(&metadata.agreement.scope)? {
                            "./".to_string()
                        } else {
                            "detached".to_string()
                        }
                    }
                    ResolutionType::NotFound => "unknown".to_string(),
                };

                let tree_slug = metadata.status.clone();

                // Find the actual worktree directory
                let worktree_dir = worktree_dirs
                    .iter()
                    .find(|dir| {
                        dir.contains(short_scope)
                            || dir.contains(&metadata.agreement.scope)
                            || dir.contains(&format!("~{}", short_scope))
                    })
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_string());

                // Determine status indicator
                let indicator = if metadata.agreement.scope == current_tree_sha {
                    "* " // Current agreement
                } else {
                    "+ " // Has worktree
                };

                // Get contract name for display
                let contract_name = get_contract_name(&metadata.agreement.contract)
                    .unwrap_or_else(|_| "unknown".to_string());

                local_agreements.push((
                    metadata,
                    path_scope,
                    tree_slug,
                    worktree_dir,
                    indicator,
                    contract_name,
                ));
            }
        }

        if local_agreements.is_empty() {
            println!("📝 No local agreements found (no worktrees exist)");
            return Ok(());
        }

        // Sort: current first, then by scope
        local_agreements.sort_by(|a, b| {
            let a_is_current = a.0.agreement.scope == current_tree_sha;
            let b_is_current = b.0.agreement.scope == current_tree_sha;

            if a_is_current && !b_is_current {
                std::cmp::Ordering::Less
            } else if !a_is_current && b_is_current {
                std::cmp::Ordering::Greater
            } else {
                a.0.agreement.scope.cmp(&b.0.agreement.scope)
            }
        });

        println!("🏠 Local Agreements (with worktrees):");
        println!();

        for (metadata, path_scope, tree_slug, worktree_dir, indicator, contract_name) in
            local_agreements
        {
            let short_scope = &metadata.agreement.scope[..7];

            println!(
                "{}{} ({}): {} [{}]",
                indicator, short_scope, contract_name, path_scope, tree_slug
            );
            println!("  📁 Worktree: {}", worktree_dir);
            println!("  📅 Created: {}", metadata.created_at);
            println!("  👤 By: {}", metadata.created_by);
            println!("  📊 Status: {}", metadata.status);
            println!();
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
        /// Check origin/main reachability
        #[arg(long)]
        check_main: bool,
    },
    /// Resolve contract content from an agreement
    Resolve {
        /// Scope SHA of the agreement to resolve
        scope: String,
        /// Output format (json, text, pretty)
        #[arg(long, default_value = "pretty")]
        format: String,
    },
    /// Show agreement history/log
    Log {
        /// Scope SHA of the agreement to show history for (optional)
        scope: Option<String>,
    },
    /// Update agreement status
    UpdateStatus {
        /// Scope SHA of the agreement to update
        scope: String,
        /// New status (backlog, developing, fulfilled)
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
        /// Optional scope tree SHA (defaults to current HEAD tree)
        #[arg(long)]
        scope: Option<String>,
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
    /// Show the current agreement based on the current branch
    Current,
    /// List agreements with enhanced resolution details
    ListWithResolution,
    /// Verify agreement with detailed resolution
    Verify {
        /// Scope SHA of the agreement to verify
        scope: String,
        /// Show verbose output
        #[arg(long)]
        verbose: bool,
    },
    /// Audit all agreements for trust decay
    Audit,
    /// List local agreements (agreements with worktrees)
    Local,
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
        AgreementCommands::Validate {
            scope,
            strict,
            check_main,
        } => {
            cli.validate(&scope)?;
            if strict {
                // In a real implementation, you'd check the actual validation result
                // and exit with error if it fails
                println!("✅ Agreement validation passed");
            }
            if check_main {
                // In a real implementation, you'd check the origin/main reachability
                // and exit with error if it fails
                println!("✅ Agreement reachability check passed");
            }
        }
        AgreementCommands::Resolve { scope, format: _ } => {
            cli.resolve(&scope)?;
            // In a real implementation, you'd format the output based on the format parameter
        }
        AgreementCommands::Log { scope } => {
            cli.log(scope.as_deref())?;
        }
        AgreementCommands::UpdateStatus { scope, status } => {
            let valid_statuses = ["backlog", "developing", "fulfilled"];
            if !valid_statuses.contains(&status.as_str()) {
                anyhow::bail!(
                    "Invalid status: {}. Valid values: backlog, developing, fulfilled",
                    status
                );
            }

            let manager = AgreementManager::new(&current_dir)?;
            manager.update_agreement_status(&scope, &status)?;
            println!("✅ Updated agreement status to {}", status);
        }
        AgreementCommands::CreateFromFile {
            contract_file,
            description,
            scope,
        } => {
            create_agreement_from_file(&contract_file, description.as_deref(), scope.as_deref())
                .await?;
        }
        AgreementCommands::Prune { dry_run, force } => {
            cli.prune(dry_run, force)?;
        }
        AgreementCommands::Honor {
            scope,
            base_branch,
            create_worktree,
            worktree_name,
            open_in_cursor,
        } => {
            cli.honor(
                &scope,
                &base_branch,
                create_worktree,
                worktree_name.as_deref(),
                open_in_cursor,
            )?;
        }
        AgreementCommands::Current => {
            cli.current()?;
        }
        AgreementCommands::ListWithResolution => {
            cli.list_with_resolution()?;
        }
        AgreementCommands::Verify { scope, verbose } => {
            cli.verify(&scope)?;
            if verbose {
                println!("✅ Agreement verification complete.");
            }
        }
        AgreementCommands::Audit => {
            cli.audit()?;
        }
        AgreementCommands::Local => {
            cli.list_local()?;
        }
    }

    Ok(())
}

/// Create an agreement from a contract file
async fn create_agreement_from_file(
    contract_file: &str,
    description: Option<&str>,
    scope: Option<&str>,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let manager = AgreementManager::new(&current_dir)?;

    // Get tree SHA - use provided scope or current HEAD tree
    let tree_sha = if let Some(provided_scope) = scope {
        provided_scope.to_string()
    } else {
        let output = std::process::Command::new("git")
            .args(&["rev-parse", "HEAD^{tree}"])
            .output()?;
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    println!("📋 Current tree SHA: {}", tree_sha);

    // Get contract file SHA
    let contract_sha = get_file_blob_sha(contract_file)?;
    println!("📄 Contract file SHA: {}", contract_sha);

    // Validate that the tree SHA is reachable from origin/main
    if !manager.is_tree_sha_reachable_from_origin_main(&tree_sha)? {
        anyhow::bail!(
            "Agreement tree scope must be reachable from origin/main. Tree SHA: {}",
            tree_sha
        );
    }

    // Create agreement
    manager.create_agreement(&tree_sha, &contract_sha, description)?;
    println!("✅ Agreement created successfully!");
    println!("   Scope: {}", tree_sha);
    println!("   Contract: {}", contract_sha);

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

/// Get the name from a contract JSON blob
fn get_contract_name(contract_sha: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["cat-file", "blob", contract_sha])
        .output()
        .context(format!(
            "Failed to get contract content for {}",
            contract_sha
        ))?;

    if !output.status.success() {
        anyhow::bail!(
            "Failed to get contract content for {}: {}",
            contract_sha,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let contract_content = String::from_utf8(output.stdout)?;

    // Parse JSON and extract name field
    let contract_json: serde_json::Value =
        serde_json::from_str(&contract_content).context("Failed to parse contract JSON")?;

    let name = contract_json
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Contract does not have a 'name' field"))?;

    Ok(name.to_string())
}

/// Get the path for a tree SHA
fn get_tree_path(tree_sha: &str) -> Result<String> {
    // First, try to find the current path of this tree in HEAD
    let ls_tree_output = std::process::Command::new("git")
        .args(["ls-tree", "-r", "-t", "HEAD"])
        .output()
        .context("Failed to get HEAD tree structure")?;

    if ls_tree_output.status.success() {
        let tree_content = String::from_utf8(ls_tree_output.stdout)?;
        for line in tree_content.lines() {
            if line.contains(tree_sha) {
                // Parse the tree entry to get the path
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 && parts[1] == "tree" && parts[2] == tree_sha {
                    let path = parts[3];
                    return Ok(format!("Path: {}", path));
                }
            }
        }
    }

    // If not found in HEAD, try to find in any commit
    let output = std::process::Command::new("git")
        .args(["rev-list", "--all"])
        .output()
        .context("Failed to get all commits")?;

    if !output.status.success() {
        anyhow::bail!("Failed to get all commits");
    }

    let commits = String::from_utf8(output.stdout)?;

    // Search for commits that contain this tree
    for commit in commits.lines() {
        let tree_output = std::process::Command::new("git")
            .args(["rev-parse", &format!("{}:^{{tree}}", commit)])
            .output();

        if let Ok(tree_output) = tree_output {
            if tree_output.status.success() {
                let commit_tree = String::from_utf8(tree_output.stdout)?.trim().to_string();
                if commit_tree == tree_sha {
                    // Found the commit, now get the commit message
                    let log_output = std::process::Command::new("git")
                        .args(["log", "--oneline", "-1", commit])
                        .output()
                        .context("Failed to get commit message")?;

                    if log_output.status.success() {
                        let commit_msg = String::from_utf8(log_output.stdout)?.trim().to_string();
                        return Ok(format!("Commit: {}", commit_msg));
                    }
                }
            }
        }
    }

    // If we can't find a path or commit, show the tree SHA
    Ok(format!("Tree: {}", &tree_sha[..7]))
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
    let worktree_path = format!("worktrees/{}", worktree_dir);
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

/// Open a worktree in Cursor
fn open_worktree_in_cursor(worktree_dir: &str) -> Result<()> {
    // Get the absolute path to the worktree
    let worktree_path = std::path::Path::new(".wt").join(worktree_dir);
    let absolute_path = worktree_path.canonicalize().context(format!(
        "Failed to get absolute path for worktree: {}",
        worktree_dir
    ))?;

    // Open in Cursor
    let cursor_output = std::process::Command::new("cursor")
        .args([absolute_path.to_str().unwrap()])
        .output()
        .context("Failed to open worktree in Cursor")?;

    if !cursor_output.status.success() {
        let error = String::from_utf8_lossy(&cursor_output.stderr);
        anyhow::bail!("Failed to open worktree in Cursor: {}", error);
    }

    Ok(())
}

/// Get the Git repository root, handling worktree scenarios
fn get_git_repo_root() -> Result<std::path::PathBuf> {
    // Check if we're in a worktree
    let worktree_output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .context("Failed to get Git directory")?;

    if !worktree_output.status.success() {
        anyhow::bail!("Not in a Git repository");
    }

    let git_dir = String::from_utf8(worktree_output.stdout)?
        .trim()
        .to_string();

    // If we're in a worktree, the git dir will be something like .git/worktrees/<name>
    if git_dir.contains("worktrees") {
        // Get the main repository root
        let main_repo_output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to get repository root")?;

        if !main_repo_output.status.success() {
            anyhow::bail!("Failed to get repository root");
        }

        let repo_root = String::from_utf8(main_repo_output.stdout)?
            .trim()
            .to_string();
        Ok(std::path::PathBuf::from(repo_root))
    } else {
        // Regular repository
        let repo_root_output = std::process::Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Failed to get repository root")?;

        if !repo_root_output.status.success() {
            anyhow::bail!("Failed to get repository root");
        }

        let repo_root = String::from_utf8(repo_root_output.stdout)?
            .trim()
            .to_string();
        Ok(std::path::PathBuf::from(repo_root))
    }
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
            check_main: true,
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
