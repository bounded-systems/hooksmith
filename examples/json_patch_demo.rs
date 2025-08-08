use hooksmith::modules::functional_contract_pipeline::{
    FunctionalContractPipeline, HookEvent, ConcernSymbol, ContractSymbol
};
use json_patch::Patch;
use serde_json::json;

/// Example demonstrating JSON Patch integration for structural diffs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 JSON Patch Integration Demo");
    println!("=============================\n");

    // Create a new pipeline for the current repository
    let pipeline = FunctionalContractPipeline::new(".");

    // Demonstrate JSON Patch generation
    demonstrate_json_patch_generation(&pipeline)?;

    // Demonstrate JSON Patch verification
    demonstrate_json_patch_verification(&pipeline)?;

    // Demonstrate patch application
    demonstrate_patch_application()?;

    // Demonstrate structural diff analysis
    demonstrate_structural_diff_analysis()?;

    println!("✅ JSON Patch demo completed successfully!");
    Ok(())
}

/// Demonstrate JSON Patch generation for hook validation
fn demonstrate_json_patch_generation(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 JSON Patch Generation");
    println!("------------------------");

    let hooks = vec![HookEvent::PreCommit, HookEvent::PrePush];

    for hook in hooks {
        println!("\n🔍 Generating patch for hook: {:?}", hook);
        
        match pipeline.generate_pipeline_patch(hook) {
            Ok(patch) => {
                println!("  ✅ Generated patch with {} operations", patch.0.len());
                
                // Show the patch operations
                for (i, op) in patch.0.iter().enumerate() {
                    println!("    {}. {:?}", i + 1, op);
                }
            }
            Err(error) => {
                println!("  ❌ Failed to generate patch: {}", error);
            }
        }
    }

    Ok(())
}

/// Demonstrate JSON Patch verification with detailed diff analysis
fn demonstrate_json_patch_verification(pipeline: &FunctionalContractPipeline) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 JSON Patch Verification");
    println!("--------------------------");

    let hook = HookEvent::PreCommit;
    println!("\n🔍 Verifying hook: {:?}", hook);
    
    match pipeline.run_hook_with_json_patch(hook) {
        Ok(diff_set) => {
            if diff_set.is_valid() {
                println!("  ✅ Validation passed");
            } else {
                println!("  ❌ Validation failed with {} differences", diff_set.diff_count());
                
                for diff in &diff_set.diffs {
                    println!("    Concern: {}", diff.concern.name());
                    println!("    Type: {:?}", diff.diff_type);
                    println!("    Description: {}", diff.description);
                    
                    // Show JSON Patch operations if available
                    if let Some(patch_value) = diff.metadata.get("json_patch") {
                        if let Ok(patch) = serde_json::from_value::<Vec<json_patch::PatchOperation>>(patch_value.clone()) {
                            println!("    JSON Patch operations:");
                            for (i, op) in patch.iter().enumerate() {
                                println!("      {}. {:?}", i + 1, op);
                            }
                        }
                    }
                    println!();
                }
            }
        }
        Err(error) => {
            println!("  ❌ Verification failed: {}", error);
        }
    }

    Ok(())
}

/// Demonstrate applying JSON Patch to transform data
fn demonstrate_patch_application() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Patch Application Demo");
    println!("------------------------");

    // Simulate observed vs expected data
    let observed = json!({
        "index": {
            "exists": false,
            "staged_files": ["a.txt"],
            "unstaged_files": ["b.txt"]
        },
        "attributes": {
            "line_endings": "mixed",
            "binary_files": []
        }
    });

    let expected = json!({
        "index": {
            "exists": true,
            "staged_files": ["a.txt", "c.txt"],
            "unstaged_files": []
        },
        "attributes": {
            "line_endings": "normalized",
            "binary_files": ["image.png"]
        }
    });

    println!("📊 Observed state:");
    println!("  {}", serde_json::to_string_pretty(&observed)?);
    
    println!("\n📊 Expected state:");
    println!("  {}", serde_json::to_string_pretty(&expected)?);

    // Generate patch
    let patch = json_patch::diff(&observed, &expected);
    println!("\n🔧 Generated JSON Patch ({} operations):", patch.len());
    
    for (i, op) in patch.iter().enumerate() {
        println!("  {}. {:?}", i + 1, op);
    }

    // Apply patch
    let mut patched = observed.clone();
    json_patch::patch(&mut patched, &json_patch::Patch(patch))?;
    
    println!("\n✅ Applied patch result:");
    println!("  {}", serde_json::to_string_pretty(&patched)?);
    
    // Verify the patch worked
    assert_eq!(patched, expected);
    println!("  ✅ Patch application verified!");

    Ok(())
}

