use anyhow::{bail, Result};
use std::process::Command;
use std::collections::HashMap;
use itertools::Itertools;

fn main() -> Result<()> {
    println!("🔍 Validating Git objects...");
    
    // Execute the exact command specified by the user
    let output = Command::new("sh")
        .arg("-c")
        .arg("git rev-list --all --objects | cut -d' ' -f1 | git cat-file --batch-check='%(objecttype)' | sort | uniq -c")
        .output()?;
    
    if !output.status.success() {
        bail!("Failed to analyze Git objects: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Parse the results
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut object_types = HashMap::new();
    let mut total_objects = 0;
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        if parts.len() == 2 {
            if let (Ok(count), object_type) = (parts[0].parse::<u32>(), parts[1]) {
                object_types.insert(object_type.to_string(), count);
                total_objects += count;
            }
        }
    }
    
    // Display results
    println!("📊 Git Object Analysis:");
    println!("   Total objects: {}", total_objects);
    println!("\n   Object types:");
    for (object_type, count) in object_types.iter().sorted_by_key(|(_, &count)| count).rev() {
        println!("     {}: {}", object_type, count);
    }
    
    // Validate object types
    let valid_types = ["blob", "tree", "commit", "tag"];
    let mut validation_errors = Vec::new();
    
    for (object_type, count) in &object_types {
        if !valid_types.contains(&object_type.as_str()) {
            validation_errors.push(format!("Unknown object type '{}' found {} times", object_type, count));
        }
    }
    
    if !validation_errors.is_empty() {
        println!("\n❌ Validation errors:");
        for error in &validation_errors {
            println!("   - {}", error);
        }
        bail!("Git object validation failed");
    }
    
    // Additional analysis for contract validation
    println!("\n🔍 Contract Validation Analysis:");
    
    // Check for potential issues
    let mut warnings = Vec::new();
    
    if let Some(blob_count) = object_types.get("blob") {
        if *blob_count > 10000 {
            warnings.push(format!("Large number of blobs ({}) - consider cleanup", blob_count));
        }
    }
    
    if let Some(tree_count) = object_types.get("tree") {
        if *tree_count > 10000 {
            warnings.push(format!("Large number of trees ({}) - consider cleanup", tree_count));
        }
    }
    
    if let Some(commit_count) = object_types.get("commit") {
        if *commit_count > 5000 {
            warnings.push(format!("Large number of commits ({}) - consider cleanup", commit_count));
        }
    }
    
    if !warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &warnings {
            println!("   - {}", warning);
        }
    }
    
    println!("\n✅ Git object validation completed successfully");
    Ok(())
}

fn validate_blob(hash: &str) -> Result<()> {
    // Basic blob validation - check if it can be read
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()?;
    
    if !output.status.success() {
        bail!("Failed to read blob content");
    }
    
    // Additional validation could be added here:
    // - Check for binary files
    // - Validate file size limits
    // - Check for malicious content patterns
    // - Validate encoding
    
    Ok(())
}

fn validate_tree(hash: &str) -> Result<()> {
    // Basic tree validation - check if it can be read
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()?;
    
    if !output.status.success() {
        bail!("Failed to read tree content");
    }
    
    // Parse tree entries and validate them
    let content = String::from_utf8_lossy(&output.stdout);
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let mode = parts[0];
            let object_type = parts[1];
            let name = parts[2];
            
            // Validate tree entry
            if let Err(e) = validate_tree_entry(mode, object_type, name) {
                bail!("Invalid tree entry: {}", e);
            }
        }
    }
    
    Ok(())
}

fn validate_tree_entry(mode: &str, object_type: &str, name: &str) -> Result<()> {
    // Validate mode (should be a valid Git mode)
    if !mode.chars().all(|c| c.is_ascii_digit()) {
        bail!("Invalid mode: {}", mode);
    }
    
    // Validate object type
    match object_type {
        "blob" | "tree" | "commit" => {}
        _ => bail!("Invalid object type: {}", object_type),
    }
    
    // Validate name (should not be empty and should not contain null bytes)
    if name.is_empty() {
        bail!("Empty name in tree entry");
    }
    
    if name.contains('\0') {
        bail!("Name contains null byte");
    }
    
    Ok(())
}

fn validate_commit(hash: &str) -> Result<()> {
    // Basic commit validation - check if it can be read
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()?;
    
    if !output.status.success() {
        bail!("Failed to read commit content");
    }
    
    // Parse commit content and validate structure
    let content = String::from_utf8_lossy(&output.stdout);
    let mut has_tree = false;
    let mut has_author = false;
    let mut has_committer = false;
    
    for line in content.lines() {
        if line.starts_with("tree ") {
            has_tree = true;
            // Validate tree hash
            let tree_hash = &line[5..];
            if tree_hash.len() != 40 || !tree_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                bail!("Invalid tree hash: {}", tree_hash);
            }
        } else if line.starts_with("parent ") {
            // Validate parent hash
            let parent_hash = &line[7..];
            if parent_hash.len() != 40 || !parent_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                bail!("Invalid parent hash: {}", parent_hash);
            }
        } else if line.starts_with("author ") {
            has_author = true;
        } else if line.starts_with("committer ") {
            has_committer = true;
        }
    }
    
    if !has_tree {
        bail!("Commit missing tree");
    }
    
    if !has_author {
        bail!("Commit missing author");
    }
    
    if !has_committer {
        bail!("Commit missing committer");
    }
    
    Ok(())
}

fn validate_tag(hash: &str) -> Result<()> {
    // Basic tag validation - check if it can be read
    let output = Command::new("git")
        .args(["cat-file", "-p", hash])
        .output()?;
    
    if !output.status.success() {
        bail!("Failed to read tag content");
    }
    
    // Parse tag content and validate structure
    let content = String::from_utf8_lossy(&output.stdout);
    let mut has_object = false;
    let mut has_type = false;
    let mut has_tag = false;
    let mut has_tagger = false;
    
    for line in content.lines() {
        if line.starts_with("object ") {
            has_object = true;
            // Validate object hash
            let object_hash = &line[7..];
            if object_hash.len() != 40 || !object_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                bail!("Invalid object hash: {}", object_hash);
            }
        } else if line.starts_with("type ") {
            has_type = true;
            // Validate type
            let object_type = &line[5..];
            match object_type {
                "blob" | "tree" | "commit" | "tag" => {}
                _ => bail!("Invalid object type: {}", object_type),
            }
        } else if line.starts_with("tag ") {
            has_tag = true;
        } else if line.starts_with("tagger ") {
            has_tagger = true;
        }
    }
    
    if !has_object {
        bail!("Tag missing object");
    }
    
    if !has_type {
        bail!("Tag missing type");
    }
    
    if !has_tag {
        bail!("Tag missing tag name");
    }
    
    if !has_tagger {
        bail!("Tag missing tagger");
    }
    
    Ok(())
}
