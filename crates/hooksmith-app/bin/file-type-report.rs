use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Clone)]
struct FileTypeStats {
    count: usize,
    total_size: u64,
    examples: Vec<PathBuf>,
}

impl FileTypeStats {
    fn new() -> Self {
        Self {
            count: 0,
            total_size: 0,
            examples: Vec::new(),
        }
    }

    fn add_file(&mut self, path: &PathBuf) {
        self.count += 1;
        if self.examples.len() < 5 {
            self.examples.push(path.clone());
        }
        // Note: We could calculate file size here if needed
    }
}

fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
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

fn analyze_file_types(files: &[PathBuf]) -> HashMap<String, FileTypeStats> {
    let mut stats = HashMap::new();

    for file in files {
        if !file.is_file() {
            continue; // Skip directories
        }

        let extension = get_file_extension(file).unwrap_or_else(|| "no-extension".to_string());

        stats
            .entry(extension)
            .or_insert_with(FileTypeStats::new)
            .add_file(file);
    }

    stats
}

fn print_file_type_report(stats: &HashMap<String, FileTypeStats>) {
    println!("📊 File Type Breakdown Report");
    println!("============================");
    println!();

    // Convert to vector and sort by count (descending)
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    let total_files: usize = stats.values().map(|s| s.count).sum();

    println!("📈 Summary:");
    println!("   Total files: {}", total_files);
    println!("   Unique extensions: {}", stats.len());
    println!();

    println!("📋 File Type Distribution:");
    println!();

    for (extension, stat) in &sorted_stats {
        let percentage = (stat.count as f64 / total_files as f64) * 100.0;
        let bar_length = ((percentage / 5.0) as usize).min(20); // Max 20 chars
        let bar = "█".repeat(bar_length);

        println!(
            "{:>6} {:>4} files ({:>5.1}%) {}",
            format!(".{}", extension),
            stat.count,
            format!("{:.1}", percentage),
            bar
        );

        // Show examples for top extensions
        if stat.count > 1 && stat.examples.len() > 0 {
            println!("        Examples:");
            for example in &stat.examples[..stat.examples.len().min(3)] {
                println!("          {}", example.display());
            }
            if stat.examples.len() > 3 {
                println!("          ... and {} more", stat.count - 3);
            }
            println!();
        }
    }

    // Policy compliance analysis
    println!("🔍 Policy Compliance Analysis:");
    println!();

    let allowed_extensions = ["rs", "jsonc"];
    let blocked_extensions = [
        "sh",
        "bash",
        "zsh",
        "fish",
        "csh",
        "ksh",
        "tcsh",
        "dash",
        "ash",
        "mksh",
        "yash",
        "posh",
        "rc",
        "es",
        "nu",
        "xonsh",
        "elvish",
        "nushell",
        "powershell",
        "ps1",
        "cmd",
        "bat",
        "com",
        "exe",
        "vbs",
        "js",
        "ts",
        "py",
        "rb",
        "pl",
        "php",
        "java",
        "cs",
        "cpp",
        "c",
        "h",
        "hpp",
        "cc",
        "cxx",
        "m",
        "mm",
        "swift",
        "go",
        "dart",
        "kt",
        "scala",
        "clj",
        "hs",
        "ml",
        "fs",
        "v",
        "zig",
        "nim",
        "crystal",
        "odin",
        "jai",
        "carbon",
        "mojo",
    ];

    let mut allowed_count = 0;
    let mut blocked_count = 0;
    let mut other_count = 0;

    for (ext, stat) in &sorted_stats {
        if allowed_extensions.contains(&ext.as_str()) {
            allowed_count += stat.count;
        } else if blocked_extensions.contains(&ext.as_str()) {
            blocked_count += stat.count;
        } else {
            other_count += stat.count;
        }
    }

    let allowed_percentage = (allowed_count as f64 / total_files as f64) * 100.0;
    let blocked_percentage = (blocked_count as f64 / total_files as f64) * 100.0;
    let other_percentage = (other_count as f64 / total_files as f64) * 100.0;

    println!(
        "✅ Policy Compliant ({} files, {:.1}%):",
        allowed_count, allowed_percentage
    );
    for ext in &allowed_extensions {
        if let Some(stat) = stats.get(*ext) {
            println!("   .{}: {} files", ext, stat.count);
        }
    }
    println!();

    if blocked_count > 0 {
        println!(
            "❌ Policy Violations ({} files, {:.1}%):",
            blocked_count, blocked_percentage
        );
        for (ext, stat) in &sorted_stats {
            if blocked_extensions.contains(&ext.as_str()) && stat.count > 0 {
                println!("   .{}: {} files", ext, stat.count);
            }
        }
        println!();
    }

    println!(
        "⚠️  Other Files ({} files, {:.1}%):",
        other_count, other_percentage
    );
    let mut other_extensions: Vec<_> = sorted_stats
        .iter()
        .filter(|(ext, _)| {
            !allowed_extensions.contains(&ext.as_str())
                && !blocked_extensions.contains(&ext.as_str())
        })
        .collect();
    other_extensions.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    for (ext, stat) in other_extensions.iter().take(10) {
        println!("   .{}: {} files", ext, stat.count);
    }
    if other_extensions.len() > 10 {
        println!("   ... and {} more extensions", other_extensions.len() - 10);
    }
    println!();

    // Recommendations
    println!("💡 Recommendations:");
    if blocked_count > 0 {
        println!(
            "   • Remove {} blocked files to achieve 100% policy compliance",
            blocked_count
        );
    } else {
        println!("   • ✅ Repository is 100% policy compliant!");
    }

    if allowed_percentage < 50.0 {
        println!("   • Consider converting more files to .rs or .jsonc");
    }

    if other_percentage > 50.0 {
        println!("   • Review other file types for potential conversion");
    }
    println!();
}

