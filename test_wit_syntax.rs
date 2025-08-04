use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing WIT file syntax...");
    
    let wit_files = [
        "crates/components/git-filter/wit/git-filter.wit",
        "wit/event-bus.wit",
        "wit/hooksmith.wit",
    ];
    
    for wit_file in &wit_files {
        println!("📄 Testing: {}", wit_file);
        
        if !Path::new(wit_file).exists() {
            println!("❌ File not found: {}", wit_file);
            continue;
        }
        
        let content = std::fs::read_to_string(wit_file)?;
        
        // Basic syntax check - look for common issues
        if content.contains(";; @generated") {
            println!("⚠️  Contains generated header - may cause parsing issues");
        }
        
        if content.contains("package ") && content.contains("world ") {
            println!("✅ Basic WIT structure looks good");
        } else {
            println!("❌ Missing package or world declaration");
        }
        
        // Check for proper comment syntax
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with(";;") {
                println!("⚠️  Line {}: Uses ;; comment syntax", i + 1);
            }
        }
        
        println!("✅ Syntax check completed for: {}", wit_file);
    }
    
    println!("🎉 WIT syntax validation completed!");
    Ok(())
} 
