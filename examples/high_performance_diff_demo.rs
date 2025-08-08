use hooksmith::modules::functional_contract_pipeline::{
    FunctionalContractPipeline, HookEvent, ConcernSymbol, ContractSymbol,
    high_performance_diff::{DiffStrategy, HighPerformanceDiffer, convenience}
};
use serde_json::json;

/// Example demonstrating high-performance diffing capabilities
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 High-Performance Diffing Demo");
    println!("================================\n");

    // Create a new pipeline for the current repository
    let pipeline = FunctionalContractPipeline::new(".");

    // Demonstrate different diffing strategies
    demonstrate_diff_strategies(&pipeline)?;

    // Benchmark all strategies
    demonstrate_benchmarking(&pipeline)?;

    // Show automatic strategy selection
    demonstrate_auto_selection(&pipeline)?;

    // Demonstrate performance comparison
    demonstrate_performance_comparison()?;

    println!("✅ High-performance diffing demo completed successfully!");
    Ok(())
}

/// Demonstrate different diffing strategies
fn demonstrate_diff_strategies(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Diff Strategy Demonstration");
    println!("-----------------------------\n");

    let strategies = vec![
        DiffStrategy::JsonPatch,
        DiffStrategy::SonicJsonPatch,
        DiffStrategy::DiffxSemantic,
        DiffStrategy::Hybrid,
    ];

    let hook = HookEvent::PreCommit;

    for strategy in strategies {
        println!("📋 Testing strategy: {:?}", strategy);
        
        let (diff_set, metrics) = pipeline.run_hook_with_diff_strategy(hook, strategy);
        
        println!("  Performance: {}", metrics.summary());
        println!("  Valid: {}", diff_set.is_valid());
        println!("  Diffs: {}", diff_set.diff_count());
        
        // Show strategy-specific metadata
        for diff in &diff_set.diffs {
            if let Some(strategy_info) = diff.metadata.get("strategy") {
                println!("    Strategy used: {}", strategy_info);
            }
        }
        println!();
    }

    Ok(())
}

/// Demonstrate benchmarking capabilities
fn demonstrate_benchmarking(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Benchmarking Demonstration");
    println!("-----------------------------\n");

    let hooks = vec![HookEvent::PreCommit, HookEvent::PrePush];

    for hook in hooks {
        println!("🔍 Benchmarking hook: {:?}", hook);
        
        let report = pipeline.benchmark_hook_strategies(hook);
        println!("{}", report);
        println!();
    }

    Ok(())
}

/// Demonstrate automatic strategy selection
fn demonstrate_auto_selection(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Automatic Strategy Selection");
    println!("-------------------------------\n");

    let hooks = vec![HookEvent::PreCommit, HookEvent::PrePush, HookEvent::PreReceive];

    for hook in hooks {
        println!("🔍 Auto-selecting strategy for hook: {:?}", hook);
        
        let (diff_set, metrics) = pipeline.run_hook_with_high_performance_diff(hook);
        
        println!("  Selected strategy: {:?}", metrics.strategy);
        println!("  Performance: {}μs", metrics.total_time_micros);
        println!("  Operations: {}", metrics.operations_generated);
        println!("  Valid: {}", diff_set.is_valid());
        println!();
    }

    Ok(())
}

/// Demonstrate performance comparison with synthetic data
fn demonstrate_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ Performance Comparison with Synthetic Data");
    println!("---------------------------------------------\n");

    // Create synthetic test data with varying complexity
    let test_cases = vec![
        ("Simple object change", create_simple_test_data()),
        ("Array modification", create_array_test_data()),
        ("Nested object changes", create_nested_test_data()),
        ("Large structure", create_large_test_data()),
    ];

    for (description, (observed, expected)) in test_cases {
        println!("📋 {}", description);
        
        let results = HighPerformanceDiffer::benchmark_strategies(&observed, &expected);
        
        // Sort by performance
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|m| m.total_time_micros);
        
        for (i, metrics) in sorted_results.iter().enumerate() {
            let rank = if i == 0 { "🥇" } else if i == 1 { "🥈" } else if i == 2 { "🥉" } else { "  " };
            println!("  {} {:?}: {}μs", rank, metrics.strategy, metrics.total_time_micros);
        }
        println!();
    }

    Ok(())
}