/// Demonstrate structural diff analysis for different concern types
fn demonstrate_structural_diff_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔬 Structural Diff Analysis");
    println!("---------------------------");

    // Test different types of structural changes
    let test_cases = vec![
        ("Object property change", 
         json!({"exists": false}), 
         json!({"exists": true})),
        
        ("Array modification", 
         json!({"files": ["a.txt", "b.txt"]}), 
         json!({"files": ["a.txt", "c.txt", "d.txt"]})),
        
        ("Nested object changes", 
         json!({"config": {"user": {"name": "old"}}}), 
         json!({"config": {"user": {"name": "new", "email": "new@example.com"}}})),
        
        ("Type changes", 
         json!({"count": 5, "enabled": false}), 
         json!({"count": "5", "enabled": true})),
    ];

    for (description, observed, expected) in test_cases {
        println!("\n📋 {}", description);
        println!("  Observed: {}", observed);
        println!("  Expected: {}", expected);
        
        let patch = json_patch::diff(&observed, &expected);
        println!("  Patch operations: {}", patch.len());
        
        for (i, op) in patch.iter().enumerate() {
            println!("    {}. {:?}", i + 1, op);
        }
    }

    Ok(())
}

/// Demonstrate performance comparison between different diff methods
fn demonstrate_performance_comparison() {
    println!("\n⚡ Performance Comparison");
    println!("-------------------------");

    let large_observed = json!({
        "files": (0..1000).map(|i| format!("file_{}.txt", i)).collect::<Vec<_>>(),
        "config": {
            "settings": (0..100).map(|i| (format!("setting_{}", i), i)).collect::<std::collections::HashMap<_, _>>()
        }
    });

    let large_expected = json!({
        "files": (0..1000).map(|i| format!("file_{}.txt", i)).collect::<Vec<_>>(),
        "config": {
            "settings": (0..100).map(|i| (format!("setting_{}", i), i + 1)).collect::<std::collections::HashMap<_, _>>()
        }
    });

    // Time JSON Patch generation
    let start = std::time::Instant::now();
    let patch = json_patch::diff(&large_observed, &large_expected);
    let json_patch_duration = start.elapsed();

    println!("📊 JSON Patch performance:");
    println!("  Operations generated: {}", patch.len());
    println!("  Time taken: {:?}", json_patch_duration);
    println!("  Operations per second: {:.0}", patch.len() as f64 / json_patch_duration.as_secs_f64());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_patch_generation() {
        let pipeline = FunctionalContractPipeline::new(".");
        let result = pipeline.generate_pipeline_patch(HookEvent::PreCommit);
        // Should not panic, even if patch is empty
        assert!(result.is_ok());
    }

    #[test]
    fn test_patch_application() {
        let observed = json!({"value": "old"});
        let expected = json!({"value": "new"});
        
        let patch = json_patch::diff(&observed, &expected);
        let mut patched = observed.clone();
        json_patch::patch(&mut patched, &json_patch::Patch(patch)).unwrap();
        
        assert_eq!(patched, expected);
    }

    #[test]
    fn test_structural_diff_analysis() {
        let observed = json!({"array": [1, 2, 3]});
        let expected = json!({"array": [1, 4, 3]});
        
        let patch = json_patch::diff(&observed, &expected);
        assert_eq!(patch.len(), 1);
        
        if let json_patch::PatchOperation::Replace { path, value } = &patch[0] {
            assert_eq!(path.as_str(), "/array/1");
            assert_eq!(value.as_u64(), Some(4));
        } else {
            panic!("Expected replace operation");
        }
    }
}
