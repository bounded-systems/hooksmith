use std::process::Command;
use std::collections::HashSet;
use hooksmith::{log_info, log_success, log_warning, log_error, log_header, run_git_command};

#[derive(Debug)]
struct SyncOptions {
    dry_run: bool,
    skip_main: bool,
    force: bool,
}

fn parse_args() -> SyncOptions {
    let args: Vec<String> = std::env::args().collect();
    let mut options = SyncOptions {
        dry_run: false,
        skip_main: false,
        force: false,
    };
    
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--dry-run" => options.dry_run = true,
            "--skip-main" => options.skip_main = true,
            "--force" => options.force = true,
            "--help" => {
                show_usage();
                std::process::exit(0);
            }
            _ => {
                log_warning(&format!("Unknown argument: {}", arg));
            }
        }
    }
    
    options
}

fn show_usage() {
    println!("Sync All Remote Branches to Worktrees");
    println!();
    println!("Usage: sync-all-remote-branches [options]");
    println!();
    println!("Options:");
    println!("  --dry-run           Show what would be done without making changes");
    println!("  --skip-main         Skip creating worktree for main branch");
    println!("  --force             Force recreation of existing worktrees");
    println!("  --help              Show this usage information");
    println!();
    println!("Examples:");
    println!("  sync-all-remote-branches                    # Sync main to base + create worktrees for other branches");
    println!("  sync-all-remote-branches --dry-run         # Show what would be synced/created");
    println!("  sync-all-remote-branches --skip-main       # Skip main sync, only create worktrees for other branches");
    println!("  sync-all-remote-branches --force           # Force recreation of existing worktrees");
    println!();
    println!("This script will:");
    println!("1. Fetch all remote branches");
    println!("2. Sync main branch to base repository (not worktree)");
    println!("3. Create worktrees for other branches that don't exist locally");
    println!("4. Skip main worktree creation (main stays in base)");
    println!("5. Show summary of created worktrees");
}

fn fetch_remote_branches() -> Result<(), String> {
    log_info("Fetching all remote branches...");
    
    let output = Command::new("git")
        .args(&["fetch", "--all", "--prune"])
        .output()
        .map_err(|e| format!("Failed to fetch remote branches: {}", e))?;
    
    if output.status.success() {
        log_success("Remote branches fetched");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to fetch remote branches: {}", stderr))
    }
}

fn get_remote_branches(skip_main: bool) -> Result<Vec<String>, String> {
    let output = run_git_command(&["branch", "-r"])?;
    
    let mut branches = Vec::new();
    
    for line in output.lines() {
        let clean_line = line.trim();
        
        // Skip empty lines
        if clean_line.is_empty() {
            continue;
        }
        
        // Skip HEAD -> main reference
        if clean_line.contains("origin/HEAD") {
            continue;
        }
        
        // Extract branch name from origin/branch-name format
        if clean_line.starts_with("origin/") {
            let branch_name = clean_line[7..].to_string(); // Remove "origin/" prefix
            
            // Skip main if requested
            if skip_main && branch_name == "main" {
                continue;
            }
            
            // Skip empty branch names
            if !branch_name.is_empty() {
                branches.push(branch_name);
            }
        }
    }
    
    branches.sort();
    Ok(branches)
}

fn worktree_exists(branch_name: &str) -> Result<bool, String> {
    let worktree_path = format!(".wt/{}", branch_name.replace("/", "\\/"));
    
    let output = run_git_command(&["worktree", "list"])?;
    
    Ok(output.lines().any(|line| line.contains(&worktree_path)))
}

