use clap::{Arg, Command};
use hooksmith::modules::functional_contract_pipeline::{
    symbols::HookEvent, FunctionalContractPipeline,
};
use std::process;

fn main() {
    let matches = Command::new("Functional Contract Runner")
        .version("1.0")
        .about("Runs functional contract validation for Git hooks")
        .arg(
            Arg::new("hook")
                .short('h')
                .long("hook")
                .value_name("HOOK")
                .help("Git hook to validate")
                .required(true)
                .value_parser([
                    "pre-commit",
                    "pre-push",
                    "pre-receive",
                    "post-receive",
                    "update",
                    "post-update",
                    "pre-auto-gc",
                    "post-merge",
                    "pre-rebase",
                    "post-checkout",
                    "post-commit",
                    "pre-apply-patch",
                    "post-apply-patch",
                    "post-rebase",
                    "pre-commit-msg",
                    "commit-msg",
                    "post-commit-msg",
                ]),
        )
        .arg(
            Arg::new("repo")
                .short('r')
                .long("repo")
                .value_name("PATH")
                .help("Repository path (default: current directory)")
                .default_value("."),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Show detailed diff information"),
        )
        .get_matches();

    let hook_name = matches.get_one::<String>("hook").unwrap();
    let repo_path = matches.get_one::<String>("repo").unwrap();
    let verbose = matches.contains_id("verbose");

    // Convert hook name to enum
    let hook = match hook_name.as_str() {
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

    // Create pipeline and run validation
    let mut pipeline = FunctionalContractPipeline::new(repo_path);

    if verbose {
        // Run with detailed diffs
        match pipeline.run_hook_with_diff(hook) {
            Ok(diff_set) => {
                println!("✅ Validation passed");
                if diff_set.diffs.len() > 0 {
                    println!("⚠️  {} diffs found:", diff_set.diffs.len());
                    for diff in &diff_set.diffs {
                        println!("  - {}: {}", diff.concern.name(), diff.description);
                    }
                }
            }
            Err(error) => {
                eprintln!("❌ Validation failed: {}", error);
                process::exit(1);
            }
        }
    } else {
        // Run with simple result
        match pipeline.run_hook(hook) {
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
