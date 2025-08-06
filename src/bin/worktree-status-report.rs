use hooksmith::{
    determine_state, generate_pr_url, get_worktree_status, get_worktrees, log_error, log_header,
    log_info, log_success, log_warning, WorktreeState, WorktreeStatus, BLUE, CYAN, NC,
};

fn print_worktree_status(worktree_path: &str, status: &WorktreeStatus) {
    let state = determine_state(status);
    let state_str = match state {
        WorktreeState::Merged => "MERGED",
        WorktreeState::Conflicted => "CONFLICTED",
        WorktreeState::Developing => "DEVELOPING",
        WorktreeState::Ready => "READY",
        WorktreeState::Outdated => "OUTDATED",
        WorktreeState::Unknown => "UNKNOWN",
    };

    println!("{}📁 Worktree:{} {}", CYAN, NC, worktree_path);
    println!("   {}Branch:{} {}", BLUE, NC, status.current_branch);
    println!("   {}State:{} {}", BLUE, NC, state_str);
    println!(
        "   {}Status:{} {}",
        BLUE,
        NC,
        if status.is_clean { "clean" } else { "dirty" }
    );
    println!("   {}Rebasing:{} {}", BLUE, NC, status.is_rebasing);
    println!("   {}Remote:{} {}", BLUE, NC, status.remote_exists);
    println!("   {}Merged:{} {}", BLUE, NC, status.is_merged);
    println!(
        "   {}Commits:{} +{} -{}",
        BLUE, NC, status.ahead_behind, status.behind_ahead
    );
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_header("WORKTREE STATUS REPORT");
    println!();

    let worktrees = get_worktrees()?;

    if worktrees.is_empty() {
        log_info("No worktrees found");
        return Ok(());
    }

    let mut ready_worktrees = Vec::new();
    let mut conflicted_worktrees = Vec::new();
    let mut merged_worktrees = Vec::new();
    let mut developing_worktrees = Vec::new();

    // Process each worktree
    for worktree_path in &worktrees {
        match get_worktree_status(worktree_path) {
            Ok(status) => {
                print_worktree_status(worktree_path, &status);

                let state = determine_state(&status);
                match state {
                    WorktreeState::Ready => ready_worktrees.push(status.current_branch.clone()),
                    WorktreeState::Conflicted => {
                        conflicted_worktrees.push(status.current_branch.clone())
                    }
                    WorktreeState::Merged => merged_worktrees.push(status.current_branch.clone()),
                    WorktreeState::Developing => {
                        developing_worktrees.push(status.current_branch.clone())
                    }
                    _ => {}
                }
            }
            Err(e) => {
                log_error(&format!(
                    "Failed to get status for {}: {}",
                    worktree_path, e
                ));
            }
        }
    }

    // Summary
    log_header("SUMMARY");
    println!();

    if !ready_worktrees.is_empty() {
        log_success(&format!("Ready for PR: {}", ready_worktrees.join(", ")));
        for branch in &ready_worktrees {
            let pr_url = generate_pr_url(branch);
            println!("   PR URL: {}", pr_url);
        }
        println!();
    }

    if !conflicted_worktrees.is_empty() {
        log_warning(&format!("Conflicted: {}", conflicted_worktrees.join(", ")));
        println!();
    }

    if !merged_worktrees.is_empty() {
        log_info(&format!(
            "Merged (ready for cleanup): {}",
            merged_worktrees.join(", ")
        ));
        println!();
    }

    if !developing_worktrees.is_empty() {
        log_info(&format!("Developing: {}", developing_worktrees.join(", ")));
        println!();
    }

    if ready_worktrees.is_empty()
        && conflicted_worktrees.is_empty()
        && merged_worktrees.is_empty()
        && developing_worktrees.is_empty()
    {
        log_info("No worktrees to process");
    }

    Ok(())
}
