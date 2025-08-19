use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde_json::Value;
use std::collections::HashSet;
use std::process::Command;

#[derive(Debug)]
struct RuleSet {
    required: Vec<String>,
    allowed: Vec<String>,
    rejected: GlobSet,
    ignored: GlobSet,
}

fn build_globs(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern)?);
    }
    Ok(builder.build()?)
}

fn get_root_tree_entries() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["ls-tree", "--name-only", "origin/main"])
        .output()
        .context("Failed to execute git ls-tree")?;

    if !output.status.success() {
        anyhow::bail!("git ls-tree failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    let entries = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in git ls-tree output")?
        .lines()
        .map(|s| s.to_string())
        .collect();

    Ok(entries)
}

fn load_contract() -> Result<RuleSet> {
    let contract_content = std::fs::read_to_string("../contracts/object-names@v1.json")
        .context("Failed to read contract file")?;
    
    let contract: Value = serde_json::from_str(&contract_content)
        .context("Failed to parse contract JSON")?;

    let spec = &contract["spec"]["git"]["tree"]["objects"]["names"];
    
    let required = spec["required"]
        .as_array()
        .context("Missing or invalid 'required' field")?
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let allowed = spec["allowed"]
        .as_array()
        .context("Missing or invalid 'allowed' field")?
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let rejected_patterns: Vec<String> = spec["rejected"]
        .as_array()
        .context("Missing or invalid 'rejected' field")?
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let ignored_patterns: Vec<String> = spec["ignored"]
        .as_array()
        .context("Missing or invalid 'ignored' field")?
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    let rejected = build_globs(&rejected_patterns)?;
    let ignored = build_globs(&ignored_patterns)?;

    Ok(RuleSet {
        required,
        allowed,
        rejected,
        ignored,
    })
}

fn validate_root(rules: &RuleSet, root_entries: &[String]) -> Result<Vec<String>> {
    let mut errors = Vec::new();
    
    // Build allowed glob set
    let allowed_globs = build_globs(&rules.allowed)?;

    // 1. Check required entries
    for required in &rules.required {
        if !root_entries.iter().any(|entry| entry == required) {
            errors.push(format!("❌ missing required: {}", required));
        }
    }

    // 2. Check rejected entries (skip ignored)
    for entry in root_entries {
        if rules.ignored.is_match(entry) {
            continue;
        }
        if rules.rejected.is_match(entry) {
            errors.push(format!("❌ rejected at root: {}", entry));
        }
    }

    // 3. Check allow-list (skip ignored)
    for entry in root_entries {
        if rules.ignored.is_match(entry) {
            continue;
        }
        if !allowed_globs.is_match(entry) {
            errors.push(format!("❌ not in allowed set: {}", entry));
        }
    }

    Ok(errors)
}

fn main() -> Result<()> {
    println!("🔍 Validating object-names contract against origin/main root tree...");
    
    // Fetch latest origin/main
    let fetch_status = Command::new("git")
        .arg("fetch")
        .arg("origin")
        .status()
        .context("Failed to fetch origin")?;

    if !fetch_status.success() {
        anyhow::bail!("git fetch failed");
    }

    let rules = load_contract()?;
    let root_entries = get_root_tree_entries()?;

    println!("📋 Root tree entries:");
    for entry in &root_entries {
        println!("  - {}", entry);
    }
    println!();

    let errors = validate_root(&rules, &root_entries)?;

    if errors.is_empty() {
        println!("✅ origin/main root satisfies object-names contract");
        Ok(())
    } else {
        println!("🚫 Contract validation failed:");
        for error in &errors {
            println!("  {}", error);
        }
        std::process::exit(1);
    }
}
