use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use git2::{Repository, ObjectType};
use serde_json::json;

/// Object descriptor for NDJSON output
#[derive(Debug, Serialize, Deserialize)]
struct ObjectDescriptor {
    oid: String,
    kind: String,
    logical_path: Option<String>,
    parent_tree_oid: Option<String>,
    size: usize,
}

/// Analysis result for NDJSON output
#[derive(Debug, Serialize, Deserialize)]
struct AnalysisResult {
    object_oid: String,
    tool_fingerprint: ToolFingerprint,
    analysis_blob_oid: String,
    cache_key: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolFingerprint {
    name: String,
    version: String,
    config_hash: String,
}

/// Report result for NDJSON output
#[derive(Debug, Serialize, Deserialize)]
struct ReportResult {
    object_oid: String,
    domain: String,
    version: String,
    report_blob_oid: String,
    cache_key: String,
    analysis_oids: Vec<String>,
    metadata: serde_json::Value,
}

/// Mandate result for NDJSON output
#[derive(Debug, Serialize, Deserialize)]
struct MandateResult {
    object_oid: String,
    contract_name: String,
    contract_oid: String,
    version: String,
    mandate_blob_oid: String,
    cache_key: String,
    object_selector: String,
    logical_path: Option<String>,
    metadata: serde_json::Value,
}

/// Audit result for NDJSON output
#[derive(Debug, Serialize, Deserialize)]
struct AuditResult {
    object_oid: String,
    contract_name: String,
    version: String,
    pass: bool,
    summary_code: String,
    verdict_blob_oid: String,
    diff_blob_oid: Option<String>,
    cache_key: String,
    report_oid: String,
    mandate_oid: String,
    metadata: serde_json::Value,
}

/// Contract specification
#[derive(Debug, Deserialize)]
struct Contract {
    name: String,
    version: String,
    spec: ContractSpec,
}

#[derive(Debug, Deserialize)]
struct ContractSpec {
    git: GitSpec,
}

#[derive(Debug, Deserialize)]
struct GitSpec {
    tree: TreeSpec,
}

#[derive(Debug, Deserialize)]
struct TreeSpec {
    objects: ObjectsSpec,
}

#[derive(Debug, Deserialize)]
struct ObjectsSpec {
    names: NamesSpec,
}

#[derive(Debug, Deserialize)]
struct NamesSpec {
    required: Vec<String>,
    allowed: Vec<String>,
    rejected: Vec<String>,
    ignored: Vec<String>,
}

/// Actor 1: Scope Resolver
fn scope_resolve(repo: &Repository, ref_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let commit = repo.find_reference(ref_name)?.peel_to_commit()?;
    Ok(commit.id().to_string())
}

fn scope_ls(repo: &Repository, commit_oid: &str, selector: &str) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    let commit = repo.find_commit(git2::Oid::from_str(commit_oid)?)?;
    let tree = commit.tree()?;
    
    match selector {
        "root-names" => select_root_names(&repo, &tree),
        "pattern" => select_by_pattern(&repo, &tree, "src/**/*.rs"),
        "depth" => select_by_depth(&repo, &tree, 2),
        _ => Err(format!("Unknown selector: {}", selector).into()),
    }
}

fn select_root_names(repo: &Repository, tree: &git2::Tree) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    let mut objects = Vec::new();
    
    for entry in tree.iter() {
        let object = ObjectDescriptor {
            oid: entry.id().to_string(),
            kind: format!("{:?}", entry.kind()),
            logical_path: Some(entry.name().unwrap_or("").to_string()),
            parent_tree_oid: Some(tree.id().to_string()),
            size: entry.id().as_bytes().len(),
        };
        objects.push(object);
    }
    
    Ok(objects)
}

fn select_by_pattern(repo: &Repository, tree: &git2::Tree, pattern: &str) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    // TODO: Implement glob pattern matching
    select_root_names(repo, tree)
}

fn select_by_depth(repo: &Repository, tree: &git2::Tree, depth: usize) -> Result<Vec<ObjectDescriptor>, Box<dyn std::error::Error>> {
    // TODO: Implement depth-based tree walking
    select_root_names(repo, tree)
}

