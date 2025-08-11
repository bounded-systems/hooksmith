use hooksmith::modules::git_native::GitNativeValidator;

fn main() {
    // Test the new attribute concerns
    let concerns = vec![
        "attr-line-ending-normalization".to_string(),
        "attr-diff-strategy".to_string(),
        "attr-merge-strategy".to_string(),
        "attr-export-control".to_string(),
        "attr-filter-driver".to_string(),
        "attr-external-tool-hint".to_string(),
        "attr-locking-hint".to_string(),
    ];

    match GitNativeValidator::validate_concerns(&concerns) {
        Ok(()) => println!("✅ All attribute concerns are valid!"),
        Err(e) => println!("❌ Validation failed: {}", e),
    }

    // Test mapping functions
    println!("\nTesting attribute type mapping:");
    println!("attr-line-ending-normalization: {:?}", 
        GitNativeValidator::map_attribute_type("attr-line-ending-normalization"));
    println!("attr-diff-strategy: {:?}", 
        GitNativeValidator::map_attribute_type("attr-diff-strategy"));
    println!("attr-merge-strategy: {:?}", 
        GitNativeValidator::map_attribute_type("attr-merge-strategy"));
    println!("attr-export-control: {:?}", 
        GitNativeValidator::map_attribute_type("attr-export-control"));
    println!("attr-filter-driver: {:?}", 
        GitNativeValidator::map_attribute_type("attr-filter-driver"));
    println!("attr-external-tool-hint: {:?}", 
        GitNativeValidator::map_attribute_type("attr-external-tool-hint"));
    println!("attr-locking-hint: {:?}", 
        GitNativeValidator::map_attribute_type("attr-locking-hint"));

    println!("\nCanonical attribute types:");
    for attr_type in GitNativeValidator::canonical_attribute_types() {
        println!("  - {}", attr_type);
    }
}
