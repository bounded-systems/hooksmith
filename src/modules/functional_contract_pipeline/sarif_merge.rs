use crate::modules::functional_contract_pipeline::sarif_roles::{SarifLog, SarifResult, SarifRun};
use std::collections::HashMap;

/// SARIF merge strategy
#[derive(Debug, Clone, PartialEq)]
pub enum MergeStrategy {
    /// Merge by concern (group results by concern symbol)
    ByConcern,
    /// Merge by contract (group results by contract symbol)
    ByContract,
    /// Merge by severity (group results by severity level)
    BySeverity,
    /// Merge by hook event (group results by hook event)
    ByHookEvent,
    /// Merge by tool (group results by tool name)
    ByTool,
    /// Custom merge strategy
    Custom(String),
}

/// Options for merging SARIF logs
pub struct MergeOptions {
    /// The merge strategy to use
    pub strategy: MergeStrategy,
    /// Whether to deduplicate results
    pub deduplicate: bool,
    /// Whether to sort results
    pub sort: bool,
    /// Custom merge function
    pub custom_merge: Option<Box<dyn Fn(&[SarifResult]) -> Vec<SarifResult>>>,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self {
            strategy: MergeStrategy::ByConcern,
            deduplicate: true,
            sort: true,
            custom_merge: None,
        }
    }
}

/// SARIF merge utilities
pub struct SarifMerger;

impl SarifMerger {
    /// Merge multiple SARIF logs into one
    pub fn merge_logs(logs: Vec<SarifLog>, options: &MergeOptions) -> SarifLog {
        let mut all_results = Vec::new();
        
        // Collect all results from all logs
        for log in logs {
            for run in log.runs {
                all_results.extend(run.results);
            }
        }
        
        // Apply merge strategy
        let merged_results = match &options.strategy {
            MergeStrategy::ByConcern => Self::merge_by_concern(&all_results),
            MergeStrategy::ByContract => Self::merge_by_contract(&all_results),
            MergeStrategy::BySeverity => Self::merge_by_severity(&all_results),
            MergeStrategy::ByHookEvent => Self::merge_by_hook_event(&all_results),
            MergeStrategy::ByTool => Self::merge_by_tool(&all_results),
            MergeStrategy::Custom(_) => {
                if let Some(ref custom_merge) = options.custom_merge {
                    custom_merge(&all_results)
                } else {
                    all_results
                }
            }
        };
        
        // Deduplicate if requested
        let final_results = if options.deduplicate {
            Self::deduplicate_results(&merged_results)
        } else {
            merged_results
        };
        
        // Sort if requested
        let final_results = if options.sort {
            Self::sort_results(&final_results)
        } else {
            final_results
        };
        
        // Create merged log
        let run = SarifRun {
            tool_name: "Hooksmith Merged Validator".to_string(),
            tool_version: "1.0.0".to_string(),
            results: final_results,
        };
        
        SarifLog {
            version: "2.1.0".to_string(),
            runs: vec![run],
        }
    }
    
    /// Merge results by concern
    fn merge_by_concern(results: &[SarifResult]) -> Vec<SarifResult> {
        let mut grouped: HashMap<String, Vec<&SarifResult>> = HashMap::new();
        
        for result in results {
            if let Some(concern) = result.properties.get("concern") {
                grouped.entry(concern.clone()).or_default().push(result);
            } else {
                // Group unclassified results
                grouped.entry("unknown".to_string()).or_default().push(result);
            }
        }
        
        let mut merged = Vec::new();
        for (_concern, group_results) in grouped {
            merged.extend(group_results.into_iter().cloned());
        }
        
        // Return merged results
        merged.into_iter().collect()
    }
    
    /// Merge results by contract
    fn merge_by_contract(results: &[SarifResult]) -> Vec<SarifResult> {
        let mut grouped: HashMap<String, Vec<&SarifResult>> = HashMap::new();
        
        for result in results {
            if let Some(contract) = result.properties.get("contract") {
                grouped.entry(contract.clone()).or_default().push(result);
            } else {
                // Group unclassified results
                grouped.entry("unknown".to_string()).or_default().push(result);
            }
        }
        
        let mut merged = Vec::new();
        for (_contract, group_results) in grouped {
            merged.extend(group_results.into_iter().cloned());
        }
        
        // Return merged results
        merged.into_iter().collect()
    }
    
    /// Merge results by severity
    fn merge_by_severity(results: &[SarifResult]) -> Vec<SarifResult> {
        let mut grouped: HashMap<String, Vec<&SarifResult>> = HashMap::new();
        
        for result in results {
            grouped.entry(result.level.clone()).or_default().push(result);
        }
        
        let mut merged = Vec::new();
        // Order by severity: error, warning, note
        let severity_order = vec!["error", "warning", "note"];
        for severity in severity_order {
            if let Some(group_results) = grouped.get(severity) {
                merged.extend(group_results.iter().cloned());
            }
        }
        
        // Convert references to owned values
        merged.into_iter().cloned().collect()
    }
    
