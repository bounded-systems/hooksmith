use std::collections::HashSet;
use std::env;
use std::process::Command;

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
    exceptions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidationResult {
    ref_name: String,
    tree_sha: String,
    contract_name: String,
    contract_version: String,
    result: ValidationDiff,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ValidationDiff {
    missing_required: Vec<String>,
    rejected: Vec<String>,
    not_allowed: Vec<String>,
    exceptions: Vec<String>,
}

struct ContractValidator {
    required_exact: HashSet<String>,
    allowed_glob: GlobSet,
    rejected_glob: GlobSet,
    ignored_glob: GlobSet,
    exceptions_glob: Option<GlobSet>,
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
        
        // Build exceptions glob set if present
        let exceptions_glob = if let Some(exceptions) = &names.exceptions {
            Some(Self::build_glob_set(exceptions)?)
        } else {
            None
        };

        Ok(Self {
            required_exact,
            allowed_glob,
            rejected_glob,
            ignored_glob,
            exceptions_glob,
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
        let mut exceptions = Vec::new();

        for entry in &tree_report.entries {
            let name = &entry.name;

            // Skip ignored entries
            if self.ignored_glob.is_match(name) {
                continue;
            }

            // Check if this is an exception (overrides rejected/allowed rules)
            if let Some(ref exceptions_glob) = self.exceptions_glob {
                if exceptions_glob.is_match(name) {
                    exceptions.push(name.clone());
                    // Track required files we've seen (even if they're exceptions)
                    if self.required_exact.contains(name) {
                        seen_required.insert(name.clone());
                    }
                    continue;
                }
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
            exceptions,
        }
    }
}

fn read_tree_from_git(ref_name: &str) -> Result<TreeReport> {
    // Get the tree SHA for the reference
    let tree_sha = Command::new("git")
        .current_dir("..")  // Run from parent directory
        .args(["rev-parse", &format!("{}^{{tree}}", ref_name)])
        .output()
        .context("Failed to get tree SHA")?;

    let tree_sha = String::from_utf8(tree_sha.stdout)
        .context("Invalid tree SHA output")?
        .trim()
        .to_string();

    // Get tree entries using git ls-tree
    let output = Command::new("git")
        .current_dir("..")  // Run from parent directory
        .args(["ls-tree", &tree_sha])
        .output()
        .context("Failed to get tree entries")?;

    let entries_str = String::from_utf8(output.stdout)
        .context("Invalid tree entries output")?;

    // Parse entries (format: MODE TYPE SHA\tNAME)
    let mut entries = Vec::new();
    
    for line in entries_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
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
            eprintln!("Example: {} origin/main .hooksmith/agreements/object-names@v1.json", args[0]);
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
        result: diff.clone(),
    };

    let output = serde_json::to_string_pretty(&result)?;
    println!("{}", output);

    // Also output a summary to stderr for CI/CD integration
    let total_violations = diff.missing_required.len() + diff.rejected.len() + diff.not_allowed.len();
    if total_violations == 0 {
        eprintln!("✅ Contract validation PASSED - No violations found");
        if !diff.exceptions.is_empty() {
            eprintln!("  - Exceptions allowed: {} files", diff.exceptions.len());
        }
    } else {
        eprintln!("❌ Contract validation FAILED - {} violations found:", total_violations);
        if !diff.missing_required.is_empty() {
            eprintln!("  - Missing required: {} files", diff.missing_required.len());
        }
        if !diff.rejected.is_empty() {
            eprintln!("  - Rejected: {} files", diff.rejected.len());
        }
        if !diff.not_allowed.is_empty() {
            eprintln!("  - Not allowed: {} files", diff.not_allowed.len());
        }
        if !diff.exceptions.is_empty() {
            eprintln!("  - Exceptions allowed: {} files", diff.exceptions.len());
        }
    }

    Ok(())
}
