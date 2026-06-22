use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    println!("🚀 Migrating to Minimal Root Structure");
    println!("=====================================");

    // Define the move operations - only the files that actually exist
    let moves = vec![
        // Configuration files that can be moved
        ("CODEOWNERS", ".github/"),
        // Docker and infrastructure
        ("Dockerfile", "infra/docker/"),
        ("docker-compose.yml", "infra/docker/"),
        ("docker-bake.hcl", "infra/docker/"),
        (".dockerignore", "infra/docker/"),
        // Contracts and agreements
        ("contracts", ".hooksmith/"),
        // Generated files
        ("gen", "tools/gen/"),
        ("generated-sources", "tools/gen/"),
        // Worktree and lifecycle
        ("worktree-lifecycle", "tools/worktree/"),
        (".worktree-config.json", "tools/worktree/"),
        (".worktree-config.jsonc", "tools/worktree/"),
        // Misc files that should be moved
        (".wb", "tools/misc/"),
        (".workbloom", "tools/misc/"),
        (".contract_cache", "tools/misc/"),
    ];

    // Create necessary directories
    let directories = vec![
        "infra/docker",
        ".hooksmith",
        "tools/gen",
        "tools/worktree",
        "tools/misc",
    ];

    println!("📁 Creating directories...");
    for dir in &directories {
        if !Path::new(dir).exists() {
            fs::create_dir_all(dir).context(format!("Failed to create directory: {}", dir))?;
            println!("   Created: {}", dir);
        }
    }

    // Perform moves
    println!("\n📦 Moving files...");
    let mut moved_count = 0;

    for (source, destination) in &moves {
        let source_path = format!("../{}", source);
        if Path::new(&source_path).exists() {
            let dest_path = format!("{}{}", destination, source);

            // Create destination directory if it doesn't exist
            let dest_path_full = format!("../{}", dest_path);
            if let Some(parent) = Path::new(&dest_path_full).parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).context(format!(
                        "Failed to create parent directory for: {}",
                        dest_path_full
                    ))?;
                }
            }

            // Move the file/directory
            fs::rename(&source_path, &dest_path_full).context(format!(
                "Failed to move {} to {}",
                source_path, dest_path_full
            ))?;
            println!("   ✅ Moved: {} → {}", source, dest_path);
            moved_count += 1;
        } else {
            println!("   ⚠️  File not found: {}", source_path);
        }
    }

    println!("\n📊 Migration Summary:");
    println!("   Files moved: {}", moved_count);
    println!("   Directories created: {}", directories.len());

    // Run validation to check if we're compliant
    println!("\n🔍 Running validation...");
    let output = Command::new("cargo")
        .args(["run", "--bin", "test_minimal_root_contract"])
        .current_dir("scripts")
        .output()
        .context("Failed to run validation")?;

    println!("Validation output:");
    println!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.stderr.is_empty() {
        println!("Errors:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if output.status.success() {
        println!("\n✅ Migration completed successfully!");
    } else {
        println!("\n⚠️  Migration completed with some issues. Please review the validation output above.");
    }

    Ok(())
}
