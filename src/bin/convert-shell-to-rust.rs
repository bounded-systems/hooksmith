use hooksmith::{log_error, log_header, log_info, log_success, log_warning};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// ANSI color codes for terminal output
const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[1;33m";
const BLUE: &str = "\x1b[0;34m";
const PURPLE: &str = "\x1b[0;35m";
const NC: &str = "\x1b[0m"; // No Color

/// Log an informational message with blue color
fn log_info_colored(message: &str) {
    println!("{}[INFO]{} {}", BLUE, NC, message);
}

/// Log a success message with green color
fn log_success_colored(message: &str) {
    println!("{}[SUCCESS]{} {}", GREEN, NC, message);
}

/// Log a warning message with yellow color
fn log_warning_colored(message: &str) {
    println!("{}[WARNING]{} {}", YELLOW, NC, message);
}

/// Log an error message with red color
fn log_error_colored(message: &str) {
    println!("{}[ERROR]{} {}", RED, NC, message);
}

/// Log a header message with purple color
fn log_header_colored(message: &str) {
    println!("{}=== {} ==={}", PURPLE, message, NC);
}

/// Analyze shell script and determine its purpose
fn analyze_script(script_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let script_name = script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    log_info_colored(&format!("Analyzing: {}", script_name));

    // Read first few lines to understand purpose
    let content = fs::read_to_string(script_path)?;
    let first_lines: String = content.lines().take(20).collect::<Vec<_>>().join("\n");

    // Look for common patterns
    if first_lines.contains("worktree") {
        Ok("worktree_management".to_string())
    } else if first_lines.contains("build") || first_lines.contains("compile") {
        Ok("build_script".to_string())
    } else if first_lines.contains("cleanup") || first_lines.contains("clean") {
        Ok("cleanup_script".to_string())
    } else if first_lines.contains("sync") || first_lines.contains("update") {
        Ok("sync_script".to_string())
    } else if first_lines.contains("verify") || first_lines.contains("check") {
        Ok("verification_script".to_string())
    } else if first_lines.contains("pr") || first_lines.contains("pull") {
        Ok("pr_management".to_string())
    } else {
        Ok("general_utility".to_string())
    }
}

/// Create Rust binary structure
fn create_rust_binary(
    script_path: &Path,
    script_type: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let script_name = script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let rust_file = PathBuf::from("src/bin").join(format!("{}.rs", script_name));

    log_info_colored(&format!("Creating Rust binary: {}", rust_file.display()));

    // Create the bin directory if it doesn't exist
    if let Some(parent) = rust_file.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create the Rust file with basic structure
    let rust_content = format!(
        r#"use std::process::Command;
use std::path::Path;
use hooksmith::{{log_info, log_success, log_warning, log_error, log_header}};

fn main() -> Result<(), Box<dyn std::error::Error>> {{
    log_header("{}");
    println!();
    
    // TODO: Implement functionality from {}
    log_info("Converting from shell script: {}");
    
    // Add specific implementation based on script type
    match "{}" {{
        "worktree_management" => {{
            log_info("This is a worktree management script");
            // TODO: Add worktree-specific functionality
        }}
        "build_script" => {{
            log_info("This is a build script");
            // TODO: Add build-specific functionality
        }}
        "cleanup_script" => {{
            log_info("This is a cleanup script");
            // TODO: Add cleanup-specific functionality
        }}
        "sync_script" => {{
            log_info("This is a sync script");
            // TODO: Add sync-specific functionality
        }}
        "verification_script" => {{
            log_info("This is a verification script");
            // TODO: Add verification-specific functionality
        }}
        "pr_management" => {{
            log_info("This is a PR management script");
            // TODO: Add PR-specific functionality
        }}
        _ => {{
            log_info("This is a general utility script");
            // TODO: Add general functionality
        }}
    }}
    
    log_success("Script execution completed");
    Ok(())
}}
"#,
        script_name,
        script_path.display(),
        script_path.display(),
        script_type
    );

    fs::write(&rust_file, rust_content)?;

    log_success_colored(&format!("Created Rust binary: {}", rust_file.display()));
    Ok(())
}

/// Extract key functions from shell script
fn extract_functions(script_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(script_path)?;
    let mut functions = Vec::new();

    for line in content.lines() {
        if let Some(function_name) = line.trim().strip_suffix("()") {
            if function_name
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
            {
                functions.push(function_name.to_string());
            }
        }
    }

    if !functions.is_empty() {
        log_info_colored(&format!("Found functions: {}", functions.join(", ")));
    } else {
        log_info_colored("No functions found in script");
    }

    Ok(functions)
}

/// Extract git commands from shell script
fn extract_git_commands(script_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(script_path)?;
    let mut git_commands = Vec::new();

    for line in content.lines() {
        if line.trim().starts_with("git ") {
            git_commands.push(line.trim().to_string());
        }
    }

    if !git_commands.is_empty() {
        log_info_colored("Found git commands:");
        for cmd in git_commands.iter().take(10) {
            println!("  {}", cmd);
        }
    } else {
        log_info_colored("No git commands found");
    }

    Ok(git_commands)
}

/// Create conversion summary
fn create_conversion_summary(
    script_path: &Path,
    script_type: &str,
    functions: &[String],
    git_commands: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let script_name = script_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let summary_file = PathBuf::from("docs/conversion-summary.md");

    // Create summary directory if it doesn't exist
    if let Some(parent) = summary_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut summary_content = String::new();

    // Append to summary file
    summary_content.push_str(&format!("\n## {}\n\n", script_name));
    summary_content.push_str(&format!("- **Original**: `{}`\n", script_path.display()));
    summary_content.push_str(&format!(
        "- **Rust Binary**: `src/bin/{}.rs`\n",
        script_name
    ));
    summary_content.push_str(&format!("- **Type**: {}\n", script_type));
    summary_content.push_str("- **Status**: Converted (basic structure)\n");
    summary_content.push_str("- **TODO**: Implement specific functionality\n\n");

    summary_content.push_str("### Key Functions\n");
    if !functions.is_empty() {
        for func in functions {
            summary_content.push_str(&format!("- {}\n", func));
        }
    } else {
        summary_content.push_str("- None found\n");
    }

    summary_content.push_str("\n### Git Commands\n");
    if !git_commands.is_empty() {
        for cmd in git_commands.iter().take(10) {
            summary_content.push_str(&format!("- {}\n", cmd));
        }
    } else {
        summary_content.push_str("- None found\n");
    }

    summary_content.push_str("\n---\n");

    // Append to existing file or create new
    if summary_file.exists() {
        let mut existing_content = fs::read_to_string(&summary_file)?;
        existing_content.push_str(&summary_content);
        fs::write(&summary_file, existing_content)?;
    } else {
        fs::write(&summary_file, summary_content)?;
    }

    log_info_colored(&format!(
        "Added to conversion summary: {}",
        summary_file.display()
    ));
    Ok(())
}

/// Find all shell scripts in the current directory
fn find_shell_scripts() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut scripts = Vec::new();

    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "sh" {
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

                    // Skip the convert script itself
                    if file_name != "convert-shell-to-rust.sh" {
                        scripts.push(path);
                    }
                }
            }
        }
    }

    Ok(scripts)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header_colored("SHELL TO RUST CONVERSION");
    println!();

    // Find all shell scripts
    let shell_scripts = find_shell_scripts()?;

    if shell_scripts.is_empty() {
        log_info_colored("No shell scripts found");
        return Ok(());
    }

    log_info_colored(&format!(
        "Found {} shell scripts to convert",
        shell_scripts.len()
    ));
    println!();

    let mut converted_count = 0;

    // Process each shell script
    for script_path in shell_scripts {
        let script_name = script_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let script_type = analyze_script(&script_path)?;

        log_header_colored(&format!("CONVERTING: {}", script_name));
        println!();

        // Create Rust binary
        create_rust_binary(&script_path, &script_type)?;

        // Extract functions and git commands
        let functions = extract_functions(&script_path)?;
        let git_commands = extract_git_commands(&script_path)?;

        // Create conversion summary
        create_conversion_summary(&script_path, &script_type, &functions, &git_commands)?;

        converted_count += 1;

        println!("---");
        println!();
    }

    log_success_colored(&format!(
        "Converted {} shell scripts to Rust",
        converted_count
    ));
    println!();
    log_info_colored("Next steps:");
    log_info_colored("1. Review the generated Rust files in src/bin/");
    log_info_colored("2. Implement specific functionality for each script");
    log_info_colored("3. Test the Rust binaries");
    log_info_colored("4. Update any references to the old shell scripts");

    Ok(())
}
