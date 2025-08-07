use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

#[derive(Debug, Clone)]
struct FileTypePolicy {
    allowed_extensions: HashSet<String>,
    blocked_extensions: HashSet<String>,
    linguist_generated_extensions: HashSet<String>,
}

impl FileTypePolicy {
    fn new() -> Self {
        let mut allowed = HashSet::new();
        allowed.insert("rs".to_string());
        allowed.insert("jsonc".to_string());

        let mut blocked = HashSet::new();
        // Shell files
        blocked.insert("sh".to_string());
        blocked.insert("bash".to_string());
        blocked.insert("zsh".to_string());
        blocked.insert("fish".to_string());
        blocked.insert("csh".to_string());
        blocked.insert("ksh".to_string());
        blocked.insert("tcsh".to_string());
        blocked.insert("dash".to_string());
        blocked.insert("ash".to_string());
        blocked.insert("mksh".to_string());
        blocked.insert("yash".to_string());
        blocked.insert("posh".to_string());
        blocked.insert("rc".to_string());
        blocked.insert("es".to_string());
        blocked.insert("nu".to_string());
        blocked.insert("xonsh".to_string());
        blocked.insert("elvish".to_string());
        blocked.insert("nushell".to_string());
        blocked.insert("powershell".to_string());
        blocked.insert("ps1".to_string());
        blocked.insert("cmd".to_string());
        blocked.insert("bat".to_string());
        blocked.insert("com".to_string());
        blocked.insert("exe".to_string());
        blocked.insert("vbs".to_string());
        
        // Other programming languages (not allowed in this policy)
        blocked.insert("js".to_string());
        blocked.insert("ts".to_string());
        blocked.insert("py".to_string());
        blocked.insert("rb".to_string());
        blocked.insert("pl".to_string());
        blocked.insert("php".to_string());
        blocked.insert("java".to_string());
        blocked.insert("cs".to_string());
        blocked.insert("cpp".to_string());
        blocked.insert("c".to_string());
        blocked.insert("h".to_string());
        blocked.insert("hpp".to_string());
        blocked.insert("cc".to_string());
        blocked.insert("cxx".to_string());
        blocked.insert("m".to_string());
        blocked.insert("mm".to_string());
        blocked.insert("swift".to_string());
        blocked.insert("go".to_string());
        blocked.insert("dart".to_string());
        blocked.insert("kt".to_string());
        blocked.insert("scala".to_string());
        blocked.insert("clj".to_string());
        blocked.insert("hs".to_string());
        blocked.insert("ml".to_string());
        blocked.insert("fs".to_string());
        blocked.insert("v".to_string());
        blocked.insert("zig".to_string());
        blocked.insert("nim".to_string());
        blocked.insert("crystal".to_string());
        blocked.insert("odin".to_string());
        blocked.insert("jai".to_string());
        blocked.insert("carbon".to_string());
        blocked.insert("mojo".to_string());

        let mut linguist_generated = HashSet::new();
        linguist_generated.insert("yaml".to_string());
        linguist_generated.insert("txt".to_string());
        linguist_generated.insert("workbloom".to_string());
        linguist_generated.insert("LICENSE".to_string());

        Self {
            allowed_extensions: allowed,
            blocked_extensions: blocked,
            linguist_generated_extensions: linguist_generated,
        }
    }

    fn is_allowed(&self, extension: &str) -> bool {
        self.allowed_extensions.contains(extension)
    }

    fn is_blocked(&self, extension: &str) -> bool {
        self.blocked_extensions.contains(extension)
    }

    fn is_linguist_generated(&self, extension: &str) -> bool {
        self.linguist_generated_extensions.contains(extension)
    }
}

#[derive(Debug)]
struct ValidationResult {
    allowed_files: Vec<PathBuf>,
    blocked_files: Vec<PathBuf>,
    linguist_generated_files: Vec<PathBuf>,
    other_files: Vec<PathBuf>,
    no_extension_files: Vec<PathBuf>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            allowed_files: Vec::new(),
            blocked_files: Vec::new(),
            linguist_generated_files: Vec::new(),
            other_files: Vec::new(),
            no_extension_files: Vec::new(),
        }
    }

    fn has_violations(&self) -> bool {
        !self.blocked_files.is_empty() || !self.no_extension_files.is_empty()
    }
}

fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

