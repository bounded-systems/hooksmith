use crate::modules::contract_validation::Contract;
use crate::modules::git_model::{GitPath, GitTree};
use git2::{Commit, Repository, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

/// Analysis of file churn patterns for split planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChurnAnalysis {
    pub file_path: PathBuf,
    pub commit_count: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub volatility_score: f64,
    pub contract_count: u64,
    pub dependency_count: u64,
}

/// Suggested crate split with rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateSplitSuggestion {
    pub crate_name: String,
    pub files: Vec<PathBuf>,
    pub rationale: String,
    pub estimated_loc: u64,
    pub contract_count: u64,
    pub volatility_score: f64,
    pub dependency_risk: DependencyRisk,
    pub recommended: bool,
}

/// Risk assessment for dependency cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyRisk {
    Low,
    Medium,
    High,
    CycleDetected,
}

/// Configuration for split planning analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitPlannerConfig {
    pub max_crate_size: u64,
    pub min_crate_size: u64,
    pub max_volatility_threshold: f64,
    pub contract_density_threshold: f64,
    pub enable_dependency_checks: bool,
    pub enable_churn_tracking: bool,
}

impl Default for SplitPlannerConfig {
    fn default() -> Self {
        Self {
            max_crate_size: 1000,
            min_crate_size: 100,
            max_volatility_threshold: 0.8,
            contract_density_threshold: 0.3,
            enable_dependency_checks: true,
            enable_churn_tracking: true,
        }
    }
}

/// Tree split planner for analyzing and suggesting crate boundaries
pub struct TreeSplitPlanner {
    repo: Repository,
    config: SplitPlannerConfig,
    churn_cache: HashMap<PathBuf, ChurnAnalysis>,
}

impl TreeSplitPlanner {
    pub fn new(repo: Repository, config: Option<SplitPlannerConfig>) -> Self {
        Self {
            repo,
            config: config.unwrap_or_default(),
            churn_cache: HashMap::new(),
        }
    }

    /// Analyze churn patterns for files in a tree
    pub fn analyze_churn(&mut self, tree_sha: &str) -> Result<Vec<ChurnAnalysis>, String> {
        let tree = self.repo.find_tree(git2::Oid::from_str(tree_sha)?)?;
        let mut analyses = Vec::new();

        self.walk_tree_recursive(&tree, PathBuf::new(), &mut analyses)?;

        Ok(analyses)
    }

