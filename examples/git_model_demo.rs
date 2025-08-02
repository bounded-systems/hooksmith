//! Git Model Demo
//!
//! This example demonstrates how to use the Git file states, actions, and hooks model
//! for contract validation and analysis, including Lefthook integration.

use hooksmith::modules::git_model::{diagrams, *};

fn main() {
    println!("🔹 Git File States, Actions, and Hooks Model Demo\n");

    // Example 1: Basic usage with individual functions
    println!("📋 Example 1: Basic API Usage");
    let staged_actions = allowed_actions(FileStateKind::Staged);
    println!("  Staged file actions: {:?}", staged_actions);

    let commit_hooks = hooks_for_action(ActionKind::Commit);
    println!(
        "  Commit hooks: {}",
        commit_hooks
            .iter()
            .map(|h| format!(
                "{} ({})",
                h.hook.filename(),
                if h.can_block {
                    "can block"
                } else {
                    "cannot block"
                }
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Example 2: Check if a hook can block an action
    println!("\n📋 Example 2: Blocking Behavior");
    println!(
        "  Can PreCommit block Commit? {}",
        can_block(ActionKind::Commit, HookKind::PreCommit)
    );
    println!(
        "  Can PostCommit block Commit? {}",
        can_block(ActionKind::Commit, HookKind::PostCommit)
    );
    println!(
        "  Can PrePush block Push? {}",
        can_block(ActionKind::Push, HookKind::PrePush)
    );

    // Example 3: Get all actions and hooks for a file state
    println!("\n📋 Example 3: Actions and Hooks for File State");
    let staged_actions_hooks = actions_for_file(FileStateKind::Staged);
    for (action, hooks) in staged_actions_hooks {
        println!("  Action: {:?}", action);
        for hook in hooks {
            println!(
                "    Hook: {} ({})",
                hook.hook.filename(),
                if hook.can_block {
                    "can block"
                } else {
                    "cannot block"
                }
            );
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
        println!(
            "    Actions: {}",
            file_info
                .actions
                .iter()
                .map(|(action, _)| format!("{:?}", action))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "    Can be blocked: {}",
            if file_info.can_be_blocked() {
                "✅ Yes"
            } else {
                "❌ No"
            }
        );

        if !file_info.blocking_hooks().is_empty() {
            println!(
                "    Blocking hooks: {}",
                file_info
                    .blocking_hooks()
                    .iter()
                    .map(|h| h.filename())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        if !file_info.non_blocking_hooks().is_empty() {
            println!(
                "    Non-blocking hooks: {}",
                file_info
                    .non_blocking_hooks()
                    .iter()
                    .map(|h| h.filename())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }

    // Example 5: Lefthook Integration
    println!("\n📋 Example 5: Lefthook Integration");

    // Create a Lefthook configuration
    let mut config = LefthookHookConfig::default();
    config.files = Some("*.rs".to_string());
    config.parallel = Some(true);
    config.commands.insert(
        "rustfmt".to_string(),
        "cargo fmt --all -- --check".to_string(),
    );
    config.commands.insert(
        "clippy".to_string(),
        "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
    );
    config.skip = Some(SkipCondition::Conditions(vec!["CI".to_string()]));

    let files_with_config = vec![
        (
            "src/main.rs".to_string(),
            FileStateKind::Staged,
            Some(config.clone()),
        ),
        (
            "src/lib.rs".to_string(),
            FileStateKind::Staged,
            Some(config.clone()),
        ),
        ("docs/README.md".to_string(), FileStateKind::Untracked, None),
        ("Cargo.toml".to_string(), FileStateKind::Clean, None),
    ];

    let file_analyses_with_config = analyze_files_with_config(files_with_config);
    for file_info in file_analyses_with_config {
        println!("  File: {}", file_info.path);
        println!("    State: {:?}", file_info.state);
        println!(
            "    Has Lefthook config: {}",
            if file_info.lefthook_config.is_some() {
                "✅ Yes"
            } else {
                "❌ No"
            }
        );

        if let Some(config) = &file_info.lefthook_config {
            println!(
                "    File pattern: {}",
                config.files.as_deref().unwrap_or("None")
            );
            println!(
                "    Parallel: {}",
                config
                    .parallel
                    .map(|p| if p { "Yes" } else { "No" })
                    .unwrap_or_else(|| "Not set")
            );
            println!(
                "    Commands: {}",
                config
                    .commands
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "    Matches pattern: {}",
                if file_info.matches_file_pattern() {
                    "✅ Yes"
                } else {
                    "❌ No"
                }
            );
            println!(
                "    Should skip: {}",
                if file_info.should_skip() {
                    "✅ Yes"
                } else {
                    "❌ No"
                }
            );
            println!(
                "    Should run: {}",
                if file_info.should_run() {
                    "✅ Yes"
                } else {
                    "❌ No"
                }
            );
        }
    }

    // Example 6: Contract validation
    println!("\n🔒 Contract Validation Examples:");

    // Valid contract
    let result = validate_contract(
        FileStateKind::Staged,
        ActionKind::Commit,
        HookKind::PreCommit,
    );
    println!("  Staged + Commit + PreCommit: {}", result.description());

    // Invalid contracts
    let result = validate_contract(
        FileStateKind::Untracked,
        ActionKind::Commit,
        HookKind::PreCommit,
    );
    println!("  Untracked + Commit + PreCommit: {}", result.description());

    let result = validate_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PrePush);
    println!("  Staged + Commit + PrePush: {}", result.description());

    let result = validate_contract(
        FileStateKind::Staged,
        ActionKind::Commit,
        HookKind::PostCommit,
    );
    println!("  Staged + Commit + PostCommit: {}", result.description());

    // Example 7: Show all allowed actions for each file state
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
        println!(
            "  • {:?}: {}",
            state,
            actions
                .iter()
                .map(|a| format!("{:?}", a))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    // Example 8: Show all hooks for each action
    println!("\n📊 Hooks by Action:");
    for action in [
        ActionKind::Commit,
        ActionKind::Push,
        ActionKind::Merge,
        ActionKind::Rebase,
        ActionKind::Checkout,
        ActionKind::ReceivePush,
        ActionKind::P4Operations,
    ] {
        let hooks = hooks_for_action(action);
        let blocking_hooks: Vec<_> = hooks
            .iter()
            .filter(|h| h.can_block)
            .map(|h| h.hook.filename())
            .collect();
        let non_blocking_hooks: Vec<_> = hooks
            .iter()
            .filter(|h| !h.can_block)
            .map(|h| h.hook.filename())
            .collect();

        println!("  • {:?}:", action);
        println!(
            "    - All hooks: {}",
            hooks
                .iter()
                .map(|h| h.hook.filename())
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("    - Blocking: {}", blocking_hooks.join(", "));
        println!("    - Non-blocking: {}", non_blocking_hooks.join(", "));
    }

    // Example 9: Advanced contract validation function
    println!("\n🔒 Advanced Contract Validation Function Example:");
    validate_git_contract(
        FileStateKind::Staged,
        ActionKind::Commit,
        HookKind::PreCommit,
    );
    validate_git_contract(
        FileStateKind::Clean,
        ActionKind::Checkout,
        HookKind::PostCheckout,
    );
    validate_git_contract(
        FileStateKind::Untracked,
        ActionKind::Commit,
        HookKind::PreCommit,
    );
    validate_git_contract(FileStateKind::Staged, ActionKind::Commit, HookKind::PrePush);

    // Example 10: Lefthook Configuration Examples
    println!("\n📋 Example 10: Lefthook Configuration Examples");

    // Example 1: Rust project configuration
    let mut rust_config = LefthookHookConfig::default();
    rust_config.files = Some("*.rs".to_string());
    rust_config.parallel = Some(true);
    rust_config
        .commands
        .insert("fmt".to_string(), "cargo fmt --all -- --check".to_string());
    rust_config.commands.insert(
        "clippy".to_string(),
        "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
    );
    rust_config.commands.insert(
        "test".to_string(),
        "cargo test --all-targets --all-features".to_string(),
    );

    println!("  Rust Project Config:");
    println!(
        "    Files: {}",
        rust_config.files.as_deref().unwrap_or("None")
    );
    println!(
        "    Parallel: {}",
        rust_config
            .parallel
            .map(|p| if p { "Yes" } else { "No" })
            .unwrap_or_else(|| "Not set")
    );
    println!(
        "    Commands: {}",
        rust_config
            .commands
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Example 2: TypeScript project configuration
    let mut ts_config = LefthookHookConfig::default();
    ts_config.files = Some("src/**/*.{ts,tsx}".to_string());
    ts_config.parallel = Some(false);
    ts_config
        .commands
        .insert("lint".to_string(), "npm run lint".to_string());
    ts_config
        .commands
        .insert("type-check".to_string(), "npm run type-check".to_string());
    ts_config.skip = Some(SkipCondition::Conditions(vec!["WIP".to_string()]));

    println!("  TypeScript Project Config:");
    println!(
        "    Files: {}",
        ts_config.files.as_deref().unwrap_or("None")
    );
    println!(
        "    Parallel: {}",
        ts_config
            .parallel
            .map(|p| if p { "Yes" } else { "No" })
            .unwrap_or_else(|| "Not set")
    );
    println!(
        "    Commands: {}",
        ts_config
            .commands
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!("    Skip: {:?}", ts_config.skip);

    // Example 11: Detailed LefthookCommand Examples
    println!("\n📋 Example 11: Detailed LefthookCommand Examples");

    // Create a detailed Rust command
    let mut rustfmt_cmd = LefthookCommand::new("cargo fmt --all -- --check".to_string());
    rustfmt_cmd.files = Some("*.rs".to_string());
    rustfmt_cmd.priority = Some(10);
    rustfmt_cmd.file_types = vec!["rust".to_string()];
    rustfmt_cmd
        .env
        .insert("RUST_BACKTRACE".to_string(), "1".to_string());
    rustfmt_cmd.fail_text =
        Some("Code formatting check failed. Run 'cargo fmt' to fix.".to_string());
    rustfmt_cmd.stage_fixed = Some(true);

    println!("  Rustfmt Command:");
    println!("    Run: {}", rustfmt_cmd.run);
    println!(
        "    Files: {}",
        rustfmt_cmd.files.as_deref().unwrap_or("None")
    );
    println!("    Priority: {}", rustfmt_cmd.execution_priority());
    println!("    File Types: {}", rustfmt_cmd.file_types.join(", "));
    println!("    Environment: {:?}", rustfmt_cmd.env);
    println!(
        "    Fail Text: {}",
        rustfmt_cmd.fail_text.as_deref().unwrap_or("None")
    );
    println!(
        "    Stage Fixed: {}",
        rustfmt_cmd.stage_fixed.unwrap_or(false)
    );
    println!(
        "    Matches src/main.rs: {}",
        rustfmt_cmd.matches_file("src/main.rs")
    );
    println!(
        "    Matches src/main.py: {}",
        rustfmt_cmd.matches_file("src/main.py")
    );

    // Create a detailed TypeScript command
    let mut ts_lint_cmd = LefthookCommand::new("npm run lint".to_string());
    ts_lint_cmd.glob = vec!["src/**/*.ts".to_string(), "src/**/*.tsx".to_string()];
    ts_lint_cmd.priority = Some(5);
    ts_lint_cmd.file_types = vec!["typescript".to_string()];
    ts_lint_cmd.exclude = Some(ExcludeCondition::Patterns(vec![
        "*.test.ts".to_string(),
        "*.spec.ts".to_string(),
    ]));
    ts_lint_cmd.interactive = Some(false);
    ts_lint_cmd.use_stdin = Some(true);

    println!("  TypeScript Lint Command:");
    println!("    Run: {}", ts_lint_cmd.run);
    println!("    Glob: {}", ts_lint_cmd.glob.join(", "));
    println!("    Priority: {}", ts_lint_cmd.execution_priority());
    println!("    File Types: {}", ts_lint_cmd.file_types.join(", "));
    println!("    Exclude: {:?}", ts_lint_cmd.exclude);
    println!(
        "    Interactive: {}",
        ts_lint_cmd.interactive.unwrap_or(false)
    );
    println!("    Use Stdin: {}", ts_lint_cmd.use_stdin.unwrap_or(false));
    println!(
        "    Matches src/components/Button.tsx: {}",
        ts_lint_cmd.matches_file("src/components/Button.tsx")
    );
    println!(
        "    Matches src/components/Button.test.ts: {}",
        ts_lint_cmd.matches_file("src/components/Button.test.ts")
    );

    // Example 12: Advanced LefthookHookConfig with Detailed Commands
    println!("\n📋 Example 12: Advanced LefthookHookConfig with Detailed Commands");

    let mut advanced_config = LefthookHookConfig::default();
    advanced_config.parallel = Some(true);
    advanced_config.files = Some("*.{rs,ts,tsx}".to_string());

    // Add detailed commands
    advanced_config.add_command("rustfmt".to_string(), rustfmt_cmd.clone());
    advanced_config.add_command("tslint".to_string(), ts_lint_cmd.clone());

    // Add a test command
    let mut test_cmd = LefthookCommand::new("npm test".to_string());
    test_cmd.priority = Some(1); // Low priority, runs last
    test_cmd.file_types = vec!["test".to_string()];
    test_cmd.skip = Some(SkipCondition::Conditions(vec!["CI".to_string()]));
    advanced_config.add_command("test".to_string(), test_cmd);

    println!("  Advanced Config:");
    println!(
        "    Parallel: {}",
        advanced_config
            .parallel
            .map(|p| if p { "Yes" } else { "No" })
            .unwrap_or_else(|| "Not set")
    );
    println!(
        "    Files: {}",
        advanced_config.files.as_deref().unwrap_or("None")
    );
    println!("    Commands by Priority:");
    for (name, cmd) in advanced_config.commands_by_priority() {
        println!("      {} (priority: {})", name, cmd.execution_priority());
    }
    println!(
        "    All File Types: {}",
        advanced_config.all_file_types().join(", ")
    );
    println!(
        "    Has Incompatible File Types: {}",
        advanced_config.has_incompatible_file_types()
    );

    // Test commands for specific files
    println!(
        "    Commands for src/main.rs: {}",
        advanced_config.commands_for_file("src/main.rs").len()
    );
    println!(
        "    Commands for src/components/Button.tsx: {}",
        advanced_config
            .commands_for_file("src/components/Button.tsx")
            .len()
    );
    println!(
        "    Commands for src/components/Button.test.ts: {}",
        advanced_config
            .commands_for_file("src/components/Button.test.ts")
            .len()
    );

    // Example 13: File Substitution Examples
    println!("\n📋 Example 13: File Substitution Examples");

    // Create a file substitution context
    let context = FileSubstitutionContext::new(
        vec!["src/main.rs".to_string(), "src/lib.rs".to_string()],
        vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "docs/README.md".to_string(),
            "Cargo.toml".to_string(),
        ],
        vec!["src/main.rs".to_string()],
        vec!["src/lib.rs".to_string()],
    );

    println!("  File Substitution Context:");
    println!("    Files: {}", context.files.join(", "));
    println!("    All Files: {}", context.all_files.join(", "));
    println!("    Staged Files: {}", context.staged_files.join(", "));
    println!("    Push Files: {}", context.push_files.join(", "));

    // Test various command substitutions
    let commands = vec![
        "cargo fmt {files}",
        "npm run lint {all_files}",
        "cargo test {staged_files}",
        "npm run build {push_files}",
        "cargo fmt {files} && npm run lint {all_files}",
    ];

    println!("  Command Substitutions:");
    for command in commands {
        let resolved = context.substitute_files(command);
        let used_subs = context.get_used_substitutions(command);
        println!("    Original: {}", command);
        println!("    Resolved: {}", resolved);
        println!("    Used substitutions: {}", used_subs.join(", "));
        println!();
    }

    // Test compatibility checking
    println!("  Compatibility Checking:");
    let compatible_commands = vec![
        "cargo fmt {files}",
        "npm run lint {all_files}",
        "cargo test {staged_files}",
        "npm run build {push_files}",
    ];

    let incompatible_commands = vec![
        "cargo fmt {staged_files} {push_files}",
        "npm run lint {push_files} {staged_files}",
    ];

    for command in compatible_commands {
        println!(
            "    {}: {}",
            command,
            if is_run_files_compatible(command) {
                "✅ Compatible"
            } else {
                "❌ Incompatible"
            }
        );
    }

    for command in incompatible_commands {
        println!(
            "    {}: {}",
            command,
            if is_run_files_compatible(command) {
                "✅ Compatible"
            } else {
                "❌ Incompatible"
            }
        );
    }

    // Example 14: LefthookCommand with File Substitutions
    println!("\n📋 Example 14: LefthookCommand with File Substitutions");

    // Create commands with file substitutions
    let mut fmt_cmd = LefthookCommand::new("cargo fmt {files}".to_string());
    fmt_cmd.files = Some("*.rs".to_string());
    fmt_cmd.priority = Some(10);

    let mut lint_cmd = LefthookCommand::new("npm run lint {all_files}".to_string());
    lint_cmd.glob = vec!["src/**/*.{ts,tsx}".to_string()];
    lint_cmd.priority = Some(5);

    let mut test_cmd = LefthookCommand::new("cargo test {staged_files}".to_string());
    test_cmd.priority = Some(1);

    let incompatible_cmd =
        LefthookCommand::new("cargo fmt {staged_files} {push_files}".to_string());

    println!("  Rustfmt Command:");
    println!("    Run: {}", fmt_cmd.run);
    println!(
        "    Uses file substitutions: {}",
        fmt_cmd.uses_file_substitutions()
    );
    println!("    Is compatible: {}", fmt_cmd.is_files_compatible());
    println!("    Resolved: {}", fmt_cmd.get_resolved_command(&context));
    println!(
        "    Used substitutions: {}",
        fmt_cmd.get_used_substitutions(&context).join(", ")
    );

    println!("  Lint Command:");
    println!("    Run: {}", lint_cmd.run);
    println!(
        "    Uses file substitutions: {}",
        lint_cmd.uses_file_substitutions()
    );
    println!("    Is compatible: {}", lint_cmd.is_files_compatible());
    println!("    Resolved: {}", lint_cmd.get_resolved_command(&context));
    println!(
        "    Used substitutions: {}",
        lint_cmd.get_used_substitutions(&context).join(", ")
    );

    println!("  Test Command:");
    println!("    Run: {}", test_cmd.run);
    println!(
        "    Uses file substitutions: {}",
        test_cmd.uses_file_substitutions()
    );
    println!("    Is compatible: {}", test_cmd.is_files_compatible());
    println!("    Resolved: {}", test_cmd.get_resolved_command(&context));
    println!(
        "    Used substitutions: {}",
        test_cmd.get_used_substitutions(&context).join(", ")
    );

    println!("  Incompatible Command:");
    println!("    Run: {}", incompatible_cmd.run);
    println!(
        "    Uses file substitutions: {}",
        incompatible_cmd.uses_file_substitutions()
    );
    println!(
        "    Is compatible: {}",
        incompatible_cmd.is_files_compatible()
    );
    println!(
        "    Used substitutions: {}",
        incompatible_cmd.get_used_substitutions(&context).join(", ")
    );

    // Example 15: Real-world Lefthook Configuration with File Substitutions
    println!("\n📋 Example 15: Real-world Lefthook Configuration with File Substitutions");

    let mut real_world_config = LefthookHookConfig::default();
    real_world_config.parallel = Some(true);

    // Add commands with file substitutions
    let mut pre_commit_fmt = LefthookCommand::new("cargo fmt --check {files}".to_string());
    pre_commit_fmt.files = Some("*.rs".to_string());
    pre_commit_fmt.priority = Some(10);
    pre_commit_fmt.fail_text =
        Some("Code formatting check failed. Run 'cargo fmt' to fix.".to_string());

    let mut pre_commit_clippy =
        LefthookCommand::new("cargo clippy --all-targets --all-features -- {files}".to_string());
    pre_commit_clippy.files = Some("*.rs".to_string());
    pre_commit_clippy.priority = Some(5);
    pre_commit_clippy.fail_text =
        Some("Clippy found issues. Fix them before committing.".to_string());

    let mut pre_push_test = LefthookCommand::new("cargo test {staged_files}".to_string());
    pre_push_test.priority = Some(1);
    pre_push_test.fail_text = Some("Tests failed. Fix them before pushing.".to_string());

    real_world_config.add_command("fmt".to_string(), pre_commit_fmt);
    real_world_config.add_command("clippy".to_string(), pre_commit_clippy);
    real_world_config.add_command("test".to_string(), pre_push_test);

    println!("  Real-world Config:");
    println!(
        "    Parallel: {}",
        real_world_config
            .parallel
            .map(|p| if p { "Yes" } else { "No" })
            .unwrap_or_else(|| "Not set")
    );
    println!("    Commands by Priority:");
    for (name, cmd) in real_world_config.commands_by_priority() {
        println!("      {} (priority: {})", name, cmd.execution_priority());
        println!("        Run: {}", cmd.run);
        println!("        Resolved: {}", cmd.get_resolved_command(&context));
        println!(
            "        Fail Text: {}",
            cmd.fail_text.as_deref().unwrap_or("None")
        );
    }

    // Example 16: Advanced Skip/Only Conditions
    println!("\n📋 Example 16: Advanced Skip/Only Conditions");

    // Create a skip checker with different Git states
    let pre_commit_state =
        || GitState::new(GitRepoState::Normal, "feature/new-feature".to_string());
    let pre_push_state = || GitState::new(GitRepoState::Normal, "main".to_string());
    let skip_checker_pre_commit = SkipChecker::new(pre_commit_state);
    let skip_checker_pre_push = SkipChecker::new(pre_push_state);

    println!("  Skip Checker Examples:");

    // Test boolean conditions
    let always_skip = AdvancedSkipCondition::Always;
    let never_skip = AdvancedSkipCondition::Conditions(vec![ConditionValue::Boolean(false)]);

    println!(
        "    Always skip (pre-commit): {}",
        skip_checker_pre_commit.check(Some(&always_skip), None)
    );
    println!(
        "    Never skip (pre-commit): {}",
        skip_checker_pre_commit.check(Some(&never_skip), None)
    );

    // Test string conditions
    let pre_commit_skip =
        AdvancedSkipCondition::Conditions(vec![ConditionValue::String("pre-commit".to_string())]);
    let pre_push_skip =
        AdvancedSkipCondition::Conditions(vec![ConditionValue::String("pre-push".to_string())]);

    println!(
        "    Skip on pre-commit (pre-commit state): {}",
        skip_checker_pre_commit.check(Some(&pre_commit_skip), None)
    );
    println!(
        "    Skip on pre-push (pre-commit state): {}",
        skip_checker_pre_commit.check(Some(&pre_push_skip), None)
    );
    println!(
        "    Skip on pre-push (pre-push state): {}",
        skip_checker_pre_push.check(Some(&pre_push_skip), None)
    );

    // Test reference conditions
    let feature_branch_skip = AdvancedSkipCondition::Conditions(vec![ConditionValue::Reference {
        ref_pattern: "feature/*".to_string(),
    }]);
    let main_branch_skip = AdvancedSkipCondition::Conditions(vec![ConditionValue::Reference {
        ref_pattern: "main".to_string(),
    }]);

    println!(
        "    Skip on feature/* (feature/new-feature): {}",
        skip_checker_pre_commit.check(Some(&feature_branch_skip), None)
    );
    println!(
        "    Skip on main (feature/new-feature): {}",
        skip_checker_pre_commit.check(Some(&main_branch_skip), None)
    );
    println!(
        "    Skip on main (main): {}",
        skip_checker_pre_push.check(Some(&main_branch_skip), None)
    );

    // Test only conditions
    let only_pre_commit =
        AdvancedOnlyCondition::Conditions(vec![ConditionValue::String("pre-commit".to_string())]);
    let only_main_branch = AdvancedOnlyCondition::Conditions(vec![ConditionValue::Reference {
        ref_pattern: "main".to_string(),
    }]);

    println!(
        "    Only on pre-commit (pre-commit state): {}",
        skip_checker_pre_commit.check(None, Some(&only_pre_commit))
    );
    println!(
        "    Only on pre-commit (pre-push state): {}",
        skip_checker_pre_push.check(None, Some(&only_pre_commit))
    );
    println!(
        "    Only on main branch (feature/new-feature): {}",
        skip_checker_pre_commit.check(None, Some(&only_main_branch))
    );
    println!(
        "    Only on main branch (main): {}",
        skip_checker_pre_push.check(None, Some(&only_main_branch))
    );

    // Example 17: LefthookCommand with Advanced Skip/Only Conditions
    println!("\n📋 Example 17: LefthookCommand with Advanced Skip/Only Conditions");

    // Create commands with advanced conditions
    let mut fmt_command = LefthookCommand::new("cargo fmt {files}".to_string());
    fmt_command.advanced_skip = Some(AdvancedSkipCondition::Conditions(vec![
        ConditionValue::String("pre-push".to_string()),
    ]));

    let mut test_command = LefthookCommand::new("cargo test {staged_files}".to_string());
    test_command.advanced_only = Some(AdvancedOnlyCondition::Conditions(vec![
        ConditionValue::Reference {
            ref_pattern: "main".to_string(),
        },
    ]));

    let mut lint_command = LefthookCommand::new("cargo clippy {files}".to_string());
    lint_command.advanced_skip = Some(AdvancedSkipCondition::Conditions(vec![
        ConditionValue::Command {
            run: "git diff --cached --quiet".to_string(),
        },
    ]));

    println!("  Format Command:");
    println!("    Run: {}", fmt_command.run);
    println!(
        "    Skip on pre-push (pre-commit state): {}",
        fmt_command.should_skip_advanced(&skip_checker_pre_commit)
    );
    println!(
        "    Skip on pre-push (pre-push state): {}",
        fmt_command.should_skip_advanced(&skip_checker_pre_push)
    );

    println!("  Test Command:");
    println!("    Run: {}", test_command.run);
    println!(
        "    Only on main (feature/new-feature): {}",
        test_command.should_skip_advanced(&skip_checker_pre_commit)
    );
    println!(
        "    Only on main (main): {}",
        test_command.should_skip_advanced(&skip_checker_pre_push)
    );

    println!("  Lint Command:");
    println!("    Run: {}", lint_command.run);
    println!(
        "    Skip if no staged changes (pre-commit state): {}",
        lint_command.should_skip_advanced(&skip_checker_pre_commit)
    );

    // Example 18: Real-world Advanced Configuration
    println!("\n📋 Example 18: Real-world Advanced Configuration");

    let mut advanced_config = LefthookHookConfig::default();
    advanced_config.parallel = Some(true);

    // Pre-commit hook that only runs on feature branches
    let mut pre_commit_fmt = LefthookCommand::new("cargo fmt --check {files}".to_string());
    pre_commit_fmt.files = Some("*.rs".to_string());
    pre_commit_fmt.priority = Some(10);
    pre_commit_fmt.advanced_only = Some(AdvancedOnlyCondition::Conditions(vec![
        ConditionValue::Reference {
            ref_pattern: "feature/*".to_string(),
        },
    ]));

    // Pre-push hook that only runs on main branch
    let mut pre_push_test = LefthookCommand::new("cargo test {staged_files}".to_string());
    pre_push_test.priority = Some(5);
    pre_push_test.advanced_only = Some(AdvancedOnlyCondition::Conditions(vec![
        ConditionValue::Reference {
            ref_pattern: "main".to_string(),
        },
    ]));

    // Hook that skips if no changes
    let mut conditional_lint = LefthookCommand::new("cargo clippy {files}".to_string());
    conditional_lint.priority = Some(1);
    conditional_lint.advanced_skip = Some(AdvancedSkipCondition::Conditions(vec![
        ConditionValue::Command {
            run: "git diff --cached --quiet".to_string(),
        },
    ]));

    advanced_config.add_command("fmt".to_string(), pre_commit_fmt);
    advanced_config.add_command("test".to_string(), pre_push_test);
    advanced_config.add_command("lint".to_string(), conditional_lint);

    println!("  Advanced Config with Conditional Execution:");
    println!(
        "    Parallel: {}",
        advanced_config
            .parallel
            .map(|p| if p { "Yes" } else { "No" })
            .unwrap_or_else(|| "Not set")
    );
    println!("    Commands by Priority:");

    for (name, cmd) in advanced_config.commands_by_priority() {
        println!("      {} (priority: {})", name, cmd.execution_priority());
        println!("        Run: {}", cmd.run);

        // Test with different states
        let pre_commit_checker = SkipChecker::new(|| {
            GitState::new(GitRepoState::Normal, "feature/new-feature".to_string())
        });
        let pre_push_checker =
            SkipChecker::new(|| GitState::new(GitRepoState::Normal, "main".to_string()));

        println!(
            "        Skip on pre-commit (feature branch): {}",
            cmd.should_skip_advanced(&pre_commit_checker)
        );
        println!(
            "        Skip on pre-push (main branch): {}",
            cmd.should_skip_advanced(&pre_push_checker)
        );
    }

    // Example 19: Diagram Generation
    println!("\n📋 Example 19: Diagram Generation");

    println!("  Generating file state diagram (DOT format):");
    let dot_diagram = diagrams::export_file_state_diagram();
    println!(
        "    {}",
        dot_diagram
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n    ")
    );
    println!("    ... (truncated)");

    println!("\n  Generating file state diagram (Mermaid format):");
    let mermaid_diagram = diagrams::export_file_state_mermaid();
    println!(
        "    {}",
        mermaid_diagram
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n    ")
    );
    println!("    ... (truncated)");

    println!("\n  Generating comprehensive diagram:");
    let comprehensive_diagram = diagrams::export_comprehensive_diagram();
    println!(
        "    {}",
        comprehensive_diagram
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n    ")
    );
    println!("    ... (truncated)");

    println!("\n  Generating commit workflow diagram:");
    let commit_workflow = diagrams::export_commit_workflow_diagram();
    println!(
        "    {}",
        commit_workflow
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n    ")
    );
    println!("    ... (truncated)");

    println!("\n  Generating skip/only conditions diagram:");
    let skip_only_diagram = diagrams::export_skip_only_diagram();
    println!(
        "    {}",
        skip_only_diagram
            .lines()
            .take(5)
            .collect::<Vec<_>>()
            .join("\n    ")
    );
    println!("    ... (truncated)");

    // Example 20: Export Diagrams to Files
    println!("\n📋 Example 20: Export Diagrams to Files");

    use std::fs;
    use std::path::Path;

    let diagrams_dir = "diagrams";
    if !Path::new(diagrams_dir).exists() {
        fs::create_dir(diagrams_dir).expect("Failed to create diagrams directory");
    }

    // Export DOT diagram
    let dot_path = format!("{}/git_file_states.dot", diagrams_dir);
    fs::write(&dot_path, diagrams::export_file_state_diagram())
        .expect("Failed to write DOT diagram");
    println!("    ✅ Exported DOT diagram to: {}", dot_path);

    // Export Mermaid diagrams
    let mermaid_path = format!("{}/git_file_states.md", diagrams_dir);
    let mermaid_content = format!(
        "# Git File States Diagram\n\n```mermaid\n{}\n```\n",
        diagrams::export_file_state_mermaid()
    );
    fs::write(&mermaid_path, mermaid_content).expect("Failed to write Mermaid diagram");
    println!("    ✅ Exported Mermaid diagram to: {}", mermaid_path);

    // Export comprehensive diagram
    let comprehensive_path = format!("{}/git_comprehensive.md", diagrams_dir);
    let comprehensive_content = format!(
        "# Git Comprehensive Diagram\n\n```mermaid\n{}\n```\n",
        diagrams::export_comprehensive_diagram()
    );
    fs::write(&comprehensive_path, comprehensive_content)
        .expect("Failed to write comprehensive diagram");
    println!(
        "    ✅ Exported comprehensive diagram to: {}",
        comprehensive_path
    );

    // Export commit workflow
    let workflow_path = format!("{}/git_commit_workflow.md", diagrams_dir);
    let workflow_content = format!(
        "# Git Commit Workflow\n\n```mermaid\n{}\n```\n",
        diagrams::export_commit_workflow_diagram()
    );
    fs::write(&workflow_path, workflow_content).expect("Failed to write commit workflow diagram");
    println!("    ✅ Exported commit workflow to: {}", workflow_path);

    // Export skip/only conditions
    let skip_only_path = format!("{}/git_skip_only_conditions.md", diagrams_dir);
    let skip_only_content = format!(
        "# Git Skip/Only Conditions\n\n```mermaid\n{}\n```\n",
        diagrams::export_skip_only_diagram()
    );
    fs::write(&skip_only_path, skip_only_content)
        .expect("Failed to write skip/only conditions diagram");
    println!(
        "    ✅ Exported skip/only conditions to: {}",
        skip_only_path
    );

    println!(
        "\n    📁 All diagrams exported to '{}' directory",
        diagrams_dir
    );
    println!("    💡 You can render these diagrams using:");
    println!("       - Graphviz: dot -Tpng git_file_states.dot -o git_file_states.png");
    println!("       - Mermaid: Use the .md files in GitHub, GitLab, or Mermaid Live Editor");
    println!("       - VS Code: Install Mermaid extension to preview .md files");
}

/// Advanced contract validation function that can be used in real-world scenarios
fn validate_git_contract(state: FileStateKind, action: ActionKind, hook: HookKind) {
    println!(
        "  Validating contract: {:?} + {:?} + {:?}",
        state, action, hook
    );

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
        ContractValidation::HookSkipped => {
            println!("    ⏭️  {}", result.description());
        }
    }
}
