use std::collections::HashSet;
use std::env;
use std::path::Path;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header};

/// Whitelist of allowed file extensions for source files
/// Only .rs and .jsonc files are allowed as manually maintained source files
/// All other file types must be code-generated
const ALLOWED_EXTENSIONS: &[&str] = &[
    // Rust source files (manually maintained)
    "rs",
    // JSON with comments configuration files (manually maintained)
    "jsonc",
];

/// Directories to exclude from validation
const EXCLUDED_DIRS: &[&str] = &[
    "target", "dist", "build", "node_modules", ".git", ".trunk", "logs",
    "status-trends", "generated_file_demo", ".hooks"
];

fn validate_file_extensions() -> Result<(), Box<dyn std::error::Error>> {
    log_header("FILE EXTENSION VALIDATION");
    println!();

    let allowed_extensions: HashSet<&str> = ALLOWED_EXTENSIONS.iter().copied().collect();
    let excluded_dirs: HashSet<&str> = EXCLUDED_DIRS.iter().copied().collect();
    
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut shell_scripts = Vec::new();
    let mut rust_files = Vec::new();
    let mut other_files = Vec::new();

    // Get all files from git
    let output = std::process::Command::new("git")
        .args(&["ls-files"])
        .output()
        .map_err(|e| format!("Failed to get git files: {}", e))?;

    let files_list = String::from_utf8_lossy(&output.stdout);
    
    for file_path in files_list.lines() {
        let file_path = file_path.trim();
        if file_path.is_empty() {
            continue;
        }

        let path = Path::new(file_path);
        
        // Skip excluded directories
        if path.components().any(|component| {
            if let std::path::Component::Normal(name) = component {
                excluded_dirs.contains(name.to_str().unwrap_or(""))
            } else {
                false
            }
        }) {
            continue;
        }

        // Check file extension
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                match ext_str {
                    "rs" => {
                        rust_files.push(file_path.to_string());
                        println!("✅ {} (Rust file)", file_path);
                    }
                    "sh" | "bash" | "zsh" => {
                        shell_scripts.push(file_path.to_string());
                        errors.push(format!(
                            "File '{}' has shell script extension '{}'. Convert to .rs for Rust-based scripts.",
                            file_path, ext_str
                        ));
                    }
                    "jsonc" => {
                        println!("✅ {} (JSONC file)", file_path);
                    }
                    _ => {
                        other_files.push(file_path.to_string());
                        warnings.push(format!(
                            "File '{}' has extension '{}'. Consider converting to .rs or ensure it's generated.",
                            file_path, ext_str
                        ));
                    }
                }
            } else {
                warnings.push(format!(
                    "File '{}' has invalid extension encoding",
                    file_path
                ));
            }
        } else {
            // Files without extensions
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            match filename {
                "Makefile" | "Dockerfile" | "README" | "LICENSE" => {
                    println!("✅ {} (standard file without extension)", file_path);
                }
                _ => {
                    other_files.push(file_path.to_string());
                    warnings.push(format!(
                        "File '{}' has no extension. Consider adding appropriate extension or converting to .rs",
                        file_path
                    ));
                }
            }
        }
    }

    // Report results
    println!();
    log_header("VALIDATION SUMMARY");
    println!();
    
    println!("📊 File Statistics:");
    println!("   Rust files (.rs): {}", rust_files.len());
    println!("   Shell scripts (.sh/.bash/.zsh): {}", shell_scripts.len());
    println!("   Other files: {}", other_files.len());
    println!();

    if !shell_scripts.is_empty() {
        log_error(&format!("Found {} shell scripts that need to be converted to Rust:", shell_scripts.len()));
        for script in &shell_scripts {
            println!("   - {}", script);
        }
        println!();
    }

    if !errors.is_empty() {
        log_error("File extension validation failed:");
        for error in &errors {
            println!("   {}", error);
        }
        println!();
    }

    if !warnings.is_empty() {
        log_warning("Warnings:");
        for warning in &warnings {
            println!("   {}", warning);
        }
        println!();
    }

    if errors.is_empty() && warnings.is_empty() {
        log_success("All file extensions validated successfully!");
        log_success("No shell scripts found - only Rust files present!");
    } else {
        log_warning("Some files need attention. Consider converting shell scripts to Rust.");
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        // Validate specific files
        let allowed_extensions: HashSet<&str> = ALLOWED_EXTENSIONS.iter().copied().collect();
        let mut errors = Vec::new();

        for file_path in &args[1..] {
            let path = Path::new(file_path);
            
            if let Some(extension) = path.extension() {
                if let Some(ext_str) = extension.to_str() {
                    if !allowed_extensions.contains(ext_str) {
                        errors.push(format!(
                            "File '{}' has disallowed extension '{}'. Only .rs and .jsonc files are allowed as source files.",
                            file_path, ext_str
                        ));
                    }
                }
            }
        }

        if !errors.is_empty() {
            for error in &errors {
                log_error(error);
            }
            std::process::exit(1);
        } else {
            log_success("All specified files have valid extensions!");
        }
    } else {
        // Validate all files
        validate_file_extensions()?;
    }

    Ok(())
} 
