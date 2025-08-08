use std::process::Command;
use std::str::FromStr;

#[derive(Debug, Clone)]
struct HygieneIssue {
    file_path: String,
    issue_type: String,
    severity: String,
    description: String,
    recommendation: String,
    potential_savings: Option<u64>,
}

#[derive(Debug, Clone)]
struct HygieneCategory {
    name: String,
    issues: Vec<HygieneIssue>,
    total_savings: u64,
    priority: String,
}

#[derive(Debug)]
struct HygieneReport {
    categories: Vec<HygieneCategory>,
    total_issues: usize,
    total_potential_savings: u64,
    gitignore_suggestions: Vec<String>,
    gitattributes_suggestions: Vec<String>,
    lfs_suggestions: Vec<String>,
    optimization_commands: Vec<String>,
}

fn analyze_frequent_write_files() -> Result<Vec<HygieneIssue>, Box<dyn std::error::Error>> {
    println!("📝 Analyzing frequent write files...");

    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;

    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut issues = Vec::new();

    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let path_lower = path.to_lowercase();

            // Check for files that should be ignored
            if should_be_ignored(&path_lower) {
                let (issue_type, severity, description, recommendation) =
                    analyze_ignore_candidate(&path_lower);

                issues.push(HygieneIssue {
                    file_path: path,
                    issue_type,
                    severity,
                    description,
                    recommendation,
                    potential_savings: None, // Will calculate later
                });
            }
        }
    }

    println!("📝 Found {} files that should be ignored", issues.len());
    Ok(issues)
}

fn should_be_ignored(path: &str) -> bool {
    // Common patterns that should be ignored
    let ignore_patterns = [
        "target/",
        "build/",
        "dist/",
        "node_modules/",
        ".cargo/",
        "*.log",
        "*.tmp",
        "*.temp",
        "*.cache",
        "*.lock",
        ".DS_Store",
        "Thumbs.db",
        "*.swp",
        "*.swo",
        "*.o",
        "*.so",
        "*.dylib",
        "*.dll",
        "*.exe",
        "*.pyc",
        "__pycache__/",
        "*.class",
        "*.min.js",
        "*.min.css",
        "*.map",
    ];

    for pattern in &ignore_patterns {
        if pattern.ends_with('/') {
            if path.contains(pattern) {
                return true;
            }
        } else if pattern.starts_with('*') {
            let ext = &pattern[1..];
            if path.ends_with(ext) {
                return true;
            }
        } else if path.contains(pattern) {
            return true;
        }
    }

    false
}

fn analyze_ignore_candidate(path: &str) -> (String, String, String, String) {
    if path.contains("target/") || path.contains("build/") {
        (
            "Build Artifacts".to_string(),
            "High".to_string(),
            "Build artifacts should not be tracked".to_string(),
            "Add to .gitignore".to_string(),
        )
    } else if path.contains(".log") || path.contains("log/") {
        (
            "Log Files".to_string(),
            "Medium".to_string(),
            "Log files change frequently and bloat history".to_string(),
            "Add *.log to .gitignore".to_string(),
        )
    } else if path.contains(".cache") || path.contains("cache/") {
        (
            "Cache Files".to_string(),
            "Medium".to_string(),
            "Cache files are regenerated and shouldn't be tracked".to_string(),
            "Add cache directories to .gitignore".to_string(),
        )
    } else if path.contains(".lock") {
        (
            "Lock Files".to_string(),
            "Low".to_string(),
            "Lock files may be regenerated".to_string(),
            "Consider if lock file should be tracked".to_string(),
        )
    } else {
        (
            "Frequent Writes".to_string(),
            "Medium".to_string(),
            "File may change frequently".to_string(),
            "Review if this should be tracked".to_string(),
        )
    }
}

