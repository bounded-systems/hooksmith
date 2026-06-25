// pre-push hook — delegates to the hook-engine policy evaluator.
//
// git passes the remote name and URL as $1 and $2.
// stdin contains ref-lines: <local-ref> <local-sha> <remote-ref> <remote-sha>
use anyhow::Result;
use std::io::Read;
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // git writes ref-lines to the hook's stdin.
    let mut stdin_buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin_buf);
    let stdin = if stdin_buf.is_empty() {
        None
    } else {
        Some(stdin_buf.as_str())
    };

    let env_vars: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| k.starts_with("GIT_") || k == "HOME")
        .collect();

    let repo_root = std::env::var("GIT_WORK_TREE")
        .or_else(|_| std::env::var("PWD"))
        .unwrap_or_else(|_| ".".to_string());

    let result = hook_engine::evaluate_impl_raw("pre-push", &args, stdin, &env_vars, &repo_root);
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
        eprintln!("\nhooksmith pre-push: {}", result.summary);
    }
}
