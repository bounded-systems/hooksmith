use std::process::Command;
use std::env;
use hooksmith::{log_info, log_warning, log_error, log_success, log_header, run_git_command, get_worktree_paths};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut dry_run = false;
    let mut create_prs = false;
    let mut force = false;

    // Parse command line arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--dry-run" => {
                dry_run = true;
                i += 1;
            }
            "--create-prs" => {
                create_prs = true;
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            "--help" => {
                show_usage();
                return Ok(());
            }
            _ => {
                log_error(&format!("Unknown option: {}", args[i]));
                show_usage();
                std::process::exit(1);
            }
        }
    }

    // Check dependencies
    check_dependencies()?;

    // Update all worktrees
    update_all_worktrees(dry_run, create_prs, force)?;

    Ok(())
}

fn show_usage() {
    println!("Update All Worktrees to Main");
    println!();
    println!("Usage: cargo run --bin update-all-worktrees-to-main -- [options]");
    println!();
    println!("Options:");
    println!("  --dry-run           Show what would be done without making changes");
    println!("  --create-prs        Create PRs for updated worktrees");
    println!("  --force             Force push even if conflicts");
    println!("  --help              Show this usage information");
    println!();
    println!("Examples:");
    println!("  cargo run --bin update-all-worktrees-to-main --                    # Update all worktrees to origin/main");
    println!("  cargo run --bin update-all-worktrees-to-main -- --dry-run         # Show what would be updated");
    println!("  cargo run --bin update-all-worktrees-to-main -- --create-prs      # Update and create PRs");
    println!("  cargo run --bin update-all-worktrees-to-main -- --force           # Force update even with conflicts");
    println!();
    println!("This script will:");
    println!("1. Update all worktrees to be based on origin/main");
    println!("2. Push updated branches to remote");
    println!("3. Create PRs if --create-prs is specified");
    println!("4. Skip main branch (stays in base)");
}

fn check_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    // Check if git is available
    if Command::new("git").arg("--version").output().is_err() {
        log_error("Missing required dependency: git");
        std::process::exit(1);
    }

    // Check if GitHub CLI is available
    if Command::new("gh").arg("--version").output().is_err() {
        log_warning("GitHub CLI (gh) not found. PR creation will be limited.");
    }

    Ok(())
}

fn get_worktree_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    let mut branches = Vec::new();

    for line in worktree_list.lines() {
        // Extract branch name from [branch] format
        if let Some(start) = line.find('[') {
            if let Some(end) = line.find(']') {
                if start < end {
                    let branch_name = &line[start + 1..end];
                    if branch_name != "main" && !branch_name.is_empty() {
                        branches.push(branch_name.to_string());
                    }
                }
            }
        }
    }

    Ok(branches)
}

fn update_worktree_to_main(worktree_path: &str, branch_name: &str, dry_run: bool, force: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Processing worktree: {}", branch_name));

    if dry_run {
        log_info(&format!("DRY RUN: Would update {} to origin/main", branch_name));
        return Ok(true);
    }

    // Change to worktree directory
    let original_dir = env::current_dir()?;
    env::set_current_dir(worktree_path)?;

    // Fetch latest origin/main
    run_git_command(&["fetch", "origin", "main"])?;

    // Check if we're behind origin/main
    let behind_count = get_behind_count()?;

    if behind_count == 0 {
        log_info(&format!("Worktree {} is already up to date with origin/main", branch_name));
        env::set_current_dir(original_dir)?;
        return Ok(true);
    }

    log_info(&format!("Worktree {} is {} commits behind origin/main", branch_name, behind_count));

    // Reset to origin/main
    if run_git_command(&["reset", "--hard", "origin/main"]).is_ok() {
        log_success(&format!("Successfully updated {} to origin/main", branch_name));
    } else {
        log_error(&format!("Failed to update {} to origin/main", branch_name));
        env::set_current_dir(original_dir)?;
        return Ok(false);
    }

    // Push to remote
    let push_args = if force {
        vec!["push", "--force", "origin", branch_name]
    } else {
        vec!["push", "--force-with-lease", "origin", branch_name]
    };

    if run_git_command(&push_args).is_ok() {
        log_success(&format!("Successfully pushed {} to remote", branch_name));
    } else {
        log_warning(&format!("Failed to push {} to remote", branch_name));
        env::set_current_dir(original_dir)?;
        return Ok(false);
    }

    env::set_current_dir(original_dir)?;
    Ok(true)
}

fn get_behind_count() -> Result<u32, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["rev-list", "HEAD..origin/main", "--count"])
        .output()?;

    let count_str = String::from_utf8(output.stdout)?.trim().to_string();
    let count: u32 = count_str.parse()?;
    Ok(count)
}

fn create_pr_for_worktree(branch_name: &str, dry_run: bool) -> Result<bool, Box<dyn std::error::Error>> {
    log_info(&format!("Creating PR for branch: {}", branch_name));

    if dry_run {
        log_info(&format!("DRY RUN: Would create PR for {}", branch_name));
        return Ok(true);
    }

    // Check if PR already exists
    let output = Command::new("gh")
        .args(&["pr", "list", "--head", branch_name, "--json", "number", "--jq", "length"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let count = String::from_utf8(output.stdout)?.trim();
            if count != "0" {
                log_warning(&format!("PR already exists for {}", branch_name));
                return Ok(true);
            }
        }
    }

    // Create PR
    let title = format!("feat: Update {} to latest main", branch_name);
    let body = format!("Updated {} to be based on the latest origin/main\n\n## Changes\n- Updated branch to latest origin/main\n- Ensured compatibility with current main branch\n- Ready for review and merge", branch_name);

    let output = Command::new("gh")
        .args(&["pr", "create", "--title", &title, "--body", &body])
        .output()?;

    if output.status.success() {
        log_success(&format!("Successfully created PR for {}", branch_name));
        Ok(true)
    } else {
        log_error(&format!("Failed to create PR for {}", branch_name));
        Ok(false)
    }
}

fn update_all_worktrees(dry_run: bool, create_prs: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    log_header("UPDATING ALL WORKTREES TO MAIN");

    // Get list of worktree branches
    let branches = get_worktree_branches()?;

    log_info(&format!("Found {} worktrees to update", branches.len()));

    let mut updated_count = 0;
    let mut failed_count = 0;
    let mut pr_count = 0;

    // Process each worktree
    for branch in &branches {
        // Get worktree path
        let worktree_path = get_worktree_path(branch)?;
        if worktree_path.is_empty() {
            log_error(&format!("Could not find worktree path for branch: {}", branch));
            failed_count += 1;
            continue;
        }

        if update_worktree_to_main(&worktree_path, branch, dry_run, force)? {
            updated_count += 1;

            // Create PR if requested
            if create_prs {
                if create_pr_for_worktree(branch, dry_run)? {
                    pr_count += 1;
                }
            }
        } else {
            failed_count += 1;
        }
    }

    // Show summary
    log_header("UPDATE SUMMARY");
    log_info(&format!("Worktrees processed: {}", branches.len()));
    log_success(&format!("Updated: {}", updated_count));
    log_error(&format!("Failed: {}", failed_count));

    if create_prs {
        log_success(&format!("PRs created: {}", pr_count));
    }

    if !dry_run {
        log_info("Use 'cargo run --bin worktree-status-report' to see updated worktree status");
    }

    Ok(())
}

fn get_worktree_path(branch_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    for line in worktree_list.lines() {
        if line.contains(&format!("[{}]", branch_name)) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                return Ok(parts[0].to_string());
            }
        }
    }

    Ok(String::new())
}