fn analyze_large_files() -> Result<Vec<HygieneIssue>, Box<dyn std::error::Error>> {
    println!("📦 Analyzing large files for LFS...");

    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;

    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut issues = Vec::new();

    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let hash = parts[1];

            // Get blob size
            let size_output = Command::new("git")
                .args(&["cat-file", "-s", hash])
                .output()?;

            if let Ok(size_str) = String::from_utf8(size_output.stdout) {
                if let Ok(blob_size) = u64::from_str(size_str.trim()) {
                    if blob_size > 50 * 1024 * 1024 {
                        // 50 MB
                        let (issue_type, severity, description, recommendation) =
                            analyze_lfs_candidate(&path, blob_size);

                        issues.push(HygieneIssue {
                            file_path: path,
                            issue_type,
                            severity,
                            description,
                            recommendation,
                            potential_savings: Some(blob_size - 130), // LFS pointer is ~130 bytes
                        });
                    }
                }
            }
        }
    }

    println!("📦 Found {} large files for LFS", issues.len());
    Ok(issues)
}

fn analyze_lfs_candidate(path: &str, size: u64) -> (String, String, String, String) {
    let path_lower = path.to_lowercase();

    if path_lower.contains(".bin") || path_lower.contains(".exe") || path_lower.contains(".so") {
        (
            "Binary Files".to_string(),
            "High".to_string(),
            format!("Large binary file ({} MB)", size / (1024 * 1024)),
            "Use Git LFS for binary files".to_string(),
        )
    } else if path_lower.contains(".zip")
        || path_lower.contains(".tar")
        || path_lower.contains(".gz")
    {
        (
            "Archive Files".to_string(),
            "High".to_string(),
            format!("Large archive file ({} MB)", size / (1024 * 1024)),
            "Use Git LFS for archives".to_string(),
        )
    } else if path_lower.contains(".pdf")
        || path_lower.contains(".doc")
        || path_lower.contains(".xls")
    {
        (
            "Document Files".to_string(),
            "Medium".to_string(),
            format!("Large document file ({} MB)", size / (1024 * 1024)),
            "Consider Git LFS for documents".to_string(),
        )
    } else {
        (
            "Large Files".to_string(),
            "Medium".to_string(),
            format!("Large file ({} MB)", size / (1024 * 1024)),
            "Consider Git LFS for large files".to_string(),
        )
    }
}

fn analyze_gitattributes_candidates() -> Result<Vec<HygieneIssue>, Box<dyn std::error::Error>> {
    println!("⚙️ Analyzing .gitattributes candidates...");

    let ls_files_output = Command::new("git")
        .args(&["ls-files", "--stage"])
        .output()?;

    let ls_files_str = String::from_utf8(ls_files_output.stdout)?;
    let mut issues = Vec::new();

    for line in ls_files_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let path = parts[3..].join(" ");
            let path_lower = path.to_lowercase();

            if should_have_gitattributes(&path_lower) {
                let (issue_type, severity, description, recommendation) =
                    analyze_gitattributes_candidate(&path_lower);

                issues.push(HygieneIssue {
                    file_path: path,
                    issue_type,
                    severity,
                    description,
                    recommendation,
                    potential_savings: None,
                });
            }
        }
    }

    println!("⚙️ Found {} files needing .gitattributes", issues.len());
    Ok(issues)
}

fn should_have_gitattributes(path: &str) -> bool {
    let gitattributes_patterns = [
        "*.zip", "*.tar.gz", "*.rar", "*.7z", "*.exe", "*.dll", "*.so", "*.dylib", "*.bin",
        "*.dat", "*.db", "*.sqlite", "*.pdf", "*.doc", "*.docx", "*.xls", "*.xlsx", "*.psd",
        "*.ai", "*.eps", "*.mp3", "*.mp4", "*.avi", "*.mov",
    ];

    for pattern in &gitattributes_patterns {
        if pattern.starts_with('*') {
            let ext = &pattern[1..];
            if path.ends_with(ext) {
                return true;
            }
        }
    }

    false
}