    /// Walk tree recursively to analyze all files
    fn walk_tree_recursive(
        &mut self,
        tree: &Tree,
        current_path: PathBuf,
        analyses: &mut Vec<ChurnAnalysis>,
    ) -> Result<(), String> {
        for entry in tree.iter() {
            let entry_path = current_path.join(entry.name().unwrap());

            match entry.kind() {
                Some(git2::ObjectType::Blob) => {
                    if self.is_rust_file(&entry_path) {
                        let analysis = self.analyze_file_churn(&entry_path)?;
                        analyses.push(analysis);
                    }
                }
                Some(git2::ObjectType::Tree) => {
                    let subtree = self.repo.find_tree(entry.id())?;
                    self.walk_tree_recursive(&subtree, entry_path, analyses)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Analyze churn for a specific file
    fn analyze_file_churn(&mut self, file_path: &Path) -> Result<ChurnAnalysis, String> {
        if let Some(cached) = self.churn_cache.get(file_path) {
            return Ok(cached.clone());
        }

        let commit_count = self.count_file_commits(file_path)?;
        let last_modified = self.get_last_modified(file_path)?;
        let volatility_score = self.calculate_volatility_score(file_path, commit_count)?;
        let contract_count = self.count_contracts_in_file(file_path)?;
        let dependency_count = self.count_dependencies(file_path)?;

        let analysis = ChurnAnalysis {
            file_path: file_path.to_path_buf(),
            commit_count,
            last_modified,
            volatility_score,
            contract_count,
            dependency_count,
        };

        self.churn_cache
            .insert(file_path.to_path_buf(), analysis.clone());
        Ok(analysis)
    }

    /// Count commits that touched a file
    fn count_file_commits(&self, file_path: &Path) -> Result<u64, String> {
        let mut revwalk = self.repo.revwalk().map_err(|e| e.to_string())?;
        revwalk.push_head().map_err(|e| e.to_string())?;

        let mut count = 0;
        for commit_id in revwalk {
            let commit = self
                .repo
                .find_commit(commit_id.map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let parent = commit.parent(0).ok();

            if let Some(parent) = parent {
                let diff = self
                    .repo
                    .diff_tree_to_tree(
                        Some(&commit.tree().map_err(|e| e.to_string())?),
                        Some(&parent.tree().map_err(|e| e.to_string())?),
                        None,
                    )
                    .map_err(|e| e.to_string())?;

                for delta in diff.deltas() {
                    if let Some(path) = delta.new_file().path() {
                        if path == file_path {
                            count += 1;
                            break;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get last modified timestamp for a file
    fn get_last_modified(&self, file_path: &Path) -> Result<chrono::DateTime<chrono::Utc>, String> {
        let mut revwalk = self.repo.revwalk().map_err(|e| e.to_string())?;
        revwalk.push_head().map_err(|e| e.to_string())?;

        for commit_id in revwalk {
            let commit = self
                .repo
                .find_commit(commit_id.map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            let parent = commit.parent(0).ok();

            if let Some(parent) = parent {
                let diff = self
                    .repo
                    .diff_tree_to_tree(
                        Some(&commit.tree().map_err(|e| e.to_string())?),
                        Some(&parent.tree().map_err(|e| e.to_string())?),
                        None,
                    )
                    .map_err(|e| e.to_string())?;

                for delta in diff.deltas() {
                    if let Some(path) = delta.new_file().path() {
                        if path == file_path {
                            return Ok(chrono::DateTime::from_timestamp(
                                commit.time().seconds(),
                                0,
                            )
                            .unwrap_or_default());
                        }
                    }
                }
            }
        }

        Ok(chrono::Utc::now())
    }

    /// Calculate volatility score based on commit frequency and recency
    fn calculate_volatility_score(
        &self,
        file_path: &Path,
        commit_count: u64,
    ) -> Result<f64, String> {
        let last_modified = self.get_last_modified(file_path)?;
        let days_since_modified = (chrono::Utc::now() - last_modified).num_days() as f64;

        // Higher score for more commits and recent modifications
        let commit_factor = (commit_count as f64).min(100.0) / 100.0;
        let recency_factor = 1.0 / (1.0 + days_since_modified / 30.0);

        Ok((commit_factor + recency_factor) / 2.0)
    }

    /// Count contracts in a file (placeholder implementation)
    fn count_contracts_in_file(&self, _file_path: &Path) -> Result<u64, String> {
        // TODO: Implement actual contract counting
        Ok(0)
    }

    /// Count dependencies for a file (placeholder implementation)
    fn count_dependencies(&self, _file_path: &Path) -> Result<u64, String> {
        // TODO: Implement actual dependency counting
        Ok(0)
    }

    /// Check if file is a Rust source file
    fn is_rust_file(&self, file_path: &Path) -> bool {
        file_path.extension().map_or(false, |ext| ext == "rs")
    }

    /// Generate split suggestions based on churn analysis
    pub fn generate_split_suggestions(
        &mut self,
        tree_sha: &str,
    ) -> Result<Vec<CrateSplitSuggestion>, String> {
        let analyses = self.analyze_churn(tree_sha)?;
        let mut suggestions = Vec::new();

        // Group files by volatility and contract density
        let mut high_volatility = Vec::new();
        let mut high_contract_density = Vec::new();
        let mut stable_files = Vec::new();

        for analysis in analyses {
            if analysis.volatility_score > self.config.max_volatility_threshold {
                high_volatility.push(analysis);
            } else if analysis.contract_count > 0 {
                high_contract_density.push(analysis);
            } else {
                stable_files.push(analysis);
            }
        }

        // Suggest splits for high volatility files
        if !high_volatility.is_empty() {
            suggestions.push(self.create_volatility_split(&high_volatility));
        }

        // Suggest splits for high contract density
        if !high_contract_density.is_empty() {
            suggestions.push(self.create_contract_split(&high_contract_density));
        }

        // Check for dependency cycles
        if self.config.enable_dependency_checks {
            self.check_dependency_cycles(&mut suggestions)?;
        }

        Ok(suggestions)
    }

    /// Create split suggestion for high volatility files
    fn create_volatility_split(&self, files: &[ChurnAnalysis]) -> CrateSplitSuggestion {
        let total_loc: u64 = files.iter().map(|f| self.estimate_loc(&f.file_path)).sum();
        let total_contracts: u64 = files.iter().map(|f| f.contract_count).sum();
        let avg_volatility: f64 =
            files.iter().map(|f| f.volatility_score).sum::<f64>() / files.len() as f64;

        CrateSplitSuggestion {
            crate_name: "high_volatility".to_string(),
            files: files.iter().map(|f| f.file_path.clone()).collect(),
            rationale: format!("High volatility files (avg score: {:.2})", avg_volatility),
            estimated_loc: total_loc,
            contract_count: total_contracts,
            volatility_score: avg_volatility,
            dependency_risk: DependencyRisk::Medium,
            recommended: total_loc > self.config.min_crate_size,
        }
    }

    /// Create split suggestion for high contract density files
    fn create_contract_split(&self, files: &[ChurnAnalysis]) -> CrateSplitSuggestion {
        let total_loc: u64 = files.iter().map(|f| self.estimate_loc(&f.file_path)).sum();
        let total_contracts: u64 = files.iter().map(|f| f.contract_count).sum();

        CrateSplitSuggestion {
            crate_name: "contract_dense".to_string(),
            files: files.iter().map(|f| f.file_path.clone()).collect(),
            rationale: format!("High contract density ({} contracts)", total_contracts),
            estimated_loc: total_loc,
            contract_count: total_contracts,
            volatility_score: 0.0,
            dependency_risk: DependencyRisk::Low,
            recommended: total_contracts > 0,
        }
    }

    /// Estimate lines of code for a file
    fn estimate_loc(&self, file_path: &Path) -> u64 {
        // Simple estimation - count lines in file
        std::fs::read_to_string(file_path)
            .map(|content| content.lines().count() as u64)
            .unwrap_or(0)
    }

    /// Check for dependency cycles in suggested splits
    fn check_dependency_cycles(
        &self,
        suggestions: &mut [CrateSplitSuggestion],
    ) -> Result<(), String> {
        for suggestion in suggestions.iter_mut() {
            let cycle_detected = self.detect_cycles_in_files(&suggestion.files)?;
            suggestion.dependency_risk = if cycle_detected {
                DependencyRisk::CycleDetected
            } else {
                self.assess_dependency_risk(&suggestion.files)?
            };
        }
        Ok(())
    }

    /// Detect cycles in file dependencies
    fn detect_cycles_in_files(&self, files: &[PathBuf]) -> Result<bool, String> {
        // Build dependency graph
        let mut graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        for file in files {
            let dependencies = self.extract_file_dependencies(file)?;
            graph.insert(file.clone(), dependencies);
        }

        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for file in files {
            if !visited.contains(file) {
                if self.has_cycle_dfs(file, &graph, &mut visited, &mut rec_stack)? {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Extract dependencies for a file (placeholder)
    fn extract_file_dependencies(&self, _file_path: &Path) -> Result<Vec<PathBuf>, String> {
        // TODO: Implement actual dependency extraction
        Ok(Vec::new())
    }

    /// DFS to detect cycles
    fn has_cycle_dfs(
        &self,
        file: &PathBuf,
        graph: &HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
    ) -> Result<bool, String> {
        visited.insert(file.clone());
        rec_stack.insert(file.clone());

        if let Some(dependencies) = graph.get(file) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if self.has_cycle_dfs(dep, graph, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(dep) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(file);
        Ok(false)
    }

    /// Assess dependency risk level
    fn assess_dependency_risk(&self, files: &[PathBuf]) -> Result<DependencyRisk, String> {
        let mut total_deps = 0;
        let mut external_deps = 0;

        for file in files {
            let deps = self.extract_file_dependencies(file)?;
            total_deps += deps.len();

            // Count external dependencies (outside the suggested crate)
            external_deps += deps.iter().filter(|dep| !files.contains(dep)).count();
        }

        let external_ratio = if total_deps > 0 {
            external_deps as f64 / total_deps as f64
        } else {
            0.0
        };

        Ok(match external_ratio {
            r if r < 0.2 => DependencyRisk::Low,
            r if r < 0.5 => DependencyRisk::Medium,
            _ => DependencyRisk::High,
        })
    }

    /// Generate warnings for problematic crates
    pub fn generate_warnings(&self, suggestions: &[CrateSplitSuggestion]) -> Vec<String> {
        let mut warnings = Vec::new();

        for suggestion in suggestions {
            if suggestion.estimated_loc > self.config.max_crate_size {
                warnings.push(format!(
                    "Crate '{}' exceeds size limit ({} LOC > {} LOC)",
                    suggestion.crate_name, suggestion.estimated_loc, self.config.max_crate_size
                ));
            }

            if suggestion.contract_count > 1 {
                warnings.push(format!(
                    "Crate '{}' has multiple contracts ({}), consider splitting",
                    suggestion.crate_name, suggestion.contract_count
                ));
            }

            if matches!(suggestion.dependency_risk, DependencyRisk::CycleDetected) {
                warnings.push(format!(
                    "Crate '{}' has dependency cycles - manual intervention required",
                    suggestion.crate_name
                ));
            }
        }

        warnings
    }

    /// Export analysis results to JSON
    pub fn export_analysis(&mut self, tree_sha: &str) -> Result<serde_json::Value, String> {
        let analyses = self.analyze_churn(tree_sha)?;
        let suggestions = self.generate_split_suggestions(tree_sha)?;
        let warnings = self.generate_warnings(&suggestions);

        let result = serde_json::json!({
            "tree_sha": tree_sha,
            "churn_analyses": analyses,
            "split_suggestions": suggestions,
            "warnings": warnings,
            "config": self.config,
        });

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (Repository, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create test files and commits
        let signature = git2::Signature::now("test", "test@example.com").unwrap();

        // Create initial tree
        let tree_id = repo.treebuilder(None).unwrap().write().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let commit_id = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )
            .unwrap();

        (repo, temp_dir)
    }

    #[test]
    fn test_split_planner_creation() {
        let (repo, _temp_dir) = create_test_repo();
        let config = SplitPlannerConfig::default();
        let planner = TreeSplitPlanner::new(repo, Some(config));

        assert_eq!(planner.config.max_crate_size, 1000);
        assert_eq!(planner.config.min_crate_size, 100);
    }

    #[test]
    fn test_rust_file_detection() {
        let (repo, _temp_dir) = create_test_repo();
        let planner = TreeSplitPlanner::new(repo, None);

        assert!(planner.is_rust_file(Path::new("src/lib.rs")));
        assert!(planner.is_rust_file(Path::new("main.rs")));
        assert!(!planner.is_rust_file(Path::new("README.md")));
        assert!(!planner.is_rust_file(Path::new("Cargo.toml")));
    }
}
