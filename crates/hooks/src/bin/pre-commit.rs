// pre-commit hook — delegates to the hook-engine policy evaluator.
//
// git passes no arguments and no stdin. The host reads the staged tree
// entries via git ls-tree and supplies them via HOOKSMITH_TREE_ENTRIES.
use anyhow::Result;
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Populate HOOKSMITH_TREE_ENTRIES so the naming policy can inspect
    // what is about to be committed. Use the index (HEAD if no HEAD yet).
    let tree_entries = std::process::Command::new("git")
        .args(["ls-tree", "--name-only", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    unsafe {
        std::env::set_var("HOOKSMITH_TREE_ENTRIES", &tree_entries);
    }

    let env_vars: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| k.starts_with("GIT_") || k == "HOME" || k == "HOOKSMITH_TREE_ENTRIES")
        .collect();

    let repo_root = std::env::var("GIT_WORK_TREE")
        .or_else(|_| std::env::var("PWD"))
        .unwrap_or_else(|_| ".".to_string());

    let result = hook_engine::evaluate_impl_raw("pre-commit", &args, None, &env_vars, &repo_root);
    print_findings(&result);

    process::exit(result.exit_code as i32);
}

fn print_findings(result: &hook_engine::EvalResult) {
    for f in &result.findings {
        let prefix = match f.level {
            hook_engine::FindingLevel::Error => "error",
            hook_engine::FindingLevel::Warn => "warn ",
            hook_engine::FindingLevel::Info => "info ",
        };
        eprintln!("[{}] {} — {}", prefix, f.rule, f.message);
        if let Some(s) = &f.suggestion {
            eprintln!("       hint: {}", s);
        }
    }
    if !result.allow {
        eprintln!("\nhooksmith pre-commit: {}", result.summary);
    }
}
