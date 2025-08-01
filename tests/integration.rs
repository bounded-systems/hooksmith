//! Integration tests for pushd-cli
//! 
//! These tests verify the CLI behavior end-to-end using assert_cmd.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_cli_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Unified CLI for pushd-web development workflows"));
    Ok(())
}

#[test]
fn test_cli_version() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("pushd-cli 0.1.0"));
    Ok(())
}

#[test]
fn test_worktree_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["worktree", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Worktree management"));
    Ok(())
}

#[test]
fn test_daemon_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["daemon", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Daemon operations"));
    Ok(())
}

#[test]
fn test_docker_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["docker", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Docker operations"));
    Ok(())
}

#[test]
fn test_git_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["git", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Git safety and validation"));
    Ok(())
}

#[test]
fn test_setup_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["setup", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Environment setup"));
    Ok(())
}

#[test]
fn test_component_help() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["component", "--help"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Component management"));
    Ok(())
}

#[test]
fn test_worktree_list() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["worktree", "list"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Available worktrees"));
    Ok(())
}

#[test]
fn test_daemon_list() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["daemon", "list"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Available daemons"));
    Ok(())
}

#[test]
fn test_config_show() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["config"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Configuration"));
    Ok(())
}

#[test]
fn test_invalid_command() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("error"));
    Ok(())
}

#[test]
fn test_dry_run_flag() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["--dry-run", "daemon", "list"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Available daemons"));
    Ok(())
}

#[test]
fn test_aws_profile_flag() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["--aws-profile", "test-profile", "config"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Configuration"));
    Ok(())
}

#[test]
fn test_target_env_flag() -> anyhow::Result<()> {
    let mut cmd = Command::cargo_bin("pushd-cli")?;
    cmd.args(&["--target-env", "development", "config"]);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Configuration"));
    Ok(())
} 