fn print_detailed_analysis(stats: &HashMap<String, FileTypeStats>) {
    println!("🔬 Detailed Analysis:");
    println!();

    // Top 10 extensions
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.count.cmp(&a.1.count));

    println!("📈 Top 10 File Extensions:");
    for (i, (ext, stat)) in sorted_stats.iter().take(10).enumerate() {
        println!("{:2}. .{:15} {} files", i + 1, ext, stat.count);
    }
    println!();

    // Extension categories
    let mut categories = HashMap::new();
    categories.insert(
        "Configuration",
        vec!["toml", "yml", "yaml", "json", "jsonc", "ini", "cfg", "conf"],
    );
    categories.insert("Documentation", vec!["md", "txt", "rst", "adoc", "tex"]);
    categories.insert("Build/CI", vec!["yml", "yaml", "toml", "lock", "cargo"]);
    categories.insert(
        "Source Code",
        vec!["rs", "js", "ts", "py", "rb", "java", "cpp", "c", "go"],
    );
    categories.insert(
        "Shell/Scripts",
        vec!["sh", "bash", "zsh", "fish", "ps1", "cmd", "bat"],
    );

    println!("📂 File Categories:");
    for (category, extensions) in &categories {
        let mut category_count = 0;
        let mut category_files = Vec::new();

        for ext in extensions {
            if let Some(stat) = stats.get(*ext) {
                category_count += stat.count;
                category_files.extend(stat.examples.clone());
            }
        }

        if category_count > 0 {
            println!("   {}: {} files", category, category_count);
            for file in category_files.iter().take(3) {
                println!("     {}", file.display());
            }
            if category_files.len() > 3 {
                println!("     ... and {} more", category_count - 3);
            }
            println!();
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let detailed = args.iter().any(|arg| arg == "--detailed" || arg == "-d");
    let help = args.iter().any(|arg| arg == "--help" || arg == "-h");

    if help {
        println!("file-type-report - Generate comprehensive file type breakdown");
        println!();
        println!("Usage:");
        println!("  {} [options]", args[0]);
        println!();
        println!("Options:");
        println!("  -d, --detailed    Show detailed analysis");
        println!("  -h, --help        Show help");
        return ExitCode::SUCCESS;
    }

    // Check if we're in a git repository
    if Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .is_err()
    {
        eprintln!("❌ Error: Not in a git repository");
        return ExitCode::FAILURE;
    }

    // Get all tracked files
    let files = match get_git_files() {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ Error getting git files: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Analyze file types
    let stats = analyze_file_types(&files);

    // Print report
    print_file_type_report(&stats);

    if detailed {
        print_detailed_analysis(&stats);
    }

    ExitCode::SUCCESS
}
