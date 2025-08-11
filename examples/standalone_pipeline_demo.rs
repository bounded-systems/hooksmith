use std::collections::HashSet;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use git2::{Repository, ObjectType};

/// Git object metadata for pipeline processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitObject {
    pub oid: String,
    pub kind: ObjectType,
    pub logical_path: Option<PathBuf>,
    pub parent_tree_oid: Option<String>,
    pub size: usize,
}

/// Tool fingerprint for cache invalidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFingerprint {
    pub name: String,
    pub version: String,
    pub config_hash: String,
}

/// Analysis result from a researcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    pub tool_fingerprint: ToolFingerprint,
    pub object_oid: String,
    pub analysis_data: serde_json::Value,
    pub cache_key: String,
}

/// Report combining multiple analyses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub domain: String,
    pub version: String,
    pub object_oid: String,
    pub normalized_data: serde_json::Value,
    pub analysis_oids: Vec<String>,
    pub cache_key: String,
}

/// Mandate (expectation) for an object based on contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mandate {
    pub contract_name: String,
    pub contract_oid: String,
    pub version: String,
    pub object_selector: String,
    pub logical_path: Option<PathBuf>,
    pub expectation: serde_json::Value,
    pub cache_key: String,
}

/// Audit verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verdict {
    pub contract_name: String,
    pub version: String,
    pub pass: bool,
    pub summary_code: String,
    pub report_oid: String,
    pub mandate_oid: String,
    pub diff_oid: Option<String>,
    pub cache_key: String,
}

