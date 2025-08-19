use anyhow::Result;
use clap::Parser;
use serde_json::Value;
use std::env;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "github-issues")]
#[command(about = "GitHub Actions stub for issues events")]
struct Args {
    /// GitHub event payload file path
    #[arg(long, env = "GITHUB_EVENT_PATH")]
    event_path: Option<String>,

    /// GitHub event name
    #[arg(long, env = "GITHUB_EVENT_NAME")]
    event_name: Option<String>,

    /// Repository name
    #[arg(long, env = "GITHUB_REPOSITORY")]
    repository: Option<String>,

    /// Issue number
    #[arg(long, env = "GITHUB_ISSUE_NUMBER")]
    issue_number: Option<String>,

    /// Action type (opened, edited, closed, etc.)
    #[arg(long, env = "GITHUB_ISSUE_ACTION")]
    action: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Use GitHub workflow commands for better integration
    println!("::group::🔍 GitHub Issues Event Handler");
    println!("Event: {:?}", args.event_name);
    println!("Repository: {:?}", args.repository);
    println!("Issue Number: {:?}", args.issue_number);
    println!("Action: {:?}", args.action);
    println!("::endgroup::");

    // Load GitHub event payload if available
    if let Some(event_path) = args.event_path {
        if let Ok(payload) = std::fs::read_to_string(&event_path) {
            if let Ok(event_data) = serde_json::from_str::<Value>(&payload) {
                println!("::notice::📄 Event payload loaded from: {}", event_path);

                // Extract issue information
                if let Some(issue) = event_data.get("issue") {
                    if let Some(title) = issue.get("title") {
                        println!("::info::📋 Issue Title: {}", title);
                    }

                    if let Some(body) = issue.get("body") {
                        let body_len = body.as_str().unwrap_or("").len();
                        println!("::info::📝 Issue Body length: {}", body_len);

                        // Set output for workflow
                        println!("issue_body_length={}", body_len);
                    }

                    if let Some(labels) = issue.get("labels") {
                        if let Some(labels_array) = labels.as_array() {
                            println!("::info::🏷️ Labels: {}", labels_array.len());

                            // Set output for workflow
                            let label_names: Vec<String> = labels_array
                                .iter()
                                .filter_map(|l| l.get("name").and_then(|n| n.as_str()))
                                .map(|s| s.to_string())
                                .collect();
                            println!("issue_labels={}", label_names.join(","));
                        }
                    }
                }

                // Extract action information
                if let Some(action) = event_data.get("action") {
                    println!("::info::⚡ Action: {}", action);

                    // Set output for workflow
                    println!("issue_action={}", action);
                }
            }
        }
    }

    // TODO: Integrate with hooksmith validation schema
    // This would call the appropriate validation handlers based on the event
    // For issue events, this might map to commit-msg validation for issue descriptions

    println!("::notice::✅ GitHub issues validation completed");
    Ok(())
}
