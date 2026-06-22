use anyhow::{Context, Result};
use git2::{Repository, Signature};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Contract state stored in Git notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractStateNote {
    pub file: String,
    pub contract: String,
    pub state: String,
    pub hash: String,
    pub validated_by: String,
    pub timestamp: String,
    pub parent_scope: Option<String>,
    pub parent_hash: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Transition log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionLogEntry {
    pub transition: String,
    pub from: String,
    pub to: String,
    pub file: String,
    pub hash: String,
    pub tool: String,
    pub timestamp: String,
    pub reason: Option<String>,
    pub commit_hash: Option<String>,
    pub user: Option<String>,
    pub environment: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Git notes manager for contract states
#[allow(dead_code)]
pub struct GitNotesManager {
    repo: Repository,
    contracts_ref: String,
    transitions_ref: String,
    merkle_proofs_ref: String,
}

impl GitNotesManager {
    /// Create a new Git notes manager
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open repository at {repo_path:?}"))?;

        Ok(GitNotesManager {
            repo,
            contracts_ref: "refs/notes/contracts".to_string(),
            transitions_ref: "refs/notes/contracts-log".to_string(),
            merkle_proofs_ref: "refs/notes/merkle-proofs".to_string(),
        })
    }

    /// Store a contract state
    pub fn store_contract_state(&self, state: &ContractStateNote) -> Result<()> {
        let content =
            serde_json::to_string_pretty(state).context("Failed to serialize contract state")?;

        let signature = self.get_signature()?;
        let tree = self.get_or_create_notes_tree(&self.contracts_ref)?;

        // Create blob with state content
        let blob_oid = self
            .repo
            .blob(content.as_bytes())
            .context("Failed to create blob")?;

        // Create tree with the file path as the note name
        let mut tree_builder = self
            .repo
            .treebuilder(Some(&tree))
            .context("Failed to create tree builder")?;

        let note_name = self.sanitize_note_name(&state.file);
        tree_builder
            .insert(&note_name, blob_oid, 0o100644)
            .context("Failed to insert note")?;

        let new_tree_oid = tree_builder.write().context("Failed to write tree")?;

        // Create commit
        let new_tree = self
            .repo
            .find_tree(new_tree_oid)
            .context("Failed to find tree")?;

        let parent_commit = self.get_or_create_notes_commit(&self.contracts_ref)?;

        let commit_message = format!("Update contract state for {}", state.file);
        let commit_oid = self
            .repo
            .commit(
                Some(&self.contracts_ref),
                &signature,
                &signature,
                &commit_message,
                &new_tree,
                &[&parent_commit],
            )
            .context("Failed to create commit")?;

        println!(
            "✅ Stored contract state for {}: {}",
            state.file, commit_oid
        );
        Ok(())
    }

    /// Retrieve a contract state
    #[allow(dead_code)]
    pub fn get_contract_state(&self, file_path: &str) -> Result<Option<ContractStateNote>> {
        let note_name = self.sanitize_note_name(file_path);

        match self.get_note_content(&self.contracts_ref, &note_name)? {
            Some(content) => {
                let state: ContractStateNote = serde_json::from_str(&content)
                    .context("Failed to deserialize contract state")?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Store a transition log entry
    pub fn store_transition_log(&self, entry: &TransitionLogEntry) -> Result<()> {
        let content =
            serde_json::to_string_pretty(entry).context("Failed to serialize transition log")?;

        let signature = self.get_signature()?;
        let tree = self.get_or_create_notes_tree(&self.transitions_ref)?;

        // Create blob with transition content
        let blob_oid = self
            .repo
            .blob(content.as_bytes())
            .context("Failed to create blob")?;

        // Create tree with timestamp as the note name
        let mut tree_builder = self
            .repo
            .treebuilder(Some(&tree))
            .context("Failed to create tree builder")?;

        let note_name = format!(
            "{}-{}",
            entry.timestamp,
            self.sanitize_note_name(&entry.file)
        );
        tree_builder
            .insert(&note_name, blob_oid, 0o100644)
            .context("Failed to insert note")?;

        let new_tree_oid = tree_builder.write().context("Failed to write tree")?;

        // Create commit
        let new_tree = self
            .repo
            .find_tree(new_tree_oid)
            .context("Failed to find tree")?;

        let parent_commit = self.get_or_create_notes_commit(&self.transitions_ref)?;

        let commit_message = format!(
            "Log transition: {} -> {} for {}",
            entry.from, entry.to, entry.file
        );
        let commit_oid = self
            .repo
            .commit(
                Some(&self.transitions_ref),
                &signature,
                &signature,
                &commit_message,
                &new_tree,
                &[&parent_commit],
            )
            .context("Failed to create commit")?;

        println!("✅ Logged transition for {}: {}", entry.file, commit_oid);
        Ok(())
    }

    /// Get all contract states
    #[allow(dead_code)]
    pub fn get_all_contract_states(&self) -> Result<HashMap<String, ContractStateNote>> {
        let mut states = HashMap::new();

        match self.get_notes_tree(&self.contracts_ref)? {
            Some(tree) => {
                for entry in tree.iter() {
                    if let Some(name) = entry.name() {
                        if let Some(content) = self.get_note_content(&self.contracts_ref, name)? {
                            if let Ok(state) = serde_json::from_str::<ContractStateNote>(&content) {
                                states.insert(state.file.clone(), state);
                            }
                        }
                    }
                }
            }
            None => {
                // No notes tree exists yet
            }
        }

        Ok(states)
    }

    /// Get transition history for a file
    #[allow(dead_code)]
    pub fn get_transition_history(&self, file_path: &str) -> Result<Vec<TransitionLogEntry>> {
        let mut transitions = Vec::new();

        match self.get_notes_tree(&self.transitions_ref)? {
            Some(tree) => {
                for entry in tree.iter() {
                    if let Some(name) = entry.name() {
                        if name.contains(&self.sanitize_note_name(file_path)) {
                            if let Some(content) =
                                self.get_note_content(&self.transitions_ref, name)?
                            {
                                if let Ok(transition) =
                                    serde_json::from_str::<TransitionLogEntry>(&content)
                                {
                                    transitions.push(transition);
                                }
                            }
                        }
                    }
                }
            }
            None => {
                // No transitions tree exists yet
            }
        }

        // Sort by timestamp
        transitions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        Ok(transitions)
    }

    /// Delete a contract state
    pub fn delete_contract_state(&self, file_path: &str) -> Result<()> {
        let note_name = self.sanitize_note_name(file_path);

        match self.get_notes_tree(&self.contracts_ref)? {
            Some(tree) => {
                let mut tree_builder = self
                    .repo
                    .treebuilder(Some(&tree))
                    .context("Failed to create tree builder")?;

                tree_builder
                    .remove(&note_name)
                    .context("Failed to remove note")?;

                let new_tree_oid = tree_builder.write().context("Failed to write tree")?;

                let signature = self.get_signature()?;
                let new_tree = self
                    .repo
                    .find_tree(new_tree_oid)
                    .context("Failed to find tree")?;

                let parent_commit = self.get_or_create_notes_commit(&self.contracts_ref)?;

                let commit_message = format!("Delete contract state for {file_path}");
                let commit_oid = self
                    .repo
                    .commit(
                        Some(&self.contracts_ref),
                        &signature,
                        &signature,
                        &commit_message,
                        &new_tree,
                        &[&parent_commit],
                    )
                    .context("Failed to create commit")?;

                println!("✅ Deleted contract state for {file_path}: {commit_oid}",);
            }
            None => {
                // No notes tree exists, nothing to delete
            }
        }

        Ok(())
    }

    /// Get or create notes tree
    fn get_or_create_notes_tree(&self, ref_name: &str) -> Result<git2::Tree<'_>> {
        match self.get_notes_tree(ref_name)? {
            Some(tree) => Ok(tree),
            None => {
                // Create empty tree
                let empty_tree_oid = self
                    .repo
                    .treebuilder(None)
                    .context("Failed to create empty tree builder")?
                    .write()
                    .context("Failed to write empty tree")?;

                self.repo
                    .find_tree(empty_tree_oid)
                    .context("Failed to find empty tree")
            }
        }
    }

    /// Get notes tree
    fn get_notes_tree(&self, ref_name: &str) -> Result<Option<git2::Tree<'_>>> {
        match self.repo.find_reference(ref_name) {
            Ok(reference) => {
                let commit = reference
                    .peel_to_commit()
                    .context("Failed to peel reference to commit")?;
                let tree = commit.tree().context("Failed to get tree from commit")?;
                Ok(Some(tree))
            }
            Err(_) => Ok(None), // Reference doesn't exist
        }
    }

    /// Get or create notes commit
    fn get_or_create_notes_commit(&self, ref_name: &str) -> Result<git2::Commit<'_>> {
        match self.repo.find_reference(ref_name) {
            Ok(reference) => reference
                .peel_to_commit()
                .context("Failed to peel reference to commit"),
            Err(_) => {
                // Create initial commit
                let signature = self.get_signature()?;
                let empty_tree_oid = self
                    .repo
                    .treebuilder(None)
                    .context("Failed to create empty tree builder")?
                    .write()
                    .context("Failed to write empty tree")?;

                let empty_tree = self
                    .repo
                    .find_tree(empty_tree_oid)
                    .context("Failed to find empty tree")?;

                let commit_oid = self
                    .repo
                    .commit(
                        Some(ref_name),
                        &signature,
                        &signature,
                        "Initial commit",
                        &empty_tree,
                        &[],
                    )
                    .context("Failed to create initial commit")?;

                self.repo
                    .find_commit(commit_oid)
                    .context("Failed to find initial commit")
            }
        }
    }

    /// Get note content
    #[allow(dead_code)]
    fn get_note_content(&self, ref_name: &str, note_name: &str) -> Result<Option<String>> {
        match self.get_notes_tree(ref_name)? {
            Some(tree) => {
                if let Some(entry) = tree.get_name(note_name) {
                    let blob = self
                        .repo
                        .find_blob(entry.id())
                        .context("Failed to find blob")?;
                    let content = String::from_utf8(blob.content().to_vec())
                        .context("Failed to convert blob content to string")?;
                    Ok(Some(content))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Get signature for commits
    fn get_signature(&self) -> Result<Signature<'_>> {
        // Try to get user info from Git config
        let config = self
            .repo
            .config()
            .context("Failed to get repository config")?;

        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Hooksmith Contract Validator".to_string());
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "contract-validator@hooksmith.local".to_string());

        Signature::now(&name, &email).context("Failed to create signature")
    }

    /// Sanitize note name for Git
    fn sanitize_note_name(&self, file_path: &str) -> String {
        file_path.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
    }

    /// List all contract files
    #[allow(dead_code)]
    pub fn list_contract_files(&self) -> Result<Vec<String>> {
        let states = self.get_all_contract_states()?;
        Ok(states.keys().cloned().collect())
    }

    /// Check if a file has a contract state
    #[allow(dead_code)]
    pub fn has_contract_state(&self, file_path: &str) -> Result<bool> {
        let state = self.get_contract_state(file_path)?;
        Ok(state.is_some())
    }

    /// Get repository path
    #[allow(dead_code)]
    pub fn repo_path(&self) -> &Path {
        self.repo.path().parent().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_notes_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = git2::Repository::init(temp_dir.path()).unwrap();

        let manager = GitNotesManager::new(temp_dir.path()).unwrap();
        assert_eq!(manager.repo_path(), temp_dir.path());
    }

    #[test]
    fn test_sanitize_note_name() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = git2::Repository::init(temp_dir.path()).unwrap();
        let manager = GitNotesManager::new(temp_dir.path()).unwrap();

        let sanitized = manager.sanitize_note_name("src/modules/git_model.rs");
        assert_eq!(sanitized, "src_modules_git_model.rs");
    }
}
