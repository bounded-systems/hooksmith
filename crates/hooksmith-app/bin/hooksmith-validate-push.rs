use anyhow::{bail, Result};
use std::process::Command;

fn main() -> Result<()> {
    println!("🔍 Validating Git push concerns...");

    // Validate refs
    println!("📋 Checking Git references...");
    let refs_output = Command::new("git")
        .args(["for-each-ref", "--format=%(refname)"])
        .output()?;

    if !refs_output.status.success() {
        bail!(
            "Failed to list Git references: {}",
            String::from_utf8_lossy(&refs_output.stderr)
        );
    }

    let refs_str = String::from_utf8_lossy(&refs_output.stdout);
    let ref_count = refs_str.lines().count();
    println!("   Found {} references", ref_count);

    // Validate remotes
    println!("🌐 Checking Git remotes...");
    let remotes_output = Command::new("git").args(["remote"]).output()?;

    if !remotes_output.status.success() {
        bail!(
            "Failed to list Git remotes: {}",
            String::from_utf8_lossy(&remotes_output.stderr)
        );
    }

    let remotes_str = String::from_utf8_lossy(&remotes_output.stdout);
    let remote_count = remotes_str.lines().count();
    println!("   Found {} remotes", remote_count);

    // Validate worktrees
    println!("🌳 Checking Git worktrees...");
    let worktrees_output = Command::new("git").args(["worktree", "list"]).output()?;

    if !worktrees_output.status.success() {
        bail!(
            "Failed to list Git worktrees: {}",
            String::from_utf8_lossy(&worktrees_output.stderr)
        );
    }

    let worktrees_str = String::from_utf8_lossy(&worktrees_output.stdout);
    let worktree_count = worktrees_str.lines().count();
    println!("   Found {} worktrees", worktree_count);

    // Summary
    println!("\n📊 Push Validation Summary:");
    println!("   References: {}", ref_count);
    println!("   Remotes: {}", remote_count);
    println!("   Worktrees: {}", worktree_count);

    // Validation checks
    let mut warnings = Vec::new();

    if ref_count == 0 {
        warnings.push("No Git references found - repository may be empty".to_string());
    }

    if remote_count == 0 {
        warnings.push("No Git remotes configured - push may fail".to_string());
    }

    if worktree_count > 1 {
        warnings.push(format!(
            "Multiple worktrees ({}) - ensure all are clean",
            worktree_count
        ));
    }

    if !warnings.is_empty() {
        println!("\n⚠️  Warnings:");
        for warning in &warnings {
            println!("   - {}", warning);
        }
    }

    println!("\n✅ Git push validation completed successfully");
    Ok(())
}
