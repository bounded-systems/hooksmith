use git_filter::{
    blob_contract::{BlobContract, BlobValidator},
    git_object_contract::{GitObjectType, GitObjectValidator},
    tree_contract::{TreeEntryContract, TreeObjectContract, TreeValidator},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Combined Contract Demo - Attributes with Generated Files\n");

    // Example 1: Tree Entry with Attributes
    demo_tree_entry_attributes()?;

    // Example 2: Blob Contract with Attributes
    demo_blob_contract_attributes()?;

    // Example 3: Git Object Contract with Attributes
    demo_git_object_contract_attributes()?;

    // Example 4: Generated Files Validation
    demo_generated_files_validation()?;

    // Example 5: Complete Workflow
    demo_complete_workflow()?;

    Ok(())
}

fn demo_tree_entry_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 1: Tree Entry with Attributes");
    println!("==========================================\n");

    // Create a tree entry for a generated file
    let entry = TreeEntryContract::new_with_attributes(
        "100644",
        "target/build/app.js".to_string(),
        "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        Some(vec![
            "linguist-generated=true".to_string(),
            "-diff".to_string(),
        ]),
    );

    println!("✅ Generated file with correct attributes:");
    println!("  {}", entry.summary());

    // Create a tree entry for a non-generated file
    let entry2 = TreeEntryContract::new_with_attributes(
        "100644",
        "src/main.rs".to_string(),
        "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        Some(vec!["text".to_string()]),
    );

    println!("✅ Source file with appropriate attributes:");
    println!("  {}", entry2.summary());

    // Create a tree entry for a generated file without required attributes
    let entry3 = TreeEntryContract::new_with_attributes(
        "100644",
        "target/build/file.js".to_string(),
        "c3d4e5f6789012345678901234567890abcdef".to_string(),
        None, // Missing linguist-generated=true
    );

    println!("❌ Generated file missing required attributes:");
    println!("  {}", entry3.summary());

    println!();
    Ok(())
}

fn demo_blob_contract_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 2: Blob Contract with Attributes");
    println!("============================================\n");

    let mut blob = BlobContract::new_with_attributes(
        "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        1024,
        Some(vec![
            "linguist-generated=true".to_string(),
            "-diff".to_string(),
        ]),
    );

    println!("✅ Blob with attributes:");
    println!("  {}", blob.summary());

    // Validate attributes for a generated file path
    let is_valid = blob.validate_attributes_for_path("target/build/app.js");
    println!(
        "  Validation for generated file path: {}",
        if is_valid { "✅ PASS" } else { "❌ FAIL" }
    );

    // Validate attributes for a non-generated file path
    let is_valid2 = blob.validate_attributes_for_path("src/main.rs");
    println!(
        "  Validation for source file path: {}",
        if is_valid2 { "✅ PASS" } else { "❌ FAIL" }
    );

    println!();
    Ok(())
}

fn demo_git_object_contract_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 3: Git Object Contract with Attributes");
    println!("==================================================\n");

    let tree_validator = TreeValidator::new(true, true, true);
    let validator = GitObjectValidator::new(true, true, true, true, tree_validator);

    // Validate a blob object with attributes
    let contract = validator.validate_object(
        GitObjectType::Blob,
        "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        1024,
        Some(vec![
            "linguist-generated=true".to_string(),
            "-diff".to_string(),
        ]),
        Some("target/build/app.js"),
    );

    println!("✅ Git object with attributes:");
    println!("  {}", contract.summary());

    // Validate a tree object with attributes
    let tree_contract = validator.validate_object(
        GitObjectType::Tree,
        "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        512,
        Some(vec!["-diff".to_string()]),
        Some("target/"),
    );

    println!("✅ Tree object with attributes:");
    println!("  {}", tree_contract.summary());

    println!();
    Ok(())
}

