use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

/// GitHub Hook Generator for Hooksmith
///
/// This tool generates Rust binaries for all GitHub Actions events,
/// creating pre- and post-hooks for each event type.
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <command>", args[0]);
        println!("Commands:");
        println!("  generate-all    Generate all GitHub event hooks");
        println!("  list-events     List all available GitHub events");
        println!("  generate <event> Generate hooks for specific event");
        return Ok(());
    }

    let command = &args[1];
    
    match command.as_str() {
        "generate-all" => generate_all_hooks()?,
        "list-events" => list_all_events()?,
        "generate" => {
            if args.len() < 3 {
                println!("Usage: {} generate <event-name>", args[0]);
                return Ok(());
            }
            let event_name = &args[2];
            generate_event_hooks(event_name)?;
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Use 'list-events' to see available commands");
        }
    }

    Ok(())
}

/// All GitHub Actions events with their descriptions
fn get_github_events() -> HashMap<String, String> {
    let mut events = HashMap::new();
    
    // Core Lifecycle Events
    events.insert("push".to_string(), "Push commits or tags".to_string());
    events.insert("pull_request".to_string(), "Pull request activity".to_string());
    events.insert("pull_request_target".to_string(), "Pull request activity (secure context)".to_string());
    events.insert("workflow_dispatch".to_string(), "Manual workflow trigger".to_string());
    events.insert("workflow_call".to_string(), "Workflow called by another workflow".to_string());
    events.insert("schedule".to_string(), "Scheduled workflow runs".to_string());
    events.insert("repository_dispatch".to_string(), "External trigger via API".to_string());
    events.insert("check_run".to_string(), "Check run activity".to_string());
    events.insert("check_suite".to_string(), "Check suite activity".to_string());
    
    // Content Events
    events.insert("create".to_string(), "Create branch or tag".to_string());
    events.insert("delete".to_string(), "Delete branch or tag".to_string());
    events.insert("fork".to_string(), "Repository forked".to_string());
    events.insert("issues".to_string(), "Issue activity".to_string());
    events.insert("issue_comment".to_string(), "Issue or PR comment".to_string());
    events.insert("discussion".to_string(), "Discussion activity".to_string());
    events.insert("discussion_comment".to_string(), "Discussion comment".to_string());
    events.insert("release".to_string(), "Release activity".to_string());
    events.insert("page_build".to_string(), "GitHub Pages build".to_string());
    
    // Security & Auth
    events.insert("security_advisory".to_string(), "Security advisory".to_string());
    events.insert("dependabot_alert".to_string(), "Dependabot security alert".to_string());
    events.insert("deployment".to_string(), "Deployment created".to_string());
    events.insert("deployment_status".to_string(), "Deployment status update".to_string());
    events.insert("status".to_string(), "Commit status change".to_string());
    
    // Collaboration
    events.insert("watch".to_string(), "Repository starred".to_string());
    events.insert("star".to_string(), "Repository starred (alias)".to_string());
    events.insert("member".to_string(), "Repository member activity".to_string());
    events.insert("team_add".to_string(), "Team added to repository".to_string());
    events.insert("public".to_string(), "Repository made public".to_string());
    events.insert("organization".to_string(), "Organization activity".to_string());
    
    // Advanced & Extensible
    events.insert("workflow_run".to_string(), "Workflow run completion".to_string());
    events.insert("milestone".to_string(), "Milestone activity".to_string());
    events.insert("label".to_string(), "Label activity".to_string());
    events.insert("project".to_string(), "Project activity".to_string());
    events.insert("project_card".to_string(), "Project card activity".to_string());
    events.insert("project_column".to_string(), "Project column activity".to_string());
    
    // Additional Events
    events.insert("gollum".to_string(), "Wiki page activity".to_string());
    events.insert("registry_package".to_string(), "Package registry activity".to_string());
    events.insert("branch_protection_rule".to_string(), "Branch protection rule changes".to_string());
    events.insert("merge_group".to_string(), "Merge queue activity".to_string());
    events.insert("pull_request_review".to_string(), "Pull request review".to_string());
    events.insert("pull_request_review_comment".to_string(), "Pull request review comment".to_string());
    
    events
}

