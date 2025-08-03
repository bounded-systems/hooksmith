#!/usr/bin/env rust-script
//! Script to add checksums to the registry for files that don't have them
//! This updates the config/generated-files.jsonc registry

use std::fs;
use std::path::Path;
use serde_json::Value;

fn remove_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut i = 0;
    
    while i < content.len() {
        let c = content.chars().nth(i).unwrap();
        
        if escape_next {
            result.push(c);
            escape_next = false;
            i += 1;
            continue;
        }
        
        if c == '\\' {
            escape_next = true;
            result.push(c);
            i += 1;
            continue;
        }
        
        if c == '"' {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        
        if !in_string && c == '/' && i + 1 < content.len() {
            let next_c = content.chars().nth(i + 1).unwrap();
            if next_c == '/' {
                // Single line comment
                while i < content.len() && content.chars().nth(i).unwrap() != '\n' {
                    i += 1;
                }
                if i < content.len() {
                    result.push('\n');
                }
                i += 1;
                continue;
            } else if next_c == '*' {
                // Multi-line comment
                i += 2;
                while i + 1 < content.len() {
                    if content.chars().nth(i).unwrap() == '*' && content.chars().nth(i + 1).unwrap() == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }
        
        result.push(c);
        i += 1;
    }
    
    result
}

fn compute_checksum(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..8].to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Adding Checksums to Registry");
    println!("===============================");
    
    // Read the registry
    let registry_content = fs::read_to_string("config/generated-files.jsonc")?;
    let json_content = remove_jsonc_comments(&registry_content);
    let mut registry: Value = serde_json::from_str(&json_content)?;
    
    let files_array = registry["files"].as_array_mut().unwrap();
    let mut updated_count = 0;
    let mut missing_files = Vec::new();
    
    for i in 0..files_array.len() {
        let file_entry = &mut files_array[i];
        let path = file_entry["path"].as_str().unwrap().to_string();
        let has_checksum = file_entry.get("checksum").is_some();
        
        if !has_checksum {
            let file_path = Path::new(&path);
            
            if file_path.exists() {
                match fs::read_to_string(file_path) {
                    Ok(content) => {
                        // Remove the @checksum line if present
                        let lines: Vec<&str> = content.lines().collect();
                        let content_without_checksum: String = lines
                            .iter()
                            .filter(|line| !line.trim().starts_with("@checksum"))
                            .map(|line| format!("{}\n", line))
                            .collect();
                        
                        let checksum = compute_checksum(&content_without_checksum);
                        file_entry["checksum"] = Value::String(checksum.clone());
                        
                        println!("✅ {}: Added checksum {}", path, checksum);
                        updated_count += 1;
                    }
                    Err(e) => {
                        println!("❌ {}: Failed to read file: {}", path, e);
                        missing_files.push(path);
                    }
                }
            } else {
                println!("⚠️  {}: File does not exist", path);
                missing_files.push(path);
            }
        }
    }
    
    // Write back to registry
    let updated_content = serde_json::to_string_pretty(&registry)?;
    fs::write("config/generated-files.jsonc", updated_content)?;
    
    println!("\n📊 Summary:");
    println!("- Files updated: {}", updated_count);
    println!("- Missing files: {}", missing_files.len());
    
    if !missing_files.is_empty() {
        println!("\n⚠️  Missing files:");
        for file in missing_files.iter().take(10) {
            println!("  - {}", file);
        }
        if missing_files.len() > 10 {
            println!("  - ... and {} more", missing_files.len() - 10);
        }
    }
    
    println!("\n✅ Registry updated successfully!");
    Ok(())
} 
