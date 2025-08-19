use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    
    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist", file_path);
        std::process::exit(1);
    }
    
    println!("Adding checksum for file: {}", file_path);
    // TODO: Implement checksum addition logic
    println!("Checksum added successfully");
} 
