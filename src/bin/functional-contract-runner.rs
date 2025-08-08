use clap::{App, Arg};
use hooksmith::modules::functional_contract_pipeline::{
    FunctionalContractPipeline, HookEvent, run_hook, run_hook_with_diffs
};
use std::process;

fn main() {
    let matches = App::new("Functional Contract Runner")
        .version("1.0")
        .about("Runs functional contract validation for Git hooks")
        .arg(
            Arg::with_name("hook")
                .short("h")
                .long("hook")
                .value_name("HOOK")
                .help("Git hook to validate")
                .required(true)
                .possible_values(&[
                    "pre-commit", "pre-push", "pre-receive", "post-receive",
                    "update", "post-update", "pre-auto-gc", "post-merge",
                    "pre-rebase", "post-checkout", "post-commit", "pre-apply-patch",
                    "post-apply-patch", "post-rebase", "pre-commit-msg",
                    "commit-msg", "post-commit-msg"
                ])
        )
        .arg(
            Arg::with_name("repo")
                .short("r")
                .long("repo")
                .value_name("PATH")
                .help("Repository path (default: current directory)")
                .default_value(".")
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Show detailed diff information")
        )
        .get_matches();

    let hook_name = matches.value_of("hook").unwrap();
    let repo_path = matches.value_of("repo").unwrap();
    let verbose = matches.is_present("verbose");

    // Convert hook name to enum
    let hook = match hook_name {
        "pre-commit" => HookEvent::PreCommit,
        "pre-push" => HookEvent::PrePush,
        "pre-receive" => HookEvent::PreReceive,
        "post-receive" => HookEvent::PostReceive,
        "update" => HookEvent::Update,
        "post-update" => HookEvent::PostUpdate,
        "pre-auto-gc" => HookEvent::PreAutoGc,
        "post-merge" => HookEvent::PostMerge,
        "pre-rebase" => HookEvent::PreRebase,
        "post-checkout" => HookEvent::PostCheckout,
        "post-commit" => HookEvent::PostCommit,
        "pre-apply-patch" => HookEvent::PreApplyPatch,
        "post-apply-patch" => HookEvent::PostApplyPatch,
        "post-rebase" => HookEvent::PostRebase,
        "pre-commit-msg" => HookEvent::PreCommitMsg,
        "commit-msg" => HookEvent::CommitMsg,
        "post-commit-msg" => HookEvent::PostCommitMsg,
        _ => {
            eprintln!("❌ Unknown hook: {}", hook_name);
            process::exit(1);
        }
    };

    println!("🔧 Functional Contract Validation Pipeline");
    println!("==========================================");
    println!("Hook: {}", hook_name);
    println!("Repository: {}", repo_path);
    println!();

    if verbose {
        // Run with detailed diffs
        let diff_set = run_hook_with_diffs(hook, repo_path);
        
        if diff_set.is_valid() {
            println!("✅ Validation passed");
            if diff_set.diff_count() > 0 {
                println!("⚠️  {} warnings found:", diff_set.warnings().len());
                for diff in diff_set.warnings() {
                    println!("  - {}: {}", diff.concern.name(), diff.description);
                }
            }
        } else {
            println!("❌ Validation failed");
            println!("Errors:");
            for diff in diff_set.errors() {
                println!("  - {}: {}", diff.concern.name(), diff.description);
                if let Some(observed) = &diff.observed {
                    println!("    Observed: {}", observed);
                }
                if let Some(expected) = &diff.expected {
                    println!("    Expected: {}", expected);
                }
            }
            
            if diff_set.warnings().len() > 0 {
                println!("Warnings:");
                for diff in diff_set.warnings() {
                    println!("  - {}: {}", diff.concern.name(), diff.description);
                }
            }
            
            process::exit(1);
        }
    } else {
        // Run with simple result
        match run_hook(hook, repo_path) {
            Ok(()) => {
                println!("✅ Validation passed");
            }
            Err(error) => {
                eprintln!("❌ Validation failed: {}", error);
                process::exit(1);
            }
        }
    }
}
