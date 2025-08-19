use hooksmith::{log_error, log_header, log_info, log_success, log_warning};
use std::env;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if any arguments were passed
    if args.len() == 1 {
        show_worktree_guidance();
        println!();
        show_worktree_status()?;
        return Ok(());
    }

    // If arguments were passed, show guidance but also suggest the direct command
    show_worktree_guidance();
    println!();
    log_warning("🔧 To execute the original git worktree command, use:");
    log_info("git xworktree <args>");
    println!();
    log_warning("Or use the worktree aliases:");
    log_info("git wtl  - List worktrees");
    log_info("git wtc  - Create worktree");
    log_info("git wts  - Switch worktree");
    log_info("git wtr  - Remove worktree");
    println!();

    Ok(())
}

fn show_worktree_guidance() {
    log_header("🌳 Worktree Management");
    println!();
    log_error("❌ Please use worktree commands instead of git worktree:");
    println!();
    log_info("  📋 List worktrees:     cargo xtask worktree list");
    log_info("  ➕ Create worktree:    cargo xtask worktree create --branch <branch>");
    log_info("  🔄 Switch worktree:    cargo xtask worktree switch --worktree <name>");
    log_info("  🗑️  Remove worktree:    cargo xtask worktree remove --worktree <name>");
    log_info("  🛠️  Setup tools:        cargo xtask worktree setup");
    println!();
    log_warning("  📁 Or use git xworktree for direct git worktree access");
    println!();
    log_success("💡 Tip: Worktrees are now created in worktrees/ directory by default");
    println!();
    log_info("🔧 Available aliases:");
    log_info("  git wtl  - List worktrees");
    log_info("  git wtc  - Create worktree");
    log_info("  git wts  - Switch worktree");
    log_info("  git wtr  - Remove worktree");
    println!();
}

fn show_worktree_status() -> Result<(), Box<dyn std::error::Error>> {
    log_info("📊 Current Worktree Status:");
    log_info("==========================");

    // Try to run cargo xtask worktree list
    match Command::new("cargo")
        .args(&["xtask", "worktree", "list"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8(output.stdout)?;
                for line in output_str.lines() {
                    if !line.trim().is_empty() {
                        log_info(&format!("  {}", line));
                    }
                }
            } else {
                log_warning("⚠️  Cargo xtask not available or failed");
            }
        }
        Err(_) => {
            log_warning("⚠️  Cargo not available");
        }
    }

    println!();
    log_info("📁 .wt Directory Contents:");

    let wt_dir = std::path::Path::new(".wt");
    if wt_dir.exists() {
        match std::fs::read_dir(wt_dir) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let file_name = entry.file_name();
                            let file_name_str = file_name.to_string_lossy();
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.is_dir() {
                                    log_info(&format!("  📁 {}", file_name_str));
                                } else {
                                    log_info(&format!("  📄 {}", file_name_str));
                                }
                            } else {
                                log_info(&format!("  ❓ {}", file_name_str));
                            }
                        }
                        Err(_) => {
                            log_warning("⚠️  Cannot read .wt directory entry");
                        }
                    }
                }
            }
            Err(_) => {
                log_warning("⚠️  Cannot list .wt directory");
            }
        }
    } else {
        log_warning("⚠️  .wt directory not found");
    }

    Ok(())
}
