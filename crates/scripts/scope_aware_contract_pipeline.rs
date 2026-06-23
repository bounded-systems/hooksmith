use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use validate_object_names_contract::{HooksmithScopeManager, ScopeRefManager};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct ValidationScope {
    name: String,
    tree_sha: String,
    scope_type: String,
    contract_ids: Vec<String>,
    cache_key: String,
    path: String,
    entry_count: usize,
    needs_validation: bool,
}

#[derive(Debug)]
struct ValidationResult {
    scope: ValidationScope,
    success: bool,
    violations: Vec<String>,
    sarif: Value,
    fix_plan: Option<Value>,
    cache_hit: bool,
    execution_time_ms: u64,
    stability_metrics: Option<Value>,
}

#[derive(Debug)]
struct ScopeAwareContractValidator {
    cache_dir: String,
    scope_manager: HooksmithScopeManager,
    ref_manager: ScopeRefManager,
}

impl ScopeAwareContractValidator {
    fn new() -> Result<Self> {
        let cache_dir = ".contract_cache".to_string();
        fs::create_dir_all(&cache_dir)?;

        let scope_manager = HooksmithScopeManager::new(".")?;
        let ref_manager = ScopeRefManager::new(".")?;

        Ok(ScopeAwareContractValidator {
            cache_dir,
            scope_manager,
            ref_manager,
        })
    }

