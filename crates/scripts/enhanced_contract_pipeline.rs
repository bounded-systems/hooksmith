use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct ValidationScope {
    tree_sha: String,
    scope_type: String,
    contract_ids: Vec<String>,
    cache_key: String,
    path: String,
    entry_count: usize,
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
struct EnhancedContractValidator {
    cache_dir: String,
}

impl EnhancedContractValidator {
    fn new() -> Result<Self> {
        let cache_dir = ".contract_cache".to_string();
        fs::create_dir_all(&cache_dir)?;

        Ok(EnhancedContractValidator { cache_dir })
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

    fn detect_scopes(&self, base_ref: &str, head_ref: &str) -> Result<Vec<ValidationScope>> {
        println!("🔍 Detecting validation scopes using Git object walker...");

        // Use the git_object_walker binary to get validation scopes
        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "git_object_walker",
                "--manifest-path",
                "scripts/Cargo.toml",
                "scopes",
                base_ref,
                head_ref,
            ])
            .output()
            .context("Failed to run git_object_walker")?;

        if !output.status.success() {
            anyhow::bail!(
                "git_object_walker failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let scope_data: Vec<Value> = serde_json::from_str(&String::from_utf8(output.stdout)?)
            .context("Failed to parse scope data")?;

        let mut scopes = Vec::new();

        for scope_json in scope_data {
            let tree_sha = scope_json["tree_sha"].as_str().unwrap_or("").to_string();
            let scope_type = scope_json["scope_type"].as_str().unwrap_or("").to_string();
            let path = scope_json["path"].as_str().unwrap_or("").to_string();
            let entry_count = scope_json["entry_count"].as_u64().unwrap_or(0) as usize;

            let contract_ids: Vec<String> = scope_json["contract_ids"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| v.as_str().unwrap_or("").to_string())
                .collect();

            for contract_id in contract_ids {
                let cache_key = self.compute_cache_key(&tree_sha, &contract_id, "v1");

                let scope = ValidationScope {
                    tree_sha: tree_sha.clone(),
                    scope_type: scope_type.clone(),
                    contract_ids: vec![contract_id],
                    cache_key,
                    path: path.clone(),
                    entry_count,
                };
                scopes.push(scope);
            }
        }

        Ok(scopes)
    }

    fn validate_scope(&self, scope: &ValidationScope) -> Result<ValidationResult> {
        let start_time = SystemTime::now();

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
                        "name": "enhanced-object-names-contract-validator",
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

    fn run_pipeline(&self, base_ref: &str, head_ref: &str) -> Result<Vec<ValidationResult>> {
        println!("🚀 Enhanced Contract Validation Pipeline");
        println!("Base: {}", base_ref);
        println!("Head: {}", head_ref);
        println!();

        let scopes = self.detect_scopes(base_ref, head_ref)?;

        println!("📋 Found {} scopes to validate:", scopes.len());
        for scope in &scopes {
            println!(
                "  - {} (contracts: {:?}) - {} entries",
                scope.scope_type, scope.contract_ids, scope.entry_count
            );
        }
        println!();

        let mut results = Vec::new();
        let mut cache_hits = 0;
        let mut total_time = 0;

        for scope in &scopes {
            println!("🔍 Validating {}...", scope.scope_type);
            let result = self.validate_scope(&scope)?;

            if result.cache_hit {
                cache_hits += 1;
                println!("  ✅ Cache hit ({}ms)", result.execution_time_ms);
            } else {
                println!("  ⚡ Fresh validation ({}ms)", result.execution_time_ms);
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

        println!("📊 Enhanced Pipeline Summary:");
        println!("  - Total scopes: {}", scopes.len());
        println!("  - Cache hits: {}", cache_hits);
        println!("  - Total execution time: {}ms", total_time);
        println!(
            "  - Failed validations: {}",
            results.iter().filter(|r| !r.success).count()
        );

        // Show object graph statistics
        let output = Command::new("cargo")
            .args([
                "run",
                "--bin",
                "git_object_walker",
                "--manifest-path",
                "scripts/Cargo.toml",
                "analyze",
                head_ref,
            ])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                if let Ok(analysis) =
                    serde_json::from_str::<Value>(&String::from_utf8_lossy(&output.stdout))
                {
                    if let Some(total_objects) = analysis["total_objects"].as_u64() {
                        println!("  - Total objects in graph: {}", total_objects);
                    }
                    if let Some(root_stability) =
                        analysis["root_stability"]["stability_level"].as_str()
                    {
                        println!("  - Root tree stability: {}", root_stability);
                    }
                }
            }
        }

        Ok(results)
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

    if args.len() < 3 {
        println!("Usage: cargo run --bin enhanced_contract_pipeline <base_ref> <head_ref>");
        println!();
        println!("Examples:");
        println!("  cargo run --bin enhanced_contract_pipeline origin/main HEAD");
        println!("  cargo run --bin enhanced_contract_pipeline main feature-branch");
        std::process::exit(1);
    }

    let base_ref = &args[1];
    let head_ref = &args[2];

    let validator = EnhancedContractValidator::new()?;
    let results = validator.run_pipeline(base_ref, head_ref)?;

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
                result.scope.scope_type,
                result.violations.len()
            );
        }

        // Save SARIF report
        let sarif_path = "enhanced-contract-validation-results.sarif";
        let sarif_results: Vec<Value> = results.iter().map(|r| r.sarif.clone()).collect();
        let sarif_report = json!({
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0-rtm.5.json",
            "runs": sarif_results
        });

        fs::write(sarif_path, serde_json::to_string_pretty(&sarif_report)?)?;
        println!("📄 Enhanced SARIF report saved to: {}", sarif_path);

        std::process::exit(1);
    }
}