/// Object names contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNamesContract {
    pub name: String,
    pub version: String,
    pub spec: ObjectNamesSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectNamesSpec {
    pub git: GitSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSpec {
    pub tree: TreeSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeSpec {
    pub objects: ObjectsSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectsSpec {
    pub names: NamesSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamesSpec {
    pub required: Vec<String>,
    pub allowed: Vec<String>,
    pub rejected: Vec<String>,
    pub ignored: Vec<String>,
}

/// Actor 1: Researcher - analyzes tree objects for naming patterns
pub struct ObjectNamesResearcher {
    tool_fingerprint: ToolFingerprint,
}

impl ObjectNamesResearcher {
    pub fn new() -> Self {
        Self {
            tool_fingerprint: ToolFingerprint {
                name: "object-names-validator".to_string(),
                version: "1.0.0".to_string(),
                config_hash: "default".to_string(),
            },
        }
    }

    pub fn analyze_tree(&self, repo: &Repository, object: &GitObject) -> Result<Analysis, Box<dyn std::error::Error>> {
        let tree = repo.find_tree(git2::Oid::from_str(&object.oid)?)?;
        
        let mut entries = Vec::new();
        let mut entry_names = Vec::new();
        
        for entry in tree.iter() {
            let name = entry.name().unwrap_or("").to_string();
            entry_names.push(name.clone());
            
            entries.push(serde_json::json!({
                "name": name,
                "oid": entry.id().to_string(),
                "kind": format!("{:?}", entry.kind()),
                "mode": entry.filemode(),
            }));
        }
        
        let analysis_data = serde_json::json!({
            "type": "tree_analysis",
            "entry_count": entries.len(),
            "entries": entries,
            "entry_names": entry_names,
            "is_root": object.logical_path.as_ref().map_or(true, |p| p.to_string_lossy().is_empty()),
        });
        
        let cache_key = self.compute_cache_key(&object.oid);
        
        Ok(Analysis {
            tool_fingerprint: self.tool_fingerprint.clone(),
            object_oid: object.oid.clone(),
            analysis_data,
            cache_key,
        })
    }
    
    fn compute_cache_key(&self, object_oid: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("analysis-{}@{}", self.tool_fingerprint.name, self.tool_fingerprint.version).as_bytes());
        hasher.update(object_oid.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Actor 2: Reporter - normalizes tree analysis into a standardized report
pub struct ObjectNamesReporter {
    domain: String,
    version: String,
}

impl ObjectNamesReporter {
    pub fn new() -> Self {
        Self {
            domain: "object-names".to_string(),
            version: "1.0.0".to_string(),
        }
    }
    
    pub fn create_report(&self, object: &GitObject, analyses: &[Analysis]) -> Result<Report, Box<dyn std::error::Error>> {
        let analysis_oids: Vec<String> = analyses.iter().map(|a| a.cache_key.clone()).collect();
        let cache_key = self.compute_cache_key(&analysis_oids);
        
        // Extract and normalize the analysis data
        let mut normalized_data = serde_json::Map::new();
        
        for analysis in analyses {
            if let Some(entry_names) = analysis.analysis_data.get("entry_names") {
                normalized_data.insert("entry_names".to_string(), entry_names.clone());
            }
            if let Some(is_root) = analysis.analysis_data.get("is_root") {
                normalized_data.insert("is_root".to_string(), is_root.clone());
            }
        }
        
        Ok(Report {
            domain: self.domain.clone(),
            version: self.version.clone(),
            object_oid: object.oid.clone(),
            normalized_data: serde_json::Value::Object(normalized_data),
            analysis_oids,
            cache_key,
        })
    }
    
    fn compute_cache_key(&self, analysis_oids: &[String]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("report@{}", self.version).as_bytes());
        for oid in analysis_oids {
            hasher.update(oid.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Actor 3: Mandator - creates expectations based on the contract
pub struct ObjectNamesMandator {
    contract: ObjectNamesContract,
}

impl ObjectNamesMandator {
    pub fn new(contract: ObjectNamesContract) -> Self {
        Self { contract }
    }
    
    pub fn create_mandate(&self, object: &GitObject) -> Result<Mandate, Box<dyn std::error::Error>> {
        let object_selector = self.compute_object_selector(object);
        let cache_key = self.compute_cache_key(&object_selector, &object.logical_path);
        
        // Determine if this is a root tree
        let is_root = object.logical_path.as_ref().map_or(true, |p| p.to_string_lossy().is_empty());
        
        let expectation = if is_root {
            // Root tree expectations
            serde_json::json!({
                "type": "root_tree",
                "required_entries": self.contract.spec.git.tree.objects.names.required,
                "allowed_entries": self.contract.spec.git.tree.objects.names.allowed,
                "rejected_entries": self.contract.spec.git.tree.objects.names.rejected,
                "ignored_entries": self.contract.spec.git.tree.objects.names.ignored,
            })
        } else {
            // Non-root tree expectations (can be more permissive)
            serde_json::json!({
                "type": "sub_tree",
                "allowed_entries": self.contract.spec.git.tree.objects.names.allowed,
                "ignored_entries": self.contract.spec.git.tree.objects.names.ignored,
            })
        };
        
        Ok(Mandate {
            contract_name: self.contract.name.clone(),
            contract_oid: "contract-oid".to_string(),
            version: self.contract.version.clone(),
            object_selector,
            logical_path: object.logical_path.clone(),
            expectation,
            cache_key,
        })
    }
    
    fn compute_object_selector(&self, object: &GitObject) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", object.kind).as_bytes());
        if let Some(path) = &object.logical_path {
            hasher.update(path.to_string_lossy().as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
    
    fn compute_cache_key(&self, selector: &str, logical_path: &Option<PathBuf>) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("mandate@{}", self.contract.version).as_bytes());
        hasher.update(selector.as_bytes());
        if let Some(path) = logical_path {
            hasher.update(path.to_string_lossy().as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Actor 4: Auditor - compares report against mandate
pub struct ObjectNamesAuditor {
    contract_name: String,
    version: String,
}

impl ObjectNamesAuditor {
    pub fn new(contract_name: String, version: String) -> Self {
        Self {
            contract_name,
            version,
        }
    }
    
    pub fn audit(&self, report: &Report, mandate: &Mandate) -> Result<Verdict, Box<dyn std::error::Error>> {
        let cache_key = self.compute_cache_key(&report.cache_key, &mandate.cache_key);
        
        // Extract data from report and mandate
        let entry_names = if let Some(names) = report.normalized_data.get("entry_names") {
            names.as_array().unwrap_or(&Vec::new()).iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        };
        
        let is_root = report.normalized_data.get("is_root")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Perform validation
        let (pass, summary_code, diff_oid) = if is_root {
            self.validate_root_tree(&entry_names, &mandate.expectation)?
        } else {
            self.validate_sub_tree(&entry_names, &mandate.expectation)?
        };
        
        Ok(Verdict {
            contract_name: self.contract_name.clone(),
            version: self.version.clone(),
            pass,
            summary_code,
            report_oid: report.cache_key.clone(),
            mandate_oid: mandate.cache_key.clone(),
            diff_oid,
            cache_key,
        })
    }
    
    fn validate_root_tree(&self, entry_names: &[String], expectation: &serde_json::Value) -> Result<(bool, String, Option<String>), Box<dyn std::error::Error>> {
        let required = expectation.get("required_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
            
        let allowed = expectation.get("allowed_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
            
        let rejected = expectation.get("rejected_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
            
        let ignored = expectation.get("ignored_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
        
        let entry_set: HashSet<String> = entry_names.iter().cloned().collect();
        let mut errors = Vec::new();
        
        // Check required entries
        for req in &required {
            if !entry_set.contains(req) {
                errors.push(format!("missing required: {}", req));
            }
        }
        
        // Check rejected entries (skip ignored)
        for entry in entry_names {
            if ignored.contains(entry) {
                continue;
            }
            if rejected.contains(entry) {
                errors.push(format!("rejected at root: {}", entry));
            }
        }
        
        // Check allow-list (skip ignored)
        for entry in entry_names {
            if ignored.contains(entry) {
                continue;
            }
            if !allowed.contains(entry) {
                errors.push(format!("not in allowed set: {}", entry));
            }
        }
        
        let pass = errors.is_empty();
        let summary_code = if pass { "PASS".to_string() } else { "FAIL".to_string() };
        
        let diff_oid = if !pass {
            Some("diff-oid".to_string())
        } else {
            None
        };
        
        Ok((pass, summary_code, diff_oid))
    }
    
    fn validate_sub_tree(&self, entry_names: &[String], expectation: &serde_json::Value) -> Result<(bool, String, Option<String>), Box<dyn std::error::Error>> {
        // For sub-trees, we're more permissive - just check against allowed/ignored
        let allowed = expectation.get("allowed_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
            
        let ignored = expectation.get("ignored_entries")
            .and_then(|v| v.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<HashSet<String>>();
        
        let mut errors = Vec::new();
        
        // Check allow-list (skip ignored)
        for entry in entry_names {
            if ignored.contains(entry) {
                continue;
            }
            if !allowed.contains(entry) {
                errors.push(format!("not in allowed set: {}", entry));
            }
        }
        
        let pass = errors.is_empty();
        let summary_code = if pass { "PASS".to_string() } else { "FAIL".to_string() };
        
        let diff_oid = if !pass {
            Some("diff-oid".to_string())
        } else {
            None
        };
        
        Ok((pass, summary_code, diff_oid))
    }
    
    fn compute_cache_key(&self, report_oid: &str, mandate_oid: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(format!("audit@{}", self.version).as_bytes());
        hasher.update(report_oid.as_bytes());
        hasher.update(mandate_oid.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Main validator that orchestrates the four actors
pub struct ObjectNamesValidator {
    researcher: ObjectNamesResearcher,
    reporter: ObjectNamesReporter,
    mandator: ObjectNamesMandator,
    auditor: ObjectNamesAuditor,
}

impl ObjectNamesValidator {
    pub fn new(contract: ObjectNamesContract) -> Self {
        let researcher = ObjectNamesResearcher::new();
        let reporter = ObjectNamesReporter::new();
        let mandator = ObjectNamesMandator::new(contract.clone());
        let auditor = ObjectNamesAuditor::new(contract.name.clone(), contract.version.clone());
        
        Self {
            researcher,
            reporter,
            mandator,
            auditor,
        }
    }
    
    pub fn validate_tree(&self, repo: &Repository, object: &GitObject) -> Result<Verdict, Box<dyn std::error::Error>> {
        println!("🔬 Researcher: Analyzing tree object {}", object.oid);
        let analysis = self.researcher.analyze_tree(repo, object)?;
        
        println!("📊 Reporter: Creating normalized report");
        let report = self.reporter.create_report(object, &[analysis])?;
        
        println!("📋 Mandator: Creating expectation from contract");
        let mandate = self.mandator.create_mandate(object)?;
        
        println!("🔍 Auditor: Comparing report vs mandate");
        let verdict = self.auditor.audit(&report, &mandate)?;
        
        Ok(verdict)
    }
    
    pub fn validate_root_tree(&self, repo: &Repository, commit_oid: &str) -> Result<Vec<Verdict>, Box<dyn std::error::Error>> {
        let commit = repo.find_commit(git2::Oid::from_str(commit_oid)?)?;
        let tree = commit.tree()?;
        
        let object = GitObject {
            oid: tree.id().to_string(),
            kind: ObjectType::Tree,
            logical_path: Some(PathBuf::new()), // Root path
            parent_tree_oid: None,
            size: tree.id().as_bytes().len(),
        };
        
        let verdict = self.validate_tree(repo, &object)?;
        Ok(vec![verdict])
    }
}

/// Load contract from JSON
pub fn load_contract(contract_json: &str) -> Result<ObjectNamesContract, Box<dyn std::error::Error>> {
    let contract: ObjectNamesContract = serde_json::from_str(contract_json)?;
    Ok(contract)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Object Names Pipeline Demo");
    println!("==============================");
    
    // Load the contract
    let contract_json = r#"
    {
        "name": "object-names",
        "version": "1.0.0",
        "spec": {
            "git": {
                "tree": {
                    "objects": {
                        "names": {
                            "required": [".gitignore", "projects"],
                            "allowed": [".gitignore", ".gitattributes", ".meta", "docs", "generated", "projects", "src", "tests", "tools", "wit"],
                            "rejected": ["README.md", "Cargo.toml", "rustfmt.toml"],
                            "ignored": [".DS_Store", "Thumbs.db", ".idea", ".vscode"]
                        }
                    }
                }
            }
        }
    }
    "#;
    
    let contract = load_contract(contract_json)?;
    println!("📄 Loaded contract: {} v{}", contract.name, contract.version);
    
    // Open the repository
    let repo = Repository::open(".")?;
    println!("📁 Opened repository");
    
    // Get the current HEAD
    let head = repo.head()?;
    let commit_oid = head.target().unwrap().to_string();
    println!("🎯 Target commit: {}", commit_oid);
    
    // Create validator
    let validator = ObjectNamesValidator::new(contract);
    
    println!("\n🔄 Running four-actor pipeline...");
    println!("==================================");
    
    // Validate the root tree
    let verdicts = validator.validate_root_tree(&repo, &commit_oid)?;
    
    println!("\n📋 Results:");
    println!("===========");
    
    for verdict in verdicts {
        if verdict.pass {
            println!("✅ {}: {}", verdict.contract_name, verdict.summary_code);
        } else {
            println!("❌ {}: {}", verdict.contract_name, verdict.summary_code);
            if let Some(diff_oid) = verdict.diff_oid {
                println!("   Diff: {}", diff_oid);
            }
        }
    }
    
    println!("\n🎉 Pipeline completed!");
    
    Ok(())
}
