//! Tests for the dynamic hooks generation system

use pushd_cli::hooks::{HookConfig, HookGenerator, ProjectFeatures};

#[test]
fn test_hook_config_builder() {
    let hook = HookConfig::new("test-hook", "pre-commit", "echo 'test'", "Test hook")
        .parallel(true)
        .glob("*.rs")
        .stage_fixed(true)
        .conditional("rust_workspace");

    assert_eq!(hook.name, "test-hook");
    assert_eq!(hook.stage, "pre-commit");
    assert_eq!(hook.command, "echo 'test'");
    assert_eq!(hook.description, "Test hook");
    assert!(hook.parallel);
    assert_eq!(hook.glob, Some("*.rs".to_string()));
    assert!(hook.stage_fixed);
    assert!(hook.conditional);
    assert_eq!(hook.required_feature, Some("rust_workspace".to_string()));
}

#[test]
fn test_hook_generator_default_hooks() {
    let generator = HookGenerator::new();
    let hooks = generator.default_hooks();
    
    // Should have CLI-based hooks
    assert!(hooks.iter().any(|h| h.name == "cli-pre-commit"));
    assert!(hooks.iter().any(|h| h.name == "cli-pre-push"));
    assert!(hooks.iter().any(|h| h.name == "cli-commit-msg"));
    
    // Should have Rust-specific hooks
    assert!(hooks.iter().any(|h| h.name == "format"));
    assert!(hooks.iter().any(|h| h.name == "lint"));
    assert!(hooks.iter().any(|h| h.name == "test"));
}

#[test]
fn test_hook_generator_with_rust_workspace() {
    let generator = HookGenerator::new();
    let mut features = ProjectFeatures::default();
    features.has_rust_workspace = true;
    features.has_wit_interfaces = true;
    features.has_safety_checks = true;
    
    let config = generator.generate_config(&features).unwrap();
    
    // Should include CLI hooks (always included)
    assert!(config.contains("cli-pre-commit"));
    assert!(config.contains("cli-pre-push"));
    
    // Should include Rust-specific hooks
    assert!(config.contains("format"));
    assert!(config.contains("lint"));
    assert!(config.contains("test"));
    
    // Should include WIT validation
    assert!(config.contains("wit-validate"));
    
    // Should include safety checks
    assert!(config.contains("worktree-guard"));
    assert!(config.contains("prevent-main-files"));
    
    // Should have parallel execution for pre-commit
    assert!(config.contains("parallel: true"));
    
    // Should have proper YAML structure
    assert!(config.contains("pre-commit:"));
    assert!(config.contains("pre-push:"));
    assert!(config.contains("commands:"));
    
    // Should include metadata
    assert!(config.contains("Configuration metadata"));
    assert!(config.contains("Features detected:"));
    assert!(config.contains("Rust workspace: true"));
    assert!(config.contains("WIT interfaces: true"));
    assert!(config.contains("Safety checks: true"));
}

#[test]
fn test_hook_generator_without_features() {
    let generator = HookGenerator::new();
    let features = ProjectFeatures::default(); // All features false
    
    let config = generator.generate_config(&features).unwrap();
    
    // Should only include CLI hooks (non-conditional)
    assert!(config.contains("cli-pre-commit"));
    assert!(config.contains("cli-pre-push"));
    assert!(config.contains("cli-commit-msg"));
    assert!(config.contains("cli-post-commit"));
    
    // Should not include conditional hooks
    assert!(!config.contains("format"));
    assert!(!config.contains("lint"));
    assert!(!config.contains("wit-validate"));
    assert!(!config.contains("worktree-guard"));
    
    // Should include metadata showing all features as false
    assert!(config.contains("Rust workspace: false"));
    assert!(config.contains("WIT interfaces: false"));
    assert!(config.contains("Safety checks: false"));
}

#[test]
fn test_hook_generator_stats() {
    let generator = HookGenerator::new();
    let mut features = ProjectFeatures::default();
    features.has_rust_workspace = true;
    features.has_safety_checks = true;
    
    let stats = generator.stats(&features);
    
    // Should count all active hooks
    assert!(stats.total_hooks > 0);
    
    // Should have counts for each stage
    assert!(stats.stage_counts.contains_key("pre-commit"));
    assert!(stats.stage_counts.contains_key("pre-push"));
    
    // Should reflect the features
    assert!(stats.features.has_rust_workspace);
    assert!(stats.features.has_safety_checks);
    assert!(!stats.features.has_wit_interfaces);
}

