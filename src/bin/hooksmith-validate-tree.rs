use anyhow::{bail, Result};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

fn main() -> Result<()> {
    println!("🔍 Validating Git objects...");
    
    // Get all objects from the repository
    let objects_output = Command::new("git")
        .args(["rev-list", "--all", "--objects"])
        .output()?;
    
    if !objects_output.status.success() {
        bail!("Failed to get Git objects: {}", String::from_utf8_lossy(&objects_output.stderr));
    }
    
    // Process objects through git cat-file to get their types
    let mut cat_file = Command::new("git")
        .args(["cat-file", "--batch-check=%(objectname) %(objecttype) %(rest)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Write object hashes to stdin
    if let Some(stdin) = cat_file.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(&objects_output.stdout)?;
    }
    
    // Read and process the output
    let output = cat_file.wait_with_output()?;
    
    if !output.status.success() {
        bail!("Failed to process Git objects: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Parse the output to validate objects
    let mut blob_count = 0;
    let mut tree_count = 0;
    let mut commit_count = 0;
    let mut tag_count = 0;
    let mut invalid_objects = Vec::new();
    
    let reader = BufReader::new(&output.stdout[..]);
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() >= 2 {
            let object_hash = parts[0];
            let object_type = parts[1];
            
            match object_type {
                "blob" => {
                    blob_count += 1;
                    // Validate blob content if needed
                    if let Err(e) = validate_blob(object_hash) {
                        invalid_objects.push(format!("Blob {}: {}", object_hash, e));
                    }
                }
                "tree" => {
                    tree_count += 1;
                    // Validate tree structure if needed
                    if let Err(e) = validate_tree(object_hash) {
                        invalid_objects.push(format!("Tree {}: {}", object_hash, e));
                    }
                }
                "commit" => {
                    commit_count += 1;
                    // Validate commit structure if needed
                    if let Err(e) = validate_commit(object_hash) {
                        invalid_objects.push(format!("Commit {}: {}", object_hash, e));
                    }
                }
                "tag" => {
                    tag_count += 1;
                    // Validate tag structure if needed
                    if let Err(e) = validate_tag(object_hash) {
                        invalid_objects.push(format!("Tag {}: {}", object_hash, e));
                    }
                }
                _ => {
                    invalid_objects.push(format!("Unknown object type '{}' for {}", object_type, object_hash));
                }
            }
        }
    }
    
    // Print validation summary
    println!("📊 Git Object Validation Summary:");
    println!("   Blobs: {}", blob_count);
    println!("   Trees: {}", tree_count);
    println!("   Commits: {}", commit_count);
    println!("   Tags: {}", tag_count);
    println!("   Invalid objects: {}", invalid_objects.len());
    
    if !invalid_objects.is_empty() {
        println!("\n❌ Invalid objects found:");
        for invalid in &invalid_objects {
            println!("   - {}", invalid);
        }
        bail!("Validation failed: {} invalid objects found", invalid_objects.len());
    }
    
    println!("✅ All Git objects are valid!");
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
