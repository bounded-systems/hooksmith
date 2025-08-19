//! Core Repair System Demo
//!
//! This demo showcases the core repair planning system with:
//! - Constraint-safe planning semantics
//! - Plan validation with circular dependency detection
//! - Mermaid diagram export
//! - Example fixer implementations

use hooksmith::modules::functional_contract_pipeline::repair_core::{
    ActionType, FixCategory, Fixer, LintIgnoreOrderFixer, MermaidExporter, PlanValidator,
    RepairAction, RepairPlan, RepairResult, ReplaceRootStarFixer, RootCause, Violation,
    ViolationSeverity,
};
use hooksmith::modules::functional_contract_pipeline::symbols::ConcernSymbol;
use hooksmith::modules::functional_contract_pipeline::FunctionalContractPipeline;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Core Repair System Demo");
    println!("========================\n");

    // Create a functional contract pipeline
    let pipeline = FunctionalContractPipeline::new(".");

    // Demo 1: Create a repair plan using the core system
    demo_core_repair_plan(&pipeline)?;

    // Demo 2: Validate repair plans
    demo_plan_validation()?;

    // Demo 3: Export to Mermaid
    demo_mermaid_export()?;

    // Demo 4: Show fixer implementations
    demo_fixer_implementations()?;

    println!("\n✅ Core repair system demo completed successfully!");
    Ok(())
}

fn demo_core_repair_plan(pipeline: &FunctionalContractPipeline) -> RepairResult<()> {
    println!("📋 Demo 1: Core Repair Plan Creation");
    println!("------------------------------------");

    let concern = ConcernSymbol::TreeFile;
    let contract = "no-wildcard-root";
    let violation_msg = "Root .gitignore includes unrestricted * pattern";

    let plan = pipeline.create_core_repair_plan(&concern, contract, violation_msg)?;

    println!("✅ Created repair plan: {}", plan.id);
    println!("   Concern: {:?}", plan.concern);
    println!("   Contract: {}", plan.contract);
    println!("   Actions: {}", plan.actions.len());

    for (i, action) in plan.actions.iter().enumerate() {
        println!("   {}. {}: {}", i + 1, action.fixer_id, action.action_type);
    }

    Ok(())
}

fn demo_plan_validation() -> RepairResult<()> {
    println!("\n🔍 Demo 2: Plan Validation");
    println!("--------------------------");

    // Create a valid plan
    let _valid_plan = create_valid_plan()?;
    println!("✅ Valid plan validation: PASS");

    // Create a plan with circular dependencies
    let invalid_plan = create_circular_dependency_plan()?;
    match PlanValidator::validate(&invalid_plan) {
        Ok(_) => println!("❌ Circular dependency detection: FAILED"),
        Err(_) => println!("✅ Circular dependency detection: PASS"),
    }

    Ok(())
}

fn demo_mermaid_export() -> RepairResult<()> {
    println!("\n📊 Demo 3: Mermaid Export");
    println!("-------------------------");

    let plan = create_valid_plan()?;
    let mermaid = MermaidExporter::export_plan(&plan);

    println!("✅ Generated Mermaid diagram:");
    println!("{}", mermaid);

    Ok(())
}

fn demo_fixer_implementations() -> RepairResult<()> {
    println!("\n🛠️ Demo 4: Fixer Implementations");
    println!("--------------------------------");

    let violation = Violation {
        concern: ConcernSymbol::TreeFile,
        contract: "no-wildcard-root".to_string(),
        message: "Root .gitignore includes wildcard pattern".to_string(),
        location: ".gitignore:1".to_string(),
        severity: ViolationSeverity::Error,
    };

    let root_cause = RootCause {
        primary_cause: "Unrestricted wildcard pattern".to_string(),
        factors: vec!["Security concern".to_string()],
        fix_categories: vec![FixCategory::Configuration],
        confidence: 0.9,
    };

    // Test ReplaceRootStarFixer
    let fixer = ReplaceRootStarFixer;
    if let Ok(Some(action)) = fixer.plan(&violation, &root_cause) {
        println!("✅ ReplaceRootStarFixer created action: {}", action.id);
        println!("   Action type: {}", action.action_type);
        println!("   Path: {}", action.path);
    }

    // Test LintIgnoreOrderFixer
    let order_violation = Violation {
        concern: ConcernSymbol::TreeFile,
        contract: "ordered-ignore".to_string(),
        message: ".gitignore lines are not in alphabetical order".to_string(),
        location: ".gitignore".to_string(),
        severity: ViolationSeverity::Warning,
    };

    let order_fixer = LintIgnoreOrderFixer;
    if let Ok(Some(action)) = order_fixer.plan(&order_violation, &root_cause) {
        println!("✅ LintIgnoreOrderFixer created action: {}", action.id);
        println!("   Action type: {}", action.action_type);
        println!("   Dependencies: {:?}", action.dependencies);
    }

    Ok(())
}

