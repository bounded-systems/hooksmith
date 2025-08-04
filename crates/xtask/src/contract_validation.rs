use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::git_notes_manager::{ContractStateNote, GitNotesManager, TransitionLogEntry};

/// Contract validation proof structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractProof {
    /// File path being validated
    pub file_path: String,
    /// SHA-256 hash of the file content
    pub blob_hash: String,
    /// SHA-256 hash of the extracted AST
    pub ast_hash: String,
    /// SHA-256 hash of the JSON schema
    pub schema_hash: String,
    /// Contract type (e.g., "rust_validation", "json_schema")
    pub contract_type: String,
    /// Validation timestamp
    pub validated_at: String,
    /// Tool version that performed validation
    pub validated_by: String,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Contract validation commands
#[derive(Debug, Subcommand)]
pub enum ContractValidationCommands {
    /// Validate a contract file and store proof in Git notes
    Validate {
        /// File path to validate
        #[arg(required = true)]
        file: String,
        /// Contract type
        #[arg(long, default_value = "rust_validation")]
        contract_type: String,
        /// Store proof in Git notes
        #[arg(long, default_value = "true")]
        store: bool,
        /// Validate AST extraction
        #[arg(long, default_value = "true")]
        validate_ast: bool,
        /// Validate JSON schema generation
        #[arg(long, default_value = "true")]
        validate_schema: bool,
    },
    /// Verify existing contract proofs
    Verify {
        /// File path to verify
        #[arg(required = true)]
        file: String,
        /// Strict verification (fail on any mismatch)
        #[arg(long, default_value = "true")]
        strict: bool,
    },
    /// Generate contract proof without storing
    Generate {
        /// File path to generate proof for
        #[arg(required = true)]
        file: String,
        /// Contract type
        #[arg(long, default_value = "rust_validation")]
        contract_type: String,
    },
    /// List all contract proofs
    List {
        /// Show detailed information
        #[arg(long, default_value = "false")]
        detailed: bool,
    },
    /// Clean up old contract proofs
    Cleanup {
        /// Days to keep proofs
        #[arg(long, default_value = "30")]
        days: u32,
        /// Dry run (don't actually delete)
        #[arg(long, default_value = "false")]
        dry_run: bool,
    },
}

/// Contract validator implementation
pub struct ContractValidator {
    notes_manager: GitNotesManager,
}

impl ContractValidator {
    /// Create a new contract validator
    pub fn new() -> Result<Self> {
        let notes_manager = GitNotesManager::new(Path::new("."))?;
        Ok(Self { notes_manager })
    }

    /// Run contract validation commands
    pub async fn run(&self, command: ContractValidationCommands) -> Result<()> {
        match command {
            ContractValidationCommands::Validate {
                file,
                contract_type,
                store,
                validate_ast,
                validate_schema,
            } => {
                self.validate_contract(&file, &contract_type, store, validate_ast, validate_schema)
                    .await?;
            }
            ContractValidationCommands::Verify { file, strict } => {
                self.verify_contract(&file, strict).await?;
            }
            ContractValidationCommands::Generate {
                file,
                contract_type,
            } => {
                self.generate_proof(&file, &contract_type).await?;
            }
            ContractValidationCommands::List { detailed } => {
                self.list_proofs(detailed).await?;
            }
            ContractValidationCommands::Cleanup { days, dry_run } => {
                self.cleanup_proofs(days, dry_run).await?;
            }
        }
        Ok(())
    }

    /// Validate a contract file
    async fn validate_contract(
        &self,
        file_path: &str,
        contract_type: &str,
        store: bool,
        validate_ast: bool,
        validate_schema: bool,
    ) -> Result<()> {
        println!("🔍 Validating contract: {file_path}");
        println!("   Type: {contract_type}");
        println!("   Store: {store}");

        // Validate file exists
        let path = Path::new(file_path);
        if !path.exists() {
            anyhow::bail!("File does not exist: {}", file_path);
        }

        // Generate proof
        let proof = self
            .generate_proof_internal(file_path, contract_type, validate_ast, validate_schema)
            .await?;

        // Verify against existing proof if it exists
        if let Some(existing_proof) = self.get_existing_proof(file_path).await? {
            if self.verify_proofs(&proof, &existing_proof)? {
                println!("   ✅ Proof matches existing validation");
            } else {
                anyhow::bail!("   ❌ Proof mismatch detected - file may have changed");
            }
        } else {
            println!("   ✅ New proof generated");
        }

        // Store proof if requested
        if store {
            self.store_proof(&proof).await?;
            println!("   💾 Proof stored in Git notes");
        }

        println!("   ✅ Contract validation complete");
        Ok(())
    }