/// Create simple test data
fn create_simple_test_data() -> (Vec<hooksmith::modules::functional_contract_pipeline::types::ConcernSnapshot>, Vec<hooksmith::modules::functional_contract_pipeline::types::ExpectedSnapshot>) {
    use hooksmith::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
    use std::collections::HashMap;

    let observed = vec![
        ConcernSnapshot::new(
            ConcernSymbol::Index,
            json!({"exists": false, "count": 5}),
            HashMap::new(),
        ),
    ];
    
    let expected = vec![
        ExpectedSnapshot::new(
            ConcernSymbol::Index,
            json!({"exists": true, "count": 10}),
            "test".to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
    ];

    (observed, expected)
}

/// Create array test data
fn create_array_test_data() -> (Vec<hooksmith::modules::functional_contract_pipeline::types::ConcernSnapshot>, Vec<hooksmith::modules::functional_contract_pipeline::types::ExpectedSnapshot>) {
    use hooksmith::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
    use std::collections::HashMap;

    let observed = vec![
        ConcernSnapshot::new(
            ConcernSymbol::Index,
            json!({"files": ["a.txt", "b.txt"], "config": {"enabled": false}}),
            HashMap::new(),
        ),
    ];
    
    let expected = vec![
        ExpectedSnapshot::new(
            ConcernSymbol::Index,
            json!({"files": ["a.txt", "c.txt", "d.txt"], "config": {"enabled": true}}),
            "test".to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
    ];

    (observed, expected)
}

/// Create nested test data
fn create_nested_test_data() -> (Vec<hooksmith::modules::functional_contract_pipeline::types::ConcernSnapshot>, Vec<hooksmith::modules::functional_contract_pipeline::types::ExpectedSnapshot>) {
    use hooksmith::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
    use std::collections::HashMap;

    let observed = vec![
        ConcernSnapshot::new(
            ConcernSymbol::Index,
            json!({
                "config": {
                    "user": {"name": "old", "email": "old@example.com"},
                    "settings": {"theme": "dark", "notifications": false}
                },
                "files": ["a.txt", "b.txt"]
            }),
            HashMap::new(),
        ),
    ];
    
    let expected = vec![
        ExpectedSnapshot::new(
            ConcernSymbol::Index,
            json!({
                "config": {
                    "user": {"name": "new", "email": "new@example.com", "role": "admin"},
                    "settings": {"theme": "light", "notifications": true}
                },
                "files": ["a.txt", "c.txt", "d.txt"]
            }),
            "test".to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
    ];

    (observed, expected)
}

/// Create large test data
fn create_large_test_data() -> (Vec<hooksmith::modules::functional_contract_pipeline::types::ConcernSnapshot>, Vec<hooksmith::modules::functional_contract_pipeline::types::ExpectedSnapshot>) {
    use hooksmith::modules::functional_contract_pipeline::types::{ConcernSnapshot, ExpectedSnapshot};
    use std::collections::HashMap;

    // Create large arrays and objects
    let large_array: Vec<String> = (0..100).map(|i| format!("file_{}.txt", i)).collect();
    let large_config: serde_json::Value = {
        let mut config = serde_json::Map::new();
        for i in 0..50 {
            config.insert(format!("setting_{}", i), json!(i));
        }
        serde_json::Value::Object(config)
    };

    let observed = vec![
        ConcernSnapshot::new(
            ConcernSymbol::Index,
            json!({
                "files": large_array,
                "config": large_config,
                "metadata": {"version": "1.0", "status": "old"}
            }),
            HashMap::new(),
        ),
    ];
    
    let expected = vec![
        ExpectedSnapshot::new(
            ConcernSymbol::Index,
            json!({
                "files": large_array,
                "config": large_config,
                "metadata": {"version": "2.0", "status": "new", "updated": true}
            }),
            "test".to_string(),
            "1.0".to_string(),
            HashMap::new(),
        ),
    ];

    (observed, expected)
}

/// Demonstrate strategy-specific features
fn demonstrate_strategy_features() {
    println!("🎯 Strategy-Specific Features");
    println!("-----------------------------\n");

    println!("📋 JSON Patch Strategy:");
    println!("  • RFC 6902 compliant");
    println!("  • Machine-readable output");
    println!("  • Precise structural operations");
    println!("  • Best for: Contract validation, CI/CD integration");
    println!();

    println!("📋 Sonic JSON Patch Strategy:");
    println!("  • SIMD-optimized parsing");
    println!("  • Zero temporary data structures");
    println!("  • Memory arena optimization");
    println!("  • Best for: High-throughput JSON workloads");
    println!();

    println!("📋 Diffx Semantic Strategy:");
    println!("  • Human-readable output");
    println!("  • Semantic difference detection");
    println!("  • Ignores formatting differences");
    println!("  • Best for: Debugging, reporting");
    println!();

    println!("📋 Hybrid Strategy:");
    println!("  • Combines Sonic parsing with JSON Patch");
    println!("  • Best of both worlds");
    println!("  • Automatic fallback");
    println!("  • Best for: Balanced performance and precision");
    println!();
}

/// Demonstrate real-world usage patterns
fn demonstrate_usage_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Real-World Usage Patterns");
    println!("-----------------------------\n");

    // Pattern 1: CI/CD Integration
    println!("1️⃣ CI/CD Integration Pattern");
    println!("   Use JSON Patch for machine-readable output");
    let (diff_set, metrics) = convenience::diff_with_strategy(
        DiffStrategy::JsonPatch,
        &create_simple_test_data().0,
        &create_simple_test_data().1,
    );
    println!("   Strategy: {:?}, Time: {}μs", metrics.strategy, metrics.total_time_micros);
    println!();

    // Pattern 2: Development Workflow
    println!("2️⃣ Development Workflow Pattern");
    println!("   Use Diffx for human-readable debugging");
    let (diff_set, metrics) = convenience::diff_with_strategy(
        DiffStrategy::DiffxSemantic,
        &create_nested_test_data().0,
        &create_nested_test_data().1,
    );
    println!("   Strategy: {:?}, Time: {}μs", metrics.strategy, metrics.total_time_micros);
    println!();

    // Pattern 3: High-Performance Processing
    println!("3️⃣ High-Performance Processing Pattern");
    println!("   Use Sonic for maximum throughput");
    let (diff_set, metrics) = convenience::diff_with_strategy(
        DiffStrategy::SonicJsonPatch,
        &create_large_test_data().0,
        &create_large_test_data().1,
    );
    println!("   Strategy: {:?}, Time: {}μs", metrics.strategy, metrics.total_time_micros);
    println!();

    // Pattern 4: Adaptive Processing
    println!("4️⃣ Adaptive Processing Pattern");
    println!("   Use auto-selection for optimal performance");
    let (diff_set, metrics) = convenience::auto_diff(
        &create_array_test_data().0,
        &create_array_test_data().1,
    );
    println!("   Auto-selected: {:?}, Time: {}μs", metrics.strategy, metrics.total_time_micros);
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_strategies() {
        let pipeline = FunctionalContractPipeline::new(".");
        let result = demonstrate_diff_strategies(&pipeline);
        assert!(result.is_ok());
    }

    #[test]
    fn test_benchmarking() {
        let pipeline = FunctionalContractPipeline::new(".");
        let result = demonstrate_benchmarking(&pipeline);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auto_selection() {
        let pipeline = FunctionalContractPipeline::new(".");
        let result = demonstrate_auto_selection(&pipeline);
        assert!(result.is_ok());
    }

    #[test]
    fn test_performance_comparison() {
        let result = demonstrate_performance_comparison();
        assert!(result.is_ok());
    }

    #[test]
    fn test_usage_patterns() {
        let result = demonstrate_usage_patterns();
        assert!(result.is_ok());
    }
}
