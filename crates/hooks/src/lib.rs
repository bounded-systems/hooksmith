pub mod schema;
pub mod test_framework;

use anyhow::Result;
use schema::{GitHook, HookContext};

/// Main hook execution function
pub fn execute_hook(_hook_name: &str, args: Vec<String>) -> Result<()> {
    let context = HookContext::from_args(args)?;

    // Validate the hook context
    context.validate()?;

    // Execute the appropriate hook logic
    match context.hook {
        GitHook::PreCommit => execute_pre_commit(&context)?,
        GitHook::CommitMsg => execute_commit_msg(&context)?,
        GitHook::PrePush => execute_pre_push(&context)?,
        GitHook::PostCommit => execute_post_commit(&context)?,
        GitHook::PostCheckout => execute_post_checkout(&context)?,
        GitHook::PrepareCommitMsg => execute_prepare_commit_msg(&context)?,
        GitHook::PostMerge => execute_post_merge(&context)?,
        GitHook::PreRebase => execute_pre_rebase(&context)?,
        GitHook::PostRebase => execute_post_rebase(&context)?,
        GitHook::PostRewrite => execute_post_rewrite(&context)?,
        GitHook::ApplyPatchMsg => execute_applypatch_msg(&context)?,
        GitHook::PreApplyPatch => execute_pre_applypatch(&context)?,
        GitHook::PostApplyPatch => execute_post_applypatch(&context)?,
        GitHook::PreMergeCommit => execute_pre_merge_commit(&context)?,
        GitHook::ReferenceTransaction => execute_reference_transaction(&context)?,
        GitHook::SendEmailValidate => execute_sendemail_validate(&context)?,
        GitHook::FSMonitorWatchman => execute_fsmonitor_watchman(&context)?,
        GitHook::PostIndexChange => execute_post_index_change(&context)?,
        // Server-side hooks
        GitHook::PreReceive => execute_pre_receive(&context)?,
        GitHook::Update => execute_update(&context)?,
        GitHook::PostReceive => execute_post_receive(&context)?,
        GitHook::PostUpdate => execute_post_update(&context)?,
        GitHook::PushToCheckout => execute_push_to_checkout(&context)?,
    }

    Ok(())
}

// Hook execution functions (currently all no-ops)
fn execute_pre_commit(_context: &HookContext) -> Result<()> {
    println!("✅ Pre-commit hook (no-op mode) - would validate directory structure");
    Ok(())
}

fn execute_commit_msg(_context: &HookContext) -> Result<()> {
    println!("✅ Commit-msg hook (no-op mode) - would validate commit message");
    Ok(())
}

fn execute_pre_push(_context: &HookContext) -> Result<()> {
    println!("✅ Pre-push hook (no-op mode) - would validate both directory and file structure");
    Ok(())
}

fn execute_post_commit(_context: &HookContext) -> Result<()> {
    println!("✅ Post-commit hook (no-op mode) - would perform post-commit actions");
    Ok(())
}

fn execute_post_checkout(_context: &HookContext) -> Result<()> {
    println!("✅ Post-checkout hook (no-op mode) - would perform post-checkout actions");
    Ok(())
}

fn execute_prepare_commit_msg(_context: &HookContext) -> Result<()> {
    println!("✅ Prepare-commit-msg hook (no-op mode) - would prepare commit message");
    Ok(())
}

fn execute_post_merge(_context: &HookContext) -> Result<()> {
    println!("✅ Post-merge hook (no-op mode) - would perform post-merge actions");
    Ok(())
}

fn execute_pre_rebase(_context: &HookContext) -> Result<()> {
    println!("✅ Pre-rebase hook (no-op mode) - would validate rebase operation");
    Ok(())
}

fn execute_post_rebase(_context: &HookContext) -> Result<()> {
    println!("✅ Post-rebase hook (no-op mode) - would perform post-rebase actions");
    Ok(())
}

fn execute_post_rewrite(_context: &HookContext) -> Result<()> {
    println!("✅ Post-rewrite hook (no-op mode) - would perform post-rewrite actions");
    Ok(())
}

