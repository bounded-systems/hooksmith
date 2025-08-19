use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str::FromStr;

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct TreeEntry {
    mode: String,
    #[serde(rename = "type")]
    entry_type: String,
    sha: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TreeReport {
    ref_name: String,
    tree_sha: String,
    entries: Vec<TreeEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Contract {
    name: String,
    version: String,
    spec: ContractSpec,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContractSpec {
    git: GitSpec,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitSpec {
    tree: TreeSpec,
}

#[derive(Debug, Serialize, Deserialize)]
struct TreeSpec {
    objects: ObjectsSpec,
}

#[derive(Debug, Serialize, Deserialize)]
struct ObjectsSpec {
    names: NamesSpec,
}

#[derive(Debug, Serialize, Deserialize)]
struct NamesSpec {
    required: Vec<String>,
    allowed: Vec<String>,
    rejected: Vec<String>,
    ignored: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    ref_name: String,
    tree_sha: String,
    contract_name: String,
    contract_version: String,
    result: ValidationDiff,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationDiff {
    missing_required: Vec<String>,
    rejected: Vec<String>,
    not_allowed: Vec<String>,
}

struct ContractValidator {
    required_exact: HashSet<String>,
    allowed_glob: GlobSet,
    rejected_glob: GlobSet,
    ignored_glob: GlobSet,
}

impl ContractValidator {
    fn new(contract: &Contract) -> Result<Self> {
        let names = &contract.spec.git.tree.objects.names;
        
        // Build required set (exact matches only)
        let required_exact: HashSet<String> = names.required.iter().cloned().collect();
        
        // Build glob sets
        let allowed_glob = Self::build_glob_set(&names.allowed)?;
        let rejected_glob = Self::build_glob_set(&names.rejected)?;
        let ignored_glob = Self::build_glob_set(&names.ignored)?;
        
        Ok(Self {
            required_exact,
            allowed_glob,
            rejected_glob,
            ignored_glob,
        })
    }
    
    fn build_glob_set(patterns: &[String]) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();

        for pattern in patterns {
            let glob = Glob::new(pattern)
                .with_context(|| format!("Invalid glob pattern: {}", pattern))?;
            builder.add(glob);
        }

        builder.build().context("Failed to build glob set")
    }
    
    fn validate_tree(&self, tree_report: &TreeReport) -> ValidationDiff {
        let mut seen_required = HashSet::new();
        let mut rejected = Vec::new();
        let mut not_allowed = Vec::new();

        for entry in &tree_report.entries {
            let name = &entry.name;

            // Skip ignored entries
            if self.ignored_glob.is_match(name) {
                continue;
            }

            // Check if explicitly rejected
            if self.rejected_glob.is_match(name) {
                rejected.push(name.clone());
                continue;
            }

            // Check if allowed
            if !self.allowed_glob.is_match(name) {
                not_allowed.push(name.clone());
                continue;
            }

            // Track required files we've seen
            if self.required_exact.contains(name) {
                seen_required.insert(name.clone());
            }
        }

        // Find missing required files
        let missing_required: Vec<String> = self
            .required_exact
            .difference(&seen_required)
            .cloned()
            .collect();

        ValidationDiff {
            missing_required,
            rejected,
            not_allowed,
        }
    }
}

fn read_tree_from_git(ref_name: &str) -> Result<TreeReport> {
    // Get the tree SHA for the reference
    let tree_sha = Command::new("git")
        .args(["rev-parse", &format!("{}^{{tree}}", ref_name)])
        .output()
        .context("Failed to get tree SHA")?;

    let tree_sha = String::from_utf8(tree_sha.stdout)
        .context("Invalid tree SHA output")?
        .trim()
        .to_string();

    // Get tree entries using git ls-tree
    let output = Command::new("git")
        .args(["ls-tree", "-z", &tree_sha])
        .output()
        .context("Failed to get tree entries")?;

    let entries_str = String::from_utf8(output.stdout)
        .context("Invalid tree entries output")?;

    // Parse entries (format: MODE TYPE SHA\tNAME\0)
    let mut entries = Vec::new();
    for entry in entries_str.split('\0') {
        if entry.is_empty() {
            continue;
    }
    
        let parts: Vec<&str> = entry.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }

        let header = parts[0];
        let name = parts[1];

        let header_parts: Vec<&str> = header.split_whitespace().collect();
        if header_parts.len() != 3 {
            continue;
        }

        let mode = header_parts[0];
        let entry_type = header_parts[1];
        let sha = header_parts[2];

        entries.push(TreeEntry {
            mode: mode.to_string(),
            entry_type: entry_type.to_string(),
            sha: sha.to_string(),
            name: name.to_string(),
        });
    }

    Ok(TreeReport {
        ref_name: ref_name.to_string(),
        tree_sha,
        entries,
    })
}

fn read_contract(contract_path: &str) -> Result<Contract> {
    let content = std::fs::read_to_string(contract_path)
        .with_context(|| format!("Failed to read contract: {}", contract_path))?;

    serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse contract JSON: {}", contract_path))
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <ref> <contract-path>", args[0]);
        eprintln!("Example: {} origin/main contracts/object-names@v1.json", args[0]);
        std::process::exit(1);
    }

    let ref_name = &args[1];
    let contract_path = &args[2];

    // Read and validate the contract
    let contract = read_contract(contract_path)?;
    let validator = ContractValidator::new(&contract)?;

    // Read the tree from Git
    let tree_report = read_tree_from_git(ref_name)?;

    // Validate the tree against the contract
    let diff = validator.validate_tree(&tree_report);

    // Output the result
    let result = ValidationResult {
        ref_name: tree_report.ref_name,
        tree_sha: tree_report.tree_sha,
        contract_name: contract.name,
        contract_version: contract.version,
        result: diff,
    };

    let output = serde_json::to_string_pretty(&result)?;
    println!("{}", output);

    Ok(())
}
