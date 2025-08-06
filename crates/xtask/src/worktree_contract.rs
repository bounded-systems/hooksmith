use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

/// Worktree naming contract rules
#[derive(Debug, Clone)]
pub struct WorktreeContract {
    /// Whether to strip prefixes like "feature/", "fix/", etc.
    pub strip_prefixes: bool,
    /// Prefixes to strip (e.g., ["feature/", "fix/", "hotfix/"])
    pub prefixes: Vec<String>,
    /// Whether to normalize to kebab-case
    pub normalize_case: bool,
    /// Whether to allow underscores to hyphens conversion
    pub underscore_to_hyphen: bool,
    /// Custom mapping rules (branch_name -> expected_dir_name)
    pub custom_mappings: HashMap<String, String>,
}

impl Default for WorktreeContract {
    fn default() -> Self {
        Self {
            strip_prefixes: true,
            prefixes: vec![
                "feature/".to_string(),
                "fix/".to_string(),
                "hotfix/".to_string(),
                "bugfix/".to_string(),
                "chore/".to_string(),
                "docs/".to_string(),
                "test/".to_string(),
            ],
            normalize_case: true,
            underscore_to_hyphen: true,
            custom_mappings: HashMap::new(),
        }
    }
}

impl WorktreeContract {
    /// Create a new contract with custom rules
    pub fn new(
        strip_prefixes: bool,
        prefixes: Vec<String>,
        normalize_case: bool,
        underscore_to_hyphen: bool,
    ) -> Self {
        Self {
            strip_prefixes,
            prefixes,
            normalize_case,
            underscore_to_hyphen,
            custom_mappings: HashMap::new(),
        }
    }

    /// Add a custom mapping rule
    pub fn add_mapping(&mut self, branch_name: &str, expected_dir_name: &str) {
        self.custom_mappings
            .insert(branch_name.to_string(), expected_dir_name.to_string());
    }

    /// Get the expected directory name for a branch
    pub fn get_expected_dir_name(&self, branch_name: &str) -> String {
        let mut name = branch_name.to_string();

        // Apply custom mappings first
        if let Some(mapped) = self.custom_mappings.get(&name) {
            return mapped.clone();
        }

        // Strip prefixes if enabled
        if self.strip_prefixes {
            for prefix in &self.prefixes {
                if name.starts_with(prefix) {
                    name = name[prefix.len()..].to_string();
                    break;
                }
            }
        }

        // Convert underscores to hyphens if enabled
        if self.underscore_to_hyphen {
            name = name.replace('_', "-");
        }

        // Normalize case if enabled
        if self.normalize_case {
            name = name.to_lowercase();
        }

        name
    }

    /// Validate a worktree's naming contract
    pub fn validate_worktree(
        &self,
        worktree_path: &str,
        branch_name: &str,
    ) -> Result<WorktreeValidationResult> {
        let expected_dir_name = self.get_expected_dir_name(branch_name);
        let actual_dir_name = Path::new(worktree_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let is_valid = actual_dir_name == expected_dir_name;
        let suggestion = if !is_valid {
            Some(format!("Expected: {}", expected_dir_name))
        } else {
            None
        };

        Ok(WorktreeValidationResult {
            worktree_path: worktree_path.to_string(),
            branch_name: branch_name.to_string(),
            actual_dir_name,
            expected_dir_name,
            is_valid,
            suggestion,
        })
    }

    /// Validate all worktrees in the repository
    pub fn validate_all_worktrees(&self) -> Result<Vec<WorktreeValidationResult>> {
        let output = Command::new("git")
            .args(["worktree", "list", "--porcelain"])
            .stdout(Stdio::piped())
            .output()
            .context("Failed to list worktrees")?;

        let worktree_list = String::from_utf8(output.stdout)?;
        let mut results = Vec::new();
        let mut current_worktree: Option<(String, String)> = None;

        for line in worktree_list.lines() {
            if line.starts_with("worktree ") {
                if let Some((path, branch)) = current_worktree.take() {
                    results.push(self.validate_worktree(&path, &branch)?);
                }
                let path = line[9..].trim();
                current_worktree = Some((path.to_string(), String::new()));
            } else if line.starts_with("branch ") {
                if let Some((path, _)) = &mut current_worktree {
                    let branch = line[8..].trim();
                    // Strip refs/heads/ prefix if present
                    let clean_branch = if branch.starts_with("refs/heads/") {
                        &branch[11..]
                    } else {
                        branch
                    };
                    *path = path.clone();
                    current_worktree = Some((path.clone(), clean_branch.to_string()));
                }
            }
        }

        // Handle the last worktree
        if let Some((path, branch)) = current_worktree {
            results.push(self.validate_worktree(&path, &branch)?);
        }

        Ok(results)
    }

    /// Check if a worktree would be valid before creating it
    pub fn validate_proposed_worktree(
        &self,
        proposed_path: &str,
        branch_name: &str,
    ) -> Result<WorktreeValidationResult> {
        let expected_dir_name = self.get_expected_dir_name(branch_name);
        let actual_dir_name = Path::new(proposed_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let is_valid = actual_dir_name == expected_dir_name;
        let suggestion = if !is_valid {
            Some(format!("Use: {}", expected_dir_name))
        } else {
            None
        };

        Ok(WorktreeValidationResult {
            worktree_path: proposed_path.to_string(),
            branch_name: branch_name.to_string(),
            actual_dir_name,
            expected_dir_name,
            is_valid,
            suggestion,
        })
    }

    /// Generate a valid worktree path for a branch
    pub fn generate_valid_path(&self, branch_name: &str, base_dir: &str) -> String {
        let dir_name = self.get_expected_dir_name(branch_name);

        if base_dir.ends_with('/') || base_dir.ends_with('\\') {
            format!("{}{}", base_dir, dir_name)
        } else {
            format!("{}/{}", base_dir, dir_name)
        }
    }
}

/// Result of worktree validation
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorktreeValidationResult {
    pub worktree_path: String,
    pub branch_name: String,
    pub actual_dir_name: String,
    pub expected_dir_name: String,
    pub is_valid: bool,
    pub suggestion: Option<String>,
}

impl WorktreeValidationResult {
    /// Get a human-readable message for this validation result
    pub fn message(&self) -> String {
        if self.is_valid {
            format!(
                "✅ {} matches {}",
                self.actual_dir_name, self.expected_dir_name
            )
        } else {
            format!(
                "❌ {} should be {}",
                self.actual_dir_name, self.expected_dir_name
            )
        }
    }

    /// Get a detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = format!(
            "Worktree: {}\nBranch: {}\nActual dir: {}\nExpected dir: {}\n",
            self.worktree_path, self.branch_name, self.actual_dir_name, self.expected_dir_name
        );

        if let Some(suggestion) = &self.suggestion {
            report.push_str(&format!("Suggestion: {}\n", suggestion));
        }

        report.push_str(&format!("Valid: {}\n", self.is_valid));
        report
    }
}

