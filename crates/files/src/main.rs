use dircheck_core::{git_query::GitQueryCommands, validate_files_index, FileRuleSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules = FileRuleSet::default();

    // Use the Git query module to get the command
    let git_cmd = GitQueryCommands::ls_files_index();
    println!(
        "🔍 Validating file structure using: {}",
        git_cmd.description
    );
    println!("📋 Git view: {:?}", git_cmd.view);

    let output = std::process::Command::new(&git_cmd.command)
        .args(&git_cmd.args)
        .output()?;

    if !output.status.success() {
        eprintln!(
            "❌ git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        std::process::exit(1);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect();

    let violations = validate_files_index(&paths, &rules);

    if violations.is_empty() {
        println!("✅ All file structure rules passed");
        Ok(())
    } else {
        eprintln!("❌ Found {} file structure violations:", violations.len());
        for violation in violations {
            eprintln!("  Rule: {}", violation.rule);
            eprintln!("  Path: {}", violation.path);
            eprintln!("  Error: {}", violation.message);
            if let Some(suggestion) = violation.suggestion {
                eprintln!("  Suggestion: {}", suggestion);
            }
            eprintln!();
        }
        std::process::exit(1);
    }
}
