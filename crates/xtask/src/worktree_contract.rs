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
    /// PR title template
    pub pr_title_template: Option<String>,
    /// Whether to auto-create PRs
    pub auto_create_pr: bool,
    /// Whether to auto-lock worktrees
    pub auto_lock_worktree: bool,
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
            pr_title_template: Some("{branch_name} - {description}".to_string()),
            auto_create_pr: false,
            auto_lock_worktree: true,
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
            pr_title_template: Some("{branch_name} - {description}".to_string()),
            auto_create_pr: false,
            auto_lock_worktree: true,
        }
    }

    /// Add a custom mapping rule
    pub fn add_mapping(&mut self, branch_name: &str, expected_dir_name: &str) {
        self.custom_mappings
            .insert(branch_name.to_string(), expected_dir_name.to_string());
    }

    /// Set PR title template
    pub fn set_pr_title_template(&mut self, template: &str) {
        self.pr_title_template = Some(template.to_string());
    }

    /// Enable/disable auto PR creation
    pub fn set_auto_create_pr(&mut self, enabled: bool) {
        self.auto_create_pr = enabled;
    }

    /// Enable/disable auto worktree locking
    pub fn set_auto_lock_worktree(&mut self, enabled: bool) {
        self.auto_lock_worktree = enabled;
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

    /// Generate PR title from branch name
    pub fn generate_pr_title(&self, branch_name: &str) -> String {
        if let Some(template) = &self.pr_title_template {
            let description = self.generate_description_from_branch(branch_name);
            template
                .replace("{branch_name}", branch_name)
                .replace("{description}", &description)
        } else {
            branch_name.to_string()
        }
    }

    /// Generate description from branch name
    fn generate_description_from_branch(&self, branch_name: &str) -> String {
        let mut description = branch_name.to_string();

        // Strip prefixes for description
        for prefix in &self.prefixes {
            if description.starts_with(prefix) {
                description = description[prefix.len()..].to_string();
                break;
            }
        }

        // Convert to title case
        description = description
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");

        description
    }

    /// Create a PR for a worktree
    pub fn create_pr_for_worktree(&self, worktree_path: &str, branch_name: &str) -> Result<()> {
        let pr_title = self.generate_pr_title(branch_name);

        println!("🚀 Creating PR for worktree: {}", worktree_path);
        println!("   Branch: {}", branch_name);
        println!("   Title: {}", pr_title);

        // Check if gh CLI is available
        let gh_check = Command::new("gh")
            .args(["version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if gh_check.is_err() {
            return Err(anyhow::anyhow!(
                "GitHub CLI (gh) is not installed or not in PATH"
            ));
        }

        // Create PR using gh CLI
        let output = Command::new("gh")
            .args([
                "pr",
                "create",
                "--title",
                &pr_title,
                "--body",
                &format!("Worktree: {}\nBranch: {}", worktree_path, branch_name),
                "--base",
                "main",
                "--head",
                branch_name,
            ])
            .current_dir(worktree_path)
            .output()
            .context("Failed to create PR")?;

        if output.status.success() {
            let pr_url = String::from_utf8_lossy(&output.stdout);
            println!("✅ PR created successfully!");
            println!("   URL: {}", pr_url.trim());

            // Lock the worktree if auto-lock is enabled
            if self.auto_lock_worktree {
                self.lock_worktree(worktree_path)?;
            }
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create PR: {}", error));
        }

        Ok(())
    }

    /// Lock a worktree
    pub fn lock_worktree(&self, worktree_path: &str) -> Result<()> {
        println!("🔒 Locking worktree: {}", worktree_path);

        let output = Command::new("git")
            .args(["worktree", "lock", worktree_path])
            .output()
            .context("Failed to lock worktree")?;

        if output.status.success() {
            println!("✅ Worktree locked successfully");
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to lock worktree: {}", error));
        }

        Ok(())
    }

    /// Unlock a worktree
    pub fn unlock_worktree(&self, worktree_path: &str) -> Result<()> {
        println!("🔓 Unlocking worktree: {}", worktree_path);

        let output = Command::new("git")
            .args(["worktree", "unlock", worktree_path])
            .output()
            .context("Failed to unlock worktree")?;

        if output.status.success() {
            println!("✅ Worktree unlocked successfully");
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to unlock worktree: {}", error));
        }

        Ok(())
    }

    /// Merge PR and cleanup worktree
    pub fn merge_pr_and_cleanup(&self, branch_name: &str, worktree_path: &str) -> Result<()> {
        println!("🔄 Merging PR and cleaning up worktree: {}", worktree_path);

        // Check if gh CLI is available
        let gh_check = Command::new("gh")
            .args(["version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if gh_check.is_err() {
            return Err(anyhow::anyhow!(
                "GitHub CLI (gh) is not installed or not in PATH"
            ));
        }

        // Merge PR using gh CLI
        let merge_output = Command::new("gh")
            .args(["pr", "merge", branch_name, "--squash", "--delete-branch"])
            .output()
            .context("Failed to merge PR")?;

        if !merge_output.status.success() {
            let error = String::from_utf8_lossy(&merge_output.stderr);
            return Err(anyhow::anyhow!("Failed to merge PR: {}", error));
        }

        println!("✅ PR merged successfully");

        // Unlock worktree
        self.unlock_worktree(worktree_path)?;

        // Remove worktree
        let remove_output = Command::new("git")
            .args(["worktree", "remove", worktree_path])
            .output()
            .context("Failed to remove worktree")?;

        if remove_output.status.success() {
            println!("✅ Worktree removed successfully");
        } else {
            let error = String::from_utf8_lossy(&remove_output.stderr);
            return Err(anyhow::anyhow!("Failed to remove worktree: {}", error));
        }

        Ok(())
    }

    /// Switch to next worktree (remove current, add new, open in Cursor)
    pub fn switch_next_worktree(&self, new_branch: &str, base_dir: &str) -> Result<()> {
        println!("🔄 Switching to next worktree: {}", new_branch);

        // Step 1: Detect current worktree
        let current_dir = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .stdout(Stdio::piped())
            .output()
            .context("Failed to get current worktree")?;

        let current_path = String::from_utf8(current_dir.stdout)?.trim().to_string();

        // Check if we're in a .wt/* worktree
        if !current_path.contains("/.wt/") {
            return Err(anyhow::anyhow!("❌ Not inside a .wt/* worktree"));
        }

        let current_branch = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .stdout(Stdio::piped())
            .output()
            .context("Failed to get current branch")?;

        let current_branch_name = String::from_utf8(current_branch.stdout)?.trim().to_string();
        let current_wt_name = Path::new(&current_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        println!(
            "📍 Current worktree: {} (branch: {})",
            current_wt_name, current_branch_name
        );

        // Step 2: Check for uncommitted changes
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&current_path)
            .output()
            .context("Failed to check worktree status")?;

        let status = String::from_utf8(status_output.stdout)?;
        if !status.trim().is_empty() {
            println!("⚠️  WARNING: Current worktree has uncommitted changes!");
            println!("   Changes:");
            for line in status.lines() {
                if !line.trim().is_empty() {
                    println!("      {}", line);
                }
            }
            println!("   Consider committing or stashing changes before switching");
        }

        // Step 3: Remove current worktree
        println!("🗑️  Removing current worktree: {}", current_wt_name);

        // Unlock if needed
        let unlock_output = Command::new("git")
            .args(["worktree", "unlock", &current_path])
            .output();

        if unlock_output.is_ok() && unlock_output.unwrap().status.success() {
            println!("🔓 Unlocked worktree");
        }

        // Remove worktree
        let remove_output = Command::new("git")
            .args(["worktree", "remove", &current_path])
            .output()
            .context("Failed to remove current worktree")?;

        if !remove_output.status.success() {
            let error = String::from_utf8_lossy(&remove_output.stderr);
            return Err(anyhow::anyhow!("Failed to remove worktree: {}", error));
        }

        println!("✅ Removed current worktree");

        // Step 4: Add new worktree
        let new_wt_path = self.generate_valid_path(new_branch, base_dir);
        println!("📦 Creating new worktree: {}", new_wt_path);

        // Check if branch exists
        let branch_check = Command::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{}", new_branch),
            ])
            .output();

        let branch_exists = branch_check.is_ok() && branch_check.unwrap().status.success();

        if branch_exists {
            println!("📦 Using existing branch: {}", new_branch);
            let add_output = Command::new("git")
                .args(["worktree", "add", &new_wt_path, new_branch])
                .output()
                .context("Failed to add worktree for existing branch")?;

            if !add_output.status.success() {
                let error = String::from_utf8_lossy(&add_output.stderr);
                return Err(anyhow::anyhow!("Failed to add worktree: {}", error));
            }
        } else {
            println!("🌱 Creating new branch: {} from main", new_branch);
            let add_output = Command::new("git")
                .args(["worktree", "add", &new_wt_path, "-b", new_branch, "main"])
                .output()
                .context("Failed to add worktree for new branch")?;

            if !add_output.status.success() {
                let error = String::from_utf8_lossy(&add_output.stderr);
                return Err(anyhow::anyhow!("Failed to add worktree: {}", error));
            }
        }

        // Step 5: Lock the new worktree
        self.lock_worktree(&new_wt_path)?;

        // Step 6: Open in Cursor
        println!("🚀 Opening new worktree in Cursor: {}", new_wt_path);

        #[cfg(target_os = "macos")]
        {
            let cursor_output = Command::new("open")
                .args(["-na", "Cursor", "--args", &new_wt_path])
                .output()
                .context("Failed to open Cursor")?;

            if !cursor_output.status.success() {
                println!("⚠️  Failed to open Cursor, but worktree is ready");
            } else {
                println!("✅ Opened in Cursor");
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Try to open with code (VSCode) as fallback
            let code_output = Command::new("code").args([&new_wt_path]).output();

            if code_output.is_ok() && code_output.unwrap().status.success() {
                println!("✅ Opened in VSCode");
            } else {
                println!("📁 Worktree ready at: {}", new_wt_path);
                println!("   Open manually in your preferred editor");
            }
        }

        // Step 7: Print summary
        println!("\n✅ Successfully switched worktrees!");
        println!("   From: {} ({})", current_wt_name, current_branch_name);
        println!(
            "   To: {} ({})",
            Path::new(&new_wt_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            new_branch
        );
        println!("   Path: {}", new_wt_path);

        Ok(())
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
                    if !branch.is_empty() {
                        results.push(self.validate_worktree(&path, &branch)?);
                    }
                }
                let path = line[9..].trim();
                current_worktree = Some((path.to_string(), String::new()));
            } else if line.starts_with("branch ") {
                if let Some((path, _)) = &mut current_worktree {
                    let branch = line[8..].trim();
                    // Strip refs/heads/ prefix if present (handle both refs and efs typos)
                    let clean_branch = if branch.starts_with("refs/heads/") {
                        &branch[11..]
                    } else if branch.starts_with("efs/heads/") {
                        &branch[10..]
                    } else {
                        branch
                    };
                    current_worktree = Some((path.clone(), clean_branch.to_string()));
                }
            }
        }

        // Handle the last worktree
        if let Some((path, branch)) = current_worktree {
            if !branch.is_empty() {
                results.push(self.validate_worktree(&path, &branch)?);
            }
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

/// Create PR for a worktree
pub fn create_pr_for_worktree(worktree_path: &str, branch_name: &str) -> Result<()> {
    let contract = WorktreeContract::default();
    contract.create_pr_for_worktree(worktree_path, branch_name)
}

/// Merge PR and cleanup worktree
pub fn merge_pr_and_cleanup(branch_name: &str, worktree_path: &str) -> Result<()> {
    let contract = WorktreeContract::default();
    contract.merge_pr_and_cleanup(branch_name, worktree_path)
}

/// Switch to next worktree (remove current, add new, open in Cursor)
pub fn switch_next_worktree(new_branch: &str, base_dir: &str) -> Result<()> {
    let contract = WorktreeContract::default();
    contract.switch_next_worktree(new_branch, base_dir)
}
