use git_filter::{
    blob_contract::{BlobContract, BlobValidator},
    git_object_contract::{GitObjectContract, GitObjectType, GitObjectValidator},
    tree_contract::{TreeEntryContract, TreeObjectContract, TreeValidator},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Simple Attributes Validation Test\n");

    // Test 1: Tree Entry with Attributes
    test_tree_entry_attributes()?;

    // Test 2: Blob Contract with Attributes
    test_blob_contract_attributes()?;

    // Test 3: Git Object Contract with Attributes
    test_git_object_contract_attributes()?;

    // Test 4: Generated File Detection
    test_generated_file_detection()?;

    println!("✅ All tests passed!");
    Ok(())
}

fn test_tree_entry_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Test 1: Tree Entry with Attributes");

    // Test generated file with correct attributes
    let entry = TreeEntryContract::new_with_attributes(
        "100644",
        "target/build/app.js".to_string(),
        "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        Some(vec![
            "linguist-generated=true".to_string(),
            "-diff".to_string(),
        ]),
    );

    println!("  ✅ Generated file with correct attributes: {}", entry.is_valid());
    assert!(entry.is_valid());
    assert!(entry.has_attribute("linguist-generated=true"));
    assert!(entry.has_attribute("-diff"));
    assert_eq!(entry.get_attribute_value("linguist-generated"), Some("true"));

    // Test generated file without required attributes
    let entry2 = TreeEntryContract::new_with_attributes(
        "100644",
        "target/build/file.js".to_string(),
        "b2c3d4e5f6789012345678901234567890abcde1".to_string(),
        None,
    );

    println!("  ❌ Generated file missing required attributes: {}", !entry2.is_valid());
    println!("    Entry2 errors: {:?}", entry2.errors);
    println!("    Entry2 valid: {}", entry2.is_valid());
    assert!(!entry2.is_valid());
    
    let has_linguist_error = entry2.errors.iter().any(|e| e.contains("linguist-generated=true"));
    if !has_linguist_error {
        println!("    Entry2 errors: {:?}", entry2.errors);
    }
    assert!(has_linguist_error);

    // Test source file without linguist-generated (should be valid)
    let entry3 = TreeEntryContract::new_with_attributes(
        "100644",
        "src/main.rs".to_string(),
        "1234567890abcdef1234567890abcdef12345678".to_string(),
        Some(vec!["text=true".to_string()]),
    );

    println!("  ✅ Source file without linguist-generated: {}", entry3.is_valid());
    println!("    Entry3 errors: {:?}", entry3.errors);
    assert!(entry3.is_valid());

    println!();
    Ok(())
}

fn test_blob_contract_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Test 2: Blob Contract with Attributes");

    let mut blob = BlobContract::new_with_attributes(
        "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        1024,
        Some(vec![
            "linguist-generated=true".to_string(),
            "-diff".to_string(),
        ]),
    );

    // Test validation for generated file path
    let is_valid = blob.validate_attributes_for_path("target/build/app.js");
    println!("  ✅ Generated file path validation: {}", is_valid);
    assert!(is_valid);

    // Test validation for non-generated file path
    let is_valid2 = blob.validate_attributes_for_path("src/main.rs");
    println!("  ✅ Non-generated file path validation: {}", is_valid2);
    assert!(is_valid2);

    // Test blob without linguist-generated for generated file
    let mut blob2 = BlobContract::new(
        "b2c3d4e5f6789012345678901234567890abcde1".to_string(),
        1024,
    );
    let is_valid3 = blob2.validate_attributes_for_path("target/build/file.js");
    println!("  ❌ Generated file without linguist-generated: {}", !is_valid3);
    assert!(!is_valid3);

    println!();
    Ok(())
}

fn test_git_object_contract_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Test 3: Git Object Contract with Attributes");

    let tree_validator = TreeValidator::new(true, true, true);
    let validator = GitObjectValidator::new(true, true, tree_validator);

    // Test generated file with linguist-generated=true
    let contract = validator.validate_object(
        GitObjectType::Blob,
        "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        1024,
        Some(vec!["linguist-generated=true".to_string()]),
        Some("target/build/app.js"),
    );

    println!("  ✅ Generated file with linguist-generated=true: {}", contract.is_valid());
    assert!(contract.is_valid());

    // Test generated file without linguist-generated=true
    let contract2 = validator.validate_object(
        GitObjectType::Blob,
        "b2c3d4e5f6789012345678901234567890abcde1".to_string(),
        1024,
        None,
        Some("target/build/file.js"),
    );

    println!("  ❌ Generated file without linguist-generated=true: {}", !contract2.is_valid());
    assert!(!contract2.is_valid());

    // Test source file without linguist-generated=true (should be valid)
    let contract3 = validator.validate_object(
        GitObjectType::Blob,
        "c3d4e5f6789012345678901234567890abcdef1".to_string(),
        1024,
        None,
        Some("src/main.rs"),
    );

    println!("  ✅ Source file without linguist-generated=true: {}", contract3.is_valid());
    assert!(contract3.is_valid());

    println!();
    Ok(())
}

fn test_generated_file_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Test 4: Generated File Detection");

    let tree_validator = TreeValidator::new(true, true, true);
    let validator = GitObjectValidator::new(true, true, tree_validator);

    // Test directory patterns
    let directory_patterns = vec![
        "target/file.js",
        "gen/proto/message.rs",
        "generated/api/client.ts",
        "build/dist/app.js",
        "dist/bundle.js",
        "node_modules/react/index.js",
    ];

    for filepath in directory_patterns {
        let contract = validator.validate_object(
            GitObjectType::Blob,
            format!("{:040x}", filepath.len() as u64),
            1024,
            None, // No linguist-generated attribute
            Some(filepath),
        );
        println!("  ❌ {} (should require linguist-generated=true): {}", filepath, !contract.is_valid());
        assert!(!contract.is_valid());
    }

    // Test file patterns
    let file_patterns = vec![
        "app.min.js",
        "styles.min.css",
        "bundle.js",
        "vendor.bundle.css",
    ];

    for filepath in file_patterns {
        let contract = validator.validate_object(
            GitObjectType::Blob,
            format!("{:040x}", filepath.len() as u64),
            1024,
            None, // No linguist-generated attribute
            Some(filepath),
        );
        println!("  ❌ {} (should require linguist-generated=true): {}", filepath, !contract.is_valid());
        assert!(!contract.is_valid());
    }

    // Test non-generated files
    let non_generated_files = vec![
        "src/main.rs",
        "docs/README.md",
        "tests/test.rs",
        "examples/demo.rs",
        "Cargo.toml",
        ".gitignore",
    ];

    for filepath in non_generated_files {
        let contract = validator.validate_object(
            GitObjectType::Blob,
            format!("{:040x}", filepath.len() as u64),
            1024,
            None, // No linguist-generated attribute
            Some(filepath),
        );
        println!("  ✅ {} (should be valid without linguist-generated=true): {}", filepath, contract.is_valid());
        assert!(contract.is_valid());
    }

    println!();
    Ok(())
} 