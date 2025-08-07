use hooksmith::modules::functional_contract_pipeline::{
    ConcernSymbol, ContractRule, ContractSpec, FunctionalContractPipeline, HookEvent, RuleSeverity, RuleType
};
use std::collections::HashMap;

/// Example demonstrating the functional contract validation pipeline
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Functional Contract Validation Pipeline Demo");
    println!("==============================================\n");

    // Create a new pipeline for the current repository
    let mut pipeline = FunctionalContractPipeline::new(".");

    // Register contracts for different concerns
    register_example_contracts(&mut pipeline)?;

    // Demonstrate the pipeline with different hook events
    demonstrate_pipeline(&pipeline)?;

    Ok(())
}

/// Register example contracts for demonstration
fn register_example_contracts(pipeline: &mut FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Registering example contracts...");

    // Contract for Index concern (pre-commit)
    let index_contract = ContractSpec {
        name: "index-validation".to_string(),
        version: "1.0".to_string(),
        concern: ConcernSymbol::Index,
        rules: vec![
            ContractRule {
                name: "no-unstaged-changes".to_string(),
                description: Some("Ensure no unstaged changes before commit".to_string()),
                rule_type: RuleType::Custom,
                parameters: HashMap::new(),
                required: true,
                severity: RuleSeverity::Error,
            },
            ContractRule {
                name: "no-untracked-files".to_string(),
                description: Some("Ensure no untracked files before commit".to_string()),
                rule_type: RuleType::Custom,
                parameters: HashMap::new(),
                required: false,
                severity: RuleSeverity::Warning,
            },
        ],
        metadata: HashMap::new(),
    };

    // Contract for line ending normalization (pre-commit)
    let line_ending_contract = ContractSpec {
        name: "line-ending-normalization".to_string(),
        version: "1.0".to_string(),
        concern: ConcernSymbol::AttrLineEndingNormalization,
        rules: vec![
            ContractRule {
                name: "text-files-normalized".to_string(),
                description: Some("Ensure text files have normalized line endings".to_string()),
                rule_type: RuleType::Pattern,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("pattern".to_string(), serde_json::json!("text"));
                    params
                },
                required: true,
                severity: RuleSeverity::Error,
            },
        ],
        metadata: HashMap::new(),
    };

    // Contract for executable files (pre-push)
    let executable_contract = ContractSpec {
        name: "executable-file-validation".to_string(),
        version: "1.0".to_string(),
        concern: ConcernSymbol::TreeExecutable,
        rules: vec![
            ContractRule {
                name: "no-new-executables".to_string(),
                description: Some("Prevent new executable files from being pushed".to_string()),
                rule_type: RuleType::Custom,
                parameters: HashMap::new(),
                required: true,
                severity: RuleSeverity::Error,
            },
        ],
        metadata: HashMap::new(),
    };

    // Register all contracts
    pipeline.register_contract(index_contract)?;
    pipeline.register_contract(line_ending_contract)?;
    pipeline.register_contract(executable_contract)?;

    println!("✅ Registered 3 example contracts");
    println!();

    Ok(())
}

/// Demonstrate the pipeline with different hook events
fn demonstrate_pipeline(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Demonstrating pipeline execution...\n");

    // Test pre-commit hook
    println!("📋 Testing PreCommit hook:");
    let pre_commit_result = pipeline.execute_pipeline(&HookEvent::PreCommit)?;
    print_pipeline_result(&pre_commit_result);

    println!();

    // Test pre-push hook
    println!("📤 Testing PrePush hook:");
    let pre_push_result = pipeline.execute_pipeline(&HookEvent::PrePush)?;
    print_pipeline_result(&pre_push_result);

    println!();

    // Test individual pipeline steps
    println!("🔍 Demonstrating individual pipeline steps:");
    demonstrate_individual_steps(pipeline)?;

    Ok(())
}

/// Print pipeline result in a formatted way
fn print_pipeline_result(result: &hooksmith::modules::functional_contract_pipeline::DiffSet) {
    println!("   Result: {}", result.summary);
    println!("   Valid: {}", result.is_valid);
    println!("   Differences: {}", result.diffs.len());
    
    for (i, diff) in result.diffs.iter().enumerate() {
        println!("   {}. {} ({:?})", i + 1, diff.description, diff.severity);
    }
}

/// Demonstrate individual pipeline steps
fn demonstrate_individual_steps(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    let hook = &HookEvent::PreCommit;

    println!("   1. Identifying concerns...");
    let concerns = pipeline.identify_concerns(hook);
    println!("      Found {} concerns: {:?}", concerns.len(), concerns);

    println!("   2. Archiving concerns...");
    let observed = pipeline.archive_concerns(&concerns)?;
    println!("      Archived {} concerns", observed.snapshots.len());

    println!("   3. Mapping contracts...");
    let contracts = pipeline.map_contracts(&concerns);
    println!("      Mapped {} contracts", contracts.len());

    println!("   4. Specifying expectations...");
    let expected = pipeline.specify_expectations(&contracts)?;
    println!("      Generated {} expectations", expected.snapshots.len());

    println!("   5. Verifying...");
    let diff = pipeline.verify(&observed, &expected)?;
    println!("      Verification complete: {}", diff.summary);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = FunctionalContractPipeline::new(".");
        assert!(pipeline.identify_concerns(&HookEvent::PreCommit).len() > 0);
    }

    #[test]
    fn test_contract_registration() {
        let mut pipeline = FunctionalContractPipeline::new(".");
        
        let contract = ContractSpec {
            name: "test".to_string(),
            version: "1.0".to_string(),
            concern: ConcernSymbol::Index,
            rules: vec![],
            metadata: HashMap::new(),
        };
        
        assert!(pipeline.register_contract(contract).is_ok());
    }

    #[test]
    fn test_pipeline_execution() {
        let pipeline = FunctionalContractPipeline::new(".");
        let result = pipeline.execute_pipeline(&HookEvent::PreCommit);
        assert!(result.is_ok());
    }
}