/// Actor 2: Researcher
fn research_objects(repo: &Repository, objects: &[ObjectDescriptor], tool: &ToolFingerprint) -> Result<Vec<AnalysisResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    for object in objects {
        // Check cache first
        let cache_key = compute_analysis_cache_key(tool, &object.oid);
        if let Some(cached_oid) = get_cached_analysis(repo, &object.oid, tool)? {
            results.push(AnalysisResult {
                object_oid: object.oid.clone(),
                tool_fingerprint: tool.clone(),
                analysis_blob_oid: cached_oid,
                cache_key,
                metadata: json!({"cached": true}),
            });
            continue;
        }
        
        // Perform analysis
        let analysis_data = analyze_object(repo, object, tool)?;
        
        // Store analysis blob
        let analysis_blob_oid = store_analysis_blob(repo, &analysis_data)?;
        
        results.push(AnalysisResult {
            object_oid: object.oid.clone(),
            tool_fingerprint: tool.clone(),
            analysis_blob_oid,
            cache_key,
            metadata: json!({"cached": false}),
        });
    }
    
    Ok(results)
}

fn analyze_object(repo: &Repository, object: &ObjectDescriptor, tool: &ToolFingerprint) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match object.kind.as_str() {
        "Tree" => analyze_tree(repo, object),
        "Blob" => analyze_blob(repo, object),
        _ => Ok(json!({"error": "unsupported_object_type"})),
    }
}

fn analyze_tree(repo: &Repository, object: &ObjectDescriptor) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let tree = repo.find_tree(git2::Oid::from_str(&object.oid)?)?;
    let mut entries = Vec::new();
    let mut entry_names = Vec::new();
    
    for entry in tree.iter() {
        let name = entry.name().unwrap_or("").to_string();
        entry_names.push(name.clone());
        
        entries.push(json!({
            "name": name,
            "oid": entry.id().to_string(),
            "kind": format!("{:?}", entry.kind()),
            "mode": entry.filemode(),
        }));
    }
    
    Ok(json!({
        "type": "tree_analysis",
        "entry_count": entries.len(),
        "entries": entries,
        "entry_names": entry_names,
        "is_root": object.logical_path.as_ref().map_or(true, |p| p.is_empty()),
    }))
}

fn analyze_blob(repo: &Repository, object: &ObjectDescriptor) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let blob = repo.find_blob(git2::Oid::from_str(&object.oid)?)?;
    let content = blob.content();
    
    Ok(json!({
        "type": "blob_analysis",
        "size": content.len(),
        "is_binary": content.iter().any(|&b| b < 32 && b != 9 && b != 10 && b != 13),
        "has_null_bytes": content.iter().any(|&b| b == 0),
    }))
}

/// Actor 3: Reporter
fn create_reports(repo: &Repository, analyses: &[AnalysisResult], domain: &str, version: &str) -> Result<Vec<ReportResult>, Box<dyn std::error::Error>> {
    // Group analyses by object_oid
    let mut object_analyses: std::collections::HashMap<String, Vec<&AnalysisResult>> = std::collections::HashMap::new();
    
    for analysis in analyses {
        object_analyses
            .entry(analysis.object_oid.clone())
            .or_insert_with(Vec::new)
            .push(analysis);
    }
    
    let mut results = Vec::new();
    
    for (object_oid, analyses) in object_analyses {
        // Check cache first
        let analysis_oids: Vec<String> = analyses.iter().map(|a| a.analysis_blob_oid.clone()).collect();
        let cache_key = compute_report_cache_key(domain, version, &analysis_oids);
        
        if let Some(cached_oid) = get_cached_report(repo, &object_oid, domain, version)? {
            results.push(ReportResult {
                object_oid,
                domain: domain.to_string(),
                version: version.to_string(),
                report_blob_oid: cached_oid,
                cache_key,
                analysis_oids,
                metadata: json!({"cached": true}),
            });
            continue;
        }
        
        // Create normalized report
        let report_data = create_normalized_report(repo, analyses)?;
        
        // Store report blob
        let report_blob_oid = store_report_blob(repo, &report_data)?;
        
        results.push(ReportResult {
            object_oid,
            domain: domain.to_string(),
            version: version.to_string(),
            report_blob_oid,
            cache_key,
            analysis_oids,
            metadata: json!({"cached": false}),
        });
    }
    
    Ok(results)
}

