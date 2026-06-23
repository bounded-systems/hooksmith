// commit-msg hook — delegates to the hook-engine policy evaluator.
//
// git passes the path to the commit message file as $1.
// The host reads the file and passes it as stdin to the engine.
use anyhow::Result;
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Read the commit message file that git provided as the first arg.
    let stdin = args.first().and_then(|path| std::fs::read_to_string(path).ok());

    let env_vars: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| k.starts_with("GIT_") || k == "HOME")
        .collect();

    let repo_root = std::env::var("GIT_WORK_TREE")
        .or_else(|_| std::env::var("PWD"))
        .unwrap_or_else(|_| ".".to_string());

    let result = hook_engine::evaluate_impl_raw(
        "commit-msg",
        &args,
        stdin.as_deref(),
        &env_vars,
        &repo_root,
    );
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
        eprintln!("\nhooksmith commit-msg: {}", result.summary);
    }
}
