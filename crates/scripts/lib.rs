// Shared types for the validate-object-names-contract script suite.
// Extracted here so scope_aware_contract_pipeline can import ScopeRefManager
// and HooksmithScopeManager without cross-binary module imports (which Rust
// does not allow — each [[bin]] target is a separate crate root).
pub use scope_types::*;

mod scope_types {
    use anyhow::{Context, Result};
    use git2::{Oid, Repository};
    use serde_json::{json, Value};

    #[derive(Debug, Clone, serde::Serialize)]
    pub struct ScopeRef {
        pub name: String,
        pub commit_sha: String,
        pub tree_sha: String,
        pub last_validated: Option<String>,
        pub contract_ids: Vec<String>,
        pub stability_level: Option<String>,
    }

    pub struct ScopeRefManager {
        repo: Repository,
        namespace: String,
    }

    impl ScopeRefManager {
        pub fn new(repo_path: &str) -> Result<Self> {
            let repo = Repository::open(repo_path).context("Failed to open Git repository")?;
            Ok(ScopeRefManager { repo, namespace: "refs/hooksmith/scopes".to_string() })
        }

        pub fn with_namespace(repo_path: &str, namespace: &str) -> Result<Self> {
            let repo = Repository::open(repo_path).context("Failed to open Git repository")?;
            Ok(ScopeRefManager { repo, namespace: namespace.to_string() })
        }

        pub fn set_scope_ref(&self, scope_name: &str, commit_sha: &str) -> Result<()> {
            let ref_name = format!("{}/{}", self.namespace, scope_name);
            let oid = Oid::from_str(commit_sha).context(format!("Invalid commit SHA: {}", commit_sha))?;
            self.repo.reference(&ref_name, oid, true, "Update scope ref")
                .context(format!("Failed to create/update ref: {}", ref_name))?;
            println!("✅ Set scope ref {} -> {}", ref_name, commit_sha);
            Ok(())
        }

        pub fn get_scope_ref(&self, scope_name: &str) -> Result<Option<String>> {
            let ref_name = format!("{}/{}", self.namespace, scope_name);
            match self.repo.find_reference(&ref_name) {
                Ok(reference) => {
                    let commit_sha = reference.target().context("Reference has no target")?.to_string();
                    Ok(Some(commit_sha))
                }
                Err(git2::Error::from_str("reference not found")) => Ok(None),
                Err(e) => Err(anyhow::anyhow!("Failed to find reference {}: {}", ref_name, e)),
            }
        }

        pub fn get_scope_tree_sha(&self, scope_name: &str) -> Result<Option<String>> {
            if let Some(commit_sha) = self.get_scope_ref(scope_name)? {
                let oid = Oid::from_str(&commit_sha).context(format!("Invalid commit SHA: {}", commit_sha))?;
                let commit = self.repo.find_commit(oid).context(format!("Failed to find commit: {}", commit_sha))?;
                let tree = commit.tree().context("Failed to get commit tree")?;
                Ok(Some(tree.id().to_string()))
            } else {
                Ok(None)
            }
        }

        pub fn list_scope_refs(&self) -> Result<Vec<ScopeRef>> {
            let mut scope_refs = Vec::new();
            let references = self.repo.references_glob(&format!("{}/*", self.namespace))
                .context("Failed to get scope references")?;
            for reference in references {
                let reference = reference?;
                let ref_name = reference.name().context("Reference has no name")?;
                let scope_name = ref_name.strip_prefix(&format!("{}/", self.namespace))
                    .unwrap_or(ref_name).to_string();
                let commit_sha = reference.target().context("Reference has no target")?.to_string();
                let tree_sha = if let Some(ts) = self.get_scope_tree_sha(&scope_name)? { ts } else { continue; };
                let metadata = self.get_scope_metadata(&scope_name)?;
                scope_refs.push(ScopeRef {
                    name: scope_name,
                    commit_sha,
                    tree_sha,
                    last_validated: metadata.get("last_validated").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    contract_ids: metadata.get("contract_ids").and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                        .unwrap_or_default(),
                    stability_level: metadata.get("stability_level").and_then(|v| v.as_str()).map(|s| s.to_string()),
                });
            }
            Ok(scope_refs)
        }