fn detect_file_type_with_hyperpolyglot(path: &Path) -> Option<String> {
    // Try to run hyperpolyglot to detect the file type
    let output = Command::new("hyply")
        .args(&["--breakdown", path.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let output_str = String::from_utf8_lossy(&output.stdout);

            // Parse hyperpolyglot output to extract the primary language
            for line in output_str.lines() {
                if line.contains('%') {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let language = parts[1].to_lowercase();
                        return Some(language);
                    }
                }
            }
            None
        }
        _ => None
    }
}

fn validate_files(files: &[PathBuf], policy: &FileTypePolicy) -> ValidationResult {
    let mut result = ValidationResult::new();

    for file in files {
        if !file.is_file() {
            continue; // Skip directories
        }

        match get_file_extension(file) {
            Some(ext) => {
                if policy.is_allowed(&ext) {
                    result.allowed_files.push(file.clone());
                } else if policy.is_blocked(&ext) {
                    result.blocked_files.push(file.clone());
                } else if policy.is_linguist_generated(&ext) {
                    result.linguist_generated_files.push(file.clone());
                } else {
                    result.other_files.push(file.clone());
                }
            }
            None => {
                // Try to detect file type using hyperpolyglot for files without extensions
                if let Some(detected_type) = detect_file_type_with_hyperpolyglot(file) {
                    // Map detected types to our policy categories
                    match detected_type.as_str() {
                        "rust" => result.allowed_files.push(file.clone()),
                        "shell" | "bash" | "zsh" => result.blocked_files.push(file.clone()),
                        "yaml" | "markdown" | "json" => result.linguist_generated_files.push(file.clone()),
                        _ => result.other_files.push(file.clone()),
                    }
                } else {
                    // If hyperpolyglot can't detect, check for known file patterns
                    let file_name = file.file_name().unwrap_or_default().to_str().unwrap_or("");
                    match file_name {
                        ".gitattributes" | ".gitignore" => result.linguist_generated_files.push(file.clone()),
                        "CODEOWNERS" => result.linguist_generated_files.push(file.clone()),
                        ".editorconfig" => result.linguist_generated_files.push(file.clone()),
                        ".workbloom" => result.linguist_generated_files.push(file.clone()),
                        "LICENSE" => result.linguist_generated_files.push(file.clone()),
                        "generate-gitattributes" => result.allowed_files.push(file.clone()), // Rust script
                        _ => result.no_extension_files.push(file.clone()),
                    }
                }
            }
        }
    }

    result
}

fn get_git_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-files", "--cached", "--full-name"])
        .output()?;

    if !output.status.success() {
        return Err("Failed to get git files".into());
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| PathBuf::from(s))
        .collect();

    Ok(files)
}

fn get_staged_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["diff", "--cached", "--name-only"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new()); // No staged files
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(s))
        .collect();

    Ok(files)
}

fn get_pr_changed_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let base_ref = env::var("GITHUB_BASE_REF").unwrap_or_else(|_| "main".to_string());
    let output = Command::new("git")
        .args(&["diff", "--name-only", &format!("origin/{}...HEAD", base_ref)])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new()); // No changed files
    }

    let files = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| PathBuf::from(s))
        .collect();

    Ok(files)
}