fn create_normalized_report(repo: &Repository, analyses: &[&AnalysisResult]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut normalized_data = serde_json::Map::new();
    
    for analysis in analyses {
        // Load analysis blob
        let analysis_blob = repo.find_blob(git2::Oid::from_str(&analysis.analysis_blob_oid)?)?;
        let analysis_content = String::from_utf8(analysis_blob.content().to_vec())?;
        let analysis_data: serde_json::Value = serde_json::from_str(&analysis_content)?;
        
        // Create tool key
        let tool_key = format!("{}@{}", analysis.tool_fingerprint.name, analysis.tool_fingerprint.version);
        
        // Extract relevant data based on analysis type
        if let Some(entry_names) = analysis_data.get("entry_names") {
            normalized_data.insert("entry_names".to_string(), entry_names.clone());
        }
        if let Some(is_root) = analysis_data.get("is_root") {
            normalized_data.insert("is_root".to_string(), is_root.clone());
        }
        
        // Store tool-specific data
        normalized_data.insert(tool_key, analysis_data);
    }
    
    Ok(serde_json::Value::Object(normalized_data))
}

/// Actor 4: Mandator
fn create_mandates(repo: &Repository, objects: &[ObjectDescriptor], contract: &Contract, version: &str) -> Result<Vec<MandateResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    for object in objects {
        // Compute object selector
        let object_selector = compute_object_selector(object);
        
        // Check cache first
        let cache_key = compute_mandate_cache_key(version, &contract.name, &object_selector, &object.logical_path);
        
        if let Some(cached_oid) = get_cached_mandate(repo, &contract.name, &object_selector)? {
            results.push(MandateResult {
                object_oid: object.oid.clone(),
                contract_name: contract.name.clone(),
                contract_oid: contract.name.clone(),
                version: version.to_string(),
                mandate_blob_oid: cached_oid,
                cache_key,
                object_selector,
                logical_path: object.logical_path.clone(),
                metadata: json!({"cached": true}),
            });
            continue;
        }
        
        // Create mandate
        let mandate_data = create_mandate(contract, object)?;
        
        // Store mandate blob
        let mandate_blob_oid = store_mandate_blob(repo, &mandate_data)?;
        
        results.push(MandateResult {
            object_oid: object.oid.clone(),
            contract_name: contract.name.clone(),
            contract_oid: contract.name.clone(),
            version: version.to_string(),
            mandate_blob_oid,
            cache_key,
            object_selector,
            logical_path: object.logical_path.clone(),
            metadata: json!({"cached": false}),
        });
    }
    
    Ok(results)
}

fn create_mandate(contract: &Contract, object: &ObjectDescriptor) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    // Determine if this is a root tree
    let is_root = object.logical_path.as_ref().map_or(true, |p| p.is_empty());
    
    let expectation = if is_root {
        // Root tree expectations
        json!({
            "type": "root_tree",
            "required_entries": contract.spec.git.tree.objects.names.required,
            "allowed_entries": contract.spec.git.tree.objects.names.allowed,
            "rejected_entries": contract.spec.git.tree.objects.names.rejected,
            "ignored_entries": contract.spec.git.tree.objects.names.ignored,
        })
    } else {
        // Non-root tree expectations (can be more permissive)
        json!({
            "type": "sub_tree",
            "allowed_entries": contract.spec.git.tree.objects.names.allowed,
            "ignored_entries": contract.spec.git.tree.objects.names.ignored,
        })
    };
    
    Ok(expectation)
}

