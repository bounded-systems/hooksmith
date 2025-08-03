#!/usr/bin/env rust-script
//! Enhanced file checksum report with actionable suggestions
//! Provides recommendations for improving checksum coverage and migrating shell scripts to Rust

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process;

#[derive(Debug)]
struct FileInfo {
    path: String,
    extension: String,
    is_generated: bool,
    has_checksum: bool,
    checksum_valid: Option<bool>,
}

#[derive(Debug)]
struct ExtensionStats {
    total_files: usize,
    generated_files: usize,
    checksum_ok: usize,
    checksum_fail: usize,
    files: Vec<String>,
}

fn remove_jsonc_comments(content: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut i = 0;
    
    while i < content.len() {
        let c = content.chars().nth(i).unwrap();
        
        if escape_next {
            result.push(c);
            escape_next = false;
            i += 1;
            continue;
        }
        
        if c == '\\' {
            escape_next = true;
            result.push(c);
            i += 1;
            continue;
        }
        
        if c == '"' {
            in_string = !in_string;
            result.push(c);
            i += 1;
            continue;
        }
        
        if !in_string && c == '/' && i + 1 < content.len() {
            let next_c = content.chars().nth(i + 1).unwrap();
            if next_c == '/' {
                // Single line comment
                while i < content.len() && content.chars().nth(i).unwrap() != '\n' {
                    i += 1;
                }
                if i < content.len() {
                    result.push('\n');
                }
                i += 1;
                continue;
            } else if next_c == '*' {
                // Multi-line comment
                i += 2;
                while i + 1 < content.len() {
                    if content.chars().nth(i).unwrap() == '*' && content.chars().nth(i + 1).unwrap() == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }
        
        result.push(c);
        i += 1;
    }
    
    result
}

fn compute_checksum(content: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..8].to_string()
}

fn validate_checksum(file_path: &str, expected_checksum: &str) -> bool {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            // Remove the @checksum line if present
            let lines: Vec<&str> = content.lines().collect();
            let content_without_checksum: String = lines
                .iter()
                .filter(|line| !line.trim().starts_with("@checksum"))
                .map(|line| format!("{}\n", line))
                .collect();
            
            let actual_checksum = compute_checksum(&content_without_checksum);
            actual_checksum == expected_checksum
        }
        Err(_) => false,
    }
}

