use std::process::Command;
use std::env;
use hooksmith::{log_info, log_warning, log_error, log_success, log_header, run_git_command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("verify");

    match command {
        "verify" => verify_worktree_sync()?,
        "status" => show_status()?,
        "help" | "--help" | "-h" => show_help(),
        _ => {
            log_error(&format!("Unknown command: {}", command));
            println!("Use 'cargo run --bin verify-worktree-1to1 -- help' for usage information");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn verify_worktree_sync() -> Result<(), Box<dyn std::error::Error>> {
    log_header("VERIFYING WORKTREE 1:1 SYNC");

    // Fetch latest remote branches
    log_info("Fetching remote branches...");
    run_git_command(&["fetch", "--all", "--prune"])?;

    // Get current state
    let remote_branches = get_remote_branches()?;
    let worktree_branches = get_worktree_branches()?;

    log_info(&format!("Remote branches ({}): {}", remote_branches.len(), remote_branches.join(", ")));
    log_info(&format!("Worktree branches ({}): {}", worktree_branches.len(), worktree_branches.join(", ")));

    // Find missing worktrees (remote branches without worktrees)
    let mut missing_worktrees = Vec::new();
    for branch in &remote_branches {
        if !worktree_exists(branch)? {
            missing_worktrees.push(branch.clone());
        }
    }

    // Find orphaned worktrees (worktrees without remote branches)
    let mut orphaned_worktrees = Vec::new();
    for branch in &worktree_branches {
        if !remote_branch_exists(branch)? {
            orphaned_worktrees.push(branch.clone());
        }
    }

    // Report results
    println!();
    log_header("VERIFICATION RESULTS");

    if missing_worktrees.is_empty() && orphaned_worktrees.is_empty() {
        log_success("✅ PERFECT SYNC: All worktrees have corresponding remote branches");
        log_success("✅ All remote branches have corresponding worktrees");
        Ok(())
    } else {
        if !missing_worktrees.is_empty() {
            log_warning(&format!("⚠️  MISSING WORKTREES ({}): {}", missing_worktrees.len(), missing_worktrees.join(", ")));
        }

        if !orphaned_worktrees.is_empty() {
            log_warning(&format!("⚠️  ORPHANED WORKTREES ({}): {}", orphaned_worktrees.len(), orphaned_worktrees.join(", ")));
        }

        log_info("💡 Run 'cargo run --bin sync-all-remote-branches' to fix the sync");
        Ok(())
    }
}

fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    log_header("CURRENT STATUS");

    println!("Worktrees:");
    let worktree_output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let worktree_list = String::from_utf8(worktree_output.stdout)?;
    for line in worktree_list.lines() {
        if !line.trim().is_empty() {
            log_info(line);
        }
    }

    println!();
    println!("Remote branches:");
    let remote_output = Command::new("git")
        .args(&["branch", "-r"])
        .output()?;

    let remote_list = String::from_utf8(remote_output.stdout)?;
    for line in remote_list.lines() {
        if line.contains("origin/") && !line.contains("origin/main") && !line.contains("origin/HEAD") {
            log_info(line.trim());
        }
    }

    Ok(())
}

fn show_help() {
    println!("Usage: cargo run --bin verify-worktree-1to1 -- [verify|status|help]");
    println!();
    println!("Commands:");
    println!("  verify  - Check if worktrees are in 1:1 sync with remote branches (default)");
    println!("  status  - Show current worktree and remote branch status");
    println!("  help    - Show this help message");
}

fn get_remote_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["branch", "-r"])
        .output()?;

    let branches = String::from_utf8(output.stdout)?
        .lines()
        .filter(|line| line.contains("origin/") && !line.contains("origin/main") && !line.contains("origin/HEAD"))
        .map(|line| line.replace("origin/", "").trim().to_string())
        .filter(|branch| !branch.is_empty())
        .collect();

    Ok(branches)
}

fn get_worktree_branches() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let branches = String::from_utf8(output.stdout)?
        .lines()
        .filter(|line| !line.contains("main"))
        .filter_map(|line| {
            // Extract branch name from [branch] format
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    if start < end {
                        return Some(line[start + 1..end].to_string());
                    }
                }
            }
            None
        })
        .collect();

    Ok(branches)
}

fn worktree_exists(branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["worktree", "list"])
        .output()?;

    let worktree_list = String::from_utf8(output.stdout)?;
    Ok(worktree_list.contains(&format!("[{}]", branch_name)))
}

fn remote_branch_exists(branch_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(&["ls-remote", "--heads", "origin", branch_name])
        .output()?;

    let remote_list = String::from_utf8(output.stdout)?;
    Ok(remote_list.contains(branch_name))
}
