#!/usr/bin/env rust-script
//! Script to add checksums to generated files
//! Uses the same logic as the validation script

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// Compute checksum using the same logic as validation script
fn compute_checksum(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())[..8].to_string()
}

/// Check if a line contains a generated header
fn is_generated_header(line: &str) -> bool {
    line.contains("@generated") && (
        line.contains("xtask gen-files") || 
        line.contains("xtask gen-config") ||
        line.contains("xtask gen-docs") ||
        line.contains("xtask gen-jsonl") ||
        line.contains("xtask gen-json") ||
        line.contains("xtask gen-yaml") ||
        line.contains("xtask gen-toml") ||
        line.contains("xtask gen-md") ||
        line.contains("xtask gen-wit") ||
        line.contains("xtask gen-jql")
    )
}

/// Add checksum to a file
fn add_checksum_to_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        println!("⚠️  {}: Empty file", file_path.display());
        return Ok(());
    }
    
    // Check if already has checksum
    if lines.len() > 1 && lines[1].contains("@checksum") {
        println!("⏭️  {}: Already has checksum", file_path.display());
        return Ok(());
    }
    
    // Check if has generated header
    if !is_generated_header(lines[0]) {
        println!("⚠️  {}: No generated header found", file_path.display());
        return Ok(());
    }
    
    // Get content without header (lines 3 onwards for validation)
    // But we need to account for the checksum line we're about to add
    let content_without_header = if lines.len() > 1 {
        lines[1..].join("\n")
    } else {
        String::new()
    };
    
    // Compute checksum
    let checksum = compute_checksum(&content_without_header);
    
    // Create new content
    let mut new_lines = vec![lines[0].to_string()];
    
    // Determine checksum prefix based on file type
    let checksum_prefix = if lines[0].contains("<!--") {
        "<!-- @checksum:"
    } else if lines[0].contains("//") {
        "// @checksum:"
    } else if lines[0].contains(";;") {
        ";; @checksum:"
    } else {
        "# @checksum:"
    };
    
    new_lines.push(format!("{} {}", checksum_prefix, checksum));
    
    // Add remaining lines
    if lines.len() > 1 {
        new_lines.extend(lines[1..].iter().map(|s| s.to_string()));
    }
    
    // Write back to file
    let new_content = new_lines.join("\n");
    fs::write(file_path, new_content)?;
    
    println!("✅ {}: Added checksum {}", file_path.display(), checksum);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file1> [file2] ...", args[0]);
        std::process::exit(1);
    }
    
    println!("🔧 Adding Checksums to Generated Files");
    println!("======================================");
    
    for file_path in &args[1..] {
        let path = Path::new(file_path);
        if !path.exists() {
            eprintln!("❌ {}: File does not exist", file_path);
            continue;
        }
        
        if let Err(e) = add_checksum_to_file(path) {
            eprintln!("❌ {}: {}", file_path, e);
        }
    }
    
    println!("\n✅ Checksum addition completed!");
    Ok(())
} 