        pub fn delete_scope_ref(&self, scope_name: &str) -> Result<()> {
            let ref_name = format!("{}/{}", self.namespace, scope_name);
            match self.repo.find_reference(&ref_name) {
                Ok(reference) => {
                    reference.delete().context(format!("Failed to delete ref: {}", ref_name))?;
                    println!("🗑️  Deleted scope ref: {}", ref_name);
                    Ok(())
                }
                Err(git2::Error::from_str("reference not found")) => {
                    println!("⚠️  Scope ref not found: {}", ref_name);
                    Ok(())
                }
                Err(e) => Err(anyhow::anyhow!("Failed to find reference {}: {}", ref_name, e)),
            }
        }

        pub fn set_scope_metadata(&self, scope_name: &str, metadata: &Value) -> Result<()> {
            if let Some(commit_sha) = self.get_scope_ref(scope_name)? {
                let oid = Oid::from_str(&commit_sha).context(format!("Invalid commit SHA: {}", commit_sha))?;
                let note_ref = "refs/notes/hooksmith-scopes".to_string();
                let note_message = serde_json::to_string_pretty(metadata)?;
                let signature = self.repo.signature().context("Failed to get signature")?;
                let note_oid = self.repo.note(&note_ref, oid, &signature, &signature, &note_message)
                    .context("Failed to create/update note")?;
                println!("📝 Set metadata for scope {}: {}", scope_name, note_oid);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Scope ref not found: {}", scope_name))
            }
        }

        pub fn get_scope_metadata(&self, scope_name: &str) -> Result<Value> {
            if let Some(commit_sha) = self.get_scope_ref(scope_name)? {
                let oid = Oid::from_str(&commit_sha).context(format!("Invalid commit SHA: {}", commit_sha))?;
                let note_ref = "refs/notes/hooksmith-scopes".to_string();
                match self.repo.find_note(&note_ref, oid) {
                    Ok(note) => {
                        let message = note.message().context("Note has no message")?;
                        serde_json::from_str(message).context("Failed to parse note as JSON")
                    }
                    Err(git2::Error::from_str("note not found")) => Ok(json!({})),
                    Err(e) => Err(anyhow::anyhow!("Failed to find note: {}", e)),
                }
            } else {
                Ok(json!({}))
            }
        }

        pub fn update_scope_after_validation(&self, scope_name: &str, commit_sha: &str, contract_ids: &[String], stability_level: Option<&str>) -> Result<()> {
            self.set_scope_ref(scope_name, commit_sha)?;
            let metadata = json!({
                "last_validated": chrono::Utc::now().to_rfc3339(),
                "contract_ids": contract_ids,
                "stability_level": stability_level,
                "validation_status": "passed"
            });
            self.set_scope_metadata(scope_name, &metadata)?;
            println!("✅ Updated scope {} after successful validation", scope_name);
            Ok(())
        }

        pub fn scope_needs_validation(&self, scope_name: &str, current_commit_sha: &str) -> Result<bool> {
            if let Some(last_commit_sha) = self.get_scope_ref(scope_name)? {
                if last_commit_sha != current_commit_sha {
                    return Ok(true);
                }
                let metadata = self.get_scope_metadata(scope_name)?;
                if let Some(status) = metadata.get("validation_status").and_then(|v| v.as_str()) {
                    if status == "failed" { return Ok(true); }
                }
                Ok(false)
            } else {
                Ok(true)
            }
        }

        pub fn get_scope_cache_key(&self, scope_name: &str, contract_id: &str, fix_hash: &str) -> Result<Option<String>> {
            if let Some(tree_sha) = self.get_scope_tree_sha(scope_name)? {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(format!("{}:{}:{}", tree_sha, contract_id, fix_hash).as_bytes());
                Ok(Some(format!("{:x}", hasher.finalize())))
            } else {
                Ok(None)
            }
        }

