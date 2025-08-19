use std::collections::HashSet;
use std::process::Command;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use globset::{Glob, GlobSet, GlobSetBuilder};

#[derive(Debug, Deserialize)]
struct TreeConfig {
    version: u32,
    mode: String,
    default: String,
    precedence: String,
    allow_dirs: Vec<String>,
    allow_files: Vec<String>,
    assertions: Option<Assertions>,
}

#[derive(Debug, Deserialize)]
struct Assertions {
    must_exist: Option<Vec<String>>,
    unique_files: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    violations: Vec<String>,
    missing_required: Vec<String>,
    duplicate_files: Vec<String>,
}

struct TreeDirChecker {
    config: TreeConfig,
    allow_file_patterns: GlobSet,
}

impl TreeDirChecker {
    fn new(config: TreeConfig) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in &config.allow_files {
            let glob = Glob::new(pattern)
                .context(format!("Invalid glob pattern: {}", pattern))?;
            builder.add(glob);
        }
        let allow_file_patterns = builder.build()
            .context("Failed to build glob set")?;
        
        Ok(Self {
            config,
            allow_file_patterns,
        })
    }

    fn get_root_entries(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["ls-tree", "--name-only", "HEAD"])
            .output()
            .context("Failed to get root tree entries")?;

        let entries: Vec<String> = String::from_utf8(output.stdout)?
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(entries)
    }

    fn is_directory(&self, entry: &str) -> Result<bool> {
        let output = Command::new("git")
            .args(["ls-tree", "-t", "HEAD", entry])
            .output()
            .context(format!("Failed to check type of entry: {}", entry))?;

        let output_str = String::from_utf8(output.stdout)?;
        // Tree entries start with "tree", blob entries start with "blob"
        Ok(output_str.trim().starts_with("tree"))
    }

    fn validate_entry(&self, entry: &str) -> Result<bool> {
        // Tree mode guarantee: no slashes allowed
        if entry.contains('/') {
            return Ok(false);
        }

        let is_dir = self.is_directory(entry)?;

        if is_dir {
            // Check if directory is in allow list
            Ok(self.config.allow_dirs.contains(&entry.to_string()))
        } else {
            // Check if file matches any allowed pattern
            Ok(self.allow_file_patterns.is_match(entry))
        }
    }

    fn check_assertions(&self, entries: &[String]) -> Result<(Vec<String>, Vec<String>)> {
        let mut missing_required = Vec::new();
        let mut duplicate_files = Vec::new();
        let entry_set: HashSet<&String> = entries.iter().collect();

        if let Some(assertions) = &self.config.assertions {
            // Check must_exist
            if let Some(required) = &assertions.must_exist {
                for req in required {
                    if !entry_set.contains(req) {
                        missing_required.push(req.clone());
                    }
                }
            }

            // Check unique_files
            if let Some(unique) = &assertions.unique_files {
                for file in unique {
                    let count = entries.iter().filter(|e| *e == file).count();
                    if count > 1 {
                        duplicate_files.push(file.clone());
                    }
                }
            }
        }

        Ok((missing_required, duplicate_files))
    }

    fn validate(&self) -> Result<ValidationResult> {
        let entries = self.get_root_entries()?;
        let mut violations = Vec::new();

        // Validate each entry
        for entry in &entries {
            if !self.validate_entry(entry)? {
                violations.push(format!("Unexpected entry at root: {}", entry));
            }
        }

        // Check assertions
        let (missing_required, duplicate_files) = self.check_assertions(&entries)?;

        let valid = violations.is_empty() && missing_required.is_empty() && duplicate_files.is_empty();

        Ok(ValidationResult {
            valid,
            violations,
            missing_required,
            duplicate_files,
        })
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let config_path = args.get(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: tree_dircheck <config.yml>"))?;

    let config_content = std::fs::read_to_string(config_path)
        .context(format!("Failed to read config file: {}", config_path))?;

    let config: TreeConfig = serde_yaml::from_str(&config_content)
        .context("Failed to parse config YAML")?;

    if config.mode != "tree" {
        anyhow::bail!("Config mode must be 'tree'");
    }

    let checker = TreeDirChecker::new(config)?;
    let result = checker.validate()?;

    // Output results
    if result.valid {
        println!("✅ Tree validation passed!");
    } else {
        println!("❌ Tree validation failed!");
        
        if !result.violations.is_empty() {
            println!("\n🚫 Violations:");
            for violation in &result.violations {
                println!("   {}", violation);
            }
        }

        if !result.missing_required.is_empty() {
            println!("\n❌ Missing required files:");
            for missing in &result.missing_required {
                println!("   {}", missing);
            }
        }

        if !result.duplicate_files.is_empty() {
            println!("\n⚠️  Duplicate files:");
            for duplicate in &result.duplicate_files {
                println!("   {}", duplicate);
            }
        }

        std::process::exit(1);
    }

    Ok(())
}
