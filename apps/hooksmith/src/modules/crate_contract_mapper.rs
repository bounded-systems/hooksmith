use crate::modules::contract_validation::{Contract, ContractScope};
use crate::modules::git_model::{GitPath, GitTree};
use git2::{Oid, Repository, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Mapping of contract to crate boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCrateMapping {
    pub contract_id: String,
    pub crate_name: String,
    pub file_path: PathBuf,
    pub line_range: Option<(u32, u32)>,
    pub scope: ContractScope,
    pub isolation_level: IsolationLevel,
    pub dependencies: Vec<String>,
}

/// Isolation level for contract boundaries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IsolationLevel {
    Private,  // Only accessible within the crate
    Internal, // Accessible within the workspace
    Public,   // Accessible from external crates
    Unstable, // Marked for future changes
}

/// Analysis of contract distribution across crates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDistributionAnalysis {
    pub crate_name: String,
    pub contract_count: u64,
    pub total_contracts: u64,
    pub isolation_breakdown: HashMap<IsolationLevel, u64>,
    pub dependency_count: u64,
    pub cross_crate_dependencies: u64,
    pub stability_score: f64,
}

/// Boundary violation detected during mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryViolation {
    pub contract_id: String,
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub suggested_fix: Option<String>,
}

/// Types of boundary violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    CrossCrateDependency,
    UnstableApiExposure,
    ContractScopeMismatch,
    CircularDependency,
    IsolationLevelViolation,
}

/// Severity levels for violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Configuration for contract mapping analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMapperConfig {
    pub enable_isolation_checks: bool,
    pub enable_dependency_analysis: bool,
    pub enable_stability_tracking: bool,
    pub max_contracts_per_crate: u64,
    pub min_isolation_score: f64,
    pub allowed_cross_crate_deps: Vec<String>,
}

impl Default for ContractMapperConfig {
    fn default() -> Self {
        Self {
            enable_isolation_checks: true,
            enable_dependency_analysis: true,
            enable_stability_tracking: true,
            max_contracts_per_crate: 10,
            min_isolation_score: 0.7,
            allowed_cross_crate_deps: vec!["core".to_string(), "shared".to_string()],
        }
    }
}

/// Contract-to-crate boundary mapper for isolation auditing
pub struct CrateContractMapper {
    repo: Repository,
    config: ContractMapperConfig,
    mappings: HashMap<String, ContractCrateMapping>,
    violations: Vec<BoundaryViolation>,
}

impl CrateContractMapper {
    pub fn new(repo: Repository, config: Option<ContractMapperConfig>) -> Self {
        Self {
            repo,
            config: config.unwrap_or_default(),
            mappings: HashMap::new(),
            violations: Vec::new(),
        }
    }

