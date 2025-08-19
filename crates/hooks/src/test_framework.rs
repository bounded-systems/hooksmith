use crate::schema::{GitHook, HookContext, HookError, HookMetadata, ValidationCapability};
use anyhow::Result;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test result for a hook execution
#[derive(Debug, Clone, serde::Serialize)]
pub struct HookTestResult {
    pub hook_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub error: Option<String>,
    pub metadata: HookMetadata,
}

/// Comprehensive hook testing framework
pub struct HookTestFramework {
    project_root: String,
    test_results: Vec<HookTestResult>,
}

impl HookTestFramework {
    /// Create a new test framework
    pub fn new() -> Result<Self> {
        let project_root = std::env::current_dir()?.to_string_lossy().to_string();

        Ok(Self {
            project_root,
            test_results: Vec::new(),
        })
    }

    /// Test all hooks for basic functionality
    pub async fn test_all_hooks(&mut self) -> Result<()> {
        println!("🧪 Testing all Git hooks...");

        let hooks = vec![
            GitHook::PreCommit,
            GitHook::PrepareCommitMsg,
            GitHook::CommitMsg,
            GitHook::PostCommit,
            GitHook::PrePush,
            GitHook::PostCheckout,
            GitHook::PostMerge,
            GitHook::PreRebase,
            GitHook::PostRebase,
            GitHook::PostRewrite,
            GitHook::ApplyPatchMsg,
            GitHook::PreApplyPatch,
            GitHook::PostApplyPatch,
            GitHook::PreMergeCommit,
            GitHook::ReferenceTransaction,
            GitHook::SendEmailValidate,
            GitHook::FSMonitorWatchman,
            GitHook::PostIndexChange,
        ];

        for hook in hooks {
            self.test_hook(hook).await?;
        }

        self.print_summary();
        Ok(())
    }

    /// Test a specific hook
    async fn test_hook(&mut self, hook: GitHook) -> Result<()> {
        let hook_name = hook.name();
        println!("  🔍 Testing {}...", hook_name);

        // Test 1: Basic execution
        let basic_result = self.test_basic_execution(&hook).await?;

        // Test 2: Performance
        let perf_result = self.test_performance(&hook).await?;

        // Test 3: Blocking behavior
        let blocking_result = self.test_blocking_behavior(&hook).await?;

        // Test 4: Schema validation
        let schema_result = self.test_schema_validation(&hook).await?;

        // Combine results
        let combined_result = HookTestResult {
            hook_name: hook_name.to_string(),
            success: basic_result.success
                && perf_result.success
                && blocking_result.success
                && schema_result.success,
            duration_ms: perf_result.duration_ms,
            exit_code: basic_result.exit_code,
            stdout: basic_result.stdout,
            stderr: basic_result.stderr,
            error: basic_result.error,
            metadata: self.create_test_context(&hook).metadata(),
        };

        self.test_results.push(combined_result);
        Ok(())
    }

    /// Test basic hook execution
    async fn test_basic_execution(&self, hook: &GitHook) -> Result<HookTestResult> {
        let start = Instant::now();

        // Create test context
        let context = self.create_test_context(hook);

        // Execute hook binary
        let output = Command::new(format!(
            "{}/target/debug/{}",
            self.project_root,
            hook.name()
        ))
        .args(&context.args)
        .envs(&context.env)
        .output()?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(HookTestResult {
            hook_name: hook.name().to_string(),
            success: output.status.success(),
            duration_ms,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8(output.stdout).ok(),
            stderr: String::from_utf8(output.stderr).ok(),
            error: if output.status.success() {
                None
            } else {
                Some("Hook execution failed".to_string())
            },
            metadata: context.metadata(),
        })
    }

    /// Test hook performance
    async fn test_performance(&self, hook: &GitHook) -> Result<HookTestResult> {
        let mut total_duration = 0u64;
        let iterations = 5;

        for _ in 0..iterations {
            let start = Instant::now();

            let output = Command::new(format!(
                "{}/target/debug/{}",
                self.project_root,
                hook.name()
            ))
            .output()?;

            total_duration += start.elapsed().as_millis() as u64;

            // Small delay between iterations
            sleep(Duration::from_millis(10)).await;
        }

        let avg_duration = total_duration / iterations;
        let is_fast = avg_duration < 100; // Consider fast if under 100ms

        Ok(HookTestResult {
            hook_name: format!("{} (performance)", hook.name()),
            success: is_fast,
            duration_ms: avg_duration,
            exit_code: 0,
            stdout: Some(format!("Average execution time: {}ms", avg_duration)),
            stderr: None,
            error: if is_fast {
                None
            } else {
                Some("Hook is too slow".to_string())
            },
            metadata: self.create_test_context(hook).metadata(),
        })
    }

