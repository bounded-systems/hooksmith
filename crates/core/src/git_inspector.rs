use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::process::Command;

/// Git repository inspection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInspectionReport {
    pub refs: RefAnalysis,
    pub unreachable: UnreachableAnalysis,
    pub sync_status: SyncStatus,
    pub summary: InspectionSummary,
}

/// Analysis of Git references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefAnalysis {
    pub head: Option<String>,
    pub local_branches: Vec<String>,
    pub remote_branches: Vec<String>,
    pub tags: Vec<String>,
    pub notes: Vec<String>,
    pub total_refs: usize,
}

/// Analysis of unreachable objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreachableAnalysis {
    pub objects: Vec<UnreachableObject>,
    pub by_type: HashMap<String, usize>,
    pub total_unreachable: usize,
    pub oldest_object: Option<String>,
    pub newest_object: Option<String>,
}

/// Individual unreachable object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreachableObject {
    pub sha: String,
    pub object_type: String,
    pub path: Option<String>,
    pub size: Option<usize>,
    pub first_line: Option<String>,
}

/// Local/remote synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub local_only: Vec<String>,
    pub remote_only: Vec<String>,
    pub out_of_sync: Vec<String>,
    pub in_sync: Vec<String>,
    pub total_local: usize,
    pub total_remote: usize,
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionSummary {
    pub total_objects: usize,
    pub reachable_objects: usize,
    pub unreachable_objects: usize,
    pub local_branches: usize,
    pub remote_branches: usize,
    pub tags: usize,
    pub notes: usize,
    pub sync_score: f64, // 0.0 to 1.0
}

/// Git repository inspector
pub struct GitInspector;

impl GitInspector {
    /// Perform comprehensive repository inspection
    pub fn inspect_repository() -> Result<GitInspectionReport, Box<dyn std::error::Error + Send + Sync>> {
        let refs = Self::analyze_refs()?;
        let unreachable = Self::analyze_unreachable()?;
        let sync_status = Self::analyze_sync_status(&refs)?;
        let summary = Self::create_summary(&refs, &unreachable, &sync_status);

        Ok(GitInspectionReport {
            refs,
            unreachable,
            sync_status,
            summary,
        })
    }

    /// Analyze all Git references
    pub fn analyze_refs() -> Result<RefAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("git").args(["show-ref", "--head"]).output()?;

        if !output.status.success() {
            return Err(format!(
                "git show-ref failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut head = None;
        let mut local_branches = Vec::new();
        let mut remote_branches = Vec::new();
        let mut tags = Vec::new();
        let mut notes = Vec::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let sha = parts[0];
                let ref_name = parts[1];

                match ref_name {
                    "HEAD" => head = Some(sha.to_string()),
                    s if s.starts_with("refs/heads/") => {
                        local_branches.push(s.replace("refs/heads/", ""));
                    }
                    s if s.starts_with("refs/remotes/") => {
                        remote_branches.push(s.replace("refs/remotes/", ""));
                    }
                    s if s.starts_with("refs/tags/") => {
                        tags.push(s.replace("refs/tags/", ""));
                    }
                    s if s.starts_with("refs/notes/") => {
                        notes.push(s.replace("refs/notes/", ""));
                    }
                    _ => {}
                }
            }
        }

        let total_refs = local_branches.len()
            + remote_branches.len()
            + tags.len()
            + notes.len()
            + if head.is_some() { 1 } else { 0 };

        Ok(RefAnalysis {
            head,
            local_branches,
            remote_branches,
            tags,
            notes,
            total_refs,
        })
    }

    /// Analyze unreachable objects
    pub fn analyze_unreachable() -> Result<UnreachableAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("git")
            .args(["fsck", "--unreachable"])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "git fsck failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        let stdout = String::from_utf8(output.stdout)?;
        let mut objects = Vec::new();
        let mut by_type = HashMap::new();

        for line in stdout.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let object_type = parts[1];
                let sha = parts[2];

                // Get object details
                let path = Self::get_object_path(sha)?;
                let size = Self::get_object_size(sha)?;
                let first_line = Self::get_object_first_line(sha, object_type)?;

                let object = UnreachableObject {
                    sha: sha.to_string(),
                    object_type: object_type.to_string(),
                    path,
                    size,
                    first_line,
                };

