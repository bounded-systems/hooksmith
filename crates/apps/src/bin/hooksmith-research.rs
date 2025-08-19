use std::io::{self, BufRead, Write};
use std::collections::HashMap;
use clap::Parser;
use git2::{Repository, ObjectType};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Parser)]
#[command(name = "hooksmith-research")]
#[command(about = "Git object researcher for Hooksmith pipeline")]
struct Cli {
    /// Tool to use for analysis
    #[arg(long, default_value = "name-linter")]
    tool: String,
    
    /// Tool version
    #[arg(long, default_value = "1.0.0")]
    version: String,
    
    /// Tool config hash
    #[arg(long, default_value = "default")]
    config_hash: String,
}

/// Input object descriptor from NDJSON
#[derive(Debug, Deserialize)]
struct ObjectDescriptor {
    oid: String,
    kind: String,
    logical_path: Option<String>,
    parent_tree_oid: Option<String>,
    size: usize,
}

/// Analysis result for NDJSON output
#[derive(Debug, Serialize)]
struct AnalysisResult {
    object_oid: String,
    tool_fingerprint: ToolFingerprint,
    analysis_blob_oid: String,
    cache_key: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ToolFingerprint {
    name: String,
    version: String,
    config_hash: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let repo = Repository::open(".")?;
    
    let tool_fingerprint = ToolFingerprint {
        name: cli.tool,
        version: cli.version,
        config_hash: cli.config_hash,
    };
    
    // Read NDJSON from stdin
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    // Output NDJSON to stdout
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    
    while let Some(line) = lines.next() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        let object: ObjectDescriptor = serde_json::from_str(&line)?;
        
        // Check cache first
        let cache_key = compute_cache_key(&tool_fingerprint, &object.oid);
        if let Some(cached_oid) = get_cached_analysis(&repo, &object.oid, &tool_fingerprint)? {
            let result = AnalysisResult {
                object_oid: object.oid,
                tool_fingerprint: tool_fingerprint.clone(),
                analysis_blob_oid: cached_oid,
                cache_key,
                metadata: json!({"cached": true}),
            };
            let json = serde_json::to_string(&result)?;
            writeln!(handle, "{}", json)?;
            continue;
        }
        
        // Perform analysis
        let analysis_data = analyze_object(&repo, &object, &tool_fingerprint)?;
        
        // Store analysis blob
        let analysis_blob_oid = store_analysis_blob(&repo, &analysis_data)?;
        
        let result = AnalysisResult {
            object_oid: object.oid,
            tool_fingerprint: tool_fingerprint.clone(),
            analysis_blob_oid,
            cache_key,
            metadata: json!({"cached": false}),
        };
        
        let json = serde_json::to_string(&result)?;
        writeln!(handle, "{}", json)?;
    }
    
    Ok(())
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

fn compute_cache_key(tool: &ToolFingerprint, object_oid: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("analysis-{}@{}", tool.name, tool.version).as_bytes());
    hasher.update(object_oid.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn get_cached_analysis(repo: &Repository, object_oid: &str, tool: &ToolFingerprint) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // TODO: Implement cache lookup from refs/hooksmith/cache/analysis/
    // For now, return None (no cache hit)
    Ok(None)
}

fn store_analysis_blob(repo: &Repository, analysis_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    // Convert to canonical JSON (sorted keys, no whitespace)
    let json_string = serde_json::to_string(analysis_data)?;
    
    // Store as blob
    let blob_oid = repo.blob(json_string.as_bytes())?;
    
    Ok(blob_oid.to_string())
}
