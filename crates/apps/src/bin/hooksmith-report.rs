use std::io::{self, BufRead, Write};
use std::collections::HashMap;
use clap::Parser;
use git2::Repository;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Sha256, Digest};

#[derive(Parser)]
#[command(name = "hooksmith-report")]
#[command(about = "Report generator for Hooksmith pipeline")]
struct Cli {
    /// Domain for the report
    #[arg(long, default_value = "object-names")]
    domain: String,
    
    /// Report version
    #[arg(long, default_value = "1.0.0")]
    version: String,
    
    /// Selector hash for caching
    #[arg(long)]
    selector_hash: String,
}

/// Input analysis result from NDJSON
#[derive(Debug, Deserialize)]
struct AnalysisResult {
    object_oid: String,
    tool_fingerprint: ToolFingerprint,
    analysis_blob_oid: String,
    cache_key: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ToolFingerprint {
    name: String,
    version: String,
    config_hash: String,
}

/// Report result for NDJSON output
#[derive(Debug, Serialize)]
struct ReportResult {
    object_oid: String,
    domain: String,
    version: String,
    report_blob_oid: String,
    cache_key: String,
    analysis_oids: Vec<String>,
    metadata: serde_json::Value,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let repo = Repository::open(".")?;
    
    // Group analysis results by object_oid
    let mut object_analyses: HashMap<String, Vec<AnalysisResult>> = HashMap::new();
    
    // Read NDJSON from stdin
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    while let Some(line) = lines.next() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        let analysis: AnalysisResult = serde_json::from_str(&line)?;
        object_analyses
            .entry(analysis.object_oid.clone())
            .or_insert_with(Vec::new)
            .push(analysis);
    }
    
    // Output NDJSON to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    for (object_oid, analyses) in object_analyses {
        // Check cache first
        let analysis_oids: Vec<String> = analyses.iter().map(|a| a.analysis_blob_oid.clone()).collect();
        let cache_key = compute_cache_key(&cli.domain, &cli.version, &analysis_oids);
        
        if let Some(cached_oid) = get_cached_report(&repo, &object_oid, &cli.domain, &cli.version)? {
            let result = ReportResult {
                object_oid,
                domain: cli.domain.clone(),
                version: cli.version.clone(),
                report_blob_oid: cached_oid,
                cache_key,
                analysis_oids,
                metadata: json!({"cached": true}),
            };
            let json = serde_json::to_string(&result)?;
            writeln!(handle, "{}", json)?;
            continue;
        }
        
        // Create normalized report
        let report_data = create_normalized_report(&repo, &analyses)?;
        
        // Store report blob
        let report_blob_oid = store_report_blob(&repo, &report_data)?;
        
        let result = ReportResult {
            object_oid,
            domain: cli.domain.clone(),
            version: cli.version.clone(),
            report_blob_oid,
            cache_key,
            analysis_oids,
            metadata: json!({"cached": false}),
        };
        
        let json = serde_json::to_string(&result)?;
        writeln!(handle, "{}", json)?;
    }
    
    Ok(())
}

fn create_normalized_report(repo: &Repository, analyses: &[AnalysisResult]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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

fn compute_cache_key(domain: &str, version: &str, analysis_oids: &[String]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("report@{}", version).as_bytes());
    for oid in analysis_oids {
        hasher.update(oid.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

fn get_cached_report(repo: &Repository, object_oid: &str, domain: &str, version: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/report/
    // For now, return None (no cache hit)
    Ok(None)
}

fn store_report_blob(repo: &Repository, report_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    // Convert to canonical JSON (sorted keys, no whitespace)
    let json_string = serde_json::to_string(report_data)?;
    
    // Store as blob
    let blob_oid = repo.blob(json_string.as_bytes())?;
    
    Ok(blob_oid.to_string())
}