                objects.push(object);
                *by_type.entry(object_type.to_string()).or_insert(0) += 1;
            }
        }

        let total_unreachable = objects.len();

        Ok(UnreachableAnalysis {
            objects,
            by_type,
            total_unreachable,
            oldest_object: None, // Would need to analyze commit dates
            newest_object: None,
        })
    }

    /// Analyze local/remote synchronization
    fn analyze_sync_status(refs: &RefAnalysis) -> Result<SyncStatus, Box<dyn std::error::Error + Send + Sync>> {
        let mut local_only = Vec::new();
        let mut remote_only = Vec::new();
        let out_of_sync = Vec::new();
        let in_sync = Vec::new();

        // Create sets for comparison
        let local_set: HashSet<&str> = refs.local_branches.iter().map(|s| s.as_str()).collect();
        let remote_set: HashSet<&str> = refs
            .remote_branches
            .iter()
            .filter_map(|s| s.strip_prefix("origin/"))
            .collect();

        // Find local-only branches
        for branch in &refs.local_branches {
            if !remote_set.contains(branch.as_str()) {
                local_only.push(branch.clone());
            }
        }

        // Find remote-only branches
        for branch in &refs.remote_branches {
            let branch_name = branch.strip_prefix("origin/").unwrap_or(branch);
            if !local_set.contains(branch_name) {
                remote_only.push(branch.clone());
            }
        }

        // Check for out-of-sync branches (would need to compare SHAs)
        // This is a simplified version - full implementation would compare commit SHAs

        Ok(SyncStatus {
            local_only,
            remote_only,
            out_of_sync,
            in_sync,
            total_local: refs.local_branches.len(),
            total_remote: refs.remote_branches.len(),
        })
    }

    /// Create summary statistics
    fn create_summary(
        refs: &RefAnalysis,
        unreachable: &UnreachableAnalysis,
        sync: &SyncStatus,
    ) -> InspectionSummary {
        let total_objects = refs.total_refs + unreachable.total_unreachable;
        let reachable_objects = refs.total_refs;
        let unreachable_objects = unreachable.total_unreachable;

        let sync_score = if sync.total_local + sync.total_remote > 0 {
            let synced = sync.in_sync.len();
            let total = sync.total_local + sync.total_remote;
            synced as f64 / total as f64
        } else {
            1.0
        };

        InspectionSummary {
            total_objects,
            reachable_objects,
            unreachable_objects,
            local_branches: refs.local_branches.len(),
            remote_branches: refs.remote_branches.len(),
            tags: refs.tags.len(),
            notes: refs.notes.len(),
            sync_score,
        }
    }

    /// Get object path (for blobs)
    fn get_object_path(_sha: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        // This would need to traverse the object graph to find paths
        // For now, return None
        Ok(None)
    }

    /// Get object size
    fn get_object_size(sha: &str) -> Result<Option<usize>, Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("git").args(["cat-file", "-s", sha]).output()?;

        if output.status.success() {
            let size_str = String::from_utf8(output.stdout)?.trim().to_string();
            if let Ok(size) = size_str.parse::<usize>() {
                Ok(Some(size))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Get first line of object content
    fn get_object_first_line(
        sha: &str,
        object_type: &str,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        if object_type == "blob" {
            let output = Command::new("git").args(["cat-file", "-p", sha]).output()?;

            if output.status.success() {
                let content = String::from_utf8(output.stdout)?;
                let first_line = content.lines().next().unwrap_or("").to_string();
                if !first_line.is_empty() {
                    Ok(Some(first_line))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Generate recovery suggestions for unreachable objects
    pub fn generate_recovery_suggestions(unreachable: &UnreachableAnalysis) -> Vec<String> {
        let mut suggestions = Vec::new();

        if unreachable.total_unreachable > 0 {
            suggestions.push(format!(
                "Found {} unreachable objects",
                unreachable.total_unreachable
            ));

            if let Some(blob_count) = unreachable.by_type.get("blob") {
                if *blob_count > 0 {
                    suggestions.push(format!(
                        "- {} blob(s) may contain lost file content",
                        blob_count
                    ));
                }
            }

            if let Some(tree_count) = unreachable.by_type.get("tree") {
                if *tree_count > 0 {
                    suggestions.push(format!(
                        "- {} tree(s) may represent lost directory structures",
                        tree_count
                    ));
                }
            }

            suggestions.push("Consider running: git fsck --unreachable | grep blob | while read sha; do git cat-file -p $sha | head -5; done".to_string());
        }

        suggestions
    }

    /// Check if repository is clean (no unreachable objects)
    pub fn is_repository_clean() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let unreachable = Self::analyze_unreachable()?;
        Ok(unreachable.total_unreachable == 0)
    }

    /// Get local/remote branch comparison
    pub fn get_branch_comparison(
    ) -> Result<HashMap<String, BranchStatus>, Box<dyn std::error::Error + Send + Sync>> {
        let refs = Self::analyze_refs()?;
        let mut comparison = HashMap::new();

        let local_set: HashSet<&str> = refs.local_branches.iter().map(|s| s.as_str()).collect();
        let remote_set: HashSet<&str> = refs
            .remote_branches
            .iter()
            .filter_map(|s| s.strip_prefix("origin/"))
            .collect();

        for branch in &refs.local_branches {
            if remote_set.contains(branch.as_str()) {
                comparison.insert(branch.clone(), BranchStatus::Synced);
            } else {
                comparison.insert(branch.clone(), BranchStatus::LocalOnly);
            }
        }

        for branch in &refs.remote_branches {
            let branch_name = branch.strip_prefix("origin/").unwrap_or(branch);
            if !local_set.contains(branch_name) {
                comparison.insert(branch_name.to_string(), BranchStatus::RemoteOnly);
            }
        }

        Ok(comparison)
    }
}

/// Branch synchronization status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BranchStatus {
    Synced,
    LocalOnly,
    RemoteOnly,
    OutOfSync,
}

/// Format inspection report as markdown
pub fn format_inspection_report_markdown(report: &GitInspectionReport) -> String {
    let mut output = String::new();

    output.push_str("# Git Repository Inspection Report\n\n");

    // Summary
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- **Total Objects**: {}\n",
        report.summary.total_objects
    ));
    output.push_str(&format!(
        "- **Reachable Objects**: {}\n",
        report.summary.reachable_objects
    ));
    output.push_str(&format!(
        "- **Unreachable Objects**: {}\n",
        report.summary.unreachable_objects
    ));
    output.push_str(&format!(
        "- **Sync Score**: {:.1}%\n",
        report.summary.sync_score * 100.0
    ));

    // Refs
    output.push_str("\n## References\n\n");
    output.push_str(&format!(
        "- **HEAD**: {}\n",
        report.refs.head.as_deref().unwrap_or("None")
    ));
    output.push_str(&format!(
        "- **Local Branches**: {}\n",
        report.refs.local_branches.len()
    ));
    output.push_str(&format!(
        "- **Remote Branches**: {}\n",
        report.refs.remote_branches.len()
    ));
    output.push_str(&format!("- **Tags**: {}\n", report.refs.tags.len()));
    output.push_str(&format!("- **Notes**: {}\n", report.refs.notes.len()));

    // Sync Status
    output.push_str("\n## Synchronization Status\n\n");
    if !report.sync_status.local_only.is_empty() {
        output.push_str("### Local-Only Branches\n");
        for branch in &report.sync_status.local_only {
            output.push_str(&format!("- {}\n", branch));
        }
        output.push_str("\n");
    }

    if !report.sync_status.remote_only.is_empty() {
        output.push_str("### Remote-Only Branches\n");
        for branch in &report.sync_status.remote_only {
            output.push_str(&format!("- {}\n", branch));
        }
        output.push_str("\n");
    }

    // Unreachable Objects
    if report.unreachable.total_unreachable > 0 {
        output.push_str("## Unreachable Objects\n\n");
        output.push_str(&format!(
            "**Total**: {}\n\n",
            report.unreachable.total_unreachable
        ));

        for (object_type, count) in &report.unreachable.by_type {
            output.push_str(&format!("- **{}**: {}\n", object_type, count));
        }

        output.push_str("\n### Recovery Suggestions\n");
        let suggestions = GitInspector::generate_recovery_suggestions(&report.unreachable);
        for suggestion in suggestions {
            output.push_str(&format!("- {}\n", suggestion));
        }
    }

    output
}
