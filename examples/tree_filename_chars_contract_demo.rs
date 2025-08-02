use git_filter::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔤 Tree Filename Chars Contract Demo - Combined Filename & Character Validation\n");

    // Example 1: Character-level validation
    demo_character_validation()?;

    // Example 2: Tree filename contract with character validation
    demo_tree_filename_contract()?;

    // Example 3: Invalid characters in filenames
    demo_invalid_characters()?;

    // Example 4: Batch filename validation with character analysis
    demo_batch_filename_validation()?;

    // Example 5: Integration with tree entries
    demo_integration_with_tree_entries()?;

    // Example 6: Character-by-character analysis
    demo_character_analysis()?;

    Ok(())
}

fn demo_character_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔤 Example 1: Character-Level Validation");

    let characters = vec![
        'a', 'Z', '5', '!', ' ', '~', // Valid printable ASCII
        '\x00', '\x01', '\x1f', '\x7f', // Invalid control characters
        'é', 'ñ', '€', // Invalid non-ASCII
    ];

    for ch in characters {
        let contract = CharContract::new(ch);
        println!("  {}", contract.summary());
        
        if !contract.is_valid() {
            println!("    Error: {}", contract.error.as_ref().unwrap());
        }
        println!();
    }

    println!("  Character validation rules:");
    println!("    ✅ Valid: Printable ASCII (0x20 to 0x7e)");
    println!("    ❌ Invalid: Control characters (0x00 to 0x1f, 0x7f)");
    println!("    ❌ Invalid: Non-ASCII characters");
    println!();

    Ok(())
}

fn demo_tree_filename_contract() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Example 2: Tree Filename Contract with Character Validation");

    let filenames = vec![
        "README.md",
        "Cargo.toml",
        "src/main.rs",
        "file with spaces.txt",
        "file-with-dashes.txt",
        "file_with_underscores.txt",
    ];

    for filename in filenames {
        let contract = TreeFilenameContractChars::new(filename.to_string());
        println!("  {}", contract.summary());
        println!("    Length: {} characters", contract.len());
        println!("    Valid characters: {}", contract.get_valid_chars().len());
        println!("    Invalid characters: {}", contract.get_invalid_chars().len());
        
        if !contract.errors.is_empty() {
            println!("    Errors: {:?}", contract.errors);
        }
        println!();
    }

    println!();
    Ok(())
}

fn demo_invalid_characters() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Example 3: Invalid Characters in Filenames");

    let filenames = vec![
        "file\x00name.txt",      // NUL character
        "file\x01name.txt",      // Control character
        "file\x1fname.txt",      // Control character
        "file\x7fname.txt",      // DEL character
        "fileéname.txt",         // Non-ASCII
        "fileñame.txt",          // Non-ASCII
        "",                      // Empty filename
    ];

    for filename in filenames {
        let contract = TreeFilenameContractChars::new(filename.to_string());
        println!("  {}", contract.summary());
        
        if !contract.is_valid() {
            println!("    Errors: {:?}", contract.errors);
            
            // Show invalid characters
            let invalid_chars = contract.get_invalid_chars();
            if !invalid_chars.is_empty() {
                println!("    Invalid characters:");
                for char_contract in invalid_chars {
                    println!("      {}", char_contract.summary());
                }
            }
        }
        println!();
    }

    println!();
    Ok(())
}

fn demo_batch_filename_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 4: Batch Filename Validation with Character Analysis");

    let validator = TreeFilenameCharsValidator::new(false); // Don't allow path separators
    let filenames = vec![
        "README.md".to_string(),
        "Cargo.toml".to_string(),
        "file\x00name.txt".to_string(),
        "src/main.rs".to_string(), // Contains '/'
        "fileéname.txt".to_string(),
        "".to_string(),
    ];

    let contracts = validator.validate_filenames(filenames);
    let summary = validator.summarize_validation(&contracts);

    println!("  {}", summary);
    println!();

    println!("  Individual results:");
    for contract in &contracts {
        println!("    {}", contract.summary());
        if !contract.is_valid() {
            println!("      Errors: {:?}", contract.errors);
        }
    }

    println!();
    println!("  Validation summary:");
    println!("    All valid: {}", validator.all_valid(&contracts));
    
    let invalid_filenames = validator.get_invalid_filenames(&contracts);
    if !invalid_filenames.is_empty() {
        println!("    Invalid filenames:");
        for contract in invalid_filenames {
            println!("      - '{}': {}", contract.as_string(), contract.errors.join(", "));
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
            "file\x00name.txt".to_string(),
            "b2c3d4e5f6789012345678901234567890abcde".to_string(),
        ),
        TreeEntryContract::new(
            "100644",
            "src/main.rs".to_string(),
            "c3d4e5f6789012345678901234567890abcdef".to_string(),
        ),
    ];

    // Validate filenames with character-level validation
    let validator = TreeFilenameCharsValidator::new(true); // Allow path separators
    let filenames: Vec<String> = tree_entries.iter().map(|e| e.filename.clone()).collect();
    let filename_contracts = validator.validate_filenames(filenames);

    println!("  Tree entries with character-level filename validation:");
    for (i, (tree_entry, filename_contract)) in tree_entries.iter().zip(filename_contracts.iter()).enumerate() {
        println!("    Entry {}: {}", i + 1, tree_entry.summary());
        println!("      Filename validation: {}", filename_contract.summary());
        println!("      Character count: {} ({} valid, {} invalid)", 
            filename_contract.len(),
            filename_contract.get_valid_chars().len(),
            filename_contract.get_invalid_chars().len()
        );
        
        if !filename_contract.errors.is_empty() {
            println!("      Filename errors: {:?}", filename_contract.errors);
        }
        println!();
    }

    println!();
    Ok(())
}

fn demo_character_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Example 6: Character-by-Character Analysis");

    let filename = "file\x00with\x01invalid\x1fchars.txt";
    let contract = TreeFilenameContractChars::new(filename.to_string());

    println!("  Filename: '{}'", filename);
    println!("  Overall: {}", contract.summary());
    println!();

    println!("  Character-by-character analysis:");
    for (i, char_contract) in contract.filename.iter().enumerate() {
        let status = if char_contract.is_valid() { "✅" } else { "❌" };
        println!("    Position {}: {} '{}'", i, status, char_contract.char);
        
        if !char_contract.is_valid() {
            println!("      Error: {}", char_contract.error.as_ref().unwrap());
        }
    }

    println!();
    println!("  Summary:");
    println!("    Total characters: {}", contract.len());
    println!("    Valid characters: {}", contract.get_valid_chars().len());
    println!("    Invalid characters: {}", contract.get_invalid_chars().len());
    
    let valid_chars: String = contract.get_valid_chars().iter().map(|c| c.char).collect();
    let invalid_chars: Vec<char> = contract.get_invalid_chars().iter().map(|c| c.char).collect();
    
    println!("    Valid string: '{}'", valid_chars);
    println!("    Invalid chars: {:?}", invalid_chars);

    println!();
    Ok(())
} 