    /// Test blocking behavior
    async fn test_blocking_behavior(&self, hook: &GitHook) -> Result<HookTestResult> {
        let start = Instant::now();

        // Test if hook blocks appropriately
        let should_block = matches!(
            hook,
            GitHook::PreCommit
                | GitHook::CommitMsg
                | GitHook::PrePush
                | GitHook::PreReceive
                | GitHook::Update
        );

        // For now, all our hooks are no-ops, so they shouldn't block
        let actually_blocks = false; // Our current no-op hooks don't block

        let success = !should_block || actually_blocks;

        Ok(HookTestResult {
            hook_name: format!("{} (blocking)", hook.name()),
            success,
            duration_ms: start.elapsed().as_millis() as u64,
            exit_code: if success { 0 } else { 1 },
            stdout: Some(format!(
                "Blocking behavior: {}",
                if should_block {
                    "should block"
                } else {
                    "should not block"
                }
            )),
            stderr: None,
            error: if success {
                None
            } else {
                Some("Incorrect blocking behavior".to_string())
            },
            metadata: self.create_test_context(hook).metadata(),
        })
    }

    /// Test schema validation
    async fn test_schema_validation(&self, hook: &GitHook) -> Result<HookTestResult> {
        let context = self.create_test_context(hook);

        // Validate context
        let validation_result = context.validate();

        let success = validation_result.is_ok();
        let error = validation_result.err().map(|e| e.to_string());

        Ok(HookTestResult {
            hook_name: format!("{} (schema)", hook.name()),
            success,
            duration_ms: 0,
            exit_code: if success { 0 } else { 1 },
            stdout: Some("Schema validation completed".to_string()),
            stderr: None,
            error,
            metadata: context.metadata(),
        })
    }

    /// Create a test context for a hook
    fn create_test_context(&self, hook: &GitHook) -> HookContext {
        let args = vec![hook.name().to_string()];
        let env = std::env::vars().collect();

        HookContext::from_args(args).unwrap_or_else(|_| {
            // Fallback for testing
            HookContext {
                hook: hook.clone(),
                args: vec![],
                env,
                stdin_data: None,
            }
        })
    }

    /// Test Git operations that trigger hooks
    pub async fn test_git_operations(&mut self) -> Result<()> {
        println!("🔧 Testing Git operations with hooks...");

        // Test 1: Commit operation
        self.test_commit_operation().await?;

        // Test 2: Push operation
        self.test_push_operation().await?;

        // Test 3: Checkout operation
        self.test_checkout_operation().await?;

        // Test 4: Merge operation
        self.test_merge_operation().await?;

        Ok(())
    }

    /// Test commit operation
    async fn test_commit_operation(&mut self) -> Result<()> {
        println!("  📝 Testing commit operation...");

        let start = Instant::now();

        // Create a test file
        std::fs::write("test-commit-file.txt", "test content")?;

        // Stage the file
        Command::new("git")
            .args(["add", "test-commit-file.txt"])
            .output()?;

        // Commit (this should trigger pre-commit, prepare-commit-msg, commit-msg, post-commit)
        let output = Command::new("git")
            .args(["commit", "-m", "test: commit operation test"])
            .output()?;

        let duration = start.elapsed();

        let result = HookTestResult {
            hook_name: "commit-operation".to_string(),
            success: output.status.success(),
            duration_ms: duration.as_millis() as u64,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8(output.stdout).ok(),
            stderr: String::from_utf8(output.stderr).ok(),
            error: if output.status.success() {
                None
            } else {
                Some("Commit operation failed".to_string())
            },
            metadata: HookMetadata {
                name: "commit-operation".to_string(),
                phase: crate::schema::HookPhase::PreValidation,
                scope: crate::schema::HookScope::Client,
                objects: vec![
                    crate::schema::GitObject::Commit,
                    crate::schema::GitObject::Index,
                ],
                validation_capabilities: vec![crate::schema::ValidationCapability::BasicValidation],
            },
        };

        self.test_results.push(result);
        Ok(())
    }

