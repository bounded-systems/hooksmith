use hooksmith::modules::git_native::GitNativeValidator;

fn main() {
    // Test the comprehensive coverage with all the new concerns
    let comprehensive_concerns = vec![
        // Core Git Objects
        "blob", "tree", "commit", "tag",
        // Tree Entry Types
        "tree-file", "tree-executable", "tree-symlink", "tree-directory", "tree-submodule",
        // Metadata Types
        "ref", "note", "attr", "index", "stash", "worktree", "remote", "branch", "head", "reflog",
        // Structured Attributes
        "attr-line-ending-normalization", "attr-diff-strategy", "attr-merge-strategy",
        "attr-export-control", "attr-filter-driver", "attr-external-tool-hint", "attr-locking-hint",
        // Storage Files
        "head-pointer", "index-entry", "index-file", "index-stage", "ref-branch", "ref-remote-branch",
        "ref-tag", "ref-packed", "note-ref", "attr-repo-only", "ignore-repo-only", "config-local",
        "hook-script", "hook-lifecycle", "ref-log", "ref-log-entry", "worktree-meta", "worktree-lock",
        "rebase-plan", "rr-cache-entry", "merge-state", "merge-head", "orig-head-pointer",
        "commit-message-draft", "fetch-head-pointer", "repo-description", "fs-monitor-state", "shallow-clone-depth",
        // Pattern Types
        "tree-ignore-pattern", "ignore-global-pattern", "attr-pattern", "attr-global",
        // Remote & Network
        "remote-origin", "remote-config", "push-strategy-config", "credential-helper-config", "remote-url-alias",
        // State Management
        "stash-entry", "stash-ref", "stash-meta", "rebase-step", "merge-conflict-marker",
        "bisect-state", "bisect-log", "tag-object", "worktree-index", "worktree-branch-link",
        "hook-trigger", "index-conflict", "index-mode", "config-global", "config-system",
        // Config Sections
        "config-user", "config-core", "config-branch", "config-remote", "config-init", "config-color",
        "config-alias", "config-diff", "config-merge", "config-gpg", "config-commit", "config-pull",
        "config-push", "config-rebase", "config-fetch", "config-status", "config-tar", "config-rerere",
        "config-advice", "config-interactive", "config-submodule", "config-filter", "config-include",
        "config-credential", "config-http", "config-url", "config-safe", "config-notes", "config-gc",
        "config-maintenance", "config-pager", "config-worktree"
    ];

    let concerns: Vec<String> = comprehensive_concerns.iter().map(|s| s.to_string()).collect();

    match GitNativeValidator::validate_concerns(&concerns) {
        Ok(()) => println!("✅ All {} comprehensive concerns are valid!", concerns.len()),
        Err(e) => println!("❌ Validation failed: {}", e),
    }

    println!("\n📊 Coverage Summary:");
    println!("  - Core Git Objects: 4");
    println!("  - Tree Entry Types: 5");
    println!("  - Metadata Types: 10");
    println!("  - Config Sections: 30+");
    println!("  - Structured Attributes: 7");
    println!("  - Storage Files: 28");
    println!("  - Pattern Types: 4");
    println!("  - Remote & Network: 5");
    println!("  - State Management: 15");
    println!("  - **Total: {} concerns**", concerns.len());

    println!("\n🎯 All concerns are Git-native and map directly to Git's internal structure!");
}
