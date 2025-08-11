use std::io::{self, BufRead, Write};
use std::collections::HashSet;
use clap::Parser;
use git2::Repository;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};

#[derive(Parser)]
#[command(name = "hooksmith-audit")]
#[command(about = "Auditor for Hooksmith pipeline")]
struct Cli {
    /// Contract name
    #[arg(long, default_value = "object-names")]
    contract_name: String,
    
    /// Contract version
    #[arg(long, default_value = "1.0.0")]
    version: String,
}

/// Input report result from NDJSON
#[derive(Debug, Deserialize)]
struct ReportResult {
    object_oid: String,
    domain: String,
    version: String,
    report_blob_oid: String,
    cache_key: String,
    analysis_oids: Vec<String>,
    metadata: serde_json::Value,
}

/// Input mandate result from NDJSON
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Serialize)]
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let repo = Repository::open(".")?;
    
    // Read report results from stdin
    let mut reports: Vec<ReportResult> = Vec::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    while let Some(line) = lines.next() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        let report: ReportResult = serde_json::from_str(&line)?;
        reports.push(report);
    }
    
    // For now, we'll create dummy mandates for each report
    // In a real implementation, you'd read mandate results from another NDJSON stream
    let mut mandates: Vec<MandateResult> = Vec::new();
    for report in &reports {
        let mandate = MandateResult {
            object_oid: report.object_oid.clone(),
            contract_name: cli.contract_name.clone(),
            contract_oid: "contract-oid".to_string(),
            version: cli.version.clone(),
            mandate_blob_oid: "mandate-blob-oid".to_string(),
            cache_key: "mandate-cache-key".to_string(),
            object_selector: "object-selector".to_string(),
            logical_path: None,
            metadata: json!({}),
        };
        mandates.push(mandate);
    }
    
    // Output NDJSON to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    for (report, mandate) in reports.iter().zip(mandates.iter()) {
        // Check cache first
        let cache_key = compute_cache_key(&cli.version, &report.cache_key, &mandate.cache_key);
        
        if let Some(cached_verdict) = get_cached_verdict(&repo, &report.cache_key, &mandate.cache_key)? {
            let result = AuditResult {
                object_oid: report.object_oid.clone(),
                contract_name: cli.contract_name.clone(),
                version: cli.version.clone(),
                pass: cached_verdict.pass,
                summary_code: cached_verdict.summary_code,
                verdict_blob_oid: cached_verdict.verdict_blob_oid,
                diff_blob_oid: cached_verdict.diff_blob_oid,
                cache_key,
                report_oid: report.report_blob_oid.clone(),
                mandate_oid: mandate.mandate_blob_oid.clone(),
                metadata: json!({"cached": true}),
            };
            let json = serde_json::to_string(&result)?;
            writeln!(handle, "{}", json)?;
            continue;
        }
        
        // Load report and mandate data
        let report_data = load_report_data(&repo, &report.report_blob_oid)?;
        let mandate_data = load_mandate_data(&repo, &mandate.mandate_blob_oid)?;
        
        // Perform audit
        let (pass, summary_code, diff_data) = audit_object(&report_data, &mandate_data)?;
        
        // Store verdict blob
        let verdict_data = json!({
            "contract_name": cli.contract_name,
            "version": cli.version,
            "pass": pass,
            "summary_code": summary_code,
            "report_oid": report.report_blob_oid,
            "mandate_oid": mandate.mandate_blob_oid,
        });
        let verdict_blob_oid = store_verdict_blob(&repo, &verdict_data)?;
        
        // Store diff blob if needed
        let diff_blob_oid = if !pass && diff_data.is_some() {
            let diff_blob_oid = store_diff_blob(&repo, &diff_data.unwrap())?;
            Some(diff_blob_oid)
        } else {
            None
        };
        
        let result = AuditResult {
            object_oid: report.object_oid.clone(),
            contract_name: cli.contract_name.clone(),
            version: cli.version.clone(),
            pass,
            summary_code,
            verdict_blob_oid,
            diff_blob_oid,
            cache_key,
            report_oid: report.report_blob_oid.clone(),
            mandate_oid: mandate.mandate_blob_oid.clone(),
            metadata: json!({"cached": false}),
        };
        
        let json = serde_json::to_string(&result)?;
        writeln!(handle, "{}", json)?;
    }
    
    Ok(())
}

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

fn compute_cache_key(version: &str, report_oid: &str, mandate_oid: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("audit@{}", version).as_bytes());
    hasher.update(report_oid.as_bytes());
    hasher.update(mandate_oid.as_bytes());
    format!("{:x}", hasher.finalize())
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
    // For now, return None (no cache hit)
    Ok(None)
}

fn store_verdict_blob(repo: &Repository, verdict_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    // Convert to canonical JSON (sorted keys, no whitespace)
    let json_string = serde_json::to_string(verdict_data)?;
    
    // Store as blob
    let blob_oid = repo.blob(json_string.as_bytes())?;
    
    Ok(blob_oid.to_string())
}

fn store_diff_blob(repo: &Repository, diff_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    // Convert to canonical JSON (sorted keys, no whitespace)
    let json_string = serde_json::to_string(diff_data)?;
    
    // Store as blob
    let blob_oid = repo.blob(json_string.as_bytes())?;
    
    Ok(blob_oid.to_string())
}
