use std::collections::HashSet;
use std::process::Command;
use serde_json::Value;
use anyhow::{Result, Context};

fn main() -> Result<()> {
    println!("🔍 Testing Minimal Root Contract");
    println!("================================");
    
    // Load the minimal root contract
    let contract_content = std::fs::read_to_string("../contracts/object-names@root-minimal.json")
        .context("Failed to read contract file")?;
    
    let contract: Value = serde_json::from_str(&contract_content)
        .context("Failed to parse contract JSON")?;
    
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
    
    println!("📄 Contract loaded:");
    println!("   Required: {:?}", required);
    println!("   Allowed patterns: {} items", allowed.len());
    println!("   Rejected patterns: {} items", rejected_patterns.len());
    println!("   Ignored patterns: {:?}", ignored_patterns);
    
    // Get current root tree entries
    let output = Command::new("git")
        .args(["ls-tree", "--name-only", "HEAD^{tree}"])
        .current_dir("..")
        .output()
        .context("Failed to get root tree entries")?;
    
    if !output.status.success() {
        anyhow::bail!("git ls-tree failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    let root_entries: Vec<String> = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    println!("\n📁 Current root entries ({} items):", root_entries.len());
    for entry in &root_entries {
        println!("   {}", entry);
    }
    
    // Check for violations
    let mut violations = Vec::new();
    let mut missing_required = Vec::new();
    let mut rejected_entries = Vec::new();
    let mut not_allowed = Vec::new();
    
    // Check required entries
    for req in &required {
        if !root_entries.iter().any(|entry| entry == req) {
            missing_required.push(req.clone());
            violations.push(format!("missing required: {}", req));
        }
    }
    
    // Check rejected entries (skip ignored)
    for entry in &root_entries {
        if ignored_patterns.iter().any(|pattern| glob_match(pattern, entry)) {
            continue;
        }
        if rejected_patterns.iter().any(|pattern| glob_match(pattern, entry)) {
            rejected_entries.push(entry.clone());
            violations.push(format!("rejected at root: {}", entry));
        }
    }
    
    // Check allow-list (skip ignored)
    for entry in &root_entries {
        if ignored_patterns.iter().any(|pattern| glob_match(pattern, entry)) {
            continue;
        }
        if !allowed.iter().any(|pattern| glob_match(pattern, entry)) {
            not_allowed.push(entry.clone());
            violations.push(format!("not in allowed set: {}", entry));
        }
    }
    
    println!("\n🔍 Validation Results:");
    println!("=====================");
    
    if violations.is_empty() {
        println!("✅ No violations found! Root structure is compliant.");
    } else {
        println!("❌ Found {} violations:", violations.len());
        
        if !missing_required.is_empty() {
            println!("\n📋 Missing required entries:");
            for req in &missing_required {
                println!("   - {}", req);
            }
        }
        
        if !rejected_entries.is_empty() {
            println!("\n🚫 Rejected entries (need to be moved):");
            for entry in &rejected_entries {
                println!("   - {}", entry);
            }
        }
        
        if !not_allowed.is_empty() {
            println!("\n⚠️  Not in allowed set:");
            for entry in &not_allowed {
                println!("   - {}", entry);
            }
        }
        
        println!("\n📝 Suggested move plan:");
        println!("=======================");
        
        // Generate move suggestions
        for entry in &rejected_entries {
            let suggestion = suggest_move_location(entry);
            println!("   {} → {}", entry, suggestion);
        }
        
        for entry in &not_allowed {
            let suggestion = suggest_move_location(entry);
            println!("   {} → {}", entry, suggestion);
        }
    }
    
    Ok(())
}

fn glob_match(pattern: &str, entry: &str) -> bool {
    if pattern.contains('*') {
        // Simple glob matching
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let inner = &pattern[1..pattern.len()-1];
            entry.contains(inner)
        } else if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            entry.starts_with(prefix)
        } else if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            entry.ends_with(suffix)
        } else {
            entry == pattern
        }
    } else {
        entry == pattern
    }
}

fn suggest_move_location(entry: &str) -> String {
    match entry {
        // Documentation and reports
        entry if entry.ends_with(".md") && entry != "README.md" => "docs/reports/".to_string(),
        entry if entry.contains("SUMMARY") => "docs/reports/".to_string(),
        entry if entry.contains("REPORT") => "docs/reports/".to_string(),
        
        // Contracts and agreements
        "agreement.json" => ".hooksmith/agreements/".to_string(),
        entry if entry.starts_with("contract") => ".hooksmith/contracts/".to_string(),
        
        // Docker and infrastructure
        "Dockerfile" => "infra/docker/".to_string(),
        entry if entry.starts_with("docker") => "infra/docker/".to_string(),
        
        // Generated files
        entry if entry.starts_with("gen") => "tools/gen/".to_string(),
        entry if entry.contains("generated") => "tools/gen/".to_string(),
        
        // Test files
        entry if entry.starts_with("test") => "tests/".to_string(),
        
        // Configuration files
        "CODEOWNERS" => ".github/".to_string(),
        "languages.yml" => ".github/".to_string(),
        
        // Build artifacts
        "build.rs" => "crates/build-script/".to_string(),
        
        // Worktree and lifecycle
        entry if entry.contains("worktree") => "tools/worktree/".to_string(),
        entry if entry.contains("lifecycle") => "tools/lifecycle/".to_string(),
        
        // Default suggestion
        _ => "tools/misc/".to_string(),
    }
}