fn print_results(result: &ValidationResult, context: &str) {
    println!("🔍 File Type Policy Validation");
    println!("=============================");
    println!();
    println!("Policy:");
    println!("  ✅ ALLOWED: .rs, .jsonc files");
    println!("  ❌ BLOCKED: Shell files and other programming languages");
    println!("  ⚠️  WARNED: Other file types");
    println!();

    // Only show violations and warnings
    let mut has_violations = false;
    let mut has_warnings = false;

    if !result.blocked_files.is_empty() {
        has_violations = true;
        println!("❌ BLOCKED files ({}):", result.blocked_files.len());
        for file in &result.blocked_files {
            println!("   {}", file.display());
        }
        println!();
    }

    if !result.no_extension_files.is_empty() {
        has_warnings = true;
        println!("⚠️  Files with no extension ({}):", result.no_extension_files.len());
        for file in &result.no_extension_files {
            println!("   {}", file.display());
        }
        println!();
    }

    // Show summary of other categories without listing all files
    if !result.allowed_files.is_empty() {
        println!("✅ Allowed files: {} files", result.allowed_files.len());
    }

    if !result.linguist_generated_files.is_empty() {
        println!("🔧 Linguist-generated files: {} files", result.linguist_generated_files.len());
    }

    if !result.other_files.is_empty() {
        has_warnings = true;
        println!("⚠️  Other files: {} files", result.other_files.len());

        // Group by extension for better overview
        let mut extension_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for file in &result.other_files {
            if let Some(ext) = get_file_extension(file) {
                *extension_counts.entry(ext).or_insert(0) += 1;
            }
        }

        // Show top extensions
        let mut sorted_extensions: Vec<_> = extension_counts.iter().collect();
        sorted_extensions.sort_by(|a, b| b.1.cmp(a.1));

        for (ext, count) in sorted_extensions.iter().take(5) {
            println!("   {} files ({})", count, ext);
        }
        if sorted_extensions.len() > 5 {
            println!("   ... and {} more extensions", sorted_extensions.len() - 5);
        }
        println!();
    }

    // Summary
    println!("📋 Summary:");
    if has_violations {
        println!("❌ File type policy violations found!");
        println!();
        println!("💡 To fix violations:");
        println!("   • Remove blocked files");
        println!("   • Convert shell scripts to Rust");
        println!("   • Use .jsonc for configuration files");
    } else if has_warnings {
        println!("⚠️  File type policy warnings found!");
        println!();
        println!("💡 To address warnings:");
        println!("   • Add linguist-generated attributes to non-extension files");
        println!("   • Consider converting other file types to allowed formats");
    } else {
        println!("✅ All files comply with file type policy!");
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let strict_mode = args.iter().any(|arg| arg == "--strict" || arg == "-s");
    let verbose = args.iter().any(|arg| arg == "--verbose" || arg == "-v");
    let help = args.iter().any(|arg| arg == "--help" || arg == "-h");

    if help {
        println!("validate-file-types - Enforce strict file type policies");
        println!();
        println!("Usage:");
        println!("  {} [options]", args[0]);
        println!();
        println!("Options:");
        println!("  -s, --strict     Exit with error if violations found");
        println!("  -v, --verbose    Verbose output");
        println!("  -h, --help       Show help");
        println!();
        println!("File Type Policy:");
        println!("  ✅ ALLOWED: .rs, .jsonc files");
        println!("  ❌ BLOCKED: All shell files and other programming languages");
        println!("  ⚠️  WARNED: Other file types");
        return ExitCode::SUCCESS;
    }

    let policy = FileTypePolicy::new();
    let mut has_violations = false;

    println!("🔍 File Type Policy Validation");
    println!("=============================");
    println!();
    println!("Policy:");
    println!("  ✅ ALLOWED: .rs, .jsonc files");
    println!("  ❌ BLOCKED: Shell files and other programming languages");
    println!("  ⚠️  WARNED: Other file types");
    println!();

    // Check if we're in a git repository
    if Command::new("git").args(&["rev-parse", "--git-dir"]).output().is_err() {
        eprintln!("❌ Error: Not in a git repository");
        return ExitCode::FAILURE;
    }

    // Validate staged files (for pre-commit)
    if let Ok(staged_files) = get_staged_files() {
        if !staged_files.is_empty() {
            let result = validate_files(&staged_files, &policy);
            print_results(&result, "staged");
            if result.has_violations() {
                has_violations = true;
            }
        }
    }

    // Validate all tracked files
    if let Ok(all_files) = get_git_files() {
        let result = validate_files(&all_files, &policy);
        print_results(&result, "all tracked");
        if result.has_violations() {
            has_violations = true;
        }
    }

    // Validate PR changed files (for CI)
    if env::var("GITHUB_ACTIONS").is_ok() {
        if let Ok(changed_files) = get_pr_changed_files() {
            if !changed_files.is_empty() {
                let result = validate_files(&changed_files, &policy);
                print_results(&result, "PR changes");
                if result.has_violations() {
                    has_violations = true;
                }
            }
        }
    }

    // Summary
    println!("📋 Summary:");
    if has_violations {
        println!("❌ File type policy violations found!");
        println!();
        println!("💡 To fix violations:");
        println!("   • Remove blocked files");
        println!("   • Convert shell scripts to Rust");
        println!("   • Use .jsonc for configuration files");
        println!();
        if strict_mode {
            println!("🚫 Strict mode: Exiting with error");
            ExitCode::FAILURE
        } else {
            println!("⚠️  Non-strict mode: Continuing with warnings");
            ExitCode::SUCCESS
        }
    } else {
        println!("✅ No file type policy violations found!");
        println!();
        println!("🎉 All files comply with the policy");
        ExitCode::SUCCESS
    }
}
