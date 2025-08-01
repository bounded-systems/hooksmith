use git_filter::prelude::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    
    // Read the file content
    let content = fs::read(file_path)?;
    
    // For now, create a simple file state
    // In a real implementation, you'd parse .gitattributes
    let mut attributes = HashMap::new();
    attributes.insert("safe-ascii".to_string(), Some("true"));
    attributes.insert("text".to_string(), Some("true"));
    attributes.insert("eol".to_string(), Some("lf"));
    
    let file_state = FileState::from_attributes(&attributes);
    let operation = GitOperation::Add;
    
    // Create a filter and process the file
    let mut filter = MultiFilter::new();
    filter.add_driver("safe-ascii", Box::new(SafeAsciiFilter::default()));
    
    match filter.process_file(&content, &file_state, &operation) {
        Ok(_) => {
            println!("✅ File '{}' passed validation", file_path);
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ File '{}' failed validation: {}", file_path, e);
            std::process::exit(1);
        }
    }
} 
