use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌳 Tree Contract Demo - Git Tree Object Validation\n");

    // Example 1: Tree mode validation
    demo_tree_mode_validation()?;

    // Example 2: Tree entry contracts
    demo_tree_entry_contracts()?;

    // Example 3: Tree object contracts
    demo_tree_object_contracts()?;

    // Example 4: Tree validation with errors
    demo_tree_validation_with_errors()?;

    // Example 5: Complete tree object validation
    demo_complete_tree_object_validation()?;

    // Example 6: Tree object in Git object contract
    demo_tree_in_git_object_contract()?;

    Ok(())
}

fn demo_tree_mode_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 1: Tree Mode Validation");

    let modes = vec![
        ("100644", "Regular file (non-executable)"),
        ("100755", "Regular file (executable)"),
        ("040000", "Tree (directory)"),
    ];

    for (mode_str, description) in modes {
        match TreeMode::parse_from_str(mode_str) {
            Some(mode) => {
                println!("  ✅ Mode {}: {}", mode_str, mode.description());
                println!("    Mode string: {}", mode.to_mode_string());
                println!("    Is blob: {}", mode.is_blob());
                println!("    Is tree: {}", mode.is_tree());
                println!("    Object type: {:?}", mode.object_type());
            }
            None => {
                println!("  ❌ Invalid mode: {mode_str}");
            }
        }
        println!();
    }

    // Test invalid mode
    let invalid_mode = TreeMode::parse_from_str("999999");
    println!("  Invalid mode test: {invalid_mode:?}");

    println!();
    Ok(())
}

fn demo_tree_entry_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Example 2: Tree Entry Contracts");

    let entries = vec![
        (
            "100644",
            "README.md",
            "a1b2c3d4e5f6789012345678901234567890abcd",
        ),
        (
            "100755",
            "script.sh",
            "b2c3d4e5f6789012345678901234567890abcde",
        ),
        ("040000", "src", "c3d4e5f6789012345678901234567890abcdef"),
    ];

    for (mode, filename, object_id) in entries {
        let entry = TreeEntryContract::new(mode, filename.to_string(), object_id.to_string());
        println!("  {}", entry.summary());
        println!(
            "    Mode: {} ({})",
            entry.mode_string(),
            entry.mode.description()
        );
        println!("    Filename: {}", entry.filename);
        println!("    Object ID: {}", entry.object_id);
        println!("    Object type: {:?}", entry.object_type);
        println!("    Valid: {}", entry.is_valid());
        if !entry.errors.is_empty() {
            println!("    Errors: {:?}", entry.errors);
        }
        println!();
    }

    println!();
    Ok(())
}

fn demo_tree_object_contracts() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Example 3: Tree Object Contracts");

    let validator = TreeValidator::default();

    // Create a tree with various entry types
    let raw_entries = vec![
        (
            "100644".to_string(),
            "Cargo.toml".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        ),
        (
            "100644".to_string(),
            "README.md".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ),
        (
            "040000".to_string(),
            "src".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
        ),
        (
            "100755".to_string(),
            "scripts/build.sh".to_string(),
            "d4e5f6789012345678901234567890abcdef0".to_string(),
        ),
    ];

    let tree = validator.create_tree_object(raw_entries);

    println!("  {}", tree.summary());
    println!("    Total entries: {}", tree.entries.len());
    println!("    Valid: {}", tree.is_valid());

    // Show entries by type
    let blob_entries = tree.get_blob_entries();
    let tree_entries = tree.get_tree_entries();

    println!("    Blob entries: {}", blob_entries.len());
    for entry in blob_entries {
        println!("      - {} ({})", entry.filename, entry.mode.description());
    }

    println!("    Tree entries: {}", tree_entries.len());
    for entry in tree_entries {
        println!("      - {} ({})", entry.filename, entry.mode.description());
    }

    // Find specific entry
    if let Some(entry) = tree.find_entry("README.md") {
        println!("    Found README.md: {}", entry.summary());
    }

    println!();
    Ok(())
}

fn demo_tree_validation_with_errors() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 4: Tree Validation with Errors");

    let _validator = TreeValidator::default();

    // Create entries with various validation issues
    let entries = vec![
        TreeEntryContract::new(
            "100644",
            "valid.txt".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        ),
        TreeEntryContract::new(
            "999999", // Invalid mode
            "invalid_mode.txt".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ),
        TreeEntryContract::new(
            "100644",
            "".to_string(), // Empty filename
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
        ),
        TreeEntryContract::new(
            "100644",
            "invalid_id.txt".to_string(),
            "invalid".to_string(), // Invalid object ID
        ),
        TreeEntryContract::new(
            "100644",
            "valid.txt".to_string(), // Duplicate filename
            "d4e5f6789012345678901234567890abcdef0".to_string(),
        ),
    ];

    let tree = TreeObjectContract::new(entries);

    println!("  {}", tree.summary());
    println!("    Valid: {}", tree.is_valid());

    let errors = tree.get_errors();
    if !errors.is_empty() {
        println!("    Errors:");
        for error in &errors {
            println!("      - {error}");
        }
    }

    // Show individual entry validation
    println!("    Entry validation:");
    for entry in &tree.entries {
        println!("      {}", entry.summary());
    }

    println!();
    Ok(())
}

fn demo_complete_tree_object_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 5: Complete Tree Object Validation");

    let validator = TreeValidator::new(true, true, true); // Enable all validations

    // Create a realistic tree structure
    let raw_entries = vec![
        (
            "100644".to_string(),
            "Cargo.toml".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        ),
        (
            "100644".to_string(),
            "README.md".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ),
        (
            "040000".to_string(),
            "src".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
        ),
        (
            "040000".to_string(),
            "tests".to_string(),
            "d4e5f6789012345678901234567890abcdef0".to_string(),
        ),
        (
            "100755".to_string(),
            "scripts/build.sh".to_string(),
            "e5f6789012345678901234567890abcdef01".to_string(),
        ),
        (
            "120000".to_string(),
            "link.txt".to_string(),
            "f6789012345678901234567890abcdef012".to_string(),
        ),
    ];

    let tree = validator.create_tree_object(raw_entries);
    let summary = validator.summarize_tree(&tree);

    println!("  {summary}");
    println!("    Tree structure:");
    for entry in &tree.entries {
        let type_symbol = match entry.object_type {
            TreeObjectType::Blob => "📄",
            TreeObjectType::Tree => "📁",
        };
        println!(
            "      {} {} ({})",
            type_symbol,
            entry.filename,
            entry.mode.description()
        );
    }

    println!();
    Ok(())
}

fn demo_tree_in_git_object_contract() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Example 6: Tree Object in Git Object Contract");

    let tree_validator = TreeValidator::default();
    let validator = GitObjectValidator::new(true, false, true, false, tree_validator); // Enable line and tree validation

    // Create tree entries
    let entries = vec![
        TreeEntryContract::new(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        ),
        TreeEntryContract::new(
            "040000",
            "src".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ),
    ];

    // Create a tree object from the entries
    let tree = TreeObjectContract::new(entries);

    // Validate as a Git object
    let git_object = validator.validate_tree_entry(&tree.entries[0]);

    println!("  Git Object Type: {:?}", git_object.object_type);
    println!("  {}", git_object.summary());
    println!("  Tree: {}", tree.summary());
    println!("    Entries: {}", tree.entries.len());
    println!("    Valid: {}", tree.is_valid());

    // Show entries
    for entry in &tree.entries {
        println!("    {}", entry.summary());
    }

    println!();
    Ok(())
}
