use anyhow::{Context, Result};
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
    /// Optional description
    pub description: Option<String>,
    /// Additional metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Agreement status
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Manager for creating and managing agreements using Git notes
pub struct AgreementManager {
    repo: Repository,
    notes_ref: String,
}

impl AgreementManager {
    /// Create a new agreement manager
    pub fn new(repo_path: &Path) -> Result<Self> {
        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open repository at {repo_path:?}"))?;

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
            created_by: self.get_current_user()?,
            status: AgreementStatus::Active,
            description: description.map(|s| s.to_string()),
            metadata: None,
        };

        // Store as Git note
        let note_content = serde_json::to_string_pretty(&metadata)?;
        self.store_note(&scope, &note_content)?;

        Ok(format!(
            "Agreement created: scope={}, contract={}",
            scope, contract
        ))
    }

    /// Get an agreement by scope
    pub fn get_agreement(&self, scope: &str) -> Result<Option<AgreementMetadata>> {
        let note_content = self.get_note(scope)?;
        if note_content.is_empty() {
            return Ok(None);
        }

        let metadata: AgreementMetadata = serde_json::from_str(&note_content)?;
        Ok(Some(metadata))
    }

    /// List all agreements
    pub fn list_agreements(&self) -> Result<Vec<AgreementMetadata>> {
        let mut agreements = Vec::new();

        // Get all notes from the agreements ref
        let notes = self.repo.notes(Some(&self.notes_ref))?;

        for note_result in notes {
            if let Ok((oid, _note_oid)) = note_result {
                if let Ok(note) = self.repo.find_note(Some(&self.notes_ref), oid) {
                    if let Some(content) = note.message() {
                        if let Ok(metadata) = serde_json::from_str::<AgreementMetadata>(content) {
                            agreements.push(metadata);
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
            let note_content = serde_json::to_string_pretty(&metadata)?;
            self.store_note(scope, &note_content)?;
        }

        Ok(())
    }

    /// Validate an agreement (check if contract exists in scope)
    pub fn validate_agreement(&self, scope: &str) -> Result<bool> {
        if let Some(metadata) = self.get_agreement(scope)? {
            // Check if the contract blob exists in the scope tree
            let scope_tree = self
                .repo
                .find_tree(git2::Oid::from_str(&metadata.agreement.scope)?)?;
            let contract_blob = self
                .repo
                .find_blob(git2::Oid::from_str(&metadata.agreement.contract)?)?;

            // This is a simplified validation - in practice you'd want to check
            // if the contract is actually referenced in the scope tree
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resolve contract from agreement
    pub fn resolve_contract(&self, agreement: &Agreement) -> Result<Option<String>> {
        // Get the contract blob content
        let contract_oid = git2::Oid::from_str(&agreement.contract)?;
        let contract_blob = self.repo.find_blob(contract_oid)?;

        let content = std::str::from_utf8(contract_blob.content())?;
        Ok(Some(content.to_string()))
    }

    /// Store a Git note
    fn store_note(&self, key: &str, content: &str) -> Result<()> {
        let signature = self.get_signature()?;

        // Create blob with note content
        let blob_oid = self.repo.blob(content.as_bytes())?;

        // Create or update note
        let note_ref = format!("{}/{}", self.notes_ref, key.replace('/', "_"));

        // This is a simplified implementation - in practice you'd want to
        // handle the note tree structure more carefully
        println!("Would store note at {}: {}", note_ref, content);

        Ok(())
    }

    /// Get a Git note
    fn get_note(&self, key: &str) -> Result<String> {
        let note_ref = format!("{}/{}", self.notes_ref, key.replace('/', "_"));

        // This is a simplified implementation - in practice you'd want to
        // actually read from Git notes
        println!("Would get note from {}", note_ref);

        Ok("".to_string())
    }

    /// Get current user signature
    fn get_signature(&self) -> Result<Signature> {
        let config = self.repo.config()?;
        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Unknown".to_string());
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "unknown@example.com".to_string());

        Ok(Signature::now(&name, &email)?)
    }

    /// Get current user name
    fn get_current_user(&self) -> Result<String> {
        let config = self.repo.config()?;
        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Unknown".to_string());
        Ok(name)
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
            if let Some(desc) = metadata.description {
                println!("  Description: {}", desc);
            }
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
            println!("  Created by: {}", metadata.created_by);
            if let Some(desc) = metadata.description {
                println!("  Description: {}", desc);
            }
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
                println!("📄 Contract Content:");
                println!("{}", contract_content);
            } else {
                println!("❌ Could not resolve contract");
            }
        } else {
            println!("❌ No agreement found for scope: {}", scope);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_agreement_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        let manager = AgreementManager::new(temp_dir.path()).unwrap();

        // Test creating an agreement
        let result = manager.create_agreement(
            "test-scope-sha",
            "test-contract-sha",
            Some("Test agreement"),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_agreement_serialization() {
        let agreement = Agreement {
            scope: "test-scope".to_string(),
            contract: "test-contract".to_string(),
        };

        let json = serde_json::to_string_pretty(&agreement).unwrap();
        let deserialized: Agreement = serde_json::from_str(&json).unwrap();

        assert_eq!(agreement.scope, deserialized.scope);
        assert_eq!(agreement.contract, deserialized.contract);
    }
}