fn demo_generated_files_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 4: Generated Files Validation");
    println!("==========================================\n");

    let tree_validator = TreeValidator::new(true, true, true);
    let validator = GitObjectValidator::new(true, true, true, true, tree_validator);

    // Test various generated file patterns
    let test_cases = vec![
        ("target/build/app.js", true, true), // Generated, should have linguist-generated=true
        ("gen/proto/message.rs", true, true), // Generated, should have linguist-generated=true
        ("dist/bundle.js", true, true),      // Generated, should have linguist-generated=true
        ("src/main.rs", false, false),       // Source, should not have linguist-generated=true
        ("docs/README.md", false, false),    // Docs, should not have linguist-generated=true
    ];

    for (filepath, is_generated, should_have_linguist) in test_cases {
        let attributes = if should_have_linguist {
            Some(vec!["linguist-generated=true".to_string()])
        } else {
            None
        };

        let contract = validator.validate_object(
            GitObjectType::Blob,
            format!("hash_{}", filepath.replace('/', "_")),
            1024,
            attributes,
            Some(filepath),
        );

        let status = if contract.is_valid() { "✅" } else { "❌" };
        println!("{} {} -> {}", status, filepath, contract.summary());
    }

    println!();
    Ok(())
}

fn demo_complete_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Example 5: Complete Workflow");
    println!("===============================\n");

    let tree_validator = TreeValidator::new(true, true, true);
    let git_validator = GitObjectValidator::new(true, true, true, true, tree_validator);
    let blob_validator = BlobValidator::new(true, true, 0.1, false);

    // Simulate a commit with multiple files
    let commit_files = vec![
        // Source files (should not have linguist-generated=true)
        (
            "src/main.rs",
            "fn main() { println!(\"Hello, World!\"); }",
            None,
        ),
        ("src/lib.rs", "pub fn hello() { \"Hello\" }", None),
        // Generated files (should have linguist-generated=true)
        (
            "target/build/app.js",
            "console.log('Hello, World!');",
            Some(vec![
                "linguist-generated=true".to_string(),
                "-diff".to_string(),
            ]),
        ),
        (
            "gen/proto/message.rs",
            "pub struct Message { pub content: String }",
            Some(vec!["linguist-generated=true".to_string()]),
        ),
        // Generated file missing required attribute
        ("target/build/file.js", "console.log('Generated');", None),
    ];

    let mut all_contracts = Vec::new();
    let mut tree_entries = Vec::new();

    for (filepath, content, attributes) in commit_files {
        println!("Processing: {filepath}");

        // Create blob contract
        let (blob_contract, _) = blob_validator.validate_blob_simple(
            &format!("hash_{}", filepath.replace('/', "_")),
            content.as_bytes(),
        );

        // Add attributes to blob
        if let Some(attrs) = attributes {
            let mut blob_with_attrs = blob_contract.clone();
            blob_with_attrs.add_attributes(attrs.clone());

            // Validate attributes for the filepath
            blob_with_attrs.validate_attributes_for_path(filepath);

            // Create tree entry
            let tree_entry = TreeEntryContract::new_with_attributes(
                "100644",
                filepath.to_string(),
                blob_with_attrs.oid.clone(),
                Some(attrs),
            );

            // Create git object contract
            let git_contract = git_validator.validate_blob(&blob_with_attrs, Some(filepath));

            all_contracts.push(git_contract);
            tree_entries.push(tree_entry);
        } else {
            // Create tree entry without attributes
            let tree_entry =
                TreeEntryContract::new("100644", filepath.to_string(), blob_contract.oid.clone());

            // Create git object contract
            let git_contract = git_validator.validate_blob(&blob_contract, Some(filepath));

            all_contracts.push(git_contract);
            tree_entries.push(tree_entry);
        }
    }

    // Create tree object
    let tree_object = TreeObjectContract::new(tree_entries);

    println!("\n📊 Validation Summary:");
    println!("=====================");
    println!("{}", git_validator.summarize_validation(&all_contracts));

    println!("\n🌳 Tree Object Summary:");
    println!("=====================");
    println!("{}", tree_object.summary());

    println!("\n📋 Individual File Results:");
    println!("===========================");
    for contract in &all_contracts {
        println!("  {}", contract.summary());
    }

    Ok(())
}