    /// Verify an existing contract proof
    async fn verify_contract(&self, file_path: &str, strict: bool) -> Result<()> {
        println!("🔍 Verifying contract: {file_path}");

        // Get existing proof
        let existing_proof = match self.get_existing_proof(file_path).await? {
            Some(proof) => proof,
            None => {
                if strict {
                    anyhow::bail!("No existing proof found for {}", file_path);
                } else {
                    println!("   ⚠️  No existing proof found");
                    return Ok(());
                }
            }
        };

        // Generate current proof
        let current_proof = self
            .generate_proof_internal(file_path, &existing_proof.contract_type, true, true)
            .await?;

        // Compare proofs
        if self.verify_proofs(&current_proof, &existing_proof)? {
            println!("   ✅ Proof verification successful");
        } else if strict {
            anyhow::bail!("   ❌ Proof verification failed - file has changed");
        } else {
            println!("   ⚠️  Proof verification failed - file has changed");
        }

        Ok(())
    }

    /// Generate a contract proof
    async fn generate_proof(&self, file_path: &str, contract_type: &str) -> Result<()> {
        println!("🔍 Generating proof for: {file_path}");

        let proof = self
            .generate_proof_internal(file_path, contract_type, true, true)
            .await?;

        println!("   📄 Blob Hash: {}", proof.blob_hash);
        println!("   🌳 AST Hash: {}", proof.ast_hash);
        println!("   📋 Schema Hash: {}", proof.schema_hash);
        println!("   ✅ Proof generation complete");

        Ok(())
    }

    /// List all contract proofs
    async fn list_proofs(&self, detailed: bool) -> Result<()> {
        println!("📋 Listing contract proofs...");

        let proofs = self.get_all_proofs().await?;

        if proofs.is_empty() {
            println!("   No proofs found");
            return Ok(());
        }

        let proofs_count = proofs.len();
        for (file_path, proof) in proofs {
            if detailed {
                println!("   📄 {file_path}");
                println!("      Type: {}", proof.contract_type);
                println!("      Blob Hash: {}", proof.blob_hash);
                println!("      AST Hash: {}", proof.ast_hash);
                println!("      Schema Hash: {}", proof.schema_hash);
                println!("      Validated: {}", proof.validated_at);
                println!("      By: {}", proof.validated_by);
                println!();
            } else {
                println!("   📄 {} ({})", file_path, proof.contract_type);
            }
        }

        println!("   📊 Total proofs: {proofs_count}");
        Ok(())
    }

    /// Clean up old contract proofs
    async fn cleanup_proofs(&self, days: u32, dry_run: bool) -> Result<()> {
        println!("🧹 Cleaning up contract proofs older than {days} days...");

        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let proofs = self.get_all_proofs().await?;

        let mut to_delete = Vec::new();

        for (file_path, proof) in proofs {
            if let Ok(validated_at) = DateTime::parse_from_rfc3339(&proof.validated_at) {
                if validated_at.naive_utc() < cutoff.naive_utc() {
                    to_delete.push(file_path);
                }
            }
        }

        if to_delete.is_empty() {
            println!("   ✅ No old proofs to clean up");
            return Ok(());
        }

        println!("   📋 Found {} old proofs to delete", to_delete.len());

        if dry_run {
            for file_path in &to_delete {
                println!("   🗑️  Would delete: {file_path}");
            }
        } else {
            for file_path in &to_delete {
                self.delete_proof(file_path).await?;
                println!("   🗑️  Deleted: {file_path}");
            }
        }

        Ok(())
    }

    /// Generate proof internally
    async fn generate_proof_internal(
        &self,
        file_path: &str,
        contract_type: &str,
        validate_ast: bool,
        validate_schema: bool,
    ) -> Result<ContractProof> {
        let path = Path::new(file_path);
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {file_path}"))?;

        // Generate blob hash
        let blob_hash = self.generate_blob_hash(&content);

        // Generate AST hash if requested
        let ast_hash = if validate_ast && file_path.ends_with(".rs") {
            self.generate_ast_hash(&content).await?
        } else {
            "no_ast_validation".to_string()
        };

        // Generate schema hash if requested
        let schema_hash = if validate_schema {
            self.generate_schema_hash(file_path, contract_type).await?
        } else {
            "no_schema_validation".to_string()
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            "file_size".to_string(),
            serde_json::Value::Number(content.len().into()),
        );
        metadata.insert(
            "validate_ast".to_string(),
            serde_json::Value::Bool(validate_ast),
        );
        metadata.insert(
            "validate_schema".to_string(),
            serde_json::Value::Bool(validate_schema),
        );

        Ok(ContractProof {
            file_path: file_path.to_string(),
            blob_hash,
            ast_hash,
            schema_hash,
            contract_type: contract_type.to_string(),
            validated_at: Utc::now().to_rfc3339(),
            validated_by: format!("xtask-contract-validate {}", env!("CARGO_PKG_VERSION")),
            metadata,
        })
    }

