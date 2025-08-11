use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use git2::Repository;
use hooksmith_core::object_names_validator::{ObjectNamesValidator, load_contract};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read stdin for git pre-receive hook input
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    
    // Parse the pre-receive input: old new ref
    let line = lines.next()
        .ok_or("No input received")?
        .map_err(|e| format!("Failed to read stdin: {}", e))?;
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Invalid pre-receive input format. Expected: old new ref".into());
    }
    
    let old_oid = parts[0];
    let new_oid = parts[1];
    let ref_name = parts[2];
    
    // Only validate main branch pushes
    if ref_name != "refs/heads/main" {
        eprintln!("Skipping validation for non-main branch: {}", ref_name);
        return Ok(());
    }
    
    eprintln!("🔍 Validating object-names contract for main branch push...");
    eprintln!("Old: {}, New: {}, Ref: {}", old_oid, new_oid, ref_name);
    
    // Open the repository
    let repo = Repository::open(".")?;
    
    // Load the contract
    let contract_json = include_str!("../../../contracts/object-names@v1.json");
    let contract = load_contract(contract_json)?;
    
    // Create validator
    let validator = ObjectNamesValidator::new(contract);
    
    // Validate the new commit's root tree
    let verdicts = validator.validate_root_tree(&repo, new_oid)?;
    
    let mut all_passed = true;
    let mut error_messages = Vec::new();
    
    for verdict in verdicts {
        if !verdict.pass {
            all_passed = false;
            error_messages.push(format!("❌ {}: {}", verdict.contract_name, verdict.summary_code));
            
            // If there's a diff, we could load and display it here
            if let Some(diff_oid) = verdict.diff_oid {
                error_messages.push(format!("   Diff available at: {}", diff_oid));
            }
        } else {
            eprintln!("✅ {}: {}", verdict.contract_name, verdict.summary_code);
        }
    }
    
    if all_passed {
        eprintln!("✅ Object-names contract validation passed");
        Ok(())
    } else {
        eprintln!("🚫 Object-names contract validation failed");
        eprintln!();
        eprintln!("Errors:");
        for error in &error_messages {
            eprintln!("  {}", error);
        }
        eprintln!();
        eprintln!("Required actions:");
        eprintln!("1. Ensure .gitignore and projects/ exist at root");
        eprintln!("2. Move rejected files to allowed directories");
        eprintln!("3. Only include files/directories from the allowed list at root");
        
        std::process::exit(1);
    }
}