/// Actor 5: Auditor
fn audit_objects(repo: &Repository, reports: &[ReportResult], mandates: &[MandateResult], contract_name: &str, version: &str) -> Result<Vec<AuditResult>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    // Create a map of object_oid to mandate for easy lookup
    let mandate_map: std::collections::HashMap<String, &MandateResult> = mandates
        .iter()
        .map(|m| (m.object_oid.clone(), m))
        .collect();
    
    for report in reports {
        let mandate = mandate_map.get(&report.object_oid)
            .ok_or_else(|| format!("No mandate found for object {}", report.object_oid))?;
        
        // Check cache first
        let cache_key = compute_audit_cache_key(version, &report.cache_key, &mandate.cache_key);
        
        if let Some(cached_verdict) = get_cached_verdict(repo, &report.cache_key, &mandate.cache_key)? {
            results.push(AuditResult {
                object_oid: report.object_oid.clone(),
                contract_name: contract_name.to_string(),
                version: version.to_string(),
                pass: cached_verdict.pass,
                summary_code: cached_verdict.summary_code,
                verdict_blob_oid: cached_verdict.verdict_blob_oid,
                diff_blob_oid: cached_verdict.diff_blob_oid,
                cache_key,
                report_oid: report.report_blob_oid.clone(),
                mandate_oid: mandate.mandate_blob_oid.clone(),
                metadata: json!({"cached": true}),
            });
            continue;
        }
        
        // Load report and mandate data
        let report_data = load_report_data(repo, &report.report_blob_oid)?;
        let mandate_data = load_mandate_data(repo, &mandate.mandate_blob_oid)?;
        
        // Perform audit
        let (pass, summary_code, diff_data) = audit_object(&report_data, &mandate_data)?;
        
        // Store verdict blob
        let verdict_data = json!({
            "contract_name": contract_name,
            "version": version,
            "pass": pass,
            "summary_code": summary_code,
            "report_oid": report.report_blob_oid,
            "mandate_oid": mandate.mandate_blob_oid,
        });
        let verdict_blob_oid = store_verdict_blob(repo, &verdict_data)?;
        
        // Store diff blob if needed
        let diff_blob_oid = if !pass && diff_data.is_some() {
            let diff_blob_oid = store_diff_blob(repo, &diff_data.unwrap())?;
            Some(diff_blob_oid)
        } else {
            None
        };
        
        results.push(AuditResult {
            object_oid: report.object_oid.clone(),
            contract_name: contract_name.to_string(),
            version: version.to_string(),
            pass,
            summary_code,
            verdict_blob_oid,
            diff_blob_oid,
            cache_key,
            report_oid: report.report_blob_oid.clone(),
            mandate_oid: mandate.mandate_blob_oid.clone(),
            metadata: json!({"cached": false}),
        });
    }
    
    Ok(results)
}

fn audit_object(report_data: &serde_json::Value, mandate_data: &serde_json::Value) -> Result<(bool, String, Option<serde_json::Value>), Box<dyn std::error::Error>> {
    // Extract entry names from report
    let entry_names = if let Some(names) = report_data.get("entry_names") {
        names.as_array().unwrap_or(&Vec::new()).iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };
    
    let is_root = report_data.get("is_root")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Perform validation
    let (pass, summary_code, diff_data) = if is_root {
        validate_root_tree(&entry_names, mandate_data)?
    } else {
        validate_sub_tree(&entry_names, mandate_data)?
    };
    
    Ok((pass, summary_code, diff_data))
}