#[test]
fn test_hook_generator_custom_hooks() {
    let mut generator = HookGenerator::new();
    
    // Add a custom hook
    let custom_hook = HookConfig::new(
        "custom-test",
        "pre-commit",
        "echo 'custom test'",
        "Custom test hook"
    ).glob("*.txt");
    
    generator.add_hook(custom_hook);
    
    let features = ProjectFeatures::default();
    let config = generator.generate_config(&features).unwrap();
    
    // Should include the custom hook
    assert!(config.contains("custom-test"));
    assert!(config.contains("echo 'custom test'"));
    assert!(config.contains("glob: \"*.txt\""));
}

#[test]
fn test_yaml_structure() {
    let generator = HookGenerator::new();
    let mut features = ProjectFeatures::default();
    features.has_rust_workspace = true;
    
    let config = generator.generate_config(&features).unwrap();
    
    // Check YAML structure
    let lines: Vec<&str> = config.lines().collect();
    
    // Should start with header comments
    assert!(lines[0].starts_with("#"));
    assert!(lines[1].starts_with("#"));
    
    // Should have proper indentation
    let pre_commit_index = lines.iter().position(|&line| line == "pre-commit:").unwrap();
    assert_eq!(lines[pre_commit_index + 1], "  parallel: true");
    assert_eq!(lines[pre_commit_index + 2], "  commands:");
    
    // Should have proper hook indentation
    let cli_hook_index = lines.iter().position(|&line| line == "    cli-pre-commit:").unwrap();
    assert_eq!(lines[cli_hook_index + 1], "      run: cargo run --bin pushd-cli -- hooks pre-commit");
    assert_eq!(lines[cli_hook_index + 2], "      glob: \"*\"");
}

#[test]
fn test_conditional_hook_filtering() {
    let generator = HookGenerator::new();
    let hooks = generator.default_hooks();
    
    // Find conditional hooks
    let conditional_hooks: Vec<_> = hooks.iter()
        .filter(|h| h.conditional)
        .collect();
    
    // Should have conditional hooks
    assert!(!conditional_hooks.is_empty());
    
    // Check specific conditional hooks
    let format_hook = hooks.iter().find(|h| h.name == "format").unwrap();
    assert!(format_hook.conditional);
    assert_eq!(format_hook.required_feature, Some("rust_workspace".to_string()));
    
    let wit_hook = hooks.iter().find(|h| h.name == "wit-validate").unwrap();
    assert!(wit_hook.conditional);
    assert_eq!(wit_hook.required_feature, Some("wit_interfaces".to_string()));
}

#[test]
fn test_hook_stages() {
    let generator = HookGenerator::new();
    let hooks = generator.default_hooks();
    
    // Check that hooks are distributed across stages
    let stages: std::collections::HashSet<_> = hooks.iter()
        .map(|h| h.stage.as_str())
        .collect();
    
    assert!(stages.contains("pre-commit"));
    assert!(stages.contains("pre-push"));
    assert!(stages.contains("commit-msg"));
    assert!(stages.contains("post-commit"));
    assert!(stages.contains("post-checkout"));
}

#[test]
fn test_parallel_execution() {
    let generator = HookGenerator::new();
    let hooks = generator.default_hooks();
    
    // Find parallel hooks
    let parallel_hooks: Vec<_> = hooks.iter()
        .filter(|h| h.parallel)
        .collect();
    
    // Should have parallel hooks
    assert!(!parallel_hooks.is_empty());
    
    // Check specific parallel hooks
    let cli_hook = hooks.iter().find(|h| h.name == "cli-pre-commit").unwrap();
    assert!(cli_hook.parallel);
    
    let format_hook = hooks.iter().find(|h| h.name == "format").unwrap();
    assert!(format_hook.parallel);
}

#[test]
fn test_glob_patterns() {
    let generator = HookGenerator::new();
    let hooks = generator.default_hooks();
    
    // Check specific glob patterns
    let format_hook = hooks.iter().find(|h| h.name == "format").unwrap();
    assert_eq!(format_hook.glob, Some("*.rs".to_string()));
    
    let wit_hook = hooks.iter().find(|h| h.name == "wit-validate").unwrap();
    assert_eq!(wit_hook.glob, Some("wit/*.wit".to_string()));
    
    let daemon_hook = hooks.iter().find(|h| h.name == "daemon-validation").unwrap();
    assert_eq!(daemon_hook.glob, Some("daemon/*.rb".to_string()));
}

#[test]
fn test_stage_fixed_flag() {
    let generator = HookGenerator::new();
    let hooks = generator.default_hooks();
    
    // Check that format hook has stage_fixed
    let format_hook = hooks.iter().find(|h| h.name == "format").unwrap();
    assert!(format_hook.stage_fixed);
    
    // Check that other hooks don't have stage_fixed
    let lint_hook = hooks.iter().find(|h| h.name == "lint").unwrap();
    assert!(!lint_hook.stage_fixed);
} 
