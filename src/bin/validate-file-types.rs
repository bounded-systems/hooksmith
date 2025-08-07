use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Clone)]
struct FileTypePolicy {
    allowed_extensions: HashSet<String>,
    blocked_extensions: HashSet<String>,
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

        Self {
            allowed_extensions: allowed,
            blocked_extensions: blocked,
        }
    }

    fn is_allowed(&self, extension: &str) -> bool {
        self.allowed_extensions.contains(extension)
    }

    fn is_blocked(&self, extension: &str) -> bool {
        self.blocked_extensions.contains(extension)
    }
}

#[derive(Debug)]
struct ValidationResult {
    allowed_files: Vec<PathBuf>,
    blocked_files: Vec<PathBuf>,
    other_files: Vec<PathBuf>,
    no_extension_files: Vec<PathBuf>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            allowed_files: Vec::new(),
            blocked_files: Vec::new(),
            other_files: Vec::new(),
            no_extension_files: Vec::new(),
        }
    }

    fn has_violations(&self) -> bool {
        !self.blocked_files.is_empty()
    }
}

fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
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
                } else {
                    result.other_files.push(file.clone());
                }
            }
            None => {
                result.no_extension_files.push(file.clone());
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
    println!("\n📊 File Type Validation Results ({context}):");
    println!();

    if !result.allowed_files.is_empty() {
        println!("✅ Allowed files ({}):", result.allowed_files.len());
        for file in &result.allowed_files {
            println!("   {}", file.display());
        }
        println!();
    }

    if !result.blocked_files.is_empty() {
        println!("❌ BLOCKED files ({}):", result.blocked_files.len());
        for file in &result.blocked_files {
            println!("   {}", file.display());
        }
        println!();
    }

    if !result.other_files.is_empty() {
        println!("⚠️  Other files ({}):", result.other_files.len());
        for file in &result.other_files {
            if let Some(ext) = get_file_extension(file) {
                println!("   {} ({})", file.display(), ext);
            } else {
                println!("   {}", file.display());
            }
        }
        println!();
    }

    if !result.no_extension_files.is_empty() {
        println!("⚠️  Files with no extension ({}):", result.no_extension_files.len());
        for file in &result.no_extension_files {
            println!("   {}", file.display());
        }
        println!();
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
