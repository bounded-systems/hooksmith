use hooksmith::modules::lefthook;
use serde_json::json;
use tempfile::NamedTempFile;

#[test]
fn test_all_available_hooks() {
    // Test that we have all 28 hooks as defined in the official Lefthook implementation
    assert_eq!(lefthook::AVAILABLE_HOOKS.len(), 28);
    
    // Test that all hooks are known
    for hook in lefthook::AVAILABLE_HOOKS {
        assert!(lefthook::is_known_hook(hook), "Hook '{}' should be known", hook);
    }
    
    // Test that invalid hooks are not known
    assert!(!lefthook::is_known_hook("invalid-hook"));
    assert!(!lefthook::is_known_hook("custom-hook"));
}

#[test]
fn test_hook_behavior_functions() {
    // Test staged file hooks
    assert!(lefthook::hook_uses_staged_files("pre-commit"));
    assert!(!lefthook::hook_uses_staged_files("pre-push"));
    assert!(!lefthook::hook_uses_staged_files("commit-msg"));
    
    // Test push file hooks
    assert!(lefthook::hook_uses_push_files("pre-push"));
    assert!(!lefthook::hook_uses_push_files("pre-commit"));
    assert!(!lefthook::hook_uses_push_files("post-commit"));
    
    // Test hook lists
    let staged_hooks = lefthook::get_staged_file_hooks();
    assert_eq!(staged_hooks.len(), 1);
    assert_eq!(staged_hooks[0], "pre-commit");
    
    let push_hooks = lefthook::get_push_file_hooks();
    assert_eq!(push_hooks.len(), 1);
    assert_eq!(push_hooks[0], "pre-push");
}

#[test]
fn test_special_hook_constants() {
    // Test special hook names
    assert_eq!(lefthook::GHOST_HOOK_NAME, "prepare-commit-msg");
    assert_eq!(lefthook::CHECKSUM_FILE_NAME, "lefthook.checksum");
    
    // Test that ghost hook is in available hooks
    assert!(lefthook::is_known_hook(lefthook::GHOST_HOOK_NAME));
}

#[tokio::test]
async fn test_all_hooks_schema_validation() {
    // Create a configuration with all available hooks
    let mut config = json!({});
    
    for hook in lefthook::AVAILABLE_HOOKS {
        config[hook] = json!({
            "commands": {
                "test": {
                    "run": "echo 'test'"
                }
            }
        });
    }
    
    // This should pass schema validation
    let result = lefthook::validate_against_schema(&config).await;
    assert!(result.is_ok(), "Configuration with all hooks should pass schema validation");
}

#[tokio::test]
async fn test_comprehensive_config_generation() {
    let temp_file = NamedTempFile::new().unwrap();
    
    // Generate comprehensive configuration
    let result = lefthook::generate_comprehensive_config(temp_file.path(), false).await;
    assert!(result.is_ok(), "Should generate comprehensive configuration");
    
    // Read the generated file
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    
    // Check that all hooks are present in the generated file
    for hook in lefthook::AVAILABLE_HOOKS {
        assert!(
            content.contains(hook),
            "Generated configuration should contain hook '{}'",
            hook
        );
    }
}

#[test]
fn test_hook_categories() {
    // Test commit-related hooks
    let commit_hooks = ["pre-commit", "commit-msg", "post-commit", "prepare-commit-msg"];
    for hook in &commit_hooks {
        assert!(lefthook::is_known_hook(hook), "Commit hook '{}' should be known", hook);
    }
    
    // Test push-related hooks
    let push_hooks = ["pre-push", "pre-receive", "post-receive", "post-update"];
    for hook in &push_hooks {
        assert!(lefthook::is_known_hook(hook), "Push hook '{}' should be known", hook);
    }
    
    // Test merge-related hooks
    let merge_hooks = ["pre-merge-commit", "post-merge"];
    for hook in &merge_hooks {
        assert!(lefthook::is_known_hook(hook), "Merge hook '{}' should be known", hook);
    }
    
    // Test checkout-related hooks
    let checkout_hooks = ["post-checkout", "push-to-checkout"];
    for hook in &checkout_hooks {
        assert!(lefthook::is_known_hook(hook), "Checkout hook '{}' should be known", hook);
    }
    
    // Test patch-related hooks
    let patch_hooks = ["applypatch-msg", "pre-applypatch", "post-applypatch"];
    for hook in &patch_hooks {
        assert!(lefthook::is_known_hook(hook), "Patch hook '{}' should be known", hook);
    }
    
    // Test P4-related hooks
    let p4_hooks = [
        "p4-changelist",
        "p4-prepare-changelist", 
        "p4-post-changelist",
        "p4-pre-submit"
    ];
    for hook in &p4_hooks {
        assert!(lefthook::is_known_hook(hook), "P4 hook '{}' should be known", hook);
    }
}

#[test]
fn test_hook_set_operations() {
    let hook_set = lefthook::get_available_hooks_set();
    
    // Test that all hooks are in the set
    for hook in lefthook::AVAILABLE_HOOKS {
        assert!(hook_set.contains(hook), "Hook '{}' should be in the set", hook);
    }
    
    // Test set size
    assert_eq!(hook_set.len(), 28);
    
    // Test that invalid hooks are not in the set
    assert!(!hook_set.contains("invalid-hook"));
}

#[tokio::test]
async fn test_individual_hook_validation() {
    // Test each hook individually
    for hook in lefthook::AVAILABLE_HOOKS {
        let config = json!({
            hook: {
                "commands": {
                    "test": {
                        "run": "echo 'test'"
                    }
                }
            }
        });
        
        let result = lefthook::validate_against_schema(&config).await;
        assert!(
            result.is_ok(),
            "Individual hook '{}' should pass schema validation",
            hook
        );
    }
} 
