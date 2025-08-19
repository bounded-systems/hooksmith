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
        let repo = Repository::open(repo_path)
            .context("Failed to open Git repository")?;
        
        Ok(ScopeRefManager {
            repo,
            namespace: "refs/hooksmith/scopes".to_string(),
        })
    }

    pub fn with_namespace(repo_path: &str, namespace: &str) -> Result<Self> {
        let repo = Repository::open(repo_path)
            .context("Failed to open Git repository")?;
        
        Ok(ScopeRefManager {
            repo,
            namespace: namespace.to_string(),
        })
    }

    /// Create or update a scope ref to point to a specific commit
    pub fn set_scope_ref(&self, scope_name: &str, commit_sha: &str) -> Result<()> {
        let ref_name = format!("{}/{}", self.namespace, scope_name);
        let oid = Oid::from_str(commit_sha)
            .context(format!("Invalid commit SHA: {}", commit_sha))?;
        
        self.repo.reference(&ref_name, oid, true, "Update scope ref")
            .context(format!("Failed to create/update ref: {}", ref_name))?;
        
        println!("✅ Set scope ref {} -> {}", ref_name, commit_sha);
        Ok(())
    }

    /// Get the commit SHA that a scope ref points to
    pub fn get_scope_ref(&self, scope_name: &str) -> Result<Option<String>> {
        let ref_name = format!("{}/{}", self.namespace, scope_name);
        
        match self.repo.find_reference(&ref_name) {
            Ok(reference) => {
                let commit_sha = reference.target()
                    .context("Reference has no target")?
                    .to_string();
                Ok(Some(commit_sha))
            }
            Err(git2::Error::from_str("reference not found")) => {
                Ok(None)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to find reference {}: {}", ref_name, e)),
        }
    }

    /// Get the tree SHA for a scope ref
    pub fn get_scope_tree_sha(&self, scope_name: &str) -> Result<Option<String>> {
        if let Some(commit_sha) = self.get_scope_ref(scope_name)? {
            let oid = Oid::from_str(&commit_sha)
                .context(format!("Invalid commit SHA: {}", commit_sha))?;
            
            let commit = self.repo.find_commit(oid)
                .context(format!("Failed to find commit: {}", commit_sha))?;
            
            let tree = commit.tree()
                .context("Failed to get commit tree")?;
            
            Ok(Some(tree.id().to_string()))
        } else {
            Ok(None)
        }
    }

    /// List all scope refs
    pub fn list_scope_refs(&self) -> Result<Vec<ScopeRef>> {
        let mut scope_refs = Vec::new();
        
        // Get all references in the hooksmith namespace
        let references = self.repo.references_glob(&format!("{}/*", self.namespace))
            .context("Failed to get scope references")?;
        
        for reference in references {
            let reference = reference?;
            let ref_name = reference.name()
                .context("Reference has no name")?;
            
            // Extract scope name from full ref path
            let scope_name = ref_name
                .strip_prefix(&format!("{}/", self.namespace))
                .unwrap_or(ref_name)
                .to_string();
            
            let commit_sha = reference.target()
                .context("Reference has no target")?
                .to_string();
            
            // Get tree SHA
            let tree_sha = if let Some(tree_sha) = self.get_scope_tree_sha(&scope_name)? {
                tree_sha
            } else {
                continue; // Skip invalid refs
            };
            
            // Try to get metadata from Git notes
            let metadata = self.get_scope_metadata(&scope_name)?;
            
            scope_refs.push(ScopeRef {
                name: scope_name,
                commit_sha,
                tree_sha,
                last_validated: metadata.get("last_validated").and_then(|v| v.as_str()).map(|s| s.to_string()),
                contract_ids: metadata.get("contract_ids")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
                    .unwrap_or_default(),
                stability_level: metadata.get("stability_level").and_then(|v| v.as_str()).map(|s| s.to_string()),
            });
        }
        
        Ok(scope_refs)
    }

    /// Delete a scope ref
    pub fn delete_scope_ref(&self, scope_name: &str) -> Result<()> {
        let ref_name = format!("{}/{}", self.namespace, scope_name);
        
        match self.repo.find_reference(&ref_name) {
            Ok(reference) => {
                reference.delete()
                    .context(format!("Failed to delete ref: {}", ref_name))?;
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

    /// Store metadata for a scope ref using Git notes
    pub fn set_scope_metadata(&self, scope_name: &str, metadata: &Value) -> Result<()> {
        if let Some(commit_sha) = self.get_scope_ref(scope_name)? {
            let oid = Oid::from_str(&commit_sha)
                .context(format!("Invalid commit SHA: {}", commit_sha))?;
            
            let note_ref = format!("refs/notes/hooksmith-scopes");
            let note_message = serde_json::to_string_pretty(metadata)?;
            
            // Create or update the note
            let signature = self.repo.signature()
                .context("Failed to get signature")?;
            
            let note_oid = self.repo.note(&note_ref, oid, &signature, &signature, &note_message)
                .context("Failed to create/update note")?;
            
            println!("📝 Set metadata for scope {}: {}", scope_name, note_oid);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Scope ref not found: {}", scope_name))
        }
    }

    /// Get metadata for a scope ref from Git notes
    pub fn get_scope_metadata(&self, scope_name: &str) -> Result<Value> {
        if let Some(commit_sha) = self.get_scope_ref(scope_name)? {
            let oid = Oid::from_str(&commit_sha)
                .context(format!("Invalid commit SHA: {}", commit_sha))?;
            
            let note_ref = format!("refs/notes/hooksmith-scopes");
            
            match self.repo.find_note(&note_ref, oid) {
                Ok(note) => {
                    let message = note.message()
                        .context("Note has no message")?;
                    serde_json::from_str(message)
                        .context("Failed to parse note as JSON")
                }
                Err(git2::Error::from_str("note not found")) => {
                    Ok(json!({}))
                }
                Err(e) => Err(anyhow::anyhow!("Failed to find note: {}", e)),
            }
        } else {
            Ok(json!({}))
        }
    }

    /// Update scope ref after successful validation
    pub fn update_scope_after_validation(
        &self,
        scope_name: &str,
        commit_sha: &str,
        contract_ids: &[String],
        stability_level: Option<&str>,
    ) -> Result<()> {
        // Update the ref
        self.set_scope_ref(scope_name, commit_sha)?;
        
        // Update metadata
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

    /// Check if a scope needs validation (has changed since last validation)
    pub fn scope_needs_validation(&self, scope_name: &str, current_commit_sha: &str) -> Result<bool> {
        if let Some(last_commit_sha) = self.get_scope_ref(scope_name)? {
            // Check if the commit has changed
            if last_commit_sha != current_commit_sha {
                println!("🔄 Scope {} needs validation: {} -> {}", 
                    scope_name, last_commit_sha, current_commit_sha);
                return Ok(true);
            }
            
            // Check if metadata indicates validation is needed
            let metadata = self.get_scope_metadata(scope_name)?;
            if let Some(status) = metadata.get("validation_status").and_then(|v| v.as_str()) {
                if status == "failed" {
                    println!("⚠️  Scope {} needs re-validation (previous validation failed)", scope_name);
                    return Ok(true);
                }
            }
            
            println!("✅ Scope {} is up to date", scope_name);
            Ok(false)
        } else {
            println!("🆕 Scope {} needs initial validation", scope_name);
            Ok(true)
        }
    }

    /// Get cache key for a scope based on its ref
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

    /// Initialize scope refs for common project scopes
    pub fn initialize_project_scopes(&self, base_commit_sha: &str) -> Result<()> {
        let common_scopes = vec![
            ("project-root", vec!["object-names@v1"]),
            ("crates", vec!["crate-structure@v1"]),
            ("docs", vec!["documentation@v1"]),
            ("src", vec!["source-structure@v1"]),
            ("tests", vec!["test-structure@v1"]),
            ("scripts", vec!["script-structure@v1"]),
        ];
        
        for (scope_name, contract_ids) in common_scopes {
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

    /// Export scope refs as JSON for external tools
    pub fn export_scope_refs(&self) -> Result<Value> {
        let scope_refs = self.list_scope_refs()?;
        
        let mut export = json!({
            "namespace": self.namespace,
            "exported_at": chrono::Utc::now().to_rfc3339(),
            "scopes": {}
        });
        
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

    /// Import scope refs from JSON (useful for migration)
    pub fn import_scope_refs(&self, import_data: &Value) -> Result<()> {
        if let Some(scopes) = import_data.get("scopes").and_then(|v| v.as_object()) {
            for (scope_name, scope_data) in scopes {
                if let (Some(commit_sha), Some(contract_ids)) = (
                    scope_data.get("commit_sha").and_then(|v| v.as_str()),
                    scope_data.get("contract_ids").and_then(|v| v.as_array())
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

// Integration with existing Hooksmith pipeline
pub struct HooksmithScopeManager {
    ref_manager: ScopeRefManager,
}

impl HooksmithScopeManager {
    pub fn new(repo_path: &str) -> Result<Self> {
        let ref_manager = ScopeRefManager::new(repo_path)?;
        Ok(HooksmithScopeManager { ref_manager })
    }

    /// Get validation scopes that need checking
    pub fn get_scopes_needing_validation(&self, current_commit_sha: &str) -> Result<Vec<String>> {
        let scope_refs = self.ref_manager.list_scope_refs()?;
        let mut scopes_needing_validation = Vec::new();
        
        for scope_ref in scope_refs {
            if self.ref_manager.scope_needs_validation(&scope_ref.name, current_commit_sha)? {
                scopes_needing_validation.push(scope_ref.name);
            }
        }
        
        Ok(scopes_needing_validation)
    }

    /// Update scope after successful validation
    pub fn mark_scope_validated(
        &self,
        scope_name: &str,
        commit_sha: &str,
        contract_ids: &[String],
        stability_level: Option<&str>,
    ) -> Result<()> {
        self.ref_manager.update_scope_after_validation(
            scope_name, commit_sha, contract_ids, stability_level
        )
    }

    /// Get cache key for instant cache lookup
    pub fn get_cache_key(&self, scope_name: &str, contract_id: &str, fix_hash: &str) -> Result<Option<String>> {
        self.ref_manager.get_scope_cache_key(scope_name, contract_id, fix_hash)
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: cargo run --bin scope_ref_manager <command> [args...]");
        println!();
        println!("Commands:");
        println!("  set <scope> <commit>                    - Set scope ref to commit");
        println!("  get <scope>                             - Get scope ref commit SHA");
        println!("  list                                    - List all scope refs");
        println!("  delete <scope>                          - Delete scope ref");
        println!("  metadata <scope>                        - Get scope metadata");
        println!("  set-metadata <scope> <json>             - Set scope metadata");
        println!("  needs-validation <scope> <commit>       - Check if scope needs validation");
        println!("  mark-validated <scope> <commit> <contracts> - Mark scope as validated");
        println!("  init <base-commit>                      - Initialize project scopes");
        println!("  export                                  - Export scope refs as JSON");
        println!("  import <json-file>                      - Import scope refs from JSON");
        std::process::exit(1);
    }
    
    let command = &args[1];
    let repo_path = "."; // Current directory
    
    let ref_manager = ScopeRefManager::new(repo_path)?;
    
    match command.as_str() {
        "set" => {
            if args.len() < 4 {
                eprintln!("Error: set command requires scope and commit");
                std::process::exit(1);
            }
            let scope = &args[2];
            let commit = &args[3];
            ref_manager.set_scope_ref(scope, commit)?;
        }
        "get" => {
            if args.len() < 3 {
                eprintln!("Error: get command requires scope");
                std::process::exit(1);
            }
            let scope = &args[2];
            if let Some(commit_sha) = ref_manager.get_scope_ref(scope)? {
                println!("{}", commit_sha);
            } else {
                println!("Scope ref not found: {}", scope);
            }
        }
        "list" => {
            let scope_refs = ref_manager.list_scope_refs()?;
            println!("{}", serde_json::to_string_pretty(&scope_refs)?);
        }
        "delete" => {
            if args.len() < 3 {
                eprintln!("Error: delete command requires scope");
                std::process::exit(1);
            }
            let scope = &args[2];
            ref_manager.delete_scope_ref(scope)?;
        }
        "metadata" => {
            if args.len() < 3 {
                eprintln!("Error: metadata command requires scope");
                std::process::exit(1);
            }
            let scope = &args[2];
            let metadata = ref_manager.get_scope_metadata(scope)?;
            println!("{}", serde_json::to_string_pretty(&metadata)?);
        }
        "set-metadata" => {
            if args.len() < 4 {
                eprintln!("Error: set-metadata command requires scope and JSON");
                std::process::exit(1);
            }
            let scope = &args[2];
            let json_str = &args[3];
            let metadata: Value = serde_json::from_str(json_str)?;
            ref_manager.set_scope_metadata(scope, &metadata)?;
        }
        "needs-validation" => {
            if args.len() < 4 {
                eprintln!("Error: needs-validation command requires scope and commit");
                std::process::exit(1);
            }
            let scope = &args[2];
            let commit = &args[3];
            let needs_validation = ref_manager.scope_needs_validation(scope, commit)?;
            println!("{}", needs_validation);
        }
        "mark-validated" => {
            if args.len() < 5 {
                eprintln!("Error: mark-validated command requires scope, commit, and contracts");
                std::process::exit(1);
            }
            let scope = &args[2];
            let commit = &args[3];
            let contracts: Vec<String> = serde_json::from_str(&args[4])?;
            ref_manager.update_scope_after_validation(scope, commit, &contracts, None)?;
        }
        "init" => {
            if args.len() < 3 {
                eprintln!("Error: init command requires base commit");
                std::process::exit(1);
            }
            let base_commit = &args[2];
            ref_manager.initialize_project_scopes(base_commit)?;
        }
        "export" => {
            let export = ref_manager.export_scope_refs()?;
            println!("{}", serde_json::to_string_pretty(&export)?);
        }
        "import" => {
            if args.len() < 3 {
                eprintln!("Error: import command requires JSON file");
                std::process::exit(1);
            }
            let json_file = &args[2];
            let import_data: Value = serde_json::from_str(&std::fs::read_to_string(json_file)?)?;
            ref_manager.import_scope_refs(&import_data)?;
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }
    
    Ok(())
}
