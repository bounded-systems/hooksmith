use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::process::Command;

const REGISTRY_FILE: &str = "config/generated-files.jsonc";

#[derive(Debug)]
pub struct RegistryManager {
    registry_path: String,
    registry_data: Value,
}

impl RegistryManager {
    pub fn new() -> Result<Self> {
        let registry_path = REGISTRY_FILE.to_string();
        let content = fs::read_to_string(&registry_path)
            .with_context(|| format!("Failed to read registry file: {}", registry_path))?;

        let registry_data: Value =
            serde_json::from_str(&content).with_context(|| "Failed to parse registry JSON")?;

        Ok(Self {
            registry_path,
            registry_data,
        })
    }

    pub fn status(&self) -> Result<()> {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                REGISTRY STATUS                              ║");
        println!("╚══════════════════════════════════════════════════════════════╝");

        let files = self.registry_data["files"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid registry format: files array not found"))?;

        println!("\n📊 Registry Statistics:");
        println!("  • Total entries: {}", files.len());

        let (valid, invalid, missing, ignored) = self.validate_all()?;

        println!("  • Valid files: {}", valid);
        println!("  • Invalid checksums: {}", invalid);
        println!("  • Missing files: {}", missing);
        println!("  • Ignored files: {}", ignored);

        if invalid == 0 && missing == 0 && ignored == 0 {
            println!("\n✅ Registry is VALID and COMPLETE");
            Ok(())
        } else {
            println!("\n❌ Registry has ISSUES that need attention");
            Err(anyhow!("Registry validation failed"))
        }
    }

    pub fn validate(&self) -> Result<()> {
        println!("Validating generated files registry...");

        let (valid, invalid, missing, ignored) = self.validate_all()?;

        println!("\n=== VALIDATION SUMMARY ===");
        println!("Valid files: {}", valid);
        println!("Invalid checksums: {}", invalid);
        println!("Missing files: {}", missing);
        println!("Ignored files: {}", ignored);

        if invalid == 0 && missing == 0 && ignored == 0 {
            println!("\n✅ Registry is VALID and COMPLETE");
            Ok(())
        } else {
            println!("\n❌ Registry has ISSUES that need attention");
            Err(anyhow!("Registry validation failed"))
        }
    }

    pub fn update_checksums(&mut self) -> Result<()> {
        println!("Updating all checksums in registry...");

        let mut updated = 0;
        let mut unchanged = 0;
        let mut updates = Vec::new();

        // First pass: collect all updates
        {
            let files = self.registry_data["files"]
                .as_array()
                .ok_or_else(|| anyhow!("Invalid registry format"))?;

            for (i, file_entry) in files.iter().enumerate() {
                let path = file_entry["path"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Invalid file entry: missing path"))?;

                if Path::new(path).exists() {
                    let current_checksum = self.generate_checksum(path)?;
                    let stored_checksum = file_entry["checksum"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Invalid file entry: missing checksum"))?;

                    if current_checksum != stored_checksum {
                        updates.push((i, current_checksum.clone()));
                        println!(
                            "  Updated: {} ({} → {})",
                            path, stored_checksum, current_checksum
                        );
                        updated += 1;
                    } else {
                        unchanged += 1;
                    }
                }
            }
        }

        // Second pass: apply updates
        {
            let files = self.registry_data["files"]
                .as_array_mut()
                .ok_or_else(|| anyhow!("Invalid registry format"))?;

            for (index, checksum) in updates {
                files[index]["checksum"] = json!(checksum);
            }
        }

        self.save_registry()?;

        println!("\n=== UPDATE SUMMARY ===");
        println!("Files updated: {}", updated);
        println!("Files unchanged: {}", unchanged);

        Ok(())
    }

    pub fn fix(&mut self) -> Result<()> {
        println!("Fixing registry issues...");

        // First update all checksums
        self.update_checksums()?;

        // Then clean up ignored files
        self.cleanup_ignored_files()?;

        // Add missing files
        self.add_missing_files()?;

        println!("✅ Registry fix complete!");
        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        println!("Cleaning up registry...");
        self.cleanup_ignored_files()?;
        println!("✅ Registry cleanup complete!");
        Ok(())
    }

    fn validate_all(&self) -> Result<(usize, usize, usize, usize)> {
        let files = self.registry_data["files"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid registry format"))?;

        let mut valid = 0;
        let mut invalid = 0;
        let mut missing = 0;
        let mut ignored = 0;

        for file_entry in files {
            let path = file_entry["path"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid file entry"))?;

            if Path::new(path).exists() {
                if self.is_ignored_by_git(path)? {
                    ignored += 1;
                } else {
                    let current_checksum = self.generate_checksum(path)?;
                    let stored_checksum = file_entry["checksum"]
                        .as_str()
                        .ok_or_else(|| anyhow!("Invalid file entry"))?;

                    if current_checksum == stored_checksum {
                        valid += 1;
                    } else {
                        invalid += 1;
                    }
                }
            } else {
                missing += 1;
            }
        }

        Ok((valid, invalid, missing, ignored))
    }

    fn cleanup_ignored_files(&mut self) -> Result<()> {
        let mut to_remove = Vec::new();

        // First pass: collect indices to remove
        {
            let files = self.registry_data["files"]
                .as_array()
                .ok_or_else(|| anyhow!("Invalid registry format"))?;

            for (i, file_entry) in files.iter().enumerate() {
                let path = file_entry["path"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Invalid file entry"))?;

                let should_remove = !Path::new(path).exists() || self.is_ignored_by_git(path)?;
                if should_remove {
                    to_remove.push(i);
                    println!("  Removing: {}", path);
                }
            }
        }

        // Second pass: remove items
        {
            let files = self.registry_data["files"]
                .as_array_mut()
                .ok_or_else(|| anyhow!("Invalid registry format"))?;

            // Remove in reverse order to maintain indices
            for &index in to_remove.iter().rev() {
                files.remove(index);
            }
        }

        self.save_registry()?;
        Ok(())
    }

    fn add_missing_files(&mut self) -> Result<()> {
        let git_files = self.get_git_tracked_files()?;
        let empty_vec = Vec::new();
        let registry_files: std::collections::HashSet<_> = self.registry_data["files"]
            .as_array()
            .unwrap_or(&empty_vec)
            .iter()
            .filter_map(|f| f["path"].as_str())
            .collect();

        let missing_files: Vec<_> = git_files
            .into_iter()
            .filter(|f| !registry_files.contains(f.as_str()))
            .collect();

        if !missing_files.is_empty() {
            println!("Adding missing files to registry...");
            for file in missing_files {
                self.add_file_to_registry(&file)?;
                println!("  Added: {}", file);
            }
            self.save_registry()?;
        }

        Ok(())
    }

    fn add_file_to_registry(&mut self, path: &str) -> Result<()> {
        let checksum = self.generate_checksum(path)?;
        let slug = self.generate_slug(path);
        let file_type = self.get_file_type(path);

        let new_entry = json!({
            "path": path,
            "checksum": checksum,
            "slug": slug,
            "type": file_type
        });

        let files = self.registry_data["files"]
            .as_array_mut()
            .ok_or_else(|| anyhow!("Invalid registry format"))?;
        files.push(new_entry);

        Ok(())
    }

    fn generate_checksum(&self, path: &str) -> Result<String> {
        let output = Command::new("sha256sum")
            .arg(path)
            .output()
            .with_context(|| format!("Failed to generate checksum for {}", path))?;

        let checksum = String::from_utf8_lossy(&output.stdout);
        Ok(checksum
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("Invalid checksum output"))?
            .chars()
            .take(8)
            .collect())
    }

    fn generate_slug(&self, path: &str) -> String {
        path.replace(|c: char| !c.is_alphanumeric(), "_")
            .replace("__", "_")
            .trim_matches('_')
            .to_string()
    }

    fn get_file_type(&self, path: &str) -> String {
        let extension = Path::new(path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "md" => "md".to_string(),
            "toml" => "toml".to_string(),
            "sh" => "sh".to_string(),
            "json" => "json".to_string(),
            "wit" => "wit".to_string(),
            "hbs" => "hbs".to_string(),
            "css" => "css".to_string(),
            "sed" => "sed".to_string(),
            "jsonl" => "jsonl".to_string(),
            "yml" | "yaml" => "yaml".to_string(),
            "editorconfig" => "editorconfig".to_string(),
            "envrc" => "envrc".to_string(),
            "gitignore" => "gitignore".to_string(),
            "gitattributes" => "gitattributes".to_string(),
            _ => {
                if path.ends_with("CODEOWNERS") || path.ends_with("Makefile") {
                    if path.ends_with("CODEOWNERS") {
                        "CODEOWNERS".to_string()
                    } else {
                        "makefile".to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            }
        }
    }

    fn is_ignored_by_git(&self, path: &str) -> Result<bool> {
        // Check if file is ignored by git
        let output = Command::new("git")
            .args(["check-ignore", path])
            .output()
            .with_context(|| format!("Failed to check git ignore for {}", path))?;

        let git_ignored = output.status.success();

        // Additional patterns for temporary/artifact files
        let temp_patterns = [
            "2025",                // Timestamped files
            ".backup",             // Backup files
            ".disabled",           // Disabled files
            ".sed",                // Sed scripts
            ".jsonl",              // Log files
            ".shellcheckrc",       // ShellCheck config
            "generated_file_demo", // Demo artifacts
            "fix_format.sed",      // Development artifacts
        ];

        let matches_temp_pattern = temp_patterns.iter().any(|pattern| path.contains(pattern));

        Ok(git_ignored || matches_temp_pattern)
    }

    fn get_git_tracked_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["ls-files"])
            .output()
            .context("Failed to get git tracked files")?;

        let files = String::from_utf8_lossy(&output.stdout);
        let tracked_files: Vec<String> = files
            .lines()
            .filter(|line| {
                let path = line.trim();
                path.ends_with(".md")
                    || path.ends_with(".toml")
                    || path.ends_with(".sh")
                    || path.ends_with(".json")
                    || path.ends_with(".wit")
                    || path.ends_with(".hbs")
                    || path.ends_with(".css")
                    || path.ends_with(".sed")
                    || path.ends_with(".jsonl")
                    || path.ends_with(".yml")
                    || path.ends_with(".yaml")
                    || path.ends_with(".editorconfig")
                    || path.ends_with(".envrc")
                    || path.ends_with(".gitignore")
                    || path.ends_with(".gitattributes")
                    || path.ends_with("CODEOWNERS")
                    || path.ends_with("Makefile")
            })
            .map(|s| s.to_string())
            .collect();

        Ok(tracked_files)
    }

    fn save_registry(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.registry_data)
            .context("Failed to serialize registry")?;
        fs::write(&self.registry_path, content)
            .with_context(|| format!("Failed to write registry file: {}", self.registry_path))?;
        Ok(())
    }
}

pub fn run_registry_command(args: &[String]) -> Result<()> {
    if args.is_empty() {
        println!("Usage: cargo xtask registry <command>");
        println!("Commands:");
        println!("  status    - Show registry status");
        println!("  validate  - Validate registry integrity");
        println!("  update    - Update all checksums");
        println!("  fix       - Fix registry issues");
        println!("  cleanup   - Clean up ignored files");
        return Ok(());
    }

    let command = &args[0];
    let mut registry = RegistryManager::new()?;

    match command.as_str() {
        "status" => registry.status(),
        "validate" => registry.validate(),
        "update" => registry.update_checksums(),
        "fix" => registry.fix(),
        "cleanup" => registry.cleanup(),
        _ => {
            println!("Unknown command: {}", command);
            println!("Use 'cargo xtask registry' for help");
            Err(anyhow!("Unknown command"))
        }
    }
}
