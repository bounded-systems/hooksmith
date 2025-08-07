use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{Command, ExitCode, Stdio};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("generate-gitattributes - Generate .gitattributes using hyperpolyglot");
        println!();
        println!("Usage:");
        println!("  {} [output_file] [options]", args[0]);
        println!();
        println!("Options:");
        println!("  --check-only     Only check if .gitattributes needs updating");
        println!("  --force          Force update even if no changes detected");
        println!("  --verbose        Verbose output");
        println!("  --help           Show this help message");
        println!();
        println!("Examples:");
        println!("  {} .gitattributes", args[0]);
        println!("  {} custom.gitattributes --force", args[0]);
        println!("  {} --check-only", args[0]);
        return ExitCode::SUCCESS;
    }
    
    let mut output_file = ".gitattributes".to_string();
    let mut check_only = false;
    let mut force = false;
    let mut verbose = false;
    
    // Parse arguments
    for arg in &args[1..] {
        match arg.as_str() {
            "--check-only" => check_only = true,
            "--force" => force = true,
            "--verbose" => verbose = true,
            "--help" => {
                println!("generate-gitattributes - Generate .gitattributes using hyperpolyglot");
                println!();
                println!("Usage:");
                println!("  {} [output_file] [options]", args[0]);
                println!();
                println!("Options:");
                println!("  --check-only     Only check if .gitattributes needs updating");
                println!("  --force          Force update even if no changes detected");
                println!("  --verbose        Verbose output");
                println!("  --help           Show this help message");
                return ExitCode::SUCCESS;
            }
            _ => {
                if !arg.starts_with("--") {
                    output_file = arg.clone();
                }
            }
        }
    }
    
    if verbose {
        println!("🔍 Analyzing repository files with hyperpolyglot...");
    }
    
    // Check if we're in a git repository
    if Command::new("git").args(&["rev-parse", "--git-dir"]).output().is_err() {
        eprintln!("❌ Error: Not in a git repository");
        return ExitCode::FAILURE;
    }
    
    // Get all tracked files from git
    let files = match get_git_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error getting git files: {}", e);
            return ExitCode::FAILURE;
        }
    };
    
    if verbose {
        println!("📁 Found {} tracked files", files.len());
    }
    
    // Detect languages for each file
    let language_map = match detect_languages(&files, verbose) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("❌ Error detecting languages: {}", e);
            return ExitCode::FAILURE;
        }
    };
    
    if verbose {
        println!("🎯 Detected {} unique languages", language_map.len());
    }
    
    // Check if update is needed
    if check_only {
        if needs_update(&output_file, &language_map) {
            println!("⚠️  .gitattributes needs updating");
            return ExitCode::FAILURE;
        } else {
            println!("✅ .gitattributes is up to date");
            return ExitCode::SUCCESS;
        }
    }
    
    // Generate .gitattributes content
    let gitattributes_content = match generate_gitattributes(&language_map) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("❌ Error generating .gitattributes: {}", e);
            return ExitCode::FAILURE;
        }
    };
    
    // Write to file
    if let Err(e) = fs::write(&output_file, gitattributes_content) {
        eprintln!("❌ Error writing {}: {}", output_file, e);
        return ExitCode::FAILURE;
    }
    
    println!("✅ Generated {} with {} language entries", output_file, language_map.len());
    
    // Print summary
    print_summary(&language_map);
    
    ExitCode::SUCCESS
}

fn get_git_files() -> io::Result<Vec<String>> {
    let output = Command::new("git")
        .args(&["ls-files", "--cached", "--full-name", "--exclude-standard"])
        .output()?;
    
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get git files"
        ));
    }
    
    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    Ok(files)
}

fn detect_languages(files: &[String], verbose: bool) -> io::Result<HashMap<String, Vec<String>>> {
    let mut language_map: HashMap<String, Vec<String>> = HashMap::new();
    
    for (i, file) in files.iter().enumerate() {
        if verbose && i % 100 == 0 {
            println!("🔍 Processing file {}/{}", i + 1, files.len());
        }
        
        if let Some(language) = detect_single_file_language(file)? {
            language_map
                .entry(language)
                .or_insert_with(Vec::new)
                .push(file.clone());
        }
    }
    
    Ok(language_map)
}

fn detect_single_file_language(file_path: &str) -> io::Result<Option<String>> {
    // Skip files that don't exist or are directories
    if !Path::new(file_path).exists() {
        return Ok(None);
    }
    
    let output = Command::new("hyply")
        .args(&["--breakdown", file_path])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    
    if !output.status.success() {
        // Skip files that can't be analyzed
        return Ok(None);
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    
    // Parse hyply output to extract language
    // hyply outputs like: "100.00% Rust" for single files
    for line in output_str.lines() {
        if line.contains('%') {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let language = parts[1];
                return Ok(Some(language.to_string()));
            }
        }
    }
    
    Ok(None)
}