fn validate_root_tree(entry_names: &[String], expectation: &serde_json::Value) -> Result<(bool, String, Option<serde_json::Value>), Box<dyn std::error::Error>> {
    let required = expectation.get("required_entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();
        
    let allowed = expectation.get("allowed_entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();
        
    let rejected = expectation.get("rejected_entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();
        
    let ignored = expectation.get("ignored_entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();
    
    let entry_set: HashSet<String> = entry_names.iter().cloned().collect();
    let mut errors = Vec::new();
    
    // Check required entries
    for req in &required {
        if !entry_set.contains(req) {
            errors.push(format!("missing required: {}", req));
        }
    }
    
    // Check rejected entries (skip ignored)
    for entry in entry_names {
        if ignored.contains(entry) {
            continue;
        }
        if rejected.contains(entry) {
            errors.push(format!("rejected at root: {}", entry));
        }
    }
    
    // Check allow-list (skip ignored)
    for entry in entry_names {
        if ignored.contains(entry) {
            continue;
        }
        if !allowed.contains(entry) {
            errors.push(format!("not in allowed set: {}", entry));
        }
    }
    
    let pass = errors.is_empty();
    let summary_code = if pass { "PASS".to_string() } else { "FAIL".to_string() };
    
    let diff_data = if !pass {
        Some(json!({
            "type": "validation_errors",
            "errors": errors,
            "entry_names": entry_names,
            "required": required,
            "allowed": allowed,
            "rejected": rejected,
            "ignored": ignored,
        }))
    } else {
        None
    };
    
    Ok((pass, summary_code, diff_data))
}

fn validate_sub_tree(entry_names: &[String], expectation: &serde_json::Value) -> Result<(bool, String, Option<serde_json::Value>), Box<dyn std::error::Error>> {
    // For sub-trees, we're more permissive - just check against allowed/ignored
    let allowed = expectation.get("allowed_entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();
        
    let ignored = expectation.get("ignored_entries")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect::<HashSet<String>>();
    
    let mut errors = Vec::new();
    
    // Check allow-list (skip ignored)
    for entry in entry_names {
        if ignored.contains(entry) {
            continue;
        }
        if !allowed.contains(entry) {
            errors.push(format!("not in allowed set: {}", entry));
        }
    }
    
    let pass = errors.is_empty();
    let summary_code = if pass { "PASS".to_string() } else { "FAIL".to_string() };
    
    let diff_data = if !pass {
        Some(json!({
            "type": "validation_errors",
            "errors": errors,
            "entry_names": entry_names,
        }))
    } else {
        None
    };
    
    Ok((pass, summary_code, diff_data))
}

// Cache management functions
fn compute_analysis_cache_key(tool: &ToolFingerprint, object_oid: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(format!("analysis-{}@{}", tool.name, tool.version).as_bytes());
    hasher.update(object_oid.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compute_report_cache_key(domain: &str, version: &str, analysis_oids: &[String]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(format!("report@{}", version).as_bytes());
    for oid in analysis_oids {
        hasher.update(oid.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn compute_object_selector(object: &ObjectDescriptor) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(object.kind.as_bytes());
    if let Some(path) = &object.logical_path {
        hasher.update(path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn compute_mandate_cache_key(version: &str, contract_oid: &str, selector: &str, logical_path: &Option<String>) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(format!("mandate@{}", version).as_bytes());
    hasher.update(contract_oid.as_bytes());
    hasher.update(selector.as_bytes());
    if let Some(path) = logical_path {
        hasher.update(path.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn compute_audit_cache_key(version: &str, report_oid: &str, mandate_oid: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(format!("audit@{}", version).as_bytes());
    hasher.update(report_oid.as_bytes());
    hasher.update(mandate_oid.as_bytes());
    format!("{:x}", hasher.finalize())
}

// Cache lookup functions (stubs for now)
fn get_cached_analysis(repo: &Repository, object_oid: &str, tool: &ToolFingerprint) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/analysis/
    Ok(None)
}

fn get_cached_report(repo: &Repository, object_oid: &str, domain: &str, version: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/report/
    Ok(None)
}

fn get_cached_mandate(repo: &Repository, contract_oid: &str, selector: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/mandate/
    Ok(None)
}

#[derive(Debug)]
struct CachedVerdict {
    pass: bool,
    summary_code: String,
    verdict_blob_oid: String,
    diff_blob_oid: Option<String>,
}

fn get_cached_verdict(repo: &Repository, report_oid: &str, mandate_oid: &str) -> Result<Option<CachedVerdict>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/verdict/
    Ok(None)
}

// Blob storage functions
fn store_analysis_blob(repo: &Repository, analysis_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string(analysis_data)?;
    let blob_oid = repo.blob(json_string.as_bytes())?;
    Ok(blob_oid.to_string())
}

fn store_report_blob(repo: &Repository, report_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string(report_data)?;
    let blob_oid = repo.blob(json_string.as_bytes())?;
    Ok(blob_oid.to_string())
}

fn store_mandate_blob(repo: &Repository, mandate_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string(mandate_data)?;
    let blob_oid = repo.blob(json_string.as_bytes())?;
    Ok(blob_oid.to_string())
}

fn store_verdict_blob(repo: &Repository, verdict_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string(verdict_data)?;
    let blob_oid = repo.blob(json_string.as_bytes())?;
    Ok(blob_oid.to_string())
}

fn store_diff_blob(repo: &Repository, diff_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let json_string = serde_json::to_string(diff_data)?;
    let blob_oid = repo.blob(json_string.as_bytes())?;
    Ok(blob_oid.to_string())
}

// Data loading functions
fn load_report_data(repo: &Repository, report_blob_oid: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let report_blob = repo.find_blob(git2::Oid::from_str(report_blob_oid)?)?;
    let report_content = String::from_utf8(report_blob.content().to_vec())?;
    let report_data: serde_json::Value = serde_json::from_str(&report_content)?;
    Ok(report_data)
}

fn load_mandate_data(repo: &Repository, mandate_blob_oid: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mandate_blob = repo.find_blob(git2::Oid::from_str(mandate_blob_oid)?)?;
    let mandate_content = String::from_utf8(mandate_blob.content().to_vec())?;
    let mandate_data: serde_json::Value = serde_json::from_str(&mandate_content)?;
    Ok(mandate_data)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 NDJSON Pipeline Demo");
    println!("=======================");
    
    // Load contract
    let contract_path = "contracts/object-names@v1.json";
    let contract_content = std::fs::read_to_string(contract_path)?;
    let contract: Contract = serde_json::from_str(&contract_content)?;
    println!("📄 Loaded contract: {} v{}", contract.name, contract.version);
    
    // Open repository
    let repo = Repository::open(".")?;
    println!("📁 Opened repository");
    
    // Step 1: Scope Resolution
    println!("\n🔍 Step 1: Scope Resolution");
    let commit_oid = scope_resolve(&repo, "refs/heads/main")?;
    println!("   Commit OID: {}", commit_oid);
    
    // Step 2: Object Selection
    println!("\n📋 Step 2: Object Selection");
    let objects = scope_ls(&repo, &commit_oid, "root-names")?;
    println!("   Selected {} objects", objects.len());
    
    // Step 3: Research
    println!("\n🔬 Step 3: Research");
    let tool = ToolFingerprint {
        name: "name-linter".to_string(),
        version: "1.0.0".to_string(),
        config_hash: "default".to_string(),
    };
    let analyses = research_objects(&repo, &objects, &tool)?;
    println!("   Generated {} analyses", analyses.len());
    
    // Step 4: Reporting
    println!("\n📊 Step 4: Reporting");
    let reports = create_reports(&repo, &analyses, "object-names", "1.0.0")?;
    println!("   Created {} reports", reports.len());
    
    // Step 5: Mandating
    println!("\n📋 Step 5: Mandating");
    let mandates = create_mandates(&repo, &objects, &contract, "1.0.0")?;
    println!("   Created {} mandates", mandates.len());
    
    // Step 6: Auditing
    println!("\n🔍 Step 6: Auditing");
    let audits = audit_objects(&repo, &reports, &mandates, &contract.name, "1.0.0")?;
    println!("   Generated {} audits", audits.len());
    
    // Results
    println!("\n📋 Results:");
    println!("===========");
    
    let mut all_passed = true;
    for audit in &audits {
        if audit.pass {
            println!("✅ {}: {}", audit.object_oid, audit.summary_code);
        } else {
            println!("❌ {}: {}", audit.object_oid, audit.summary_code);
            all_passed = false;
        }
    }
    
    println!("\n🎉 Pipeline completed!");
    if all_passed {
        println!("✅ All validations passed!");
    } else {
        println!("💥 Some validations failed!");
    }
    
    // Demonstrate NDJSON output
    println!("\n📄 NDJSON Output Examples:");
    println!("==========================");
    
    // Object descriptors
    println!("\nObject Descriptors:");
    for object in &objects[..2.min(objects.len())] {
        let json = serde_json::to_string(object)?;
        println!("  {}", json);
    }
    
    // Analysis results
    println!("\nAnalysis Results:");
    for analysis in &analyses[..2.min(analyses.len())] {
        let json = serde_json::to_string(analysis)?;
        println!("  {}", json);
    }
    
    // Audit results
    println!("\nAudit Results:");
    for audit in &audits[..2.min(audits.len())] {
        let json = serde_json::to_string(audit)?;
        println!("  {}", json);
    }
    
    Ok(())
}
