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
        anyhow::bail!(
            "git ls-tree failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
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

    let contract: Value =
        serde_json::from_str(&contract_content).context("Failed to parse contract JSON")?;

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

#[derive(Debug)]
struct MigrationPlan {
    missing_required: Vec<String>,
    rejected_files: Vec<String>,
    unexpected_files: Vec<String>,
    suggested_moves: Vec<(String, String)>,
}

fn analyze_migration(rules: &RuleSet, root_entries: &[String]) -> MigrationPlan {
    let mut plan = MigrationPlan {
        missing_required: Vec::new(),
        rejected_files: Vec::new(),
        unexpected_files: Vec::new(),
        suggested_moves: Vec::new(),
    };

    let allowed_set: HashSet<&String> = rules.allowed.iter().collect();

    // 1. Check missing required
    for required in &rules.required {
        if !root_entries.iter().any(|entry| entry == required) {
            plan.missing_required.push(required.clone());
        }
    }

    // 2. Check rejected and unexpected files
    for entry in root_entries {
        if rules.ignored.is_match(entry) {
            continue;
        }

        if rules.rejected.is_match(entry) {
            plan.rejected_files.push(entry.clone());
        } else if !allowed_set.contains(entry) {
            plan.unexpected_files.push(entry.clone());
        }
    }

    // 3. Generate suggested moves
    for entry in &plan.rejected_files {
        let suggested_dest = match entry.as_str() {
            "README.md" | "*.md" => "docs/",
            "Cargo.toml" | "*.toml" => "projects/hooksmith/",
            _ => "projects/hooksmith/",
        };
        plan.suggested_moves
            .push((entry.clone(), suggested_dest.to_string()));
    }

    for entry in &plan.unexpected_files {
        let suggested_dest = if entry.ends_with(".md") {
            "docs/"
        } else if entry.ends_with(".rs") {
            "projects/hooksmith/scripts/"
        } else if entry.ends_with(".toml") || entry.ends_with(".json") {
            "projects/hooksmith/"
        } else {
            "projects/hooksmith/"
        };
        plan.suggested_moves
            .push((entry.clone(), suggested_dest.to_string()));
    }

    plan
}

fn main() -> Result<()> {
    println!("🔍 Analyzing object-names contract migration requirements...");

    let rules = load_contract()?;
    let root_entries = get_root_tree_entries()?;

    println!("📋 Current root tree entries ({}):", root_entries.len());
    for entry in &root_entries {
        println!("  - {}", entry);
    }
    println!();

    let plan = analyze_migration(&rules, &root_entries);

    println!("📊 Migration Analysis:");
    println!();

    if !plan.missing_required.is_empty() {
        println!("❌ Missing required entries:");
        for missing in &plan.missing_required {
            println!("  - {}", missing);
        }
        println!();
    }

    if !plan.rejected_files.is_empty() {
        println!("🚫 Rejected files at root:");
        for rejected in &plan.rejected_files {
            println!("  - {}", rejected);
        }
        println!();
    }

    if !plan.unexpected_files.is_empty() {
        println!("⚠️  Unexpected files (not in allowed set):");
        for unexpected in &plan.unexpected_files {
            println!("  - {}", unexpected);
        }
        println!();
    }

    if !plan.suggested_moves.is_empty() {
        println!("💡 Suggested file moves:");
        for (src, dest) in &plan.suggested_moves {
            println!("  - {} → {}", src, dest);
        }
        println!();
    }

    println!("🎯 Summary:");
    println!("  - Missing required: {}", plan.missing_required.len());
    println!("  - Rejected files: {}", plan.rejected_files.len());
    println!("  - Unexpected files: {}", plan.unexpected_files.len());
    println!("  - Total files to move: {}", plan.suggested_moves.len());

    if plan.missing_required.is_empty()
        && plan.rejected_files.is_empty()
        && plan.unexpected_files.is_empty()
    {
        println!("\n✅ No migration needed - contract is satisfied!");
    } else {
        println!("\n📝 Next steps:");
        println!("1. Create missing required directories/files");
        println!("2. Move rejected and unexpected files to suggested locations");
        println!("3. Run validation again to confirm compliance");
    }

    Ok(())
}