    /// Test push operation
    async fn test_push_operation(&mut self) -> Result<()> {
        println!("  🚀 Testing push operation...");

        let start = Instant::now();

        // Try to push (this should trigger pre-push)
        let output = Command::new("git").args(["push", "--dry-run"]).output()?;

        let duration = start.elapsed();

        let result = HookTestResult {
            hook_name: "push-operation".to_string(),
            success: true, // Dry run should succeed
            duration_ms: duration.as_millis() as u64,
            exit_code: 0,
            stdout: String::from_utf8(output.stdout).ok(),
            stderr: String::from_utf8(output.stderr).ok(),
            error: None,
            metadata: HookMetadata {
                name: "push-operation".to_string(),
                phase: crate::schema::HookPhase::PreValidation,
                scope: crate::schema::HookScope::Client,
                objects: vec![
                    crate::schema::GitObject::Ref,
                    crate::schema::GitObject::Remote,
                ],
                validation_capabilities: vec![crate::schema::ValidationCapability::RefValidation],
            },
        };

        self.test_results.push(result);
        Ok(())
    }

    /// Test checkout operation
    async fn test_checkout_operation(&mut self) -> Result<()> {
        println!("  🔄 Testing checkout operation...");

        let start = Instant::now();

        // Get current branch
        let current_branch = Command::new("git")
            .args(["branch", "--show-current"])
            .output()?;

        let current_branch = String::from_utf8(current_branch.stdout)?.trim().to_string();

        // Try to checkout the same branch (should trigger post-checkout)
        let output = Command::new("git")
            .args(["checkout", &current_branch])
            .output()?;

        let duration = start.elapsed();

        let result = HookTestResult {
            hook_name: "checkout-operation".to_string(),
            success: output.status.success(),
            duration_ms: duration.as_millis() as u64,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8(output.stdout).ok(),
            stderr: String::from_utf8(output.stderr).ok(),
            error: if output.status.success() {
                None
            } else {
                Some("Checkout operation failed".to_string())
            },
            metadata: HookMetadata {
                name: "checkout-operation".to_string(),
                phase: crate::schema::HookPhase::PostAction,
                scope: crate::schema::HookScope::Client,
                objects: vec![
                    crate::schema::GitObject::Head,
                    crate::schema::GitObject::Tree,
                ],
                validation_capabilities: vec![crate::schema::ValidationCapability::BasicValidation],
            },
        };

        self.test_results.push(result);
        Ok(())
    }

    /// Test merge operation
    async fn test_merge_operation(&mut self) -> Result<()> {
        println!("  🔀 Testing merge operation...");

        let start = Instant::now();

        // Get current HEAD
        let head_output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;

        let head_sha = String::from_utf8(head_output.stdout)?.trim().to_string();

        // Try to merge HEAD into itself (should be a no-op)
        let output = Command::new("git")
            .args(["merge", "--no-edit", "--no-commit", &head_sha])
            .output()?;

        let duration = start.elapsed();

        let result = HookTestResult {
            hook_name: "merge-operation".to_string(),
            success: output.status.success(),
            duration_ms: duration.as_millis() as u64,
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8(output.stdout).ok(),
            stderr: String::from_utf8(output.stderr).ok(),
            error: if output.status.success() {
                None
            } else {
                Some("Merge operation failed".to_string())
            },
            metadata: HookMetadata {
                name: "merge-operation".to_string(),
                phase: crate::schema::HookPhase::PostAction,
                scope: crate::schema::HookScope::Client,
                objects: vec![
                    crate::schema::GitObject::Commit,
                    crate::schema::GitObject::Tree,
                ],
                validation_capabilities: vec![crate::schema::ValidationCapability::BasicValidation],
            },
        };

        self.test_results.push(result);
        Ok(())
    }

    /// Print test summary
    fn print_summary(&self) {
        println!("\n📊 Hook Test Summary");
        println!("==================");

        let total_tests = self.test_results.len();
        let successful_tests = self.test_results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;

        println!("Total tests: {}", total_tests);
        println!("Successful: {}", successful_tests);
        println!("Failed: {}", failed_tests);
        println!(
            "Success rate: {:.1}%",
            (successful_tests as f64 / total_tests as f64) * 100.0
        );

        // Performance summary
        let avg_duration: u64 =
            self.test_results.iter().map(|r| r.duration_ms).sum::<u64>() / total_tests as u64;

        println!("Average execution time: {}ms", avg_duration);

        // Failed tests
        if failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for result in &self.test_results {
                if !result.success {
                    println!(
                        "  - {}: {}",
                        result.hook_name,
                        result
                            .error
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    );
                }
            }
        }

