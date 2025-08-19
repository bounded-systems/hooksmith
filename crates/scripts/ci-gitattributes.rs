#!/usr/bin/env rust

use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        show_usage();
        std::process::exit(1);
    }
    
    let mode = &args[1];
    
    match mode.as_str() {
        "-c" | "--check" => {
            check_gitattributes()?;
        }
        "-f" | "--force" => {
            force_update()?;
        }
        "-h" | "--help" => {
            show_usage();
            std::process::exit(0);
        }
        _ => {
            eprintln!("❌ Error: Unknown option '{}'", mode);
            show_usage();
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn show_usage() {
    println!("Usage: ci-gitattributes.rs [OPTIONS]");
    println!();
    println!("Options:");
    println!("    -c, --check     Check if .gitattributes needs updating (returns 0 if up-to-date, 1 if needs update)");
    println!("    -f, --force     Force update .gitattributes");
    println!("    -h, --help      Show this help message");
    println!();
    println!("Examples:");
    println!("    ci-gitattributes.rs -c          # Check if update is needed");
    println!("    ci-gitattributes.rs -f          # Force update .gitattributes");
}

fn check_hyperpolyglot() -> io::Result<()> {
    let output = Command::new("hyply")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    
    match output {
        Ok(_) => Ok(()),
        Err(_) => {
            eprintln!("❌ Error: hyperpolyglot (hyply) is not installed");
            eprintln!("💡 Install it with: cargo install hyperpolyglot");
            std::process::exit(1);
        }
    }
}

fn check_gitattributes() -> io::Result<()> {
    println!("🔍 Checking if .gitattributes needs updating...");
    
    // Check dependencies
    check_hyperpolyglot()?;
    
    // Create temporary files for comparison
    let temp_output_path = create_temp_file()?;
    
    // Generate new .gitattributes to temp file
    let project_root = get_project_root()?;
    env::set_current_dir(&project_root)?;
    
    // Compile and run generate-gitattributes.rs
    let temp_bin = compile_generate_script()?;
    
    // Run the script to generate temp .gitattributes
    let status = Command::new(&temp_bin)
        .arg(temp_output_path.to_str().unwrap())
        .status()?;
    
    if !status.success() {
        eprintln!("❌ Error: Failed to run generate-gitattributes");
        std::process::exit(1);
    }
    
    // Compare with existing .gitattributes
    let gitattributes_path = Path::new(".gitattributes");
    if gitattributes_path.exists() {
        let existing_content = fs::read_to_string(gitattributes_path)?;
        let new_content = fs::read_to_string(temp_output_path)?;
        
        if existing_content == new_content {
            println!("✅ .gitattributes is up to date");
            std::process::exit(0);
        } else {
            println!("⚠️  .gitattributes needs updating");
            std::process::exit(1);
        }
    } else {
        println!("⚠️  .gitattributes file not found, needs to be created");
        std::process::exit(1);
    }
}

fn force_update() -> io::Result<()> {
    println!("🔄 Force updating .gitattributes...");
    
    // Check dependencies
    check_hyperpolyglot()?;
    
    let project_root = get_project_root()?;
    env::set_current_dir(&project_root)?;
    
    // Backup existing .gitattributes if it exists
    let gitattributes_path = Path::new(".gitattributes");
    if gitattributes_path.exists() {
        fs::copy(gitattributes_path, ".gitattributes.backup")?;
        println!("📋 Backed up existing .gitattributes");
    }
    
    // Compile and run the generate-gitattributes script
    let temp_bin = compile_generate_script()?;
    
    let status = Command::new(&temp_bin)
        .arg(".gitattributes")
        .status()?;
    
    if status.success() {
        println!("✅ Successfully updated .gitattributes");
    } else {
        eprintln!("❌ Error: Failed to update .gitattributes");
        std::process::exit(1);
    }
    
    Ok(())
}

fn compile_generate_script() -> io::Result<std::path::PathBuf> {
    let temp_bin_path = create_temp_file()?;
    
    // Check if rustc is available
    let rustc_check = Command::new("rustc")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    
    match rustc_check {
        Ok(_) => {
            // Compile the generate-gitattributes.rs script
            let status = Command::new("rustc")
                .arg("-o")
                .arg(&temp_bin_path)
                .arg("scripts/generate-gitattributes.rs")
                .status()?;
            
            if status.success() {
                Ok(temp_bin_path)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to compile generate-gitattributes.rs"
                ))
            }
        }
        Err(_) => {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Rust compiler not found"
            ))
        }
    }
}

fn get_project_root() -> io::Result<std::path::PathBuf> {
    let current_dir = env::current_dir()?;
    
    // Walk up the directory tree to find the project root
    let mut current = current_dir.as_path();
    loop {
        if current.join("Cargo.toml").exists() || current.join(".git").exists() {
            return Ok(current.to_path_buf());
        }
        
        if let Some(parent) = current.parent() {
            current = parent;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not find project root"
            ));
        }
    }
}

fn create_temp_file() -> io::Result<std::path::PathBuf> {
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_path = temp_dir.join(format!("ci-gitattributes-{}", timestamp));
    
    Ok(temp_path)
}