    /// Generate blob hash
    fn generate_blob_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("sha256:{}", hex::encode(result))
    }

    /// Generate AST hash for Rust files
    async fn generate_ast_hash(&self, content: &str) -> Result<String> {
        // Use rustc to parse and get AST
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(&temp_file, content)?;

        let output = Command::new("rustc")
            .arg("--emit=metadata")
            .arg("--crate-type=lib")
            .arg(temp_file.path())
            .output()
            .context("Failed to run rustc")?;

        if !output.status.success() {
            // If rustc fails, fall back to content hash
            let mut hasher = Sha256::new();
            hasher.update(content.as_bytes());
            let result = hasher.finalize();
            return Ok(format!("sha256:{}", hex::encode(result)));
        }

        // Hash the metadata output
        let mut hasher = Sha256::new();
        hasher.update(&output.stdout);
        let result = hasher.finalize();
        Ok(format!("sha256:{}", hex::encode(result)))
    }

    /// Generate schema hash
    async fn generate_schema_hash(&self, file_path: &str, contract_type: &str) -> Result<String> {
        // For now, generate a hash based on file path and contract type
        // In a full implementation, this would extract and hash the actual schema
        let mut hasher = Sha256::new();
        hasher.update(file_path.as_bytes());
        hasher.update(contract_type.as_bytes());
        let result = hasher.finalize();
        Ok(format!("sha256:{}", hex::encode(result)))
    }

    /// Get existing proof from Git notes
    async fn get_existing_proof(&self, _file_path: &str) -> Result<Option<ContractProof>> {
        // This would read from Git notes in a full implementation
        // For now, return None to indicate no existing proof
        Ok(None)
    }

    /// Store proof in Git notes
    async fn store_proof(&self, proof: &ContractProof) -> Result<()> {
        // Convert proof to contract state note
        let mut metadata = HashMap::new();
        metadata.insert(
            "blob_hash".to_string(),
            serde_json::Value::String(proof.blob_hash.clone()),
        );
        metadata.insert(
            "ast_hash".to_string(),
            serde_json::Value::String(proof.ast_hash.clone()),
        );
        metadata.insert(
            "schema_hash".to_string(),
            serde_json::Value::String(proof.schema_hash.clone()),
        );

        let state_note = ContractStateNote {
            file: proof.file_path.clone(),
            contract: proof.contract_type.clone(),
            state: "VALIDATED".to_string(),
            hash: proof.blob_hash.clone(),
            validated_by: proof.validated_by.clone(),
            timestamp: proof.validated_at.clone(),
            parent_scope: None,
            parent_hash: None,
            metadata: Some(metadata),
        };

        self.notes_manager.store_contract_state(&state_note)?;

        // Store transition log
        let transition = TransitionLogEntry {
            transition: "validate_contract".to_string(),
            from: "UNVALIDATED".to_string(),
            to: "VALIDATED".to_string(),
            file: proof.file_path.clone(),
            hash: proof.blob_hash.clone(),
            tool: proof.validated_by.clone(),
            timestamp: proof.validated_at.clone(),
            reason: Some("Contract validation completed".to_string()),
            commit_hash: None,
            user: None,
            environment: None,
            metadata: Some(proof.metadata.clone()),
        };

        self.notes_manager.store_transition_log(&transition)?;
        Ok(())
    }

    /// Verify two proofs match
    fn verify_proofs(&self, current: &ContractProof, existing: &ContractProof) -> Result<bool> {
        Ok(current.blob_hash == existing.blob_hash
            && current.ast_hash == existing.ast_hash
            && current.schema_hash == existing.schema_hash)
    }

    /// Get all proofs
    async fn get_all_proofs(&self) -> Result<HashMap<String, ContractProof>> {
        // This would read all proofs from Git notes
        // For now, return empty map
        Ok(HashMap::new())
    }

    /// Delete a proof
    async fn delete_proof(&self, file_path: &str) -> Result<()> {
        self.notes_manager.delete_contract_state(file_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_contract_validator_creation() {
        let validator = ContractValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_blob_hash_generation() {
        let validator = ContractValidator::new().unwrap();
        let content = "fn main() { println!(\"Hello, world!\"); }";
        let hash = validator.generate_blob_hash(content);
        assert!(hash.starts_with("sha256:"));
        assert_eq!(hash.len(), 71); // sha256: + 64 hex chars
    }
}