    fn compute_cache_key(&self, tree_sha: &str, contract_id: &str, fix_hash: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(format!("{}:{}:{}", tree_sha, contract_id, fix_hash).as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn get_cache_path(&self, cache_key: &str) -> String {
        format!("{}/{}.json", self.cache_dir, cache_key)
    }

    fn load_cached_result(&self, cache_key: &str) -> Option<Value> {
        let cache_path = self.get_cache_path(cache_key);
        if let Ok(content) = fs::read_to_string(cache_path) {
            if let Ok(cached) = serde_json::from_str::<Value>(&content) {
                // Check TTL (24 hours)
                if let Some(timestamp) = cached["timestamp"].as_u64() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    if now - timestamp < 86400 {
                        return Some(cached);
                    }
                }
            }
        }
        None
    }

    fn save_cached_result(&self, cache_key: &str, result: &Value) -> Result<()> {
        let cache_path = self.get_cache_path(cache_key);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cached_result = json!({
            "timestamp": timestamp,
            "result": result
        });

        fs::write(cache_path, serde_json::to_string_pretty(&cached_result)?)?;
        Ok(())
    }

    fn detect_scopes(&self, current_commit_sha: &str) -> Result<Vec<ValidationScope>> {
        println!("🔍 Detecting validation scopes using scope refs...");

        // Get scopes that need validation
        let scopes_needing_validation = self
            .scope_manager
            .get_scopes_needing_validation(current_commit_sha)?;

        // Get all scope refs
        let scope_refs = self.ref_manager.list_scope_refs()?;

        let mut validation_scopes = Vec::new();

        for scope_ref in scope_refs {
            let needs_validation = scopes_needing_validation.contains(&scope_ref.name);

            for contract_id in &scope_ref.contract_ids {
                let cache_key = self.compute_cache_key(&scope_ref.tree_sha, contract_id, "v1");

                let scope = ValidationScope {
                    name: scope_ref.name.clone(),
                    tree_sha: scope_ref.tree_sha.clone(),
                    scope_type: if scope_ref.name == "project-root" {
                        "root".to_string()
                    } else {
                        format!("subtree:{}", scope_ref.name)
                    },
                    contract_ids: vec![contract_id.clone()],
                    cache_key,
                    path: if scope_ref.name == "project-root" {
                        "".to_string()
                    } else {
                        scope_ref.name.clone()
                    },
                    entry_count: 0, // Will be filled during validation
                    needs_validation,
                };
                validation_scopes.push(scope);
            }
        }

        Ok(validation_scopes)
    }

    fn validate_scope(
        &self,
        scope: &ValidationScope,
        current_commit_sha: &str,
    ) -> Result<ValidationResult> {
        let start_time = SystemTime::now();

        // Check if scope needs validation
        if !scope.needs_validation {
            // Scope is up to date, return cached result
            return Ok(ValidationResult {
                scope: scope.clone(),
                success: true,
                violations: vec![],
                sarif: json!({}),
                fix_plan: None,
                cache_hit: true,
                execution_time_ms: start_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
                    as u64,
                stability_metrics: None,
            });
        }

        // Check cache first
        if let Some(cached) = self.load_cached_result(&scope.cache_key) {
            let cached_result = &cached["result"];
            return Ok(ValidationResult {
                scope: scope.clone(),
                success: cached_result["success"].as_bool().unwrap_or(false),
                violations: cached_result["violations"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect(),
                sarif: cached_result["sarif"].clone(),
                fix_plan: if cached_result["fix_plan"].is_null() {
                    None
                } else {
                    Some(cached_result["fix_plan"].clone())
                },
                cache_hit: true,
                execution_time_ms: start_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
                    as u64,
                stability_metrics: if cached_result["stability_metrics"].is_null() {
                    None
                } else {
                    Some(cached_result["stability_metrics"].clone())
                },
            });
        }

        // Get stability metrics for the scope
        let stability_metrics = if scope.scope_type == "root" {
            let output = Command::new("cargo")
                .args([
                    "run",
                    "--bin",
                    "git_object_walker",
                    "--manifest-path",
                    "scripts/Cargo.toml",
                    "stability",
                    "HEAD",
                    &scope.path,
                ])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    if let Ok(stability) =
                        serde_json::from_str::<Value>(&String::from_utf8_lossy(&output.stdout))
                    {
                        Some(stability)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Run validation
        let (success, violations, sarif, fix_plan) = match scope.scope_type.as_str() {
            "root" => self.validate_root_contract(scope)?,
            _ => self.validate_subtree_contract(scope)?,
        };

        let execution_time = SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_millis() as u64;

        let result = ValidationResult {
            scope: scope.clone(),
            success,
            violations: violations.clone(),
            sarif: sarif.clone(),
            fix_plan: fix_plan.clone(),
            cache_hit: false,
            execution_time_ms: execution_time,
            stability_metrics: stability_metrics.clone(),
        };

        // Cache the result
        let cache_value = json!({
            "success": success,
            "violations": violations,
            "sarif": sarif,
            "fix_plan": fix_plan,
            "execution_time_ms": execution_time,
            "stability_metrics": stability_metrics
        });
        self.save_cached_result(&scope.cache_key, &cache_value)?;

        // Update scope ref if validation passed
        if success {
            let stability_level = stability_metrics
                .as_ref()
                .and_then(|m| m["stability_level"].as_str());

            self.scope_manager.mark_scope_validated(
                &scope.name,
                current_commit_sha,
                &scope.contract_ids,
                stability_level,
            )?;
        }

        Ok(result)
    }

    fn validate_root_contract(
        &self,
        scope: &ValidationScope,
    ) -> Result<(bool, Vec<String>, Value, Option<Value>)> {
        // Use the existing object-names validation logic
        let contract_path = "contracts/object-names@v1.json";
        let contract_content =
            fs::read_to_string(contract_path).context("Failed to read contract file")?;

        let contract: Value =
            serde_json::from_str(&contract_content).context("Failed to parse contract JSON")?;

        let spec = &contract["spec"]["git"]["tree"]["objects"]["names"];
        let required: Vec<String> = spec["required"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        let allowed: Vec<String> = spec["allowed"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        let rejected_patterns: Vec<String> = spec["rejected"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        let ignored_patterns: Vec<String> = spec["ignored"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();

        let rejected = build_globs(&rejected_patterns)?;
        let ignored = build_globs(&ignored_patterns)?;
        let allowed_globs = build_globs(&allowed)?;

        // Get root tree entries using git ls-tree
        let output = Command::new("git")
            .args(["ls-tree", "--name-only", &scope.tree_sha])
            .output()
            .context("Failed to get root tree entries")?;

        if !output.status.success() {
            anyhow::bail!(
                "git ls-tree failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let root_entries: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        let mut violations = Vec::new();

        // Check required entries
        for req in &required {
            if !root_entries.iter().any(|entry| entry == req) {
                violations.push(format!("missing required: {}", req));
            }
        }

        // Check rejected entries (skip ignored)
        for entry in &root_entries {
            if ignored.is_match(entry) {
                continue;
            }
            if rejected.is_match(entry) {
                violations.push(format!("rejected at root: {}", entry));
            }
        }

        // Check allow-list (skip ignored)
        for entry in &root_entries {
            if ignored.is_match(entry) {
                continue;
            }
            if !allowed_globs.is_match(entry) {
                violations.push(format!("not in allowed set: {}", entry));
            }
        }

        let success = violations.is_empty();

        // Generate SARIF
        let sarif = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0-rtm.5.json",
            "runs": [{
                "tool": {
                    "driver": {
                        "name": "scope-aware-object-names-contract-validator",
                        "version": "1.0.0"
                    }
                },
                "results": violations.iter().map(|v| json!({
                    "level": "error",
                    "message": {
                        "text": v
                    },
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {
                                "uri": "root"
                            }
                        }
                    }]
                })).collect::<Vec<_>>()
            }]
        });

        // Generate fix plan
        let fix_plan = if !violations.is_empty() {
            Some(json!({
                "missing_required": required.iter().filter(|r| !root_entries.contains(r)).collect::<Vec<_>>(),
                "rejected_files": root_entries.iter().filter(|e| rejected.is_match(e)).collect::<Vec<_>>(),
                "suggested_moves": violations.iter().map(|v| format!("Fix: {}", v)).collect::<Vec<_>>()
            }))
        } else {
            None
        };

        Ok((success, violations, sarif, fix_plan))
    }

    fn validate_subtree_contract(
        &self,
        _scope: &ValidationScope,
    ) -> Result<(bool, Vec<String>, Value, Option<Value>)> {
        // Placeholder for subtree validation
        // This would implement specific contract validation for different subtrees
        Ok((true, vec![], json!({}), None))
    }

    fn run_pipeline(&self, current_commit_sha: &str) -> Result<Vec<ValidationResult>> {
        println!("🚀 Scope-Aware Contract Validation Pipeline");
        println!("Current commit: {}", current_commit_sha);
        println!();

        let scopes = self.detect_scopes(current_commit_sha)?;

        println!("📋 Found {} scopes to validate:", scopes.len());
        for scope in &scopes {
            let status = if scope.needs_validation {
                "🔄 needs validation"
            } else {
                "✅ up to date"
            };
            println!(
                "  - {} (contracts: {:?}) - {}",
                scope.name, scope.contract_ids, status
            );
        }
        println!();

        let mut results = Vec::new();
        let mut cache_hits = 0;
        let mut total_time = 0;
        let mut scopes_validated = 0;

        for scope in &scopes {
            println!("🔍 Validating {}...", scope.name);
            let result = self.validate_scope(&scope, current_commit_sha)?;

            if result.cache_hit {
                cache_hits += 1;
                println!("  ✅ Cache hit ({}ms)", result.execution_time_ms);
            } else {
                println!("  ⚡ Fresh validation ({}ms)", result.execution_time_ms);
                scopes_validated += 1;
            }

            total_time += result.execution_time_ms;

            if result.success {
                println!("  ✅ Validation passed");
            } else {
                println!(
                    "  ❌ Validation failed ({} violations)",
                    result.violations.len()
                );
                for violation in &result.violations {
                    println!("    - {}", violation);
                }
            }

            // Show stability metrics if available
            if let Some(stability) = &result.stability_metrics {
                if let Some(level) = stability["stability_level"].as_str() {
                    println!(
                        "  📊 Stability: {} (tree ratio: {:.2}, blob ratio: {:.2})",
                        level,
                        stability["tree_ratio"].as_f64().unwrap_or(0.0),
                        stability["blob_ratio"].as_f64().unwrap_or(0.0)
                    );
                }
            }
            println!();

            results.push(result);
        }

        println!("📊 Scope-Aware Pipeline Summary:");
        println!("  - Total scopes: {}", scopes.len());
        println!("  - Scopes validated: {}", scopes_validated);
        println!("  - Cache hits: {}", cache_hits);
        println!("  - Total execution time: {}ms", total_time);
        println!(
            "  - Failed validations: {}",
            results.iter().filter(|r| !r.success).count()
        );

        // Show scope ref status
        let scope_refs = self.ref_manager.list_scope_refs()?;
        println!("  - Scope refs: {} configured", scope_refs.len());

        Ok(results)
    }

    fn initialize_scopes(&self, base_commit_sha: &str) -> Result<()> {
        println!("🎯 Initializing project scopes...");
        self.ref_manager
            .initialize_project_scopes(base_commit_sha)?;
        println!("✅ Project scopes initialized successfully");
        Ok(())
    }

    fn export_scope_status(&self) -> Result<Value> {
        let scope_refs = self.ref_manager.list_scope_refs()?;
        let export = self.ref_manager.export_scope_refs()?;

        println!("📊 Scope Status Report:");
        for scope_ref in &scope_refs {
            let status = if scope_ref.last_validated.is_some() {
                "✅ validated"
            } else {
                "⏳ pending"
            };
            println!(
                "  - {}: {} ({})",
                scope_ref.name, status, scope_ref.commit_sha
            );
        }

        Ok(export)
    }
}

fn build_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: cargo run --bin scope_aware_contract_pipeline <command> [args...]");
        println!();
        println!("Commands:");
        println!("  validate [commit]                       - Validate all scopes");
        println!("  init <base-commit>                      - Initialize project scopes");
        println!("  status                                  - Show scope status");
        println!("  export                                  - Export scope status");
        std::process::exit(1);
    }

    let command = &args[1];

    let validator = ScopeAwareContractValidator::new()?;

    match command.as_str() {
        "validate" => {
            let current_commit = if args.len() > 2 {
                &args[2]
            } else {
                // Get current HEAD
                let output = Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .output()
                    .context("Failed to get current HEAD")?;

                if !output.status.success() {
                    anyhow::bail!(
                        "git rev-parse failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }

                String::from_utf8(output.stdout)?.trim().to_string()
            };

            let results = validator.run_pipeline(&current_commit)?;

            // Generate overall report
            let failed_results: Vec<_> = results.iter().filter(|r| !r.success).collect();

            if failed_results.is_empty() {
                println!("🎉 All contract validations passed!");
                std::process::exit(0);
            } else {
                println!(
                    "❌ Contract validation failed for {} scopes:",
                    failed_results.len()
                );
                for result in failed_results {
                    println!(
                        "  - {}: {} violations",
                        result.scope.name,
                        result.violations.len()
                    );
                }

                // Save SARIF report
                let sarif_path = "scope-aware-contract-validation-results.sarif";
                let sarif_results: Vec<Value> = results.iter().map(|r| r.sarif.clone()).collect();
                let sarif_report = json!({
                    "version": "2.1.0",
                    "$schema": "https://json.schemastore.org/sarif-2.1.0-rtm.5.json",
                    "runs": sarif_results
                });

                fs::write(sarif_path, serde_json::to_string_pretty(&sarif_report)?)?;
                println!("📄 Scope-aware SARIF report saved to: {}", sarif_path);

                std::process::exit(1);
            }
        }
        "init" => {
            if args.len() < 3 {
                eprintln!("Error: init command requires base commit");
                std::process::exit(1);
            }
            let base_commit = &args[2];
            validator.initialize_scopes(base_commit)?;
        }
        "status" => {
            validator.export_scope_status()?;
        }
        "export" => {
            let export = validator.export_scope_status()?;
            println!("{}", serde_json::to_string_pretty(&export)?);
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            std::process::exit(1);
        }
    }

    Ok(())
}
