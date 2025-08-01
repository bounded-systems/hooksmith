//! Git Model Demo
//!
//! This example demonstrates how to use the Git file states, actions, and hooks model
//! for contract validation and analysis.

use hooksmith::modules::git_model::*;

fn main() {
    println!("🔹 Git File States, Actions, and Hooks Model Demo\n");

    // Example 1: Basic usage with individual functions
    println!("📋 Example 1: Basic API Usage");
    let staged_actions = allowed_actions(FileStateKind::Staged);
    println!("  Staged file actions: {:?}", staged_actions);
    
    let commit_hooks = hooks_for_action(ActionKind::Commit);
    println!("  Commit hooks: {}", commit_hooks.iter().map(|h| format!("{} ({})", h.hook.filename(), if h.can_block { "can block" } else { "cannot block" })).collect::<Vec<_>>().join(", "));

    // Example 2: Check if a hook can block an action
    println!("\n📋 Example 2: Blocking Behavior");
    println!("  Can PreCommit block Commit? {}", can_block(ActionKind::Commit, HookKind::PreCommit));
    println!("  Can PostCommit block Commit? {}", can_block(ActionKind::Commit, HookKind::PostCommit));
    println!("  Can PrePush block Push? {}", can_block(ActionKind::Push, HookKind::PrePush));

    // Example 3: Get all actions and hooks for a file state
    println!("\n📋 Example 3: Actions and Hooks for File State");
    let staged_actions_hooks = actions_for_file(FileStateKind::Staged);
    for (action, hooks) in staged_actions_hooks {
        println!("  Action: {:?}", action);
        for hook in hooks {
            println!("    Hook: {} ({})", hook.hook.filename(), if hook.can_block { "can block" } else { "cannot block" });
        }
    }

    // Example 4: FileActionInfo - per-file analysis
    println!("\n📋 Example 4: Per-File Analysis with FileActionInfo");
    let files = vec![
        ("src/main.rs".to_string(), FileStateKind::Staged),
        ("src/lib.rs".to_string(), FileStateKind::ModifiedUnstaged),
        ("docs/README.md".to_string(), FileStateKind::Untracked),
        ("target/debug/app".to_string(), FileStateKind::Ignored),
    ];
    
    let file_analyses = analyze_files(files);
    for file_info in file_analyses {
        println!("  File: {}", file_info.path);
        println!("    State: {:?}", file_info.state);
        println!("    Actions: {}", file_info.actions.iter().map(|(action, _)| format!("{:?}", action)).collect::<Vec<_>>().join(", "));
        println!("    Can be blocked: {}", if file_info.can_be_blocked() { "✅ Yes" } else { "❌ No" });
        
        if !file_info.blocking_hooks().is_empty() {
            println!("    Blocking hooks: {}", file_info.blocking_hooks().iter().map(|h| h.filename()).collect::<Vec<_>>().join(", "));
        }
        
        if !file_info.non_blocking_hooks().is_empty() {
            println!("    Non-blocking hooks: {}", file_info.non_blocking_hooks().iter().map(|h| h.filename()).collect::<Vec<_>>().join(", "));
        }
    }

    // Example 5: Contract validation
    println!("\n🔒 Contract Validation Examples:");
    
    // Valid contract
    let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PreCommit);
    println!("  Staged + Commit + PreCommit: {}", result.description());
    
    // Invalid contracts
    let result = validate_contract(FileStateKind::Untracked, ActionKind::Commit, HookKind::PreCommit);
    println!("  Untracked + Commit + PreCommit: {}", result.description());
    
    let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PrePush);
    println!("  Staged + Commit + PrePush: {}", result.description());
    
    let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PostCommit);
    println!("  Staged + Commit + PostCommit: {}", result.description());

    // Example 6: Show all allowed actions for each file state
    println!("\n📊 Allowed Actions by File State:");
    for state in [
        FileStateKind::Clean,
        FileStateKind::ModifiedUnstaged,
        FileStateKind::Staged,
        FileStateKind::StagedAndModified,
        FileStateKind::Added,
        FileStateKind::DeletedStaged,
        FileStateKind::Untracked,
        FileStateKind::Ignored,
    ] {
        let actions = allowed_actions(state);
        println!("  • {:?}: {}", state, actions.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>().join(", "));
    }

    // Example 7: Show all hooks for each action
    println!("\n📊 Hooks by Action:");
    for action in [
        ActionKind::Commit,
        ActionKind::Push,
        ActionKind::Merge,
        ActionKind::Rebase,
        ActionKind::Checkout,
    ] {
        let hooks = hooks_for_action(action);
        let blocking_hooks: Vec<_> = hooks.iter().filter(|h| h.can_block).map(|h| h.hook.filename()).collect();
        let non_blocking_hooks: Vec<_> = hooks.iter().filter(|h| !h.can_block).map(|h| h.hook.filename()).collect();
        
        println!("  • {:?}:", action);
        println!("    - All hooks: {}", hooks.iter().map(|h| h.hook.filename()).collect::<Vec<_>>().join(", "));
        println!("    - Blocking: {}", blocking_hooks.join(", "));
        println!("    - Non-blocking: {}", non_blocking_hooks.join(", "));
    }

    // Example 8: Advanced contract validation function
    println!("\n🔒 Advanced Contract Validation Function Example:");
    validate_git_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PreCommit);
    validate_git_contract(FileStateKind::Clean, ActionKind::Checkout, HookKind::PostCheckout);
    validate_git_contract(FileStateKind::Untracked, ActionKind::Commit, HookKind::PreCommit);
    validate_git_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PrePush);
}

/// Advanced contract validation function that can be used in real-world scenarios
fn validate_git_contract(state: FileStateKind, action: ActionKind, hook: HookKind) {
    println!("  Validating contract: {:?} + {:?} + {:?}", state, action, hook);
    
    let result = validate_contract(state, action, hook);
    
    match result {
        ContractValidation::Valid => {
            println!("    ✅ {}", result.description());
        }
        ContractValidation::ActionNotAllowed => {
            println!("    ❌ {}", result.description());
        }
        ContractValidation::HookNotRelevant => {
            println!("    ❌ {}", result.description());
        }
        ContractValidation::HookCannotBlock => {
            println!("    ⚠️  {}", result.description());
        }
    }
} 