fn execute_applypatch_msg(_context: &HookContext) -> Result<()> {
    println!("✅ Applypatch-msg hook (no-op mode) - would validate patch message");
    Ok(())
}

fn execute_pre_applypatch(_context: &HookContext) -> Result<()> {
    println!("✅ Pre-applypatch hook (no-op mode) - would validate before patch application");
    Ok(())
}

fn execute_post_applypatch(_context: &HookContext) -> Result<()> {
    println!("✅ Post-applypatch hook (no-op mode) - would perform post-patch actions");
    Ok(())
}

fn execute_pre_merge_commit(_context: &HookContext) -> Result<()> {
    println!("✅ Pre-merge-commit hook (no-op mode) - would validate merge commit");
    Ok(())
}

fn execute_reference_transaction(_context: &HookContext) -> Result<()> {
    println!("✅ Reference-transaction hook (no-op mode) - would validate reference transactions");
    Ok(())
}

fn execute_sendemail_validate(_context: &HookContext) -> Result<()> {
    println!("✅ Sendemail-validate hook (no-op mode) - would validate email");
    Ok(())
}

fn execute_fsmonitor_watchman(_context: &HookContext) -> Result<()> {
    println!("✅ Fsmonitor-watchman hook (no-op mode) - would handle file system monitoring");
    Ok(())
}

fn execute_post_index_change(_context: &HookContext) -> Result<()> {
    println!("✅ Post-index-change hook (no-op mode) - would handle index changes");
    Ok(())
}

// Server-side hook functions
fn execute_pre_receive(_context: &HookContext) -> Result<()> {
    println!("✅ Pre-receive hook (no-op mode) - would validate incoming references");
    Ok(())
}

fn execute_update(_context: &HookContext) -> Result<()> {
    println!("✅ Update hook (no-op mode) - would validate single reference update");
    Ok(())
}

fn execute_post_receive(_context: &HookContext) -> Result<()> {
    println!("✅ Post-receive hook (no-op mode) - would perform post-receive actions");
    Ok(())
}

fn execute_post_update(_context: &HookContext) -> Result<()> {
    println!("✅ Post-update hook (no-op mode) - would perform post-update actions");
    Ok(())
}

fn execute_push_to_checkout(_context: &HookContext) -> Result<()> {
    println!("✅ Push-to-checkout hook (no-op mode) - would handle push to working directory");
    Ok(())
}

/// Run comprehensive hook tests
pub async fn run_hook_tests() -> Result<()> {
    use test_framework::HookTestFramework;

    let mut framework = HookTestFramework::new()?;

    // Test all individual hooks
    framework.test_all_hooks().await?;

    // Test Git operations
    framework.test_git_operations().await?;

    // Export results
    let results = framework.export_results()?;
    std::fs::write("hook-test-results.json", results)?;

    println!("📄 Test results exported to hook-test-results.json");

    Ok(())
}

/// Get hook metadata for a specific hook
pub fn get_hook_metadata(hook_name: &str) -> Result<schema::HookMetadata> {
    let _hook = GitHook::from_name(hook_name)?;
    let context = HookContext::from_args(vec![hook_name.to_string()])?;
    Ok(context.metadata())
}

/// List all available hooks
pub fn list_hooks() -> Vec<String> {
    vec![
        "applypatch-msg".to_string(),
        "pre-applypatch".to_string(),
        "post-applypatch".to_string(),
        "pre-commit".to_string(),
        "prepare-commit-msg".to_string(),
        "commit-msg".to_string(),
        "post-commit".to_string(),
        "pre-merge-commit".to_string(),
        "pre-rebase".to_string(),
        "post-rebase".to_string(),
        "post-rewrite".to_string(),
        "post-checkout".to_string(),
        "post-merge".to_string(),
        "pre-push".to_string(),
        "pre-receive".to_string(),
        "update".to_string(),
        "post-receive".to_string(),
        "post-update".to_string(),
        "push-to-checkout".to_string(),
        "sendemail-validate".to_string(),
        "fsmonitor-watchman".to_string(),
        "reference-transaction".to_string(),
        "post-index-change".to_string(),
    ]
}
