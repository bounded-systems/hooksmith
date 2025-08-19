use std::env;
use std::process::{exit, Command};

fn main() {
    let event = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    let enabled = env::var("ENABLE_HOOKSMITH_VALIDATION").unwrap_or_else(|_| "false".into());

    // Log event information
    println!("::group::🔍 Hooksmith Dispatch");
    println!("Event: {}", event);
    println!(
        "Repository: {}",
        env::var("GITHUB_REPOSITORY").unwrap_or_default()
    );
    println!("Actor: {}", env::var("GITHUB_ACTOR").unwrap_or_default());
    println!("Run ID: {}", env::var("GITHUB_RUN_ID").unwrap_or_default());
    println!("::endgroup::");

    if enabled != "true" {
        println!("::notice::🔇 Hooksmith validation disabled");
        println!("Set ENABLE_HOOKSMITH_VALIDATION=true to enable validation");
        return;
    }

    // Map event to binary
    let binary = match event.as_str() {
        "push" => "github-push",
        "pull_request" => "github-pull-request",
        "issues" => "github-issues",
        "release" => "github-release",
        "create" => "github-create",
        "delete" => "github-delete",
        "workflow_dispatch" => "github-workflow-dispatch",
        "schedule" => "github-schedule",
        "watch" => "github-watch",
        "fork" => "github-fork",
        "gollum" => "github-gollum",
        "page_build" => "github-page-build",
        "public" => "github-public",
        "label" => "github-label",
        "milestone" => "github-milestone",
        "discussion" => "github-discussion",
        "discussion_comment" => "github-discussion-comment",
        "issue_comment" => "github-issue-comment",
        "pull_request_review" => "github-pull-request-review",
        "pull_request_review_comment" => "github-pull-request-review-comment",
        "check_suite" => "github-check-suite",
        "check_run" => "github-check-run",
        "status" => "github-status",
        "deployment" => "github-deployment",
        "deployment_status" => "github-deployment-status",
        "repository_dispatch" => "github-repository-dispatch",
        "registry_package" => "github-registry-package",
        "branch_protection_rule" => "github-branch-protection-rule",
        _ => {
            println!("::notice::📋 No validation binary for event: {}", event);
            println!("Available events: push, pull_request, issues, release, create, delete, workflow_dispatch, schedule, watch, fork, gollum, page_build, public, label, milestone, discussion, discussion_comment, issue_comment, pull_request_review, pull_request_review_comment, check_suite, check_run, status, deployment, deployment_status, repository_dispatch, registry_package, branch_protection_rule");
            return;
        }
    };

    let binary_path = format!("./target/release/{}", binary);

    println!("::group::🚀 Running Hooksmith Validation");
    println!("Binary: {}", binary);
    println!("Path: {}", binary_path);
    println!("::endgroup::");

    // Execute the binary
    let status = Command::new(&binary_path)
        .env("GITHUB_EVENT_NAME", &event)
        .env(
            "GITHUB_REPOSITORY",
            env::var("GITHUB_REPOSITORY").unwrap_or_default(),
        )
        .env("GITHUB_ACTOR", env::var("GITHUB_ACTOR").unwrap_or_default())
        .env(
            "GITHUB_RUN_ID",
            env::var("GITHUB_RUN_ID").unwrap_or_default(),
        )
        .env(
            "GITHUB_WORKFLOW",
            env::var("GITHUB_WORKFLOW").unwrap_or_default(),
        )
        .env("GITHUB_REF", env::var("GITHUB_REF").unwrap_or_default())
        .env("GITHUB_SHA", env::var("GITHUB_SHA").unwrap_or_default())
        .status();

    match status {
        Ok(exit_status) => {
            if exit_status.success() {
                println!("::notice::✅ {} completed successfully", binary);
            } else {
                println!(
                    "::warning::❌ {} failed with exit code: {}",
                    binary,
                    exit_status.code().unwrap_or(1)
                );
                exit(exit_status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            println!("::error::💥 Failed to execute {}: {}", binary, e);
            println!("::info::Make sure the binary exists: {}", binary_path);
            exit(1);
        }
    }
}
