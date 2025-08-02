//! Integration tests for hooksmith
//!
//! These tests verify the CLI behavior end-to-end.

use std::process::Command;

#[test]
fn test_cli_help() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "--help"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Main CLI application for Hooksmith"));
    assert!(stdout.contains("worktree"));
    Ok(())
}

#[test]
fn test_cli_version() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "--version"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hooksmith 0.1.0"));
    Ok(())
}

#[test]
fn test_test_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "test"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Test successful"));
    Ok(())
}

#[test]
fn test_test_command_with_message() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "test", "--message", "Custom message"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Custom message"));
    Ok(())
}

#[test]
fn test_list_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "list"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available hooks"));
    Ok(())
}

#[test]
fn test_build_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "build", "test-hook"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Building hook"));
    assert!(stdout.contains("test-hook"));
    Ok(())
}

#[test]
fn test_generate_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "generate"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generating Lefthook config"));
    Ok(())
}

#[test]
fn test_install_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "install"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Installing hooks"));
    Ok(())
}

#[test]
fn test_wasm_build_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "wasm", "build", "test.wit"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Building WASM from WIT"));
    assert!(stdout.contains("test.wit"));
    Ok(())
}

#[test]
fn test_wasm_run_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "run",
        "--",
        "wasm",
        "run",
        "test.wasm",
        "--function",
        "test",
        "--args",
        "arg1,arg2",
    ]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Running WASM"));
    assert!(stdout.contains("test.wasm"));
    assert!(stdout.contains("test"));
    Ok(())
}

#[test]
fn test_wasm_bindings_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "wasm", "bindings", "test.wit"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Generating bindings from WIT"));
    assert!(stdout.contains("test.wit"));
    Ok(())
}

#[test]
fn test_worktree_create_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "worktree", "create", "feature/test"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Creating worktree"));
    assert!(stdout.contains("feature/test"));
    Ok(())
}

#[test]
fn test_worktree_list_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "worktree", "list"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Listing worktrees"));
    Ok(())
}

#[test]
fn test_worktree_switch_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "worktree", "switch", "test-worktree"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Switching to worktree"));
    assert!(stdout.contains("test-worktree"));
    Ok(())
}

#[test]
fn test_worktree_remove_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "worktree", "remove", "test-worktree"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Removing worktree"));
    assert!(stdout.contains("test-worktree"));
    Ok(())
}

#[test]
fn test_worktree_tools_command() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--", "worktree", "tools"]);
    let output = cmd.output()?;

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Available worktree tools"));
    Ok(())
}