fn create_worktree(branch_name: &str, force: bool) -> Result<bool, String> {
    log_info(&format!("Processing branch: {}", branch_name));
    
    let worktree_path = format!(".wt/{}", branch_name.replace("/", "\\/"));
    
    // Check if worktree already exists
    if worktree_exists(branch_name)? {
        if force {
            log_warning(&format!("Removing existing worktree for {}", branch_name));
            
            // Remove existing worktree
            let _ = Command::new("git")
                .args(&["worktree", "remove", &worktree_path])
                .output();
            
            // Delete local branch
            let _ = Command::new("git")
                .args(&["branch", "-D", branch_name])
                .output();
        } else {
            log_warning(&format!("Worktree already exists for {}, skipping", branch_name));
            return Ok(false);
        }
    }
    
    // Create the worktree
    log_info(&format!("Creating worktree for branch: {}", branch_name));
    log_info(&format!("Worktree path: {}", worktree_path));
    
    // Check if local branch exists
    let local_branch_exists = Command::new("git")
        .args(&["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch_name)])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    
    let output = if local_branch_exists {
        // Local branch exists, create worktree without -b flag
        Command::new("git")
            .args(&["worktree", "add", &worktree_path, branch_name])
            .output()
    } else {
        // Local branch doesn't exist, create new branch from remote
        Command::new("git")
            .args(&["worktree", "add", &worktree_path, "-b", branch_name, &format!("origin/{}", branch_name)])
            .output()
    };
    
    match output {
        Ok(output) => {
            if output.status.success() {
                let action = if local_branch_exists { "existing" } else { "new" };
                log_success(&format!("Successfully created worktree for {} branch: {}", action, branch_name));
                Ok(true)
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error(&format!("Failed to create worktree for branch {}: {}", branch_name, stderr));
                Ok(false)
            }
        }
        Err(e) => {
            log_error(&format!("Failed to create worktree for branch {}: {}", branch_name, e));
            Ok(false)
        }
    }
}

fn sync_main_branch(dry_run: bool) -> Result<(), String> {
    log_info("Syncing main branch to base repository");
    
    if dry_run {
        log_info("DRY RUN: Would sync main branch to base");
        return Ok(());
    }
    
    // Fetch latest main
    let output = Command::new("git")
        .args(&["fetch", "origin", "main"])
        .output()
        .map_err(|e| format!("Failed to fetch origin/main: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to fetch origin/main: {}", stderr));
    }
    
    // Check if we're behind
    let behind_count = run_git_command(&["rev-list", "HEAD..origin/main", "--count"])?;
    let behind_count: i32 = behind_count.trim().parse().unwrap_or(0);
    
    if behind_count > 0 {
        log_info(&format!("Main branch is {} commits behind origin/main, updating...", behind_count));
        
        let output = Command::new("git")
            .args(&["pull", "origin", "main"])
            .output()
            .map_err(|e| format!("Failed to pull origin/main: {}", e))?;
        
        if output.status.success() {
            log_success("Successfully synced main branch to origin/main");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to sync main branch: {}", stderr))
        }
    } else {
        log_info("Main branch is already up to date");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("SYNC ALL REMOTE BRANCHES");
    println!();
    
    let options = parse_args();
    
    // Fetch remote branches
    fetch_remote_branches()?;
    
    // Sync main branch if not skipped
    if !options.skip_main {
        sync_main_branch(options.dry_run)?;
        println!();
    }
    
    // Get remote branches
    let branches = get_remote_branches(options.skip_main)?;
    
    if branches.is_empty() {
        log_info("No remote branches found to sync");
        return Ok(());
    }
    
    log_info(&format!("Found {} remote branches to process", branches.len()));
    println!();
    
    let mut created_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;
    
    // Process each branch
    for branch_name in &branches {
        if options.dry_run {
            log_info(&format!("DRY RUN: Would create worktree for branch: {}", branch_name));
            created_count += 1;
        } else {
            match create_worktree(branch_name, options.force) {
                Ok(true) => created_count += 1,
                Ok(false) => skipped_count += 1,
                Err(_) => failed_count += 1,
            }
        }
        println!();
    }
    
    // Summary
    log_header("SUMMARY");
    println!();
    
    if options.dry_run {
        log_info(&format!("DRY RUN: Would create {} worktrees", created_count));
    } else {
        log_success(&format!("Created {} worktrees", created_count));
        if skipped_count > 0 {
            log_warning(&format!("Skipped {} worktrees (already exist)", skipped_count));
        }
        if failed_count > 0 {
            log_error(&format!("Failed to create {} worktrees", failed_count));
        }
    }
    
    Ok(())
}