fn get_shell_script_migration_suggestions() -> HashMap<String, Vec<String>> {
    let mut suggestions = HashMap::new();
    
    // Map shell scripts to potential Rust xtask commands
    suggestions.insert("scripts/advanced-git-aliases.sh".to_string(), vec![
        "cargo xtask git-aliases --advanced".to_string(),
        "cargo xtask git --aliases --advanced".to_string(),
    ]);
    
    suggestions.insert("scripts/build-stats.sh".to_string(), vec![
        "cargo xtask build-stats".to_string(),
        "cargo xtask stats --build".to_string(),
    ]);
    
    suggestions.insert("scripts/check-errors.sh".to_string(), vec![
        "cargo xtask check-errors".to_string(),
        "cargo xtask validate --errors".to_string(),
    ]);
    
    suggestions.insert("scripts/ci-build.sh".to_string(), vec![
        "cargo xtask ci-build".to_string(),
        "cargo xtask build --ci".to_string(),
    ]);
    
    suggestions.insert("scripts/cleanup-logs.sh".to_string(), vec![
        "cargo xtask cleanup-logs".to_string(),
        "cargo xtask logs --cleanup".to_string(),
    ]);
    
    suggestions.insert("scripts/debug-pre-push.sh".to_string(), vec![
        "cargo xtask debug-pre-push".to_string(),
        "cargo xtask hooks --debug --pre-push".to_string(),
    ]);
    
    suggestions.insert("scripts/demo_jql_analysis.sh".to_string(), vec![
        "cargo xtask jql-demo".to_string(),
        "cargo xtask demo --jql".to_string(),
    ]);
    
    suggestions.insert("scripts/dev-cycle.sh".to_string(), vec![
        "cargo xtask dev-cycle".to_string(),
        "cargo xtask dev --cycle".to_string(),
    ]);
    
    suggestions.insert("scripts/enforce_structured_logging.sh".to_string(), vec![
        "cargo xtask enforce-logging".to_string(),
        "cargo xtask logging --enforce".to_string(),
    ]);
    
    suggestions.insert("scripts/generate-files-config.sh".to_string(), vec![
        "cargo xtask generate-files-config".to_string(),
        "cargo xtask config --generate-files".to_string(),
    ]);
    
    suggestions.insert("scripts/install_logging_tools.sh".to_string(), vec![
        "cargo xtask install-logging-tools".to_string(),
        "cargo xtask tools --install --logging".to_string(),
    ]);
    
    suggestions.insert("scripts/log-stats.sh".to_string(), vec![
        "cargo xtask log-stats".to_string(),
        "cargo xtask stats --logs".to_string(),
    ]);
    
    suggestions.insert("scripts/macos-optimize.sh".to_string(), vec![
        "cargo xtask macos-optimize".to_string(),
        "cargo xtask optimize --macos".to_string(),
    ]);
    
    suggestions.insert("scripts/monitor_errors.sh".to_string(), vec![
        "cargo xtask monitor-errors".to_string(),
        "cargo xtask monitor --errors".to_string(),
    ]);
    
    suggestions.insert("scripts/monitor-errors.sh".to_string(), vec![
        "cargo xtask monitor-errors".to_string(),
        "cargo xtask monitor --errors".to_string(),
    ]);
    
    suggestions.insert("scripts/optimize-build.sh".to_string(), vec![
        "cargo xtask optimize-build".to_string(),
        "cargo xtask build --optimize".to_string(),
    ]);
    
    suggestions.insert("scripts/safe-commit.sh".to_string(), vec![
        "cargo xtask safe-commit".to_string(),
        "cargo xtask commit --safe".to_string(),
    ]);
    
    suggestions.insert("scripts/safe-git-aliases.sh".to_string(), vec![
        "cargo xtask git-aliases --safe".to_string(),
        "cargo xtask git --aliases --safe".to_string(),
    ]);
    
    suggestions.insert("scripts/safe-pre-push-hook.sh".to_string(), vec![
        "cargo xtask safe-pre-push-hook".to_string(),
        "cargo xtask hooks --safe --pre-push".to_string(),
    ]);
    
    suggestions.insert("scripts/safe-push.sh".to_string(), vec![
        "cargo xtask safe-push".to_string(),
        "cargo xtask push --safe".to_string(),
    ]);
    
    suggestions.insert("scripts/security-check.sh".to_string(), vec![
        "cargo xtask security-check".to_string(),
        "cargo xtask check --security".to_string(),
    ]);
    
    suggestions.insert("scripts/setup-default.sh".to_string(), vec![
        "cargo xtask setup-default".to_string(),
        "cargo xtask setup --default".to_string(),
    ]);
    
    suggestions.insert("scripts/setup-env.sh".to_string(), vec![
        "cargo xtask setup-env".to_string(),
        "cargo xtask setup --env".to_string(),
    ]);
    
    suggestions.insert("scripts/setup-log-cleanup.sh".to_string(), vec![
        "cargo xtask setup-log-cleanup".to_string(),
        "cargo xtask setup --log-cleanup".to_string(),
    ]);
    
    suggestions.insert("scripts/simple-git-aliases.sh".to_string(), vec![
        "cargo xtask git-aliases --simple".to_string(),
        "cargo xtask git --aliases --simple".to_string(),
    ]);
    
    suggestions.insert("scripts/update-ci-integration.sh".to_string(), vec![
        "cargo xtask update-ci-integration".to_string(),
        "cargo xtask ci --update-integration".to_string(),
    ]);
    
    suggestions.insert("scripts/validate-ci-setup.sh".to_string(), vec![
        "cargo xtask validate-ci-setup".to_string(),
        "cargo xtask ci --validate-setup".to_string(),
    ]);
    
    suggestions.insert("scripts/validation_summary.sh".to_string(), vec![
        "cargo xtask validation-summary".to_string(),
        "cargo xtask summary --validation".to_string(),
    ]);
    
    suggestions.insert("scripts/verify-ci-readiness.sh".to_string(), vec![
        "cargo xtask verify-ci-readiness".to_string(),
        "cargo xtask ci --verify-readiness".to_string(),
    ]);
    
    suggestions.insert("scripts/watch-dashboard.sh".to_string(), vec![
        "cargo xtask watch-dashboard".to_string(),
        "cargo xtask dashboard --watch".to_string(),
    ]);
    
    suggestions
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the generated files registry
    let registry_content = fs::read_to_string("config/generated-files.jsonc")?;
    let json_content = remove_jsonc_comments(&registry_content);
    let registry: serde_json::Value = serde_json::from_str(&json_content)?;
    
    let files_array = registry["files"].as_array().unwrap();
    let mut generated_files = HashSet::new();
    let mut file_checksums = HashMap::new();
    
    for file_entry in files_array {
        let path = file_entry["path"].as_str().unwrap();
        generated_files.insert(path.to_string());
        
        if let Some(checksum) = file_entry["checksum"].as_str() {
            file_checksums.insert(path.to_string(), checksum.to_string());
        }
    }
    
    // Get all files in the repository
    let output = process::Command::new("git")
        .args(&["ls-files"])
        .output()?;
    
    let all_files: Vec<String> = String::from_utf8(output.stdout)?
        .lines()
        .map(|s| s.to_string())
        .collect();
    
    let mut file_infos = Vec::new();
    
    for file_path in all_files {
        let path = Path::new(&file_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let is_generated = generated_files.contains(&file_path);
        let has_checksum = file_checksums.contains_key(&file_path);
        
        let checksum_valid = if has_checksum {
            let expected_checksum = file_checksums.get(&file_path).unwrap();
            Some(validate_checksum(&file_path, expected_checksum))
        } else {
            None
        };
        
        file_infos.push(FileInfo {
            path: file_path,
            extension,
            is_generated,
            has_checksum,
            checksum_valid,
        });
    }
    
    // Aggregate statistics by extension
    let mut extension_stats: HashMap<String, ExtensionStats> = HashMap::new();
    
    for file_info in &file_infos {
        let stats = extension_stats.entry(file_info.extension.clone()).or_insert(ExtensionStats {
            total_files: 0,
            generated_files: 0,
            checksum_ok: 0,
            checksum_fail: 0,
            files: Vec::new(),
        });
        
        stats.total_files += 1;
        stats.files.push(file_info.path.clone());
        
        if file_info.is_generated {
            stats.generated_files += 1;
            
            if let Some(is_valid) = file_info.checksum_valid {
                if is_valid {
                    stats.checksum_ok += 1;
                } else {
                    stats.checksum_fail += 1;
                }
            }
        }
    }
    
    // Print the report
    println!("# 📊 File Checksum Report\n");
    
    let total_files: usize = file_infos.len();
    let total_generated: usize = file_infos.iter().filter(|f| f.is_generated).count();
    let total_with_checksums: usize = file_infos.iter().filter(|f| f.has_checksum).count();
    let total_valid_checksums: usize = file_infos.iter()
        .filter(|f| f.checksum_valid.unwrap_or(false))
        .count();
    
    println!("## 📈 Summary Statistics");
    println!("- **Total files**: {}", total_files);
    println!("- **Generated files**: {} ({:.1}%)", total_generated, (total_generated as f64 / total_files as f64) * 100.0);
    println!("- **Files with checksums**: {} ({:.1}%)", total_with_checksums, (total_with_checksums as f64 / total_generated as f64) * 100.0);
    println!("- **Valid checksums**: {} ({:.1}%)", total_valid_checksums, (total_valid_checksums as f64 / total_generated as f64) * 100.0);
    println!();
    
    // Print the table
    println!("## 📋 Extension Breakdown");
    println!("| Extension | Total Files | Generated Files | Checksum OK | Checksum Fail | Coverage % |");
    println!("|-----------|-------------|-----------------|-------------|---------------|------------|");
    
    let mut sorted_extensions: Vec<_> = extension_stats.iter().collect();
    sorted_extensions.sort_by(|a, b| b.1.generated_files.cmp(&a.1.generated_files));
    
    for (extension, stats) in sorted_extensions {
        let coverage = if stats.generated_files > 0 {
            (stats.checksum_ok as f64 / stats.generated_files as f64) * 100.0
        } else {
            0.0
        };
        
        println!("| {} | {} | {} | {} | {} | {:.1}% |",
            extension,
            stats.total_files,
            stats.generated_files,
            stats.checksum_ok,
            stats.checksum_fail,
            coverage
        );
    }
    println!();
    
    // Generate actionable suggestions
    println!("## 🚀 Actionable Suggestions\n");
    
    let shell_script_suggestions = get_shell_script_migration_suggestions();
    
    // Find shell scripts that need migration
    let shell_scripts: Vec<_> = file_infos.iter()
        .filter(|f| f.extension == "sh" && f.is_generated)
        .collect();
    
    if !shell_scripts.is_empty() {
        println!("### 🔧 Shell Script Migration Recommendations");
        println!("The following shell scripts should be migrated to Rust-based xtask commands for better maintainability and performance:\n");
        
        for file_info in shell_scripts {
            if let Some(suggestions) = shell_script_suggestions.get(&file_info.path) {
                println!("**{}**", file_info.path);
                for suggestion in suggestions {
                    println!("  - `{}`", suggestion);
                }
                println!();
            }
        }
        
        println!("**Migration Benefits:**");
        println!("- ✅ Better error handling and type safety");
        println!("- ✅ Cross-platform compatibility");
        println!("- ✅ Integration with Cargo toolchain");
        println!("- ✅ Easier testing and debugging");
        println!("- ✅ Consistent with project's Rust-first approach");
        println!();
    }
    
    // Find files missing checksums
    let missing_checksums: Vec<_> = file_infos.iter()
        .filter(|f| f.is_generated && !f.has_checksum)
        .collect();
    
    if !missing_checksums.is_empty() {
        println!("### 🔍 Missing Checksums");
        println!("The following generated files are missing checksums:\n");
        
        let mut by_extension: HashMap<String, Vec<&FileInfo>> = HashMap::new();
        for file_info in missing_checksums {
            by_extension.entry(file_info.extension.clone()).or_default().push(file_info);
        }
        
        for (extension, files) in by_extension.iter() {
            println!("**{} files ({}):**", extension.to_uppercase(), files.len());
            for file_info in files.iter().take(5) { // Show first 5
                println!("  - `{}`", file_info.path);
            }
            if files.len() > 5 {
                println!("  - ... and {} more", files.len() - 5);
            }
            println!();
        }
        
        println!("**To add checksums:**");
        println!("```bash");
        println!("# Add checksum to a specific file");
        println!("cargo xtask add-checksum --slug=<file-slug>");
        println!();
        println!("# Add checksums to all missing files");
        println!("cargo xtask add-checksums --missing");
        println!("```");
        println!();
    }
    
    // Find files with invalid checksums
    let invalid_checksums: Vec<_> = file_infos.iter()
        .filter(|f| f.checksum_valid == Some(false))
        .collect();
    
    if !invalid_checksums.is_empty() {
        println!("### ⚠️ Invalid Checksums");
        println!("The following files have invalid checksums (content has changed):\n");
        
        for file_info in invalid_checksums.iter().take(10) { // Show first 10
            println!("- `{}`", file_info.path);
        }
        
        if invalid_checksums.len() > 10 {
            println!("- ... and {} more", invalid_checksums.len() - 10);
        }
        println!();
        
        println!("**To fix invalid checksums:**");
        println!("```bash");
        println!("cargo xtask update-checksum --slug=<file-slug>");
        println!("```");
        println!();
    }
    
    // Priority recommendations
    println!("### 🎯 Priority Recommendations");
    
    let default_stats = ExtensionStats {
        total_files: 0,
        generated_files: 0,
        checksum_ok: 0,
        checksum_fail: 0,
        files: Vec::new(),
    };
    
    let sh_stats = extension_stats.get("sh").unwrap_or(&default_stats);
    
    if sh_stats.generated_files > 0 && sh_stats.checksum_ok == 0 {
        println!("1. **🔴 HIGH PRIORITY**: {} shell scripts have 0% checksum coverage", sh_stats.generated_files);
        println!("   - These should be migrated to Rust xtask commands");
        println!("   - Start with frequently used scripts like `safe-commit.sh`, `dev-cycle.sh`");
        println!();
    }
    
    let json_stats = extension_stats.get("json").unwrap_or(&default_stats);
    
    if json_stats.generated_files > 0 && json_stats.checksum_ok == 0 {
        println!("2. **🟡 MEDIUM PRIORITY**: {} JSON files have 0% checksum coverage", json_stats.generated_files);
        println!("   - Add checksums to configuration files");
        println!("   - Use `cargo xtask add-checksum --slug=<file-slug>`");
        println!();
    }
    
    let md_stats = extension_stats.get("md").unwrap_or(&default_stats);
    
    if md_stats.generated_files > 0 {
        let md_coverage = (md_stats.checksum_ok as f64 / md_stats.generated_files as f64) * 100.0;
        if md_coverage < 90.0 {
            println!("3. **🟢 LOW PRIORITY**: Markdown files have {:.1}% checksum coverage", md_coverage);
            println!("   - Add checksums to remaining documentation files");
            println!("   - Focus on frequently updated docs first");
            println!();
        }
    }
    
    // Next steps
    println!("### 📋 Next Steps");
    println!("1. **Immediate**: Run `cargo xtask add-checksums --missing` to add missing checksums");
    println!("2. **Short-term**: Migrate 2-3 shell scripts to Rust xtask commands");
    println!("3. **Medium-term**: Achieve 95%+ checksum coverage across all file types");
    println!("4. **Long-term**: Complete shell script migration to Rust");
    println!();
    
    println!("### 📊 Coverage Goals");
    println!("- **Target**: 95% checksum coverage for generated files");
    println!("- **Current**: {:.1}% coverage", (total_valid_checksums as f64 / total_generated as f64) * 100.0);
    println!("- **Gap**: {} files need checksums", total_generated - total_valid_checksums);
    
    Ok(())
} 