    /// Merge results by hook event
    fn merge_by_hook_event(results: &[SarifResult]) -> Vec<SarifResult> {
        let mut grouped: HashMap<String, Vec<&SarifResult>> = HashMap::new();
        
        for result in results {
            if let Some(hook_event) = result.properties.get("hook_event") {
                grouped.entry(hook_event.clone()).or_default().push(result);
            } else {
                // Group unclassified results
                grouped.entry("unknown".to_string()).or_default().push(result);
            }
        }
        
        let mut merged = Vec::new();
        for (_hook_event, group_results) in grouped {
            merged.extend(group_results.iter().cloned());
        }
        
        // Return merged results
        merged
    }
    
    /// Merge results by tool
    fn merge_by_tool(results: &[SarifResult]) -> Vec<SarifResult> {
        // For now, just return results as-is since we don't have tool info in results
        results.to_vec()
    }
    
    /// Deduplicate results
    fn deduplicate_results(results: &[SarifResult]) -> Vec<SarifResult> {
        let mut seen = std::collections::HashSet::new();
        let mut deduplicated = Vec::new();
        
        for result in results {
            let key = format!("{}:{}:{}", result.rule_id, result.message, result.level);
            if !seen.contains(&key) {
                seen.insert(key);
                deduplicated.push(result.clone());
            }
        }
        
        deduplicated
    }
    
    /// Sort results
    fn sort_results(results: &[SarifResult]) -> Vec<SarifResult> {
        let mut sorted = results.to_vec();
        sorted.sort_by(|a, b| {
            // Sort by severity first (error > warning > note)
            let severity_order = |level: &str| {
                match level {
                    "error" => 0,
                    "warning" => 1,
                    "note" => 2,
                    _ => 3,
                }
            };
            
            let severity_cmp = severity_order(&a.level).cmp(&severity_order(&b.level));
            if severity_cmp != std::cmp::Ordering::Equal {
                return severity_cmp;
            }
            
            // Then sort by rule_id
            a.rule_id.cmp(&b.rule_id)
        });
        
        sorted
    }
    
    /// Merge results in parallel (simulated)
    pub fn merge_parallel(logs: Vec<SarifLog>, options: &MergeOptions) -> SarifLog {
        // In a real implementation, this would use rayon or similar for parallel processing
        Self::merge_logs(logs, options)
    }
    
    /// Create a log for a single concern
    pub fn create_concern_log(concern: &str, results: Vec<SarifResult>) -> SarifLog {
        let run = SarifRun {
            tool_name: format!("Hooksmith Concern Validator ({})", concern),
            tool_version: "1.0.0".to_string(),
            results,
        };
        
        SarifLog {
            version: "2.1.0".to_string(),
            runs: vec![run],
        }
    }
    
    /// Create a log for a single contract
    pub fn create_contract_log(contract: &str, results: Vec<SarifResult>) -> SarifLog {
        let run = SarifRun {
            tool_name: format!("Hooksmith Contract Validator ({})", contract),
            tool_version: "1.0.0".to_string(),
            results,
        };
        
        SarifLog {
            version: "2.1.0".to_string(),
            runs: vec![run],
        }
    }
    
    /// Split a log by concern
    pub fn split_by_concern(log: &SarifLog) -> HashMap<String, SarifLog> {
        let mut concern_logs: HashMap<String, SarifLog> = HashMap::new();
        
        for run in &log.runs {
            for result in &run.results {
                let concern = result.properties.get("concern")
                    .unwrap_or(&"unknown".to_string())
                    .clone();
                
                let concern_log = concern_logs.entry(concern.clone()).or_insert_with(|| {
                    SarifLog {
                        version: "2.1.0".to_string(),
                        runs: vec![],
                    }
                });
                
                // Create a new run for this concern if it doesn't exist
                if concern_log.runs.is_empty() {
                    concern_log.runs.push(SarifRun {
                        tool_name: format!("Hooksmith Concern Validator ({})", concern),
                        tool_version: "1.0.0".to_string(),
                        results: vec![],
                    });
                }
                
                concern_log.runs[0].results.push(result.clone());
            }
        }
        
        concern_logs
    }
    
    /// Split a log by contract
    pub fn split_by_contract(log: &SarifLog) -> HashMap<String, SarifLog> {
        let mut contract_logs: HashMap<String, SarifLog> = HashMap::new();
        
        for run in &log.runs {
            for result in &run.results {
                let contract = result.properties.get("contract")
                    .unwrap_or(&"unknown".to_string())
                    .clone();
                
                let contract_log = contract_logs.entry(contract.clone()).or_insert_with(|| {
                    SarifLog {
                        version: "2.1.0".to_string(),
                        runs: vec![],
                    }
                });
                
                // Create a new run for this contract if it doesn't exist
                if contract_log.runs.is_empty() {
                    contract_log.runs.push(SarifRun {
                        tool_name: format!("Hooksmith Contract Validator ({})", contract),
                        tool_version: "1.0.0".to_string(),
                        results: vec![],
                    });
                }
                
                contract_log.runs[0].results.push(result.clone());
            }
        }
        
        contract_logs
    }
    
