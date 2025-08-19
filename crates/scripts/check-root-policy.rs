use std::collections::HashSet;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_dir = Path::new(".");
    let mut violations = Vec::new();
    let mut warnings = Vec::new();

    // Define allowed files and directories at root
    let allowed_files: HashSet<&str> = [
        // Essential configuration files
        "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml", 
        "clippy.toml", "deny.toml", "build.rs", ".gitignore", ".gitattributes", 
        ".gitmodules", "CODEOWNERS",
        // Docker/Infrastructure files
        "Dockerfile", "docker-compose.yml", "docker-bake.hcl", ".dockerignore",
        // Project configuration
        "README.md", ".worktree-config.json", ".worktree-config.jsonc", ".workbloom",
    ].iter().cloned().collect();

    let allowed_dirs: HashSet<&str> = [
        // Essential directories
        ".cargo", ".github", ".git", ".hooksmith", ".trunk", ".wb", ".contract_cache",
        // Core project directories
        "crates", "src", "contracts", "docs", "tests", "examples", "tools", 
        "scripts", "schemas", "config", "hooks", "wit", "gen", "generated-sources", 
        "target", "worktree-lifecycle",
    ].iter().cloned().collect();

    // Patterns that indicate files should be moved
    let summary_patterns = ["_SUMMARY.md", "_PROGRESS.md", "_COMPLETE.md"];
    let test_patterns = ["test_", "test-", "_test.rs", "_test.txt"];
    let config_patterns = [".yml", ".yaml", ".json", ".jsonc", ".toml"];
    let doc_patterns = ["README_", "ARCHITECTURE_", "IMPLEMENTATION_", "GUIDE_"];

    println!("🔍 Checking root directory policy compliance...\n");

    for entry in fs::read_dir(root_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();

        // Skip .git directory contents
        if name == ".git" {
            continue;
        }

        if path.is_file() {
            if !allowed_files.contains(name) {
                // Check for specific patterns that indicate violations
                let mut is_violation = false;

                // Check for summary files
                for pattern in &summary_patterns {
                    if name.contains(pattern) {
                        violations.push((name.to_string(), "summary file", "docs/summaries/"));
                        is_violation = true;
                        break;
                    }
                }

                // Check for test files
                for pattern in &test_patterns {
                    if name.contains(pattern) {
                        violations.push((name.to_string(), "test file", "tests/"));
                        is_violation = true;
                        break;
                    }
                }

                // Check for configuration files
                for pattern in &config_patterns {
                    if name.ends_with(pattern) && !allowed_files.contains(name) {
                        warnings.push((name.to_string(), "configuration file", "config/"));
                        break;
                    }
                }

                // Check for documentation files
                for pattern in &doc_patterns {
                    if name.contains(pattern) && !allowed_files.contains(name) {
                        warnings.push((name.to_string(), "documentation file", "docs/"));
                        break;
                    }
                }

                // Check for other common patterns
                if !is_violation {
                    if name.ends_with(".rs") && !allowed_files.contains(name) {
                        warnings.push((name.to_string(), "Rust source file", "src/ or crates/"));
                    } else if name.ends_with(".md") && !allowed_files.contains(name) {
                        warnings.push((name.to_string(), "markdown file", "docs/"));
                    } else if name.ends_with(".txt") && !allowed_files.contains(name) {
                        warnings.push((name.to_string(), "text file", "docs/ or tests/"));
                    } else if name.ends_with(".json") && !allowed_files.contains(name) {
                        warnings.push((name.to_string(), "JSON file", "config/ or schemas/"));
                    } else if name.ends_with(".yml") || name.ends_with(".yaml") {
                        if !allowed_files.contains(name) {
                            warnings.push((name.to_string(), "YAML file", "config/"));
                        }
                    }
                }
            }
        } else if path.is_dir() {
            if !allowed_dirs.contains(name) {
                warnings.push((name.to_string(), "directory", "evaluate if needed at root"));
            }
        }
    }

    // Report violations
    if !violations.is_empty() {
        println!("❌ POLICY VIOLATIONS (must be moved):");
        for (file, file_type, suggested_location) in &violations {
            println!("   • {} ({}) → {}", file, file_type, suggested_location);
        }
        println!();
    }

    // Report warnings
    if !warnings.is_empty() {
        println!("⚠️  POTENTIAL ISSUES (review recommended):");
        for (file, file_type, suggested_location) in &warnings {
            println!("   • {} ({}) → {}", file, file_type, suggested_location);
        }
        println!();
    }

    // Summary
    let total_issues = violations.len() + warnings.len();
    if total_issues == 0 {
        println!("✅ Root directory policy compliance: PASSED");
        println!("   All files are properly organized!");
    } else {
        println!("📊 Summary:");
        println!("   • Violations: {}", violations.len());
        println!("   • Warnings: {}", warnings.len());
        println!("   • Total issues: {}", total_issues);
        
        if !violations.is_empty() {
            println!("\n❌ Please fix violations before proceeding.");
            std::process::exit(1);
        }
    }

    Ok(())
}