fn generate_gitattributes(language_map: &HashMap<String, Vec<String>>) -> io::Result<String> {
    let mut content = String::new();
    
    // Header
    content.push_str("# @generated by generate-gitattributes.rs\n");
    content.push_str("# Do not edit manually - this file is auto-generated\n");
    content.push_str("# This file is automatically generated by generate-gitattributes.rs\n");
    content.push_str("# using hyperpolyglot for language detection\n");
    content.push_str("\n");
    
    // Add common exclusions
    content.push_str("# Common exclusions\n");
    content.push_str("*.md linguist-documentation\n");
    content.push_str("*.txt linguist-documentation\n");
    content.push_str("*.yml linguist-documentation\n");
    content.push_str("*.yaml linguist-documentation\n");
    content.push_str("*.toml linguist-documentation\n");
    content.push_str("*.json linguist-documentation\n");
    content.push_str("*.gitattributes linguist-documentation\n");
    content.push_str("*.gitignore linguist-documentation\n");
    content.push_str("*.editorconfig linguist-documentation\n");
    content.push_str("*.trunk linguist-documentation\n");
    content.push_str("*.wb linguist-documentation\n");
    content.push_str("*.worktree linguist-documentation\n");
    content.push_str("*.workbloom linguist-documentation\n");
    content.push_str("*.worktree-config linguist-documentation\n");
    content.push_str("*.worktree-config.json linguist-documentation\n");
    content.push_str("*.worktree-config.jsonc linguist-documentation\n");
    content.push_str("*.worktree-config.yml linguist-documentation\n");
    content.push_str("*.worktree-config.yaml linguist-documentation\n");
    content.push_str("*.worktree-config.toml linguist-documentation\n");
    content.push_str("*.worktree-config.md linguist-documentation\n");
    content.push_str("*.worktree-config.txt linguist-documentation\n");
    content.push_str("*.worktree-config.gitattributes linguist-documentation\n");
    content.push_str("*.worktree-config.gitignore linguist-documentation\n");
    content.push_str("*.worktree-config.editorconfig linguist-documentation\n");
    content.push_str("*.worktree-config.trunk linguist-documentation\n");
    content.push_str("*.worktree-config.wb linguist-documentation\n");
    content.push_str("*.worktree-config.worktree linguist-documentation\n");
    content.push_str("*.worktree-config.workbloom linguist-documentation\n");
    content.push_str("\n");
    
    // Add language-specific overrides
    content.push_str("# Language-specific overrides\n");
    for (language, files) in language_map {
        for file in files {
            content.push_str(&format!("{} linguist-language={}\n", file, language));
        }
    }
    
    // Add generated file markers
    content.push_str("\n");
    content.push_str("# Generated files\n");
    content.push_str("*.wit linguist-generated\n");
    content.push_str("*.wit.md linguist-generated\n");
    content.push_str("*.wit.txt linguist-generated\n");
    content.push_str("*.wit.yml linguist-generated\n");
    content.push_str("*.wit.yaml linguist-generated\n");
    content.push_str("*.wit.toml linguist-generated\n");
    content.push_str("*.wit.json linguist-generated\n");
    content.push_str("*.wit.jsonc linguist-generated\n");
    content.push_str("*.wit.gitattributes linguist-generated\n");
    content.push_str("*.wit.gitignore linguist-generated\n");
    content.push_str("*.wit.editorconfig linguist-generated\n");
    content.push_str("*.wit.trunk linguist-generated\n");
    content.push_str("*.wit.wb linguist-generated\n");
    content.push_str("*.wit.worktree linguist-generated\n");
    content.push_str("*.wit.workbloom linguist-generated\n");
    content.push_str("*.wit.worktree-config linguist-generated\n");
    content.push_str("*.wit.worktree-config.json linguist-generated\n");
    content.push_str("*.wit.worktree-config.jsonc linguist-generated\n");
    content.push_str("*.wit.worktree-config.yml linguist-generated\n");
    content.push_str("*.wit.worktree-config.yaml linguist-generated\n");
    content.push_str("*.wit.worktree-config.toml linguist-generated\n");
    content.push_str("*.wit.worktree-config.md linguist-generated\n");
    content.push_str("*.wit.worktree-config.txt linguist-generated\n");
    content.push_str("*.wit.worktree-config.gitattributes linguist-generated\n");
    content.push_str("*.wit.worktree-config.gitignore linguist-generated\n");
    content.push_str("*.wit.worktree-config.editorconfig linguist-generated\n");
    content.push_str("*.wit.worktree-config.trunk linguist-generated\n");
    content.push_str("*.wit.worktree-config.wb linguist-generated\n");
    content.push_str("*.wit.worktree-config.worktree linguist-generated\n");
    content.push_str("*.wit.worktree-config.workbloom linguist-generated\n");
    
    Ok(content)
}

fn needs_update(output_file: &str, language_map: &HashMap<String, Vec<String>>) -> bool {
    if !Path::new(output_file).exists() {
        return true;
    }
    
    // Simple check: if we have languages detected, we should have a .gitattributes file
    !language_map.is_empty()
}

fn print_summary(language_map: &HashMap<String, Vec<String>>) {
    println!();
    println!("📊 Language Detection Summary:");
    println!("=============================");
    
    let mut sorted_languages: Vec<_> = language_map.iter().collect();
    sorted_languages.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    
    for (language, files) in sorted_languages.iter().take(10) {
        println!("   {}: {} files", language, files.len());
    }
    
    if language_map.len() > 10 {
        println!("   ... and {} more languages", language_map.len() - 10);
    }
}