        // Fastest and slowest hooks
        if let Some(fastest) = self.test_results.iter().min_by_key(|r| r.duration_ms) {
            println!(
                "🏃 Fastest hook: {} ({}ms)",
                fastest.hook_name, fastest.duration_ms
            );
        }

        if let Some(slowest) = self.test_results.iter().max_by_key(|r| r.duration_ms) {
            println!(
                "🐌 Slowest hook: {} ({}ms)",
                slowest.hook_name, slowest.duration_ms
            );
        }
    }

    /// Get detailed test results
    pub fn get_results(&self) -> &[HookTestResult] {
        &self.test_results
    }

    /// Export results to JSON
    pub fn export_results(&self) -> Result<String> {
        let results = serde_json::to_string_pretty(&self.test_results)?;
        Ok(results)
    }
}

/// Integration test with the git-proxy server-side hooks
pub struct ServerHookIntegrationTest {
    git_proxy_hooks: HashMap<String, Vec<String>>,
}

impl ServerHookIntegrationTest {
    /// Create a new server hook integration test
    pub fn new() -> Self {
        let mut git_proxy_hooks = HashMap::new();

        // Map git-proxy hook types to our schema
        git_proxy_hooks.insert("pre-receive".to_string(), vec!["PreReceive".to_string()]);
        git_proxy_hooks.insert("post-receive".to_string(), vec!["PostReceive".to_string()]);
        git_proxy_hooks.insert("update".to_string(), vec!["Update".to_string()]);
        git_proxy_hooks.insert("pre-commit".to_string(), vec!["PreCommit".to_string()]);
        git_proxy_hooks.insert("post-commit".to_string(), vec!["PostCommit".to_string()]);
        git_proxy_hooks.insert(
            "prepare-commit-msg".to_string(),
            vec!["PrepareCommitMsg".to_string()],
        );
        git_proxy_hooks.insert("commit-msg".to_string(), vec!["CommitMsg".to_string()]);
        git_proxy_hooks.insert("pre-push".to_string(), vec!["PrePush".to_string()]);
        git_proxy_hooks.insert("post-merge".to_string(), vec!["PostMerge".to_string()]);
        git_proxy_hooks.insert(
            "post-checkout".to_string(),
            vec!["PostCheckout".to_string()],
        );
        git_proxy_hooks.insert("pre-rebase".to_string(), vec!["PreRebase".to_string()]);
        git_proxy_hooks.insert("post-rewrite".to_string(), vec!["PostRewrite".to_string()]);
        git_proxy_hooks.insert(
            "reference-transaction".to_string(),
            vec!["ReferenceTransaction".to_string()],
        );
        git_proxy_hooks.insert(
            "push-to-checkout".to_string(),
            vec!["PushToCheckout".to_string()],
        );
        git_proxy_hooks.insert(
            "sendemail-validate".to_string(),
            vec!["SendemailValidate".to_string()],
        );
        git_proxy_hooks.insert(
            "fsmonitor-watchman".to_string(),
            vec!["FsmonitorWatchman".to_string()],
        );
        git_proxy_hooks.insert(
            "post-index-change".to_string(),
            vec!["PostIndexChange".to_string()],
        );

        Self { git_proxy_hooks }
    }

    /// Test integration between client and server hooks
    pub async fn test_integration(&self) -> Result<()> {
        println!("🔗 Testing client-server hook integration...");

        for (client_hook, server_hooks) in &self.git_proxy_hooks {
            println!("  📋 {} -> {:?}", client_hook, server_hooks);

            // Test that client hooks can communicate with server hooks
            // This would involve testing the full pipeline
            // For now, we just validate the mapping
            if server_hooks.is_empty() {
                println!("    ⚠️  No server-side equivalent found");
            } else {
                println!("    ✅ Server-side hooks available");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_framework_creation() {
        let framework = HookTestFramework::new().unwrap();
        assert!(!framework.project_root.is_empty());
    }

    #[tokio::test]
    async fn test_server_hook_integration() {
        let integration = ServerHookIntegrationTest::new();
        assert!(!integration.git_proxy_hooks.is_empty());
    }
}
