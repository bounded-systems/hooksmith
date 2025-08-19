use hooksmith::modules::functional_contract_pipeline::{
    ConcernSymbol, ContractSymbol, FunctionalContractPipeline, HookEvent,
};
use std::collections::HashMap;

/// Example demonstrating the functional contract validation pipeline
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Functional Contract Validation Pipeline Demo");
    println!("==============================================\n");

    // Create a new pipeline for the current repository
    let mut pipeline = FunctionalContractPipeline::new(".");

    // Demonstrate the pipeline with different hook events
    demonstrate_pipeline(&pipeline)?;

    // Demonstrate individual components
    demonstrate_components()?;

    // Demonstrate custom validation scenarios
    demonstrate_custom_validation(&pipeline)?;

    println!("✅ Demo completed successfully!");
    Ok(())
}

/// Demonstrate the full pipeline
fn demonstrate_pipeline(
    pipeline: &FunctionalContractPipeline,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Pipeline Demonstration");
    println!("-------------------------");

    let hooks = vec![
        HookEvent::PreCommit,
        HookEvent::PrePush,
        HookEvent::PreReceive,
    ];

    for hook in hooks {
        println!("\n🔍 Testing hook: {:?}", hook);

        // Run with detailed diffs to see what's happening
        let diff_set = pipeline.run_hook_with_diffs(hook);

        if diff_set.is_valid() {
            println!("  ✅ Validation passed");
            if diff_set.diff_count() > 0 {
                println!("  ⚠️  {} warnings found", diff_set.warnings().len());
            }
        } else {
            println!("  ❌ Validation failed");
            println!("  Errors: {}", diff_set.errors().len());
            println!("  Warnings: {}", diff_set.warnings().len());
        }
    }

    Ok(())
}

/// Demonstrate individual components
fn demonstrate_components() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧩 Component Demonstration");
    println!("-------------------------");

    // 1. Hook to Concerns
    println!("\n1️⃣ Hook → Concerns");
    let hook = HookEvent::PreCommit;
    let concerns = hooksmith::modules::functional_contract_pipeline::hooks::get_concerns(&hook);
    println!("  Hook: {:?}", hook);
    println!("  Concerns: {:?}", concerns);

    // 2. Concerns to Snapshots
    println!("\n2️⃣ Concerns → Snapshots");
    for concern in &concerns {
        let snapshot =
            hooksmith::modules::functional_contract_pipeline::concerns::snapshot_concern(concern);
        println!("  {:?}: {}", concern, snapshot.hash);
    }

    // 3. Concerns to Contracts
    println!("\n3️⃣ Concerns → Contracts");
    for concern in &concerns {
        let contracts =
            hooksmith::modules::functional_contract_pipeline::contracts::get_contracts(concern);
        println!("  {:?}: {:?}", concern, contracts);
    }

    // 4. Contracts to Expectations
    println!("\n4️⃣ Contracts → Expectations");
    let all_contracts =
        hooksmith::modules::functional_contract_pipeline::contracts::get_all_contracts(&concerns);
    for contract in &all_contracts {
        let expectation =
            hooksmith::modules::functional_contract_pipeline::specifier::build_expectation(
                contract,
            );
        println!("  {}: {:?}", contract.name(), expectation.symbol);
    }

    Ok(())
}

/// Demonstrate custom validation scenarios
fn demonstrate_custom_validation(
    pipeline: &FunctionalContractPipeline,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Custom Validation Scenarios");
    println!("-----------------------------");

    // Custom severity mapping
    println!("\n1️⃣ Custom Severity Mapping");
    let mut severity_map = HashMap::new();
    severity_map.insert(
        ConcernSymbol::Index,
        hooksmith::modules::functional_contract_pipeline::symbols::RuleSeverity::Warning,
    );
    severity_map.insert(
        ConcernSymbol::TreeExecutable,
        hooksmith::modules::functional_contract_pipeline::symbols::RuleSeverity::Error,
    );

    let diff_set = pipeline.run_hook_with_severity(HookEvent::PrePush, &severity_map);
    println!(
        "  PrePush with custom severity: {} diffs",
        diff_set.diff_count()
    );

    // Tolerance fields
    println!("\n2️⃣ Tolerance Fields");
    let mut tolerance_fields = HashMap::new();
    tolerance_fields.insert(ConcernSymbol::Index, vec!["timestamp".to_string()]);

    // This would be used with verify_with_tolerance in a real scenario
    println!("  Tolerance configured for Index timestamp field");

    // Custom comparison
    println!("\n3️⃣ Custom Comparison");
    println!("  Custom comparison functions available for specialized validation");

    Ok(())
}

/// Demonstrate contract registration
fn demonstrate_contract_registration() {
    println!("\n📝 Contract Registration");
    println!("------------------------");

    // Show how contracts are mapped to concerns
    let concerns = vec![
        ConcernSymbol::Index,
        ConcernSymbol::TreeExecutable,
        ConcernSymbol::AttrLineEndingNormalization,
    ];

    for concern in &concerns {
        let contracts =
            hooksmith::modules::functional_contract_pipeline::contracts::get_contracts(concern);
        println!("  {:?}:", concern);
        for contract in &contracts {
            println!("    - {}", contract.name());
        }
    }
}

/// Demonstrate hook event mapping
fn demonstrate_hook_mapping() {
    println!("\n🪝 Hook Event Mapping");
    println!("----------------------");

    let hooks = vec![
        HookEvent::PreCommit,
        HookEvent::PrePush,
        HookEvent::PreReceive,
        HookEvent::PostMerge,
    ];

    for hook in &hooks {
        let concerns = hooksmith::modules::functional_contract_pipeline::hooks::get_concerns(hook);
        println!("  {:?}: {:?}", hook, concerns);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = FunctionalContractPipeline::new(".");
        assert_eq!(pipeline.repo_path(), ".");
    }

    #[test]
    fn test_hook_concerns() {
        let concerns = hooksmith::modules::functional_contract_pipeline::hooks::get_concerns(
            &HookEvent::PreCommit,
        );
        assert!(!concerns.is_empty());
        assert!(concerns.contains(&ConcernSymbol::Index));
    }

    #[test]
    fn test_contract_mapping() {
        let contracts = hooksmith::modules::functional_contract_pipeline::contracts::get_contracts(
            &ConcernSymbol::Index,
        );
        assert!(!contracts.is_empty());
        assert!(contracts.iter().any(|c| c.name() == "must-exist"));
    }

    #[test]
    fn test_expectation_building() {
        let contract = ContractSymbol::new("must-exist");
        let expectation =
            hooksmith::modules::functional_contract_pipeline::specifier::build_expectation(
                &contract,
            );
        assert_eq!(expectation.symbol, ConcernSymbol::Index);
        assert_eq!(expectation.contract, "must-exist");
    }
}