    /// Get merge statistics
    pub fn merge_stats(log: &SarifLog) -> MergeStats {
        let mut total_results = 0;
        let mut severity_counts = HashMap::new();
        let mut concern_counts = HashMap::new();
        let mut contract_counts = HashMap::new();
        
        for run in &log.runs {
            total_results += run.results.len();
            
            for result in &run.results {
                // Count by severity
                *severity_counts.entry(result.level.clone()).or_insert(0) += 1;
                
                // Count by concern
                if let Some(concern) = result.properties.get("concern") {
                    *concern_counts.entry(concern.clone()).or_insert(0) += 1;
                }
                
                // Count by contract
                if let Some(contract) = result.properties.get("contract") {
                    *contract_counts.entry(contract.clone()).or_insert(0) += 1;
                }
            }
        }
        
        MergeStats {
            total_results,
            severity_counts,
            concern_counts,
            contract_counts,
        }
    }
}

/// Merge statistics
#[derive(Debug, Clone)]
pub struct MergeStats {
    /// Total number of results
    pub total_results: usize,
    /// Count by severity level
    pub severity_counts: HashMap<String, usize>,
    /// Count by concern
    pub concern_counts: HashMap<String, usize>,
    /// Count by contract
    pub contract_counts: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_merge_by_concern() {
        let mut results = Vec::new();
        
        // Create test results
        let mut props1 = HashMap::new();
        props1.insert("concern".to_string(), "Index".to_string());
        results.push(SarifResult {
            rule_id: "test-1".to_string(),
            level: "error".to_string(),
            message: "Test 1".to_string(),
            locations: vec![],
            properties: props1,
        });
        
        let mut props2 = HashMap::new();
        props2.insert("concern".to_string(), "TreeExecutable".to_string());
        results.push(SarifResult {
            rule_id: "test-2".to_string(),
            level: "warning".to_string(),
            message: "Test 2".to_string(),
            locations: vec![],
            properties: props2,
        });
        
        let merged = SarifMerger::merge_by_concern(&results);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_merge_by_severity() {
        let mut results = Vec::new();
        
        results.push(SarifResult {
            rule_id: "test-1".to_string(),
            level: "warning".to_string(),
            message: "Test 1".to_string(),
            locations: vec![],
            properties: HashMap::new(),
        });
        
        results.push(SarifResult {
            rule_id: "test-2".to_string(),
            level: "error".to_string(),
            message: "Test 2".to_string(),
            locations: vec![],
            properties: HashMap::new(),
        });
        
        let merged = SarifMerger::merge_by_severity(&results);
        assert_eq!(merged.len(), 2);
        // Error should come before warning
        assert_eq!(merged[0].level, "error");
        assert_eq!(merged[1].level, "warning");
    }

    #[test]
    fn test_deduplicate_results() {
        let mut results = Vec::new();
        
        // Create duplicate results
        results.push(SarifResult {
            rule_id: "test-1".to_string(),
            level: "error".to_string(),
            message: "Test 1".to_string(),
            locations: vec![],
            properties: HashMap::new(),
        });
        
        results.push(SarifResult {
            rule_id: "test-1".to_string(),
            level: "error".to_string(),
            message: "Test 1".to_string(),
            locations: vec![],
            properties: HashMap::new(),
        });
        
        let deduplicated = SarifMerger::deduplicate_results(&results);
        assert_eq!(deduplicated.len(), 1);
    }

    #[test]
    fn test_merge_logs() {
        let mut logs = Vec::new();
        
        // Create test logs
        let mut results1 = Vec::new();
        results1.push(SarifResult {
            rule_id: "test-1".to_string(),
            level: "error".to_string(),
            message: "Test 1".to_string(),
            locations: vec![],
            properties: HashMap::new(),
        });
        
        let log1 = SarifLog {
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool_name: "Test Tool 1".to_string(),
                tool_version: "1.0.0".to_string(),
                results: results1,
            }],
        };
        
        let mut results2 = Vec::new();
        results2.push(SarifResult {
            rule_id: "test-2".to_string(),
            level: "warning".to_string(),
            message: "Test 2".to_string(),
            locations: vec![],
            properties: HashMap::new(),
        });
        
        let log2 = SarifLog {
            version: "2.1.0".to_string(),
            runs: vec![SarifRun {
                tool_name: "Test Tool 2".to_string(),
                tool_version: "1.0.0".to_string(),
                results: results2,
            }],
        };
        
        logs.push(log1);
        logs.push(log2);
        
        let options = MergeOptions::default();
        let merged = SarifMerger::merge_logs(logs, &options);
        
        assert_eq!(merged.runs.len(), 1);
        assert_eq!(merged.runs[0].results.len(), 2);
    }
}