/// Post-checkout hook implementation
pub fn run_post_checkout_hook() -> Result<()> {
    let contract = WorktreeContract::default();

    // Get current worktree info
    let worktree_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .stdout(Stdio::piped())
        .output()
        .context("Failed to get worktree root")?;

    let worktree_path = String::from_utf8(worktree_output.stdout)?
        .trim()
        .to_string();

    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .stdout(Stdio::piped())
        .output()
        .context("Failed to get current branch")?;

    let branch_name = String::from_utf8(branch_output.stdout)?.trim().to_string();

    // Validate the worktree
    let result = contract.validate_worktree(&worktree_path, &branch_name)?;

    if !result.is_valid {
        eprintln!("❌ Worktree naming contract violation!");
        eprintln!("   Directory: {}", result.actual_dir_name);
        eprintln!("   Branch: {}", result.branch_name);
        eprintln!("   Expected: {}", result.expected_dir_name);
        if let Some(suggestion) = result.suggestion {
            eprintln!("   Suggestion: {}", suggestion);
        }
        std::process::exit(1);
    }

    println!("✅ Worktree naming contract validated");
    Ok(())
}

/// Audit all worktrees and report violations
pub fn audit_all_worktrees() -> Result<()> {
    let contract = WorktreeContract::default();
    let results = contract.validate_all_worktrees()?;

    let mut valid_count = 0;
    let mut invalid_count = 0;

    println!("🔍 Auditing worktree naming contracts...\n");

    for result in &results {
        if result.is_valid {
            valid_count += 1;
            println!("{}", result.message());
        } else {
            invalid_count += 1;
            println!("{}", result.message());
            if let Some(suggestion) = &result.suggestion {
                println!("   💡 {}", suggestion);
            }
        }
    }

    println!("\n📊 Summary:");
    println!("   Valid worktrees: {}", valid_count);
    println!("   Invalid worktrees: {}", invalid_count);
    println!("   Total worktrees: {}", results.len());

    if invalid_count > 0 {
        println!("\n❌ Worktree naming contract violations found!");
        std::process::exit(1);
    } else {
        println!("\n✅ All worktrees comply with naming contracts!");
    }

    Ok(())
}

/// Validate a proposed worktree before creation
pub fn validate_proposed_worktree(
    proposed_path: &str,
    branch_name: &str,
) -> Result<WorktreeValidationResult> {
    let contract = WorktreeContract::default();
    contract.validate_proposed_worktree(proposed_path, branch_name)
}

/// Generate a valid worktree path for a branch
pub fn generate_valid_worktree_path(branch_name: &str, base_dir: &str) -> String {
    let contract = WorktreeContract::default();
    contract.generate_valid_path(branch_name, base_dir)
}