    /// Map contracts to their crate boundaries
    pub fn map_contracts_to_crates(
        &mut self,
        tree_sha: &str,
    ) -> Result<Vec<ContractCrateMapping>, String> {
        let tree = self
            .repo
            .find_tree(git2::Oid::from_str(tree_sha).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let mut mappings = Vec::new();

        self.walk_tree_for_contracts(&tree, PathBuf::new(), &mut mappings)?;

        // Store mappings for analysis
        for mapping in &mappings {
            self.mappings
                .insert(mapping.contract_id.clone(), mapping.clone());
        }

        Ok(mappings)
    }

    /// Walk tree to find and map contracts
    fn walk_tree_for_contracts(
        &mut self,
        tree: &Tree,
        current_path: PathBuf,
        mappings: &mut Vec<ContractCrateMapping>,
    ) -> Result<(), String> {
        for entry in tree.iter() {
            let entry_path = current_path.join(entry.name().unwrap());

            match entry.kind() {
                Some(git2::ObjectType::Blob) => {
                    if self.is_contract_file(&entry_path) {
                        let contract_mappings = self.extract_contracts_from_file(&entry_path)?;
                        mappings.extend(contract_mappings);
                    }
                }
                Some(git2::ObjectType::Tree) => {
                    let subtree = self.repo.find_tree(entry.id()).map_err(|e| e.to_string())?;
                    self.walk_tree_for_contracts(&subtree, entry_path, mappings)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Check if file contains contracts
    fn is_contract_file(&self, file_path: &Path) -> bool {
        // Check for contract-related file patterns
        let contract_patterns = ["contract", "validation", "hook", "rule", "policy"];

        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_lowercase();

        contract_patterns
            .iter()
            .any(|pattern| file_name.contains(pattern))
            || file_path.extension().map_or(false, |ext| ext == "rs")
    }

    /// Extract contracts from a file and map them to crates
    fn extract_contracts_from_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<ContractCrateMapping>, String> {
        let mut mappings = Vec::new();

        // Determine crate name from file path
        let crate_name = self.determine_crate_name(file_path)?;

        // Parse file content for contracts (simplified implementation)
        let contracts = self.parse_contracts_from_file(file_path)?;

        for contract in contracts {
            let mapping = ContractCrateMapping {
                contract_id: contract.id.clone(),
                crate_name: crate_name.clone(),
                file_path: file_path.to_path_buf(),
                line_range: contract.line_range,
                scope: contract.scope.clone(),
                isolation_level: self.determine_isolation_level(&contract, &crate_name)?,
                dependencies: self.extract_contract_dependencies(&contract)?,
            };

            mappings.push(mapping);
        }

        Ok(mappings)
    }

    /// Determine crate name from file path
    fn determine_crate_name(&self, file_path: &Path) -> Result<String, String> {
        // Look for Cargo.toml in parent directories
        let mut current_path = file_path.parent();

        while let Some(path) = current_path {
            let cargo_toml = path.join("Cargo.toml");
            if cargo_toml.exists() {
                // Extract crate name from Cargo.toml
                return self.extract_crate_name_from_cargo_toml(&cargo_toml);
            }
            current_path = path.parent();
        }

        // Fallback: use directory name
        file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| "Could not determine crate name".to_string())
    }

    /// Extract crate name from Cargo.toml
    fn extract_crate_name_from_cargo_toml(&self, cargo_toml_path: &Path) -> Result<String, String> {
        let content = std::fs::read_to_string(cargo_toml_path)
            .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

        // Simple parsing for crate name
        for line in content.lines() {
            if line.trim().starts_with("name =") {
                if let Some(name) = line.split('=').nth(1) {
                    return Ok(name.trim().trim_matches('"').to_string());
                }
            }
        }

        Err("Could not find crate name in Cargo.toml".to_string())
    }

    /// Parse contracts from file content (placeholder implementation)
    fn parse_contracts_from_file(&self, _file_path: &Path) -> Result<Vec<Contract>, String> {
        // TODO: Implement actual contract parsing
        // This would parse Rust code and extract contract definitions
        Ok(Vec::new())
    }

    /// Determine isolation level for a contract
    fn determine_isolation_level(
        &self,
        contract: &Contract,
        crate_name: &str,
    ) -> Result<IsolationLevel, String> {
        // Analyze contract visibility and scope
        match contract.scope {
            ContractScope::Private => Ok(IsolationLevel::Private),
            ContractScope::Internal => Ok(IsolationLevel::Internal),
            ContractScope::Public => {
                // Check if this is an unstable API
                if self.is_unstable_contract(contract) {
                    Ok(IsolationLevel::Unstable)
                } else {
                    Ok(IsolationLevel::Public)
                }
            }
        }
    }

    /// Check if contract is marked as unstable
    fn is_unstable_contract(&self, contract: &Contract) -> bool {
        // Check for unstable markers in contract
        contract.id.to_lowercase().contains("unstable")
            || contract.id.to_lowercase().contains("experimental")
    }

    /// Extract dependencies for a contract
    fn extract_contract_dependencies(&self, _contract: &Contract) -> Result<Vec<String>, String> {
        // TODO: Implement actual dependency extraction
        // This would analyze contract code and find dependencies
        Ok(Vec::new())
    }

    /// Analyze contract distribution across crates
    pub fn analyze_contract_distribution(&self) -> Vec<ContractDistributionAnalysis> {
        let mut crate_analyses: HashMap<String, ContractDistributionAnalysis> = HashMap::new();

        for mapping in self.mappings.values() {
            let analysis = crate_analyses
                .entry(mapping.crate_name.clone())
                .or_insert_with(|| ContractDistributionAnalysis {
                    crate_name: mapping.crate_name.clone(),
                    contract_count: 0,
                    total_contracts: self.mappings.len() as u64,
                    isolation_breakdown: HashMap::new(),
                    dependency_count: 0,
                    cross_crate_dependencies: 0,
                    stability_score: 0.0,
                });

            analysis.contract_count += 1;
            *analysis
                .isolation_breakdown
                .entry(mapping.isolation_level.clone())
                .or_insert(0) += 1;
            analysis.dependency_count += mapping.dependencies.len() as u64;

            // Count cross-crate dependencies
            for dep in &mapping.dependencies {
                if !self.config.allowed_cross_crate_deps.contains(dep) {
                    analysis.cross_crate_dependencies += 1;
                }
            }
        }

        // Calculate stability scores
        for analysis in crate_analyses.values_mut() {
            analysis.stability_score = self.calculate_stability_score(analysis);
        }

        crate_analyses.into_values().collect()
    }

    /// Calculate stability score for a crate
    fn calculate_stability_score(&self, analysis: &ContractDistributionAnalysis) -> f64 {
        let mut score = 1.0;

        // Penalize high contract count
        if analysis.contract_count > self.config.max_contracts_per_crate {
            score -= 0.2;
        }

        // Penalize cross-crate dependencies
        let dep_ratio = if analysis.dependency_count > 0 {
            analysis.cross_crate_dependencies as f64 / analysis.dependency_count as f64
        } else {
            0.0
        };
        score -= dep_ratio * 0.3;

        // Penalize unstable contracts
        let unstable_count = analysis
            .isolation_breakdown
            .get(&IsolationLevel::Unstable)
            .unwrap_or(&0);
        let unstable_ratio = *unstable_count as f64 / analysis.contract_count as f64;
        score -= unstable_ratio * 0.4;

        score.max(0.0)
    }

    /// Detect boundary violations
    pub fn detect_boundary_violations(&mut self) -> Vec<BoundaryViolation> {
        self.violations.clear();

        let mappings: Vec<_> = self.mappings.values().cloned().collect();
        for mapping in mappings {
            self.check_contract_boundaries(&mapping);
        }

        self.violations.clone()
    }

    /// Check contract boundaries for violations
    fn check_contract_boundaries(&mut self, mapping: &ContractCrateMapping) {
        // Check for cross-crate dependencies
        for dep in &mapping.dependencies {
            if !self.config.allowed_cross_crate_deps.contains(dep) {
                self.violations.push(BoundaryViolation {
                    contract_id: mapping.contract_id.clone(),
                    violation_type: ViolationType::CrossCrateDependency,
                    severity: ViolationSeverity::Warning,
                    description: format!("Contract depends on external crate '{}'", dep),
                    suggested_fix: Some(format!(
                        "Move dependency to allowed list or refactor contract"
                    )),
                });
            }
        }

        // Check for unstable API exposure
        if mapping.isolation_level == IsolationLevel::Unstable {
            let public_deps = mapping
                .dependencies
                .iter()
                .filter(|dep| !self.config.allowed_cross_crate_deps.contains(*dep))
                .count();

            if public_deps > 0 {
                self.violations.push(BoundaryViolation {
                    contract_id: mapping.contract_id.clone(),
                    violation_type: ViolationType::UnstableApiExposure,
                    severity: ViolationSeverity::Error,
                    description: "Unstable contract exposes public dependencies".to_string(),
                    suggested_fix: Some(
                        "Mark dependencies as internal or refactor contract".to_string(),
                    ),
                });
            }
        }

        // Check for scope mismatches
        if mapping.scope == ContractScope::Public
            && mapping.isolation_level == IsolationLevel::Private
        {
            self.violations.push(BoundaryViolation {
                contract_id: mapping.contract_id.clone(),
                violation_type: ViolationType::ContractScopeMismatch,
                severity: ViolationSeverity::Error,
                description: "Public contract scope with private isolation level".to_string(),
                suggested_fix: Some("Align contract scope with isolation level".to_string()),
            });
        }
    }

    /// Generate isolation audit report
    pub fn generate_isolation_report(&mut self) -> serde_json::Value {
        let distribution = self.analyze_contract_distribution();
        let violations = self.detect_boundary_violations();

        let report = serde_json::json!({
            "contract_distribution": distribution,
            "boundary_violations": violations,
            "total_contracts": self.mappings.len(),
            "total_violations": violations.len(),
            "config": self.config,
        });

        report
    }

    /// Export mappings to JSON
    pub fn export_mappings(&self) -> serde_json::Value {
        let mappings: Vec<_> = self.mappings.values().cloned().collect();
        serde_json::to_value(mappings).unwrap_or_default()
    }

    /// Get mapping for a specific contract
    pub fn get_contract_mapping(&self, contract_id: &str) -> Option<&ContractCrateMapping> {
        self.mappings.get(contract_id)
    }

    /// Get all contracts for a specific crate
    pub fn get_crate_contracts(&self, crate_name: &str) -> Vec<&ContractCrateMapping> {
        self.mappings
            .values()
            .filter(|mapping| mapping.crate_name == crate_name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (Repository, TempDir) {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(temp_dir.path()).unwrap();

        // Create test structure
        let signature = git2::Signature::now("test", "test@example.com").unwrap();
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
    fn test_contract_mapper_creation() {
        let (repo, _temp_dir) = create_test_repo();
        let config = ContractMapperConfig::default();
        let mapper = CrateContractMapper::new(repo, Some(config));

        assert_eq!(mapper.config.max_contracts_per_crate, 10);
        assert_eq!(mapper.config.min_isolation_score, 0.7);
    }

    #[test]
    fn test_contract_file_detection() {
        let (repo, _temp_dir) = create_test_repo();
        let mapper = CrateContractMapper::new(repo, None);

        assert!(mapper.is_contract_file(Path::new("src/contracts/mod.rs")));
        assert!(mapper.is_contract_file(Path::new("src/validation.rs")));
        assert!(mapper.is_contract_file(Path::new("hooks/validation.rs")));
        assert!(!mapper.is_contract_file(Path::new("README.md")));
    }
}
