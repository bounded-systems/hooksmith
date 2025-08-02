use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔗 Tree Contract Explicit Type Demo - Type Validation\n");

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
        TreeEntryContract::new_with_type(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            TreeObjectType::Blob,
        ),
        // Mode 100755 (Executable file) -> type "blob"
        TreeEntryContract::new_with_type(
            "100755",
            "script.sh".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            TreeObjectType::Blob,
        ),
        // Mode 040000 (Tree) -> type "tree"
        TreeEntryContract::new_with_type(
            "040000",
            "src".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
            TreeObjectType::Tree,
        ),
    ];

    for entry in &valid_entries {
        println!("  {}", entry.summary());
        println!("    Mode: {} -> Type: {:?}", entry.mode_string(), entry.object_type);
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
        TreeEntryContract::new_with_type(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            TreeObjectType::Tree, // Should be Blob
        ),
        // Mode 040000 (Tree) but type "blob" -> INVALID
        TreeEntryContract::new_with_type(
            "040000",
            "src".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            TreeObjectType::Blob, // Should be Tree
        ),
        // Mode 100755 (Executable file) but type "tree" -> INVALID
        TreeEntryContract::new_with_type(
            "100755",
            "script.sh".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
            TreeObjectType::Tree, // Should be Blob
        ),
    ];

    for entry in &invalid_entries {
        println!("  {}", entry.summary());
        println!("    Mode: {} -> Type: {:?}", entry.mode_string(), entry.object_type);
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

    let restricted_modes = vec![
        "100644", // ✅ Allowed - Regular file
        "100755", // ✅ Allowed - Executable file
        "040000", // ✅ Allowed - Tree
        "120000", // ❌ Not allowed - Symlink
        "160000", // ❌ Not allowed - Gitlink
        "999999", // ❌ Not allowed - Invalid
    ];

    for mode in restricted_modes {
        match TreeMode::from_str(mode) {
            Some(tree_mode) => {
                println!("  ✅ Mode {}: {} ({:?})", mode, tree_mode.description(), tree_mode.object_type());
            }
            None => {
                println!("  ❌ Mode {}: Not allowed in restricted contract", mode);
            }
        }
    }

    println!();
    println!("  Restricted TreeModeContract only allows:");
    println!("    - 100644 (Regular file) -> type: Blob");
    println!("    - 100755 (Executable file) -> type: Blob");
    println!("    - 040000 (Tree) -> type: Tree");
    println!();

    Ok(())
}

fn demo_flat_contract_structure() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 4: Flat Contract Structure");

    // Create entries with explicit type validation
    let entries = vec![
        TreeEntryContract::new_with_type(
            "100644",
            "Cargo.toml".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
            TreeObjectType::Blob,
        ),
        TreeEntryContract::new_with_type(
            "100755",
            "scripts/build.sh".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
            TreeObjectType::Blob,
        ),
        TreeEntryContract::new_with_type(
            "040000",
            "src".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
            TreeObjectType::Tree,
        ),
    ];

    // Create flat tree object contract
    let tree = TreeObjectContract::new(entries);

    println!("  TreeObjectContract = {{");
    println!("    entries: [TreeEntryContract, TreeEntryContract, ...]");
    println!("  }}");
    println!();

    println!("  Each TreeEntryContract = {{");
    println!("    mode: TreeMode,           // 100644 | 100755 | 040000");
    println!("    filename: String,         // Must not be empty");
    println!("    object_id: String,        // SHA-1 format (40 hex chars)");
    println!("    object_type: TreeObjectType, // Must match mode");
    println!("    valid: bool,              // Overall validation result");
    println!("    errors: Vec<String>,      // Validation errors");
    println!("  }}");
    println!();

    println!("  Validation Results:");
    println!("    {}", tree.summary());
    
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

    println!();
    Ok(())
} 