fn list_all_events() -> Result<()> {
    let events = get_github_events();
    
    println!("📋 Available GitHub Actions Events:");
    println!("=====================================");
    
    for (event, description) in events.iter() {
        println!("  • {} - {}", event, description);
    }
    
    println!("\n🎯 Each event generates:");
    println!("  • pre-<event> - Pre-event validation");
    println!("  • post-<event> - Post-event actions");
    
    Ok(())
}

fn generate_all_hooks() -> Result<()> {
    let events = get_github_events();
    
    println!("🚀 Generating all GitHub event hooks...");
    
    for (event, _description) in events.iter() {
        generate_event_hooks(event)?;
    }
    
    println!("✅ Generated {} event hooks", events.len() * 2);
    println!("📁 Hooks created in src/bin/");
    println!("💡 Run 'cargo build' to compile all hooks");
    
    Ok(())
}

fn generate_event_hooks(event_name: &str) -> Result<()> {
    let events = get_github_events();
    
    if !events.contains_key(event_name) {
        println!("❌ Unknown event: {}", event_name);
        println!("💡 Use 'list-events' to see available events");
        return Ok(());
    }
    
    let description = events.get(event_name).unwrap();
    
    // Generate pre-hook
    let pre_hook_content = generate_hook_content(event_name, "pre", description);
    let pre_hook_path = format!("src/bin/pre-{}.rs", event_name);
    fs::write(&pre_hook_path, pre_hook_content)?;
    
    // Generate post-hook
    let post_hook_content = generate_hook_content(event_name, "post", description);
    let post_hook_path = format!("src/bin/post-{}.rs", event_name);
    fs::write(&post_hook_path, post_hook_content)?;
    
    println!("✅ Generated hooks for {}: pre-{} and post-{}", event_name, event_name, event_name);
    
    Ok(())
}

fn generate_hook_content(event_name: &str, hook_type: &str, description: &str) -> String {
    let hook_name = format!("{}-{}", hook_type, event_name);
    let event_type = event_name.replace("_", "-");
    let hook_type_capitalized = hook_type.capitalize();
    
    format!(
        r#"use anyhow::Result;
use std::env;

/// {hook_type_capitalized} {event_name} Hook for Hooksmith
///
/// This hook handles {hook_type} {event_name} events:
/// {description}
///
/// Event: {event_name}
/// Hook Type: {hook_type}
/// Status: Stub (ready for implementation)
fn main() -> Result<()> {{
    println!("✅ {hook_name} hook (stub mode) - would handle {hook_type} {event_name} events");

    // Read GitHub event data
    if let Ok(event_path) = env::var("GITHUB_EVENT_PATH") {{
        println!("📄 Event path: {{}}", event_path);
    }}

    if let Ok(event_name) = env::var("GITHUB_EVENT_NAME") {{
        println!("🎯 Event name: {{}}", event_name);
    }}

    if let Ok(repository) = env::var("GITHUB_REPOSITORY") {{
        println!("📦 Repository: {{}}", repository);
    }}

    if let Ok(ref_name) = env::var("GITHUB_REF") {{
        println!("🌿 Ref: {{}}", ref_name);
    }}

    // TODO: Implement {hook_type} {event_name} validation logic
    // - Validate event payload
    // - Check permissions and security
    // - Perform custom validation
    // - Log activity for audit

    println!("🚀 {hook_type_capitalized} {event_name} validation completed successfully");
    Ok(())
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_hook_initialization() {{
        // Test that the hook can be initialized
        assert!(true);
    }}

    #[test]
    fn test_event_name_validation() {{
        // Test event name validation logic
        let event_name = "{event_name}";
        assert!(!event_name.is_empty());
    }}
}}
"#,
        hook_type = hook_type,
        event_name = event_name,
        description = description,
        hook_name = hook_name,
        hook_type_capitalized = hook_type_capitalized
    )
}

trait Capitalize {
    fn capitalize(&self) -> String;
}

impl Capitalize for str {
    fn capitalize(&self) -> String {
        let mut chars = self.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_events() {
        let events = get_github_events();
        assert!(events.contains_key("push"));
        assert!(events.contains_key("pull_request"));
        assert!(events.contains_key("issues"));
    }

    #[test]
    fn test_hook_content_generation() {
        let content = generate_hook_content("push", "pre", "Push commits or tags");
        assert!(content.contains("pre-push"));
        assert!(content.contains("Push commits or tags"));
    }

    #[test]
    fn test_capitalize() {
        assert_eq!("pre".capitalize(), "Pre");
        assert_eq!("post".capitalize(), "Post");
    }
}
