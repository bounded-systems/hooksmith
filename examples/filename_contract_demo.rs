use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("📁 Filename Contract Demo - Flat Unscoped Contract\n");

    // Example 1: Basic filename validation
    demo_basic_filename_validation()?;

    // Example 2: Strict filename validation (no path separators)
    demo_strict_filename_validation()?;

    // Example 3: Non-strict filename validation (allows subdirectories)
    demo_non_strict_filename_validation()?;

    // Example 4: Batch filename validation
    demo_batch_filename_validation()?;

    // Example 5: Integration with tree entries
    demo_integration_with_tree_entries()?;

    Ok(())
}

fn demo_basic_filename_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📄 Example 1: Basic Filename Validation");

    let filenames = vec![
        "README.md",
        "Cargo.toml",
        "src/main.rs",
        "",
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
    ];

    for filename in filenames {
        let contract = FilenameContract::new(filename.to_string());
        println!("  {}", contract.summary());

        if !contract.errors.is_empty() {
            println!("    Errors: {:?}", contract.errors);
        }
        println!();
    }

    println!();
    Ok(())
}

fn demo_strict_filename_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 2: Strict Filename Validation (No Path Separators)");

    let filenames = vec![
        "README.md",
        "Cargo.toml",
        "src/main.rs",         // ❌ Contains '/'
        "src/utils/helper.rs", // ❌ Contains '/'
        "file.txt",
        "script.sh",
    ];

    for filename in filenames {
        let contract = FilenameContract::new_strict(filename.to_string());
        println!("  {}", contract.summary());

        if !contract.errors.is_empty() {
            println!("    Errors: {:?}", contract.errors);
        }
        println!();
    }

    println!("  Strict validation blocks path separators:");
    println!("    ✅ Simple filenames: README.md, Cargo.toml");
    println!("    ❌ Paths with '/': src/main.rs, src/utils/helper.rs");
    println!();

    Ok(())
}

fn demo_non_strict_filename_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📂 Example 3: Non-Strict Filename Validation (Allows Subdirectories)");

    let filenames = vec![
        "README.md",
        "Cargo.toml",
        "src/main.rs",         // ✅ Allowed
        "src/utils/helper.rs", // ✅ Allowed
        "file.txt",
        "scripts/build.sh", // ✅ Allowed
        "",
    ];

    for filename in filenames {
        let contract = FilenameContract::new(filename.to_string());
        println!("  {}", contract.summary());

        if !contract.errors.is_empty() {
            println!("    Errors: {:?}", contract.errors);
        }
        println!();
    }

    println!("  Non-strict validation allows subdirectories:");
    println!("    ✅ Simple filenames: README.md, Cargo.toml");
    println!("    ✅ Paths with '/': src/main.rs, src/utils/helper.rs");
    println!("    ❌ Empty filenames: \"\"");
    println!();

    Ok(())
}

fn demo_batch_filename_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 4: Batch Filename Validation");

    let validator = FilenameValidator::new(false); // Non-strict
    let filenames = vec![
        "README.md".to_string(),
        "Cargo.toml".to_string(),
        "src/main.rs".to_string(),
        "src/utils/helper.rs".to_string(),
        "scripts/build.sh".to_string(),
        "".to_string(),
        "file.txt".to_string(),
    ];

    let contracts = validator.validate_filenames(filenames);
    let summary = validator.summarize_validation(&contracts);

    println!("  {}", summary);
    println!();

    println!("  Individual results:");
    for contract in &contracts {
        println!("    {}", contract.summary());
    }

    println!();
    println!("  Validation summary:");
    println!("    All valid: {}", validator.all_valid(&contracts));

    let invalid_filenames = validator.get_invalid_filenames(&contracts);
    if !invalid_filenames.is_empty() {
        println!("    Invalid filenames:");
        for contract in invalid_filenames {
            println!(
                "      - '{}': {}",
                contract.filename,
                contract.errors.join(", ")
            );
        }
    }

    println!();
    Ok(())
}

fn demo_integration_with_tree_entries() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Example 5: Integration with Tree Entries");

    // Create tree entries with various filenames
    let tree_entries = vec![
        TreeEntryContract::new(
            "100644",
            "README.md".to_string(),
            "a1b2c3d4e5f6789012345678901234567890abcd".to_string(),
        ),
        TreeEntryContract::new(
            "100644",
            "src/main.rs".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ),
        TreeEntryContract::new(
            "100644",
            "".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
        ),
        TreeEntryContract::new(
            "100644",
            "scripts/build.sh".to_string(),
            "d4e5f6789012345678901234567890abcdef0".to_string(),
        ),
    ];

    // Validate filenames separately using FilenameContract
    let filename_validator = FilenameValidator::new(false); // Non-strict
    let filenames: Vec<String> = tree_entries.iter().map(|e| e.filename.clone()).collect();
    let filename_contracts = filename_validator.validate_filenames(filenames);

    println!("  Tree entries with filename validation:");
    for (i, (tree_entry, filename_contract)) in tree_entries
        .iter()
        .zip(filename_contracts.iter())
        .enumerate()
    {
        println!("    Entry {}: {}", i + 1, tree_entry.summary());
        println!("      Filename validation: {}", filename_contract.summary());

        if !filename_contract.errors.is_empty() {
            println!("      Filename errors: {:?}", filename_contract.errors);
        }
        println!();
    }

    // Show how FilenameContract is unscoped (independent of Git object types)
    println!("  FilenameContract is unscoped:");
    println!("    - Independent of Git object types (blob, tree, etc.)");
    println!("    - Focuses only on filename validation");
    println!("    - Can be used with any file system path");
    println!("    - Reusable across different contexts");
    println!();

    Ok(())
}
