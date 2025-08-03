use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🌳 Tree Contract Explicit Type Demo\n");

    // Example 1: Valid type matches
    demo_valid_type_matches()?;

    // Example 2: Invalid type mismatches
    demo_invalid_type_mismatches()?;

    // Example 3: Restricted tree modes
    demo_restricted_tree_modes()?;

    // Example 4: Flat contract structure
    demo_flat_contract_structure()?;

    Ok(())
}

fn demo_valid_type_matches() -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ Example 1: Valid Type Matches");

    let valid_entries = vec![
        // Mode 100644 (Regular file) -> type "blob"
        TreeEntryContract::new_with_type_and_attributes(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            TreeObjectType::Blob,
            None,
        ),
        // Mode 100755 (Executable file) -> type "blob"
        TreeEntryContract::new_with_type_and_attributes(
            "100755",
            "script.sh".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            TreeObjectType::Blob,
            None,
        ),
        // Mode 040000 (Tree) -> type "tree"
        TreeEntryContract::new_with_type_and_attributes(
            "040000",
            "src".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
            TreeObjectType::Tree,
            None,
        ),
    ];

    for entry in &valid_entries {
        println!("  {}", entry.summary());
        println!(
            "    Mode: {} -> Type: {:?}",
            entry.mode_string(),
            entry.object_type
        );
        println!("    Valid: {}", entry.is_valid());
        if !entry.errors.is_empty() {
            println!("    Errors: {:?}", entry.errors);
        }
        println!();
    }

    // Create tree with valid entries
    let tree = TreeObjectContract::new(valid_entries);
    println!("  Tree: {}", tree.summary());
    println!();

    Ok(())
}

fn demo_invalid_type_mismatches() -> Result<(), Box<dyn std::error::Error>> {
    println!("❌ Example 2: Invalid Type Mismatches");

    let invalid_entries = vec![
        // Mode 100644 (Regular file) but type "tree" -> INVALID
        TreeEntryContract::new_with_type_and_attributes(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            TreeObjectType::Tree, // Should be Blob
            None,
        ),
        // Mode 040000 (Tree) but type "blob" -> INVALID
        TreeEntryContract::new_with_type_and_attributes(
            "040000",
            "src".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            TreeObjectType::Blob, // Should be Tree
            None,
        ),
        // Mode 100755 (Executable file) but type "tree" -> INVALID
        TreeEntryContract::new_with_type_and_attributes(
            "100755",
            "script.sh".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
            TreeObjectType::Tree, // Should be Blob
            None,
        ),
    ];

    for entry in &invalid_entries {
        println!("  {}", entry.summary());
        println!(
            "    Mode: {} -> Type: {:?}",
            entry.mode_string(),
            entry.object_type
        );
        println!("    Valid: {}", entry.is_valid());
        if !entry.errors.is_empty() {
            println!("    Errors: {:?}", entry.errors);
        }
        println!();
    }

    // Create tree with invalid entries
    let tree = TreeObjectContract::new(invalid_entries);
    println!("  Tree: {}", tree.summary());
    println!();

    Ok(())
}

fn demo_restricted_tree_modes() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 3: Restricted Tree Modes");

    let restricted_entries = vec![
        // Mode 120000 (Symlink) -> not supported in this demo
        TreeEntryContract::new(
            "120000",
            "link.txt".to_string(),
            "d4e5f6789012345678901234567890abcdef1".to_string(),
        ),
        // Mode 160000 (Gitlink) -> not supported in this demo
        TreeEntryContract::new(
            "160000",
            "submodule".to_string(),
            "e5f6789012345678901234567890abcdef12".to_string(),
        ),
    ];

    for entry in &restricted_entries {
        println!("  {}", entry.summary());
        println!(
            "    Mode: {} -> Type: {:?}",
            entry.mode_string(),
            entry.object_type
        );
        println!("    Valid: {}", entry.is_valid());
        if !entry.errors.is_empty() {
            println!("    Errors: {:?}", entry.errors);
        }
        println!();
    }

    // Create tree with restricted entries
    let tree = TreeObjectContract::new(restricted_entries);
    println!("  Tree: {}", tree.summary());
    println!();

    Ok(())
}

fn demo_flat_contract_structure() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Example 4: Flat Contract Structure");

    // Create a flat structure with mixed valid/invalid entries
    let flat_entries = vec![
        // Valid entries
        TreeEntryContract::new_with_type_and_attributes(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            TreeObjectType::Blob,
            None,
        ),
        TreeEntryContract::new_with_type_and_attributes(
            "040000",
            "src".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            TreeObjectType::Tree,
            None,
        ),
        // Invalid entry (type mismatch)
        TreeEntryContract::new_with_type_and_attributes(
            "100644",
            "config.json".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
            TreeObjectType::Tree, // Should be Blob
            None,
        ),
        // Invalid entry (restricted mode)
        TreeEntryContract::new(
            "120000",
            "link.txt".to_string(),
            "d4e5f6789012345678901234567890abcdef1".to_string(),
        ),
    ];

    // Create tree with mixed entries
    let tree = TreeObjectContract::new(flat_entries);
    println!("  Tree: {}", tree.summary());
    println!("    Total entries: {}", tree.entries.len());
    println!(
        "    Valid entries: {}",
        tree.entries.iter().filter(|e| e.is_valid()).count()
    );
    println!(
        "    Invalid entries: {}",
        tree.entries.iter().filter(|e| !e.is_valid()).count()
    );

    // Show individual entry details
    for (i, entry) in tree.entries.iter().enumerate() {
        println!("    Entry {}: {}", i + 1, entry.summary());
    }

    println!();
    Ok(())
}
