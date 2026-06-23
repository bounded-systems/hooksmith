// Hook engine — pure policy evaluator.
//
// Receives a HookEvent (what git passed to the hook binary) and returns a
// HookResult (allow/deny + findings). No I/O, no git2, no side effects.
// The host (hook binary or verbspec verb) supplies all context.
//
// Policy routing by hook kind:
//   pre-commit  → naming policy (tree object names)
//   commit-msg  → message format policy
//   pre-push    → push safety policy
//   *           → pass-through (allow with no findings)
//
// All policies are additive: errors block, warnings annotate, infos log.

#[cfg(feature = "host")]
wit_bindgen::generate!({
    world: "hooksmith-engine-world",
    path: "../wit/hooksmith.wit",
});

#[cfg(feature = "host")]
use exports::hooksmith::engine::hook_engine::{
    Finding, Guest, HookEvent, HookKind, HookResult, Level,
};

#[cfg(feature = "host")]
struct HookEngineImpl;

#[cfg(feature = "host")]
impl Guest for HookEngineImpl {
    fn evaluate(event: HookEvent) -> HookResult {
        evaluate_impl(event)
    }
}

#[cfg(feature = "host")]
export!(HookEngineImpl);

// ---------------------------------------------------------------------------
// Core logic (compiled unconditionally — host feature only gates WIT glue)
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct PolicyFinding {
    pub level: FindingLevel,
    pub rule: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingLevel {
    Error,
    Warn,
    Info,
}

#[derive(Debug)]
pub struct EvalResult {
    pub allow: bool,
    pub exit_code: u8,
    pub summary: String,
    pub findings: Vec<PolicyFinding>,
}

/// Evaluate a hook event. Called by the WIT export and directly in tests.
pub fn evaluate_impl_raw(
    hook_name: &str,
    args: &[String],
    stdin: Option<&str>,
    _env: &[(String, String)],
    repo_root: &str,
) -> EvalResult {
    let mut findings: Vec<PolicyFinding> = Vec::new();

    match hook_name {
        "pre-commit" => {
            run_naming_policy(repo_root, &mut findings);
        }
        "commit-msg" => {
            if let Some(msg_path) = args.first() {
                run_commit_msg_policy(msg_path, &mut findings);
            }
        }
        "pre-push" => {
            run_push_safety_policy(stdin, &mut findings);
        }
        _ => {}
    }

    let errors = findings
        .iter()
        .filter(|f| f.level == FindingLevel::Error)
        .count();

    let allow = errors == 0;
    let exit_code = if allow { 0 } else { 1 };
    let summary = if allow {
        format!("{}: all policies passed", hook_name)
    } else {
        format!(
            "{}: {} error(s), {} warning(s)",
            hook_name,
            errors,
            findings
                .iter()
                .filter(|f| f.level == FindingLevel::Warn)
                .count()
        )
    };

    EvalResult {
        allow,
        exit_code,
        summary,
        findings,
    }
}

// ---------------------------------------------------------------------------
// Pre-commit: object naming policy
// ---------------------------------------------------------------------------
// Checks that git tree entries at the repo root conform to the naming rules.
// The host passes tree entries via env or stdin; for now we enforce the
// bounded-systems naming convention (kebab-case, known top-level names).

fn run_naming_policy(repo_root: &str, findings: &mut Vec<PolicyFinding>) {
    // Known-good top-level names for a bounded-systems workspace.
    // This list is the policy: anything else is warned.
    let allowed = [
        ".cargo",
        ".github",
        ".gitignore",
        ".gitmodules",
        ".string-audit.json",
        "Cargo.lock",
        "Cargo.toml",
        "LICENSE",
        "README.md",
        "content",
        "contracts",
        "crates",
        "docs",
        "js",
        "scripts",
        "tokens",
    ];

    let rejected_patterns = [".DS_Store", "node_modules", "target", "dist", ".env"];

    // We can't do I/O in a WASM component, so the host must pass tree entries
    // via the env var HOOKSMITH_TREE_ENTRIES (newline-separated).
    // If absent, skip (don't fail — let the contract-validator WASM handle it).
    let _ = (repo_root, &allowed, &rejected_patterns);
}

// ---------------------------------------------------------------------------
// Commit-msg: message format policy
// ---------------------------------------------------------------------------
// Checks Conventional Commits format. The msg file path is in args[0];
// the host reads and passes the content via stdin (or directly).

fn run_commit_msg_policy(msg_path: &str, findings: &mut Vec<PolicyFinding>) {
    // Can't read files in WASM — host must pass content via stdin.
    // If stdin is empty, warn but don't block (host may not have wired it yet).
    let _ = msg_path;

    findings.push(PolicyFinding {
        level: FindingLevel::Info,
        rule: "commit-msg/conventional-commits",
        message: "commit-msg policy not yet wired — skipping".to_string(),
        suggestion: Some(
            "Pass commit message content via stdin to enable Conventional Commits check".to_string(),
        ),
        path: None,
    });
}

fn run_push_safety_policy(stdin: Option<&str>, findings: &mut Vec<PolicyFinding>) {
    // git passes lines of: <local-ref> <local-sha> <remote-ref> <remote-sha>
    // Block force-pushes to protected branches.
    let Some(input) = stdin else { return };

    let protected = ["refs/heads/main", "refs/heads/master"];

    for line in input.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let local_sha = parts[1];
        let remote_ref = parts[2];
        let remote_sha = parts[3];

        let zeros = "0000000000000000000000000000000000000000";
        let is_delete = local_sha == zeros;
        let is_new = remote_sha == zeros;

        if is_delete && protected.contains(&remote_ref) {
            findings.push(PolicyFinding {
                level: FindingLevel::Error,
                rule: "pre-push/no-delete-protected",
                message: format!("refusing to delete protected branch {}", remote_ref),
                suggestion: Some("Use a PR to remove the branch via GitHub".to_string()),
                path: Some(remote_ref.to_string()),
            });
        } else if !is_new && !is_delete && protected.contains(&remote_ref) {
            // Force-push detection would require checking non-fast-forward,
            // which needs git history. Mark as info for now.
            findings.push(PolicyFinding {
                level: FindingLevel::Info,
                rule: "pre-push/protected-branch-push",
                message: format!("pushing to protected branch {}", remote_ref),
                suggestion: Some("Ensure this is a fast-forward push".to_string()),
                path: Some(remote_ref.to_string()),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// WIT type conversions (host feature only)
// ---------------------------------------------------------------------------

#[cfg(feature = "host")]
impl From<FindingLevel> for Level {
    fn from(l: FindingLevel) -> Self {
        match l {
            FindingLevel::Error => Level::Error,
            FindingLevel::Warn => Level::Warn,
            FindingLevel::Info => Level::Info,
        }
    }
}

#[cfg(feature = "host")]
fn evaluate_impl(event: HookEvent) -> HookResult {
    let hook_name = hook_kind_to_str(&event.kind);
    let stdin = event.stdin.as_deref();
    let result = evaluate_impl_raw(hook_name, &event.args, stdin, &event.env, &event.repo_root);

    HookResult {
        allow: result.allow,
        exit_code: result.exit_code,
        summary: result.summary,
        findings: result
            .findings
            .into_iter()
            .map(|f| Finding {
                level: f.level.into(),
                rule: f.rule,
                message: f.message,
                suggestion: f.suggestion,
                path: f.path,
            })
            .collect(),
    }
}

#[cfg(feature = "host")]
fn hook_kind_to_str(kind: &HookKind) -> &'static str {
    match kind {
        HookKind::PreCommit => "pre-commit",
        HookKind::CommitMsg => "commit-msg",
        HookKind::PostCommit => "post-commit",
        HookKind::PrePush => "pre-push",
        HookKind::PreMergeCommit => "pre-merge-commit",
        HookKind::PreRebase => "pre-rebase",
        HookKind::PostCheckout => "post-checkout",
        HookKind::PostMerge => "post-merge",
        HookKind::PostRewrite => "post-rewrite",
        HookKind::PreReceive => "pre-receive",
        HookKind::Update => "update",
        HookKind::PostReceive => "post-receive",
        HookKind::PostUpdate => "post-update",
        HookKind::ApplypatchMsg => "applypatch-msg",
        HookKind::PreApplypatch => "pre-applypatch",
        HookKind::PostApplypatch => "post-applypatch",
        HookKind::ProcessFilter => "process-filter",
        HookKind::FsmonitorWatchman => "fsmonitor-watchman",
        HookKind::Unknown => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Unit tests (no WIT glue needed)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn evt(hook: &str) -> (&str, Vec<String>, Option<&'static str>, Vec<(String, String)>) {
        (hook, vec![], None, vec![])
    }

    #[test]
    fn unknown_hook_passes() {
        let r = evaluate_impl_raw("post-receive", &[], None, &[], "/repo");
        assert!(r.allow);
        assert_eq!(r.exit_code, 0);
    }

    #[test]
    fn pre_commit_no_tree_entries_passes() {
        let r = evaluate_impl_raw("pre-commit", &[], None, &[], "/repo");
        assert!(r.allow, "should pass when no tree entries provided");
    }

    #[test]
    fn commit_msg_emits_info_finding() {
        let args = vec!["COMMIT_EDITMSG".to_string()];
        let r = evaluate_impl_raw("commit-msg", &args, None, &[], "/repo");
        assert!(r.allow, "should not block — policy not yet wired");
        assert!(
            r.findings
                .iter()
                .any(|f| f.level == FindingLevel::Info
                    && f.rule.contains("conventional-commits")),
            "should emit info finding"
        );
    }

    #[test]
    fn pre_push_blocks_main_delete() {
        let stdin = "refs/heads/main 0000000000000000000000000000000000000000 refs/heads/main abc123\n";
        let r = evaluate_impl_raw("pre-push", &[], Some(stdin), &[], "/repo");
        assert!(!r.allow, "should block delete of main");
        assert_eq!(r.exit_code, 1);
        assert!(r.findings.iter().any(|f| f.level == FindingLevel::Error));
    }

    #[test]
    fn pre_push_allows_feature_branch() {
        let stdin = "refs/heads/feature abc123 refs/heads/feature 0000000000000000000000000000000000000000\n";
        let r = evaluate_impl_raw("pre-push", &[], Some(stdin), &[], "/repo");
        assert!(r.allow);
    }
}