fn analyze_gitattributes_candidate(path: &str) -> (String, String, String, String) {
    let path_lower = path.to_lowercase();

    if path_lower.ends_with(".zip") || path_lower.ends_with(".tar.gz") {
        (
            "Archive Files".to_string(),
            "Medium".to_string(),
            "Archive files don't delta well".to_string(),
            "Add -delta attribute".to_string(),
        )
    } else if path_lower.ends_with(".exe")
        || path_lower.ends_with(".dll")
        || path_lower.ends_with(".so")
    {
        (
            "Binary Files".to_string(),
            "Medium".to_string(),
            "Binary files don't delta well".to_string(),
            "Add -delta attribute".to_string(),
        )
    } else if path_lower.ends_with(".pdf") || path_lower.ends_with(".doc") {
        (
            "Document Files".to_string(),
            "Low".to_string(),
            "Document files may not delta well".to_string(),
            "Consider -delta attribute".to_string(),
        )
    } else {
        (
            "Binary-like Files".to_string(),
            "Low".to_string(),
            "File may not benefit from delta compression".to_string(),
            "Consider -delta attribute".to_string(),
        )
    }
}

fn generate_gitignore_suggestions(issues: &[HygieneIssue]) -> Vec<String> {
    let mut suggestions = Vec::new();
    let mut patterns = std::collections::HashSet::new();

    for issue in issues {
        if issue.issue_type == "Build Artifacts" {
            patterns.insert("target/".to_string());
            patterns.insert("build/".to_string());
            patterns.insert("dist/".to_string());
        } else if issue.issue_type == "Log Files" {
            patterns.insert("*.log".to_string());
            patterns.insert("logs/".to_string());
        } else if issue.issue_type == "Cache Files" {
            patterns.insert(".cache/".to_string());
            patterns.insert("cache/".to_string());
        } else if issue.issue_type == "Lock Files" {
            patterns.insert("*.lock".to_string());
        }
    }

    for pattern in patterns {
        suggestions.push(format!("echo \"{}\" >> .gitignore", pattern));
    }

    suggestions
}

fn generate_gitattributes_suggestions(issues: &[HygieneIssue]) -> Vec<String> {
    let mut suggestions = Vec::new();
    let mut patterns = std::collections::HashSet::new();

    for issue in issues {
        if issue.issue_type == "Archive Files" {
            patterns.insert("*.zip -delta".to_string());
            patterns.insert("*.tar.gz -delta".to_string());
        } else if issue.issue_type == "Binary Files" {
            patterns.insert("*.exe -delta".to_string());
            patterns.insert("*.dll -delta".to_string());
            patterns.insert("*.so -delta".to_string());
        }
    }

    for pattern in patterns {
        suggestions.push(format!("echo \"{}\" >> .gitattributes", pattern));
    }

    suggestions
}

fn generate_lfs_suggestions(issues: &[HygieneIssue]) -> Vec<String> {
    let mut suggestions = Vec::new();
    let mut patterns = std::collections::HashSet::new();

    for issue in issues {
        if issue.issue_type == "Binary Files" {
            patterns.insert("*.exe".to_string());
            patterns.insert("*.dll".to_string());
            patterns.insert("*.so".to_string());
        } else if issue.issue_type == "Archive Files" {
            patterns.insert("*.zip".to_string());
            patterns.insert("*.tar.gz".to_string());
        }
    }

    for pattern in patterns {
        suggestions.push(format!("git lfs track \"{}\"", pattern));
    }

    suggestions
}

