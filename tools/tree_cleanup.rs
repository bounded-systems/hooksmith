use std::fs;
use std::path::Path;
use std::process::Command;
use anyhow::{Result, Context};

struct TreeCleaner {
    moved_count: usize,
}

impl TreeCleaner {
    fn new() -> Self {
        Self { moved_count: 0 }
    }

    fn mv_if(&mut self, src: &str, dst: &str) -> Result<()> {
        if Path::new(src).exists() {
            let dst_path = Path::new(dst);
            
            // Create destination directory if it doesn't exist
            if let Some(parent) = dst_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .context(format!("Failed to create directory: {}", parent.display()))?;
                }
            }
            
            // Use git mv for tracked files
            let output = Command::new("git")
                .args(["ls-files", "--error-unmatch", src])
                .output();
            
            match output {
                Ok(_) => {
                    // File is tracked by git, use git mv
                    let status = Command::new("git")
                        .args(["mv", "-k", src, dst])
                        .status()
                        .context(format!("Failed to git mv {} to {}", src, dst))?;
                    
                    if status.success() {
                        println!("   ✅ Moved: {} → {}", src, dst);
                        self.moved_count += 1;
                    } else {
                        println!("   ❌ Failed to move: {} → {}", src, dst);
                    }
                }
                Err(_) => {
                    // File is not tracked by git, use regular mv
                    fs::rename(src, dst)
                        .context(format!("Failed to move {} to {}", src, dst))?;
                    println!("   ✅ Moved (untracked): {} → {}", src, dst);
                    self.moved_count += 1;
                }
            }
        } else {
            println!("   ⚠️  Not found: {}", src);
        }
        
        Ok(())
    }

    fn rm_if(&mut self, path: &str) -> Result<()> {
        if Path::new(path).exists() {
            let output = Command::new("git")
                .args(["ls-files", "--error-unmatch", path])
                .output();
            
            match output {
                Ok(_) => {
                    // File is tracked by git, use git rm
                    let status = Command::new("git")
                        .args(["rm", "-r", path])
                        .status()
                        .context(format!("Failed to git rm {}", path))?;
                    
                    if status.success() {
                        println!("   🗑️  Removed: {}", path);
                        self.moved_count += 1;
                    } else {
                        println!("   ❌ Failed to remove: {}", path);
                    }
                }
                Err(_) => {
                    // File is not tracked by git, use regular rm
                    if Path::new(path).is_dir() {
                        fs::remove_dir_all(path)
                            .context(format!("Failed to remove directory: {}", path))?;
                    } else {
                        fs::remove_file(path)
                            .context(format!("Failed to remove file: {}", path))?;
                    }
                    println!("   🗑️  Removed (untracked): {}", path);
                    self.moved_count += 1;
                }
            }
        } else {
            println!("   ⚠️  Not found: {}", path);
        }
        
        Ok(())
    }

    fn create_directories(&self) -> Result<()> {
        let directories = vec![
            "generated",
            "reports",
            "worktree",
            ".cache",
            "tests/fixtures",
        ];

        println!("📁 Creating new directory structure...");
        for dir in directories {
            if !Path::new(dir).exists() {
                fs::create_dir_all(dir)
                    .context(format!("Failed to create directory: {}", dir))?;
                println!("   Created: {}", dir);
            }
        }
        
        Ok(())
    }

    fn cleanup_generated(&mut self) -> Result<()> {
        println!("⚙️  Normalizing generated directories...");
        
        // Move gen/ to generated/ if it exists
        if Path::new("gen").exists() {
            self.mv_if("gen", "generated")?;
        }
        
        // Move generated-sources/* to generated/ and remove the directory
        if Path::new("generated-sources").exists() {
            // Move all contents
            if let Ok(entries) = fs::read_dir("generated-sources") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let src = format!("generated-sources/{}", entry.file_name().to_string_lossy());
                        let dst = format!("generated/{}", entry.file_name().to_string_lossy());
                        self.mv_if(&src, &dst)?;
                    }
                }
            }
            // Remove the now-empty directory
            self.rm_if("generated-sources")?;
        }
        
        Ok(())
    }

    fn cleanup_reports(&mut self) -> Result<()> {
        println!("📋 Consolidating reports...");
        
        // Move summary and complete reports
        let report_patterns = vec![
            "*_SUMMARY.md",
            "*_COMPLETE*.md", 
            "README_20*.md",
        ];
        
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy();
                    if name.ends_with(".md") {
                        let src = name.to_string();
                        let dst = format!("reports/{}", src);
                        
                        // Check if it matches our patterns (excluding README.md)
                        if src != "README.md" && (
                            src.contains("_SUMMARY") ||
                            src.contains("_COMPLETE") ||
                            src.starts_with("README_20")
                        ) {
                            self.mv_if(&src, &dst)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    fn cleanup_worktree(&mut self) -> Result<()> {
        println!("🌳 Consolidating worktree artifacts...");
        
        // Move worktree-lifecycle to worktree/
        if Path::new("worktree-lifecycle").exists() {
            self.mv_if("worktree-lifecycle", "worktree/lifecycle")?;
        }
        
        // Move worktree config files
        if Path::new(".worktree-config.jsonc").exists() {
            self.mv_if(".worktree-config.jsonc", "worktree/worktree.config.jsonc")?;
        }
        
        // Remove the .json version if it exists
        if Path::new(".worktree-config.json").exists() {
            self.rm_if(".worktree-config.json")?;
        }
        
        Ok(())
    }

    fn cleanup_cache(&mut self) -> Result<()> {
        println!("🗄️  Moving cache items...");
        
        // Move .wb and .workbloom to .cache/
        if Path::new(".wb").exists() {
            self.mv_if(".wb", ".cache/wb")?;
        }
        
        if Path::new(".workbloom").exists() {
            self.mv_if(".workbloom", ".cache/workbloom")?;
        }
        
        Ok(())
    }

    fn cleanup_tests(&mut self) -> Result<()> {
        println!("🧪 Organizing test fixtures...");
        
        // Move test files to fixtures
        let test_files = vec![
            "test-*.txt",
            "test_files.txt",
            "test-agreement.txt",
        ];
        
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy();
                    if name.ends_with(".txt") && (
                        name.starts_with("test-") ||
                        name == "test_files.txt" ||
                        name == "test-agreement.txt"
                    ) {
                        let src = name.to_string();
                        let dst = format!("tests/fixtures/{}", src);
                        self.mv_if(&src, &dst)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    fn get_moved_count(&self) -> usize {
        self.moved_count
    }
}

fn main() -> Result<()> {
    println!("🌳 Tree Cleanup (High-Impact, Low-Churn)");
    println!("=====================================");

    let mut cleaner = TreeCleaner::new();
    
    // Create new directory structure
    cleaner.create_directories()?;
    
    // Execute cleanup operations
    cleaner.cleanup_generated()?;
    cleaner.cleanup_reports()?;
    cleaner.cleanup_worktree()?;
    cleaner.cleanup_cache()?;
    cleaner.cleanup_tests()?;
    
    println!("\n📊 Cleanup Summary:");
    println!("   Operations completed: {}", cleaner.get_moved_count());
    println!("   Directories created: 5");
    
    println!("\n✅ Tree cleanup completed!");
    println!("📝 Review changes before committing:");
    println!("   git status");
    println!("   git diff --cached");
    
    println!("\n🔍 Next steps:");
    println!("   1. Run: cargo run --bin tree_dircheck config/dircheck.tree.yml");
    println!("   2. Review and commit changes");
    println!("   3. Update .gitignore to include .cache/ and generated/");
    
    Ok(())
}
