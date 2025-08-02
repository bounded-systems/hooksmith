use hooksmith::modules::lefthook;
use serde_json::json;

#[test]
fn test_lefthook_hook_creation() {
    // Test basic creation
    let hook = lefthook::LefthookHook::new("cargo test".to_string());
    assert_eq!(hook.run, "cargo test");
    assert_eq!(hook.execution_priority(), 0); // Default priority

    // Test builder pattern
    let hook = lefthook::LefthookHook::new("cargo fmt".to_string())
        .with_files("*.rs".to_string())
        .with_priority(10)
        .with_fail_text("Formatting failed".to_string())
        .with_stage_fixed(true);

    assert_eq!(hook.run, "cargo fmt");
    assert_eq!(hook.files, Some("*.rs".to_string()));
    assert_eq!(hook.priority, Some(10));
    assert_eq!(hook.fail_text, Some("Formatting failed".to_string()));
    assert_eq!(hook.stage_fixed, Some(true));
    assert_eq!(hook.execution_priority(), 10);
}

#[test]
fn test_skip_conditions() {
    // Test boolean skip
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_skip(json!(true));
    assert!(hook.should_skip());
    assert!(!hook.should_run());

    // Test array skip
    let hook =
        lefthook::LefthookHook::new("echo test".to_string()).with_skip(json!(["merge", "rebase"]));
    assert!(hook.should_skip());
    assert!(!hook.should_run());

    // Test empty array skip
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_skip(json!([]));
    assert!(!hook.should_skip());
    assert!(hook.should_run());

    // Test no skip
    let hook = lefthook::LefthookHook::new("echo test".to_string());
    assert!(!hook.should_skip());
    assert!(hook.should_run());
}

#[test]
fn test_tags_parsing() {
    // Test string tag
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_tags(json!("test"));
    assert_eq!(hook.get_tags(), vec!["test"]);

    // Test array of tags
    let hook = lefthook::LefthookHook::new("echo test".to_string())
        .with_tags(json!(["test", "lint", "format"]));
    assert_eq!(hook.get_tags(), vec!["test", "lint", "format"]);

    // Test no tags
    let hook = lefthook::LefthookHook::new("echo test".to_string());
    assert_eq!(hook.get_tags(), vec![]);
}

#[test]
fn test_glob_patterns() {
    // Test string glob
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_glob(json!("*.rs"));
    assert_eq!(hook.get_glob_patterns(), vec!["*.rs"]);

    // Test array of globs
    let hook = lefthook::LefthookHook::new("echo test".to_string())
        .with_glob(json!(["*.rs", "*.toml", "*.md"]));
    assert_eq!(hook.get_glob_patterns(), vec!["*.rs", "*.toml", "*.md"]);

    // Test no globs
    let hook = lefthook::LefthookHook::new("echo test".to_string());
    assert_eq!(hook.get_glob_patterns(), vec![]);
}

#[test]
fn test_exclude_patterns() {
    // Test string exclude
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_exclude(json!("*.tmp"));
    assert_eq!(hook.get_exclude_patterns(), vec!["*.tmp"]);

    // Test array of excludes
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_exclude(json!([
        "*.tmp",
        "*.log",
        "node_modules/"
    ]));
    assert_eq!(
        hook.get_exclude_patterns(),
        vec!["*.tmp", "*.log", "node_modules/"]
    );

    // Test no excludes
    let hook = lefthook::LefthookHook::new("echo test".to_string());
    assert_eq!(hook.get_exclude_patterns(), vec![]);
}

#[test]
fn test_environment_variables() {
    let mut env = std::collections::HashMap::new();
    env.insert("RUST_BACKTRACE".to_string(), "1".to_string());
    env.insert("CARGO_TERM_COLOR".to_string(), "always".to_string());

    let hook = lefthook::LefthookHook::new("cargo test".to_string()).with_env(env.clone());

    assert_eq!(hook.env, Some(env));
}

#[test]
fn test_file_types() {
    let hook = lefthook::LefthookHook::new("echo test".to_string())
        .with_file_types(vec!["rust".to_string(), "toml".to_string()]);

    assert_eq!(
        hook.file_types,
        Some(vec!["rust".to_string(), "toml".to_string()])
    );
}

#[test]
fn test_root_directory() {
    let hook = lefthook::LefthookHook::new("echo test".to_string()).with_root("src/".to_string());

    assert_eq!(hook.root, Some("src/".to_string()));
}

#[test]
fn test_interactive_and_stdin() {
    let hook = lefthook::LefthookHook::new("echo test".to_string())
        .with_interactive(true)
        .with_use_stdin(true);

    assert_eq!(hook.interactive, Some(true));
    assert_eq!(hook.use_stdin, Some(true));
}

#[test]
fn test_serialization() {
    let hook = lefthook::LefthookHook::new("cargo test".to_string())
        .with_files("*.rs".to_string())
        .with_priority(5)
        .with_fail_text("Tests failed".to_string())
        .with_stage_fixed(true);

    // Test JSON serialization
    let json = serde_json::to_string(&hook).unwrap();
    assert!(json.contains("cargo test"));
    assert!(json.contains("*.rs"));
    assert!(json.contains("5"));
    assert!(json.contains("Tests failed"));
    assert!(json.contains("stage_fixed"));

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&hook).unwrap();
    assert!(yaml.contains("cargo test"));
    assert!(yaml.contains("*.rs"));
    assert!(yaml.contains("5"));
    assert!(yaml.contains("Tests failed"));
    assert!(yaml.contains("stage_fixed"));
}

#[test]
fn test_deserialization() {
    let json = r#"{
        "run": "cargo test",
        "files": "*.rs",
        "priority": 10,
        "fail_text": "Tests failed",
        "stage_fixed": true,
        "env": {
            "RUST_BACKTRACE": "1"
        }
    }"#;

    let hook: lefthook::LefthookHook = serde_json::from_str(json).unwrap();

    assert_eq!(hook.run, "cargo test");
    assert_eq!(hook.files, Some("*.rs".to_string()));
    assert_eq!(hook.priority, Some(10));
    assert_eq!(hook.fail_text, Some("Tests failed".to_string()));
    assert_eq!(hook.stage_fixed, Some(true));

    let env = hook.env.unwrap();
    assert_eq!(env.get("RUST_BACKTRACE"), Some(&"1".to_string()));
}

#[test]
fn test_complex_skip_conditions() {
    // Test complex skip condition with array
    let hook = lefthook::LefthookHook::new("echo test".to_string())
        .with_skip(json!(["merge", "rebase", "amend"]));

    assert!(hook.should_skip());

    // Test only condition
    let hook = lefthook::LefthookHook::new("echo test".to_string())
        .with_only(json!(["pre-commit", "pre-push"]));

    // Only conditions don't affect should_skip() directly
    assert!(!hook.should_skip());
}

#[test]
fn test_priority_ordering() {
    let hook1 = lefthook::LefthookHook::new("echo first".to_string()).with_priority(1);

    let hook2 = lefthook::LefthookHook::new("echo second".to_string()).with_priority(10);

    let hook3 = lefthook::LefthookHook::new("echo third".to_string());

    // Test priority ordering
    assert!(hook1.execution_priority() < hook2.execution_priority());
    assert_eq!(hook3.execution_priority(), 0); // Default priority
}