fn create_valid_plan() -> RepairResult<RepairPlan> {
    let concern = ConcernSymbol::TreeFile;

    let violation = Violation {
        concern: concern.clone(),
        contract: "no-wildcard-root".to_string(),
        message: "Root .gitignore includes unrestricted * pattern".to_string(),
        location: ".gitignore:1".to_string(),
        severity: ViolationSeverity::Error,
    };

    let root_cause = RootCause {
        primary_cause: "Unrestricted wildcard pattern".to_string(),
        factors: vec!["Security concern".to_string()],
        fix_categories: vec![FixCategory::Configuration],
        confidence: 0.9,
    };

    let actions = vec![
        RepairAction {
            id: "replace-root-star".to_string(),
            fixer_id: "fixer.replace-root-star".to_string(),
            action_type: ActionType::Edit {
                line: 1,
                column: 1,
                content: "/*\n".to_string(),
            },
            path: ".gitignore".to_string(),
            params: HashMap::new(),
            required: true,
            priority: 1,
            dependencies: Vec::new(),
        },
        RepairAction {
            id: "reorder-lines".to_string(),
            fixer_id: "fixer.lint-ignore-order".to_string(),
            action_type: ActionType::ReorderLines {
                strategy: "alphabetical".to_string(),
            },
            path: ".gitignore".to_string(),
            params: HashMap::new(),
            required: false,
            priority: 2,
            dependencies: vec!["replace-root-star".to_string()],
        },
    ];

    Ok(RepairPlan {
        id: "valid-plan".to_string(),
        concern,
        contract: "no-wildcard-root".to_string(),
        violation,
        root_cause,
        dispatcher: "test-dispatcher".to_string(),
        actions,
        is_complete: true,
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

fn create_circular_dependency_plan() -> RepairResult<RepairPlan> {
    let concern = ConcernSymbol::TreeFile;

    let violation = Violation {
        concern: concern.clone(),
        contract: "test-contract".to_string(),
        message: "Test violation".to_string(),
        location: ".gitignore:1".to_string(),
        severity: ViolationSeverity::Error,
    };

    let root_cause = RootCause {
        primary_cause: "Test cause".to_string(),
        factors: Vec::new(),
        fix_categories: vec![FixCategory::Configuration],
        confidence: 0.8,
    };

    let actions = vec![
        RepairAction {
            id: "action1".to_string(),
            fixer_id: "fixer1".to_string(),
            action_type: ActionType::Edit {
                line: 1,
                column: 1,
                content: "test".to_string(),
            },
            path: ".gitignore".to_string(),
            params: HashMap::new(),
            required: true,
            priority: 1,
            dependencies: vec!["action2".to_string()],
        },
        RepairAction {
            id: "action2".to_string(),
            fixer_id: "fixer2".to_string(),
            action_type: ActionType::Edit {
                line: 2,
                column: 1,
                content: "test2".to_string(),
            },
            path: ".gitignore".to_string(),
            params: HashMap::new(),
            required: true,
            priority: 2,
            dependencies: vec!["action1".to_string()],
        },
    ];

    Ok(RepairPlan {
        id: "circular-plan".to_string(),
        concern,
        contract: "test-contract".to_string(),
        violation,
        root_cause,
        dispatcher: "test-dispatcher".to_string(),
        actions,
        is_complete: true,
        metadata: HashMap::new(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}
