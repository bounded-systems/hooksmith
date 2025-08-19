#!/usr/bin/env -S rustc --edition=2021 -o /tmp/migrate-shell-scripts && /tmp/migrate-shell-scripts

use std::fs;
use std::path::Path;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    let analyze = args.iter().any(|arg| arg == "--analyze");
    let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
    let dry_run = args.iter().any(|arg| arg == "--dry-run");
    
    if analyze {
        analyze_scripts(verbose, dry_run)?;
    } else {
        println!("🔄 Hooksmith Shell Script Migration Tool");
        println!("==========================================");
        println!();
        println!("Usage:");
        println!("  --analyze     Analyze all shell scripts and generate migration plan");
        println!("  --generate    Generate Rust equivalents for all scripts");
        println!("  --convert <script>  Convert a specific shell script");
        println!("  --verbose     Show detailed analysis");
        println!("  --dry-run     Show what would be done without making changes");
    }
    
    Ok(())
}

fn analyze_scripts(verbose: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing shell scripts...");
    
    let scripts_dir = Path::new("scripts");
    let mut scripts = Vec::new();
    
    // Find all .sh files
    for entry in fs::read_dir(scripts_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "sh") {
            let analysis = analyze_shell_script(&path, verbose)?;
            scripts.push(analysis);
        }
    }
    
    // Sort by lines (complexity)
    scripts.sort_by(|a, b| b.lines.cmp(&a.lines));
    
    println!("📋 MIGRATION PLAN");
    println!("================");
    println!();
    println!("📊 Summary:");
    println!("   Total scripts: {}", scripts.len());
    println!("   Total lines: {}", scripts.iter().map(|s| s.lines).sum::<usize>());
    println!();
    
    println!("🎯 Priority Order (by complexity):");
    for (i, script) in scripts.iter().enumerate() {
        let priority = if script.lines > 200 { "HIGH" } else if script.lines > 100 { "MEDIUM" } else { "LOW" };
        let priority_color = match priority {
            "HIGH" => "🔴",
            "MEDIUM" => "🟡", 
            "LOW" => "🟢",
            _ => "⚪",
        };
        
        println!("   {}. {} ({} lines, {} priority)", 
            i + 1,
            script.filename,
            script.lines,
            priority_color
        );
    }
    
    println!();
    println!("🚀 Recommended Migration Strategy:");
    println!("   1. Start with HIGH priority scripts (most complex)");
    println!("   2. Focus on scripts with many functions first");
    println!("   3. Convert dependencies to Rust equivalents");
    println!("   4. Test each conversion thoroughly");
    println!("   5. Update documentation and usage");
    
    Ok(())
}

struct ScriptAnalysis {
    filename: String,
    lines: usize,
    functions: Vec<String>,
    dependencies: Vec<String>,
}

fn analyze_shell_script(path: &Path, verbose: bool) -> Result<ScriptAnalysis, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    
    let mut functions = Vec::new();
    let mut dependencies = Vec::new();
    
    // Extract functions
    for line in &lines {
        if line.trim().ends_with("() {") {
            if let Some(func_name) = line.trim().split_whitespace().next() {
                functions.push(func_name.to_string());
            }
        }
    }
    
    // Extract dependencies (commands used)
    let commands = ["git", "cargo", "rustc", "wasmtime", "gh", "jq", "sed", "awk", "grep", "find", "xargs"];
    for line in &lines {
        for cmd in &commands {
            if line.contains(cmd) && !dependencies.contains(&cmd.to_string()) {
                dependencies.push(cmd.to_string());
            }
        }
    }
    
    if verbose {
        println!("📄 {} ({} lines)", path.display(), lines.len());
        println!("   Functions: {}", functions.join(", "));
        println!("   Dependencies: {}", dependencies.join(", "));
        println!();
    }
    
    Ok(ScriptAnalysis {
        filename: path.file_name().unwrap().to_string_lossy().to_string(),
        lines: lines.len(),
        functions,
        dependencies,
    })
}