        pub fn initialize_project_scopes(&self, base_commit_sha: &str) -> Result<()> {
            let common_scopes = vec![
                ("project-root", vec!["object-names@v1"]),
                ("crates", vec!["crate-structure@v1"]),
                ("docs", vec!["documentation@v1"]),
                ("src", vec!["source-structure@v1"]),
                ("tests", vec!["test-structure@v1"]),
                ("scripts", vec!["script-structure@v1"]),
            ];
            for (scope_name, contract_ids) in &common_scopes {
                self.set_scope_ref(scope_name, base_commit_sha)?;
                let metadata = json!({
                    "contract_ids": contract_ids,
                    "created": chrono::Utc::now().to_rfc3339(),
                    "validation_status": "pending"
                });
                self.set_scope_metadata(scope_name, &metadata)?;
            }
            println!("🎯 Initialized {} project scopes", common_scopes.len());
            Ok(())
        }

        pub fn export_scope_refs(&self) -> Result<Value> {
            let scope_refs = self.list_scope_refs()?;
            let mut export = json!({ "namespace": self.namespace, "exported_at": chrono::Utc::now().to_rfc3339(), "scopes": {} });
            for scope_ref in scope_refs {
                export["scopes"][&scope_ref.name] = json!({
                    "commit_sha": scope_ref.commit_sha,
                    "tree_sha": scope_ref.tree_sha,
                    "last_validated": scope_ref.last_validated,
                    "contract_ids": scope_ref.contract_ids,
                    "stability_level": scope_ref.stability_level
                });
            }
            Ok(export)
        }

        pub fn import_scope_refs(&self, import_data: &Value) -> Result<()> {
            if let Some(scopes) = import_data.get("scopes").and_then(|v| v.as_object()) {
                for (scope_name, scope_data) in scopes {
                    if let (Some(commit_sha), Some(contract_ids)) = (
                        scope_data.get("commit_sha").and_then(|v| v.as_str()),
                        scope_data.get("contract_ids").and_then(|v| v.as_array()),
                    ) {
                        self.set_scope_ref(scope_name, commit_sha)?;
                        let metadata = json!({
                            "contract_ids": contract_ids,
                            "imported_at": chrono::Utc::now().to_rfc3339(),
                            "validation_status": "pending"
                        });
                        self.set_scope_metadata(scope_name, &metadata)?;
                    }
                }
                println!("📥 Imported {} scope refs", scopes.len());
            }
            Ok(())
        }
    }

    pub struct HooksmithScopeManager {
        ref_manager: ScopeRefManager,
    }

    impl HooksmithScopeManager {
        pub fn new(repo_path: &str) -> Result<Self> {
            let ref_manager = ScopeRefManager::new(repo_path)?;
            Ok(HooksmithScopeManager { ref_manager })
        }

        pub fn get_scopes_needing_validation(&self, current_commit_sha: &str) -> Result<Vec<String>> {
            let scope_refs = self.ref_manager.list_scope_refs()?;
            let mut out = Vec::new();
            for scope_ref in scope_refs {
                if self.ref_manager.scope_needs_validation(&scope_ref.name, current_commit_sha)? {
                    out.push(scope_ref.name);
                }
            }
            Ok(out)
        }

        pub fn mark_scope_validated(&self, scope_name: &str, commit_sha: &str, contract_ids: &[String], stability_level: Option<&str>) -> Result<()> {
            self.ref_manager.update_scope_after_validation(scope_name, commit_sha, contract_ids, stability_level)
        }

        pub fn get_cache_key(&self, scope_name: &str, contract_id: &str, fix_hash: &str) -> Result<Option<String>> {
            self.ref_manager.get_scope_cache_key(scope_name, contract_id, fix_hash)
        }
    }
}
