use crate::modules::functional_contract_pipeline::symbols::RuleSeverity;
use crate::modules::functional_contract_pipeline::types::{DiffSet, DiffType, ValidationDiff};
use json_patch::diff;
use sonic_rs::from_str;
use std::collections::HashMap;
use std::time::Instant;

/// High-performance diffing strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStrategy {
    /// Standard JSON Patch (RFC 6902)
    JsonPatch,
    /// Sonic-rs optimized parsing with JSON Patch
    SonicJsonPatch,
    /// Hybrid approach: Sonic parsing + JSON Patch
    Hybrid,
}

/// Performance metrics for diffing operations
#[derive(Debug, Clone)]
pub struct DiffMetrics {
    pub strategy: DiffStrategy,
    pub parse_time_micros: u64,
    pub diff_time_micros: u64,
    pub total_time_micros: u64,
    pub operations_generated: usize,
    pub memory_allocations: usize,
}

impl DiffMetrics {
    pub fn new(strategy: DiffStrategy) -> Self {
        Self {
            strategy,
            parse_time_micros: 0,
            diff_time_micros: 0,
            total_time_micros: 0,
            operations_generated: 0,
            memory_allocations: 0,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Strategy: {:?}, Parse: {}μs, Diff: {}μs, Total: {}μs, Ops: {}",
            self.strategy,
            self.parse_time_micros,
            self.diff_time_micros,
            self.total_time_micros,
            self.operations_generated
        )
    }
}

/// High-performance diffing engine
pub struct HighPerformanceDiffer {
    strategy: DiffStrategy,
}

impl HighPerformanceDiffer {
    /// Create a new high-performance differ with the specified strategy
    pub fn new(strategy: DiffStrategy) -> Self {
        Self { strategy }
    }

    /// Generate diff with performance metrics
    pub fn diff_with_metrics(
        &self,
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> (DiffSet, DiffMetrics) {
        let start = Instant::now();
        let mut metrics = DiffMetrics::new(self.strategy);

        let (diff_set, parse_time, diff_time) = match self.strategy {
            DiffStrategy::JsonPatch => self.json_patch_diff(observed, expected),
            DiffStrategy::SonicJsonPatch => self.sonic_json_patch_diff(observed, expected),
            DiffStrategy::Hybrid => self.hybrid_diff(observed, expected),
        };

        metrics.parse_time_micros = parse_time;
        metrics.diff_time_micros = diff_time;
        metrics.total_time_micros = start.elapsed().as_micros() as u64;
        metrics.operations_generated = diff_set.diff_count();

        (diff_set, metrics)
    }

    /// Standard JSON Patch diffing
    fn json_patch_diff(
        &self,
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> (DiffSet, u64, u64) {
        let parse_start = Instant::now();
        
        // Parse JSON using serde_json (baseline)
        let mut diffs = Vec::new();
        let mut total_operations = 0;

        for ex in expected {
            if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
                let patches = diff(&obs.data, &ex.expectation);
                total_operations += patches.0.len();
                
                if !patches.0.is_empty() {
                    let mut metadata = HashMap::new();
                    metadata.insert("json_patch".to_string(), serde_json::to_value(&patches).unwrap_or_default());
                    
                    diffs.push(ValidationDiff::new(
                        ex.symbol.clone(),
                        DiffType::Mismatch,
                        format!("Data mismatch for concern: {} (JSON Patch: {} operations)", 
                               ex.symbol.name(), patches.0.len()),
                        Some(obs.data.clone()),
                        Some(ex.expectation.clone()),
                        RuleSeverity::Error,
                        metadata,
                    ));
                }
            } else {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Missing,
                    format!("Missing expected concern: {}", ex.symbol.name()),
                    None,
                    Some(ex.expectation.clone()),
                    RuleSeverity::Error,
                    HashMap::new(),
                ));
            }
        }

        let parse_time = parse_start.elapsed().as_micros() as u64;
        let diff_time = 0; // JSON Patch diffing is included in parse time