fn generate_hygiene_report() -> Result<HygieneReport, Box<dyn std::error::Error>> {
    println!("🧹 Git Hygiene Reporter");
    println!("======================");
    println!("Analyzing repository hygiene...");
    println!();

    // Analyze different categories
    let ignore_issues = analyze_frequent_write_files()?;
    let lfs_issues = analyze_large_files()?;
    let gitattributes_issues = analyze_gitattributes_candidates()?;

    // Create categories
    let mut categories = Vec::new();

    if !ignore_issues.is_empty() {
        let total_savings = 0; // Will calculate if needed
        categories.push(HygieneCategory {
            name: "Files to Ignore".to_string(),
            issues: ignore_issues.clone(),
            total_savings,
            priority: "High".to_string(),
        });
    }

    if !lfs_issues.is_empty() {
        let total_savings: u64 = lfs_issues.iter().filter_map(|i| i.potential_savings).sum();
        categories.push(HygieneCategory {
            name: "Large Files for LFS".to_string(),
            issues: lfs_issues.clone(),
            total_savings,
            priority: "Medium".to_string(),
        });
    }

    if !gitattributes_issues.is_empty() {
        categories.push(HygieneCategory {
            name: "Git Attributes".to_string(),
            issues: gitattributes_issues.clone(),
            total_savings: 0,
            priority: "Low".to_string(),
        });
    }

    // Generate suggestions
    let gitignore_suggestions = generate_gitignore_suggestions(&ignore_issues);
    let gitattributes_suggestions = generate_gitattributes_suggestions(&gitattributes_issues);
    let lfs_suggestions = generate_lfs_suggestions(&lfs_issues);

    // Generate optimization commands
    let mut optimization_commands = Vec::new();
    optimization_commands.push("git repack -Ad --window=250 --depth=50".to_string());
    optimization_commands.push("git gc --prune=now".to_string());

    if !lfs_issues.is_empty() {
        optimization_commands.push("git lfs install".to_string());
    }

    let total_issues = categories.iter().map(|c| c.issues.len()).sum();
    let total_potential_savings = categories.iter().map(|c| c.total_savings).sum();

    // Generate report
    println!("\n🧹 Git Hygiene Report");
    println!("====================");

    for category in &categories {
        println!(
            "\n📋 {} ({} issues, {} priority):",
            category.name,
            category.issues.len(),
            category.priority
        );

        for issue in &category.issues {
            println!("  • {} - {}", issue.file_path, issue.description);
            println!("    Recommendation: {}", issue.recommendation);

            if let Some(savings) = issue.potential_savings {
                println!("    Potential savings: {:.1} KB", savings as f64 / 1024.0);
            }
        }
    }

    // Show suggestions
    if !gitignore_suggestions.is_empty() {
        println!("\n📝 .gitignore Suggestions:");
        for suggestion in &gitignore_suggestions {
            println!("  {}", suggestion);
        }
    }

    if !gitattributes_suggestions.is_empty() {
        println!("\n⚙️ .gitattributes Suggestions:");
        for suggestion in &gitattributes_suggestions {
            println!("  {}", suggestion);
        }
    }

    if !lfs_suggestions.is_empty() {
        println!("\n📦 Git LFS Suggestions:");
        for suggestion in &lfs_suggestions {
            println!("  {}", suggestion);
        }
    }

    if !optimization_commands.is_empty() {
        println!("\n🔧 Optimization Commands:");
        for command in &optimization_commands {
            println!("  {}", command);
        }
    }

    // Summary
    println!("\n📈 Summary:");
    println!("  • Total issues: {}", total_issues);
    println!("  • Categories: {}", categories.len());
    println!(
        "  • Potential savings: {:.2} MB",
        total_potential_savings as f64 / (1024.0 * 1024.0)
    );

    Ok(HygieneReport {
        categories,
        total_issues,
        total_potential_savings,
        gitignore_suggestions,
        gitattributes_suggestions,
        lfs_suggestions,
        optimization_commands,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _report = generate_hygiene_report()?;

    println!("\n✅ Hygiene analysis complete!");
    println!("🧹 Repository hygiene assessed");
    println!("📝 Ignore patterns identified");
    println!("📦 LFS candidates found");
    println!("⚙️ Git attributes suggested");
    println!("🔧 Optimization commands ready");

    Ok(())
}