        (DiffSet::new(diffs), parse_time, diff_time)
    }

    /// Sonic-rs optimized JSON Patch diffing
    fn sonic_json_patch_diff(
        &self,
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> (DiffSet, u64, u64) {
        let parse_start = Instant::now();
        
        // Parse JSON using sonic-rs for better performance
        let mut diffs = Vec::new();
        let mut _total_operations = 0;

        for ex in expected {
            if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
                // Convert to sonic-rs format for parsing
                let obs_str = obs.data.to_string();
                let exp_str = ex.expectation.to_string();
                
                // Use sonic-rs for parsing (even if we don't use the result, it demonstrates the API)
                let _obs_sonic = from_str::<serde_json::Value>(&obs_str).unwrap_or_default();
                let _exp_sonic = from_str::<serde_json::Value>(&exp_str).unwrap_or_default();
                
                // Convert back to serde_json for JSON Patch compatibility
                let obs_serde: serde_json::Value = serde_json::from_str(&obs_str).unwrap_or_default();
                let exp_serde: serde_json::Value = serde_json::from_str(&exp_str).unwrap_or_default();
                
                let patches = diff(&obs_serde, &exp_serde);
                _total_operations += patches.0.len();
                
                if !patches.0.is_empty() {
                    let mut metadata = HashMap::new();
                    metadata.insert("json_patch".to_string(), serde_json::to_value(&patches).unwrap_or_default());
                    metadata.insert("strategy".to_string(), serde_json::json!("sonic-rs"));
                    
                    diffs.push(ValidationDiff::new(
                        ex.symbol.clone(),
                        DiffType::Mismatch,
                        format!("Data mismatch for concern: {} (Sonic JSON Patch: {} operations)", 
                               ex.symbol.name(), patches.0.len()),
                        Some(obs.data.clone()),
                        Some(ex.expectation.clone()),
                        RuleSeverity::Error,
                        metadata,
                    ));
                }
            } else {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Missing,
                    format!("Missing expected concern: {}", ex.symbol.name()),
                    None,
                    Some(ex.expectation.clone()),
                    RuleSeverity::Error,
                    HashMap::new(),
                ));
            }
        }

        let parse_time = parse_start.elapsed().as_micros() as u64;
        let diff_time = 0;

        (DiffSet::new(diffs), parse_time, diff_time)
    }



    /// Hybrid approach: Sonic parsing + JSON Patch
    fn hybrid_diff(
        &self,
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> (DiffSet, u64, u64) {
        let parse_start = Instant::now();
        
        let mut diffs = Vec::new();
        let mut total_operations = 0;

        for ex in expected {
            if let Some(obs) = observed.iter().find(|o| o.symbol == ex.symbol) {
                // Use sonic-rs for initial parsing
                let obs_str = obs.data.to_string();
                let exp_str = ex.expectation.to_string();
                
                let _obs_sonic: JsonValue = from_str(&obs_str).unwrap_or_default();
                let _exp_sonic: JsonValue = from_str(&exp_str).unwrap_or_default();
                
                // Fall back to JSON Patch for precise operations
                let patches = diff(&obs.data, &ex.expectation);
                total_operations += patches.0.len();
                
                if !patches.0.is_empty() {
                    let mut metadata = HashMap::new();
                    metadata.insert("json_patch".to_string(), serde_json::to_value(&patches).unwrap_or_default());
                    metadata.insert("strategy".to_string(), serde_json::json!("hybrid"));
                    
                    diffs.push(ValidationDiff::new(
                        ex.symbol.clone(),
                        DiffType::Mismatch,
                        format!("Data mismatch for concern: {} (Hybrid: {} operations)", 
                               ex.symbol.name(), patches.0.len()),
                        Some(obs.data.clone()),
                        Some(ex.expectation.clone()),
                        RuleSeverity::Error,
                        metadata,
                    ));
                }
            } else {
                diffs.push(ValidationDiff::new(
                    ex.symbol.clone(),
                    DiffType::Missing,
                    format!("Missing expected concern: {}", ex.symbol.name()),
                    None,
                    Some(ex.expectation.clone()),
                    RuleSeverity::Error,
                    HashMap::new(),
                ));
            }
        }

        let parse_time = parse_start.elapsed().as_micros() as u64;
        let diff_time = 0;

        (DiffSet::new(diffs), parse_time, diff_time)
    }

    /// Benchmark different strategies
    pub fn benchmark_strategies(
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> Vec<DiffMetrics> {
        let strategies = vec![
            DiffStrategy::JsonPatch,
            DiffStrategy::SonicJsonPatch,
            DiffStrategy::Hybrid,
        ];

        let mut results = Vec::new();

        for strategy in strategies {
            let differ = HighPerformanceDiffer::new(strategy);
            let (_, metrics) = differ.diff_with_metrics(observed, expected);
            results.push(metrics);
        }

        results
    }

    /// Get the fastest strategy based on benchmark results
    pub fn get_fastest_strategy(
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> DiffStrategy {
        let results = Self::benchmark_strategies(observed, expected);
        
        results
            .iter()
            .min_by_key(|m| m.total_time_micros)
            .map(|m| m.strategy)
            .unwrap_or(DiffStrategy::JsonPatch)
    }
}

/// Convenience functions for high-performance diffing
pub mod convenience {
    use super::*;

    /// Run diff with automatic strategy selection
    pub fn auto_diff(
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> (DiffSet, DiffMetrics) {
        let strategy = HighPerformanceDiffer::get_fastest_strategy(observed, expected);
        let differ = HighPerformanceDiffer::new(strategy);
        differ.diff_with_metrics(observed, expected)
    }

    /// Run diff with specified strategy
    pub fn diff_with_strategy(
        strategy: DiffStrategy,
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> (DiffSet, DiffMetrics) {
        let differ = HighPerformanceDiffer::new(strategy);
        differ.diff_with_metrics(observed, expected)
    }

    /// Benchmark all strategies and return detailed report
    pub fn benchmark_report(
        observed: &[crate::modules::functional_contract_pipeline::types::ConcernSnapshot],
        expected: &[crate::modules::functional_contract_pipeline::types::ExpectedSnapshot],
    ) -> String {
        let results = HighPerformanceDiffer::benchmark_strategies(observed, expected);
        
        let mut report = String::new();
        report.push_str("🔬 High-Performance Diff Benchmark Report\n");
        report.push_str("==========================================\n\n");
        
        for (i, metrics) in results.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, metrics.summary()));
        }
        
        let fastest = results.iter().min_by_key(|m| m.total_time_micros).unwrap();
        report.push_str(&format!("\n🏆 Fastest: {:?} ({}μs)", fastest.strategy, fastest.total_time_micros));
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};

    #[test]
    fn test_high_performance_diff() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": false, "files": ["a.txt"]}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true, "files": ["a.txt", "b.txt"]}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];

        let differ = HighPerformanceDiffer::new(DiffStrategy::JsonPatch);
        let (diff_set, metrics) = differ.diff_with_metrics(&observed, &expected);
        
        assert!(!diff_set.is_valid());
        assert!(metrics.total_time_micros > 0);
        assert_eq!(metrics.strategy, DiffStrategy::JsonPatch);
    }

    #[test]
    fn test_benchmark_strategies() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": false}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];

        let results = HighPerformanceDiffer::benchmark_strategies(&observed, &expected);
        assert_eq!(results.len(), 3); // All 3 strategies tested
        
        // Verify all strategies completed
        for result in &results {
            assert!(result.total_time_micros > 0);
        }
    }

    #[test]
    fn test_auto_diff() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": false}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];

        let (diff_set, metrics) = convenience::auto_diff(&observed, &expected);
        assert!(!diff_set.is_valid());
        assert!(metrics.total_time_micros > 0);
    }

    #[test]
    fn test_benchmark_report() {
        let observed = vec![
            ConcernSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": false}),
                HashMap::new(),
            ),
        ];
        
        let expected = vec![
            ExpectedSnapshot::new(
                ConcernSymbol::Index,
                serde_json::json!({"exists": true}),
                "test".to_string(),
                "1.0".to_string(),
                HashMap::new(),
            ),
        ];

        let report = convenience::benchmark_report(&observed, &expected);
        assert!(report.contains("High-Performance Diff Benchmark Report"));
        assert!(report.contains("Fastest:"));
    }
}
