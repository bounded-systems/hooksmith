use std::collections::HashMap;
use std::process::Command;

/// Git view/state that a command operates on
#[derive(Debug, Clone, PartialEq)]
pub enum GitView {
    /// Index (staged files)
    Index,
    /// Working directory (unstaged changes)
    WorkingDirectory,
    /// Commit tree (HEAD or specific commit)
    CommitTree,
    /// Mixed views (e.g., status shows all)
    Mixed,
}

/// Git command with its scope and description
#[derive(Debug, Clone)]
pub struct GitCommand {
    pub command: String,
    pub args: Vec<String>,
    pub view: GitView,
    pub description: String,
    pub use_case: String,
}

impl GitCommand {
    pub fn new(
        command: &str,
        args: Vec<String>,
        view: GitView,
        description: &str,
        use_case: &str,
    ) -> Self {
        Self {
            command: command.to_string(),
            args,
            view,
            description: description.to_string(),
            use_case: use_case.to_string(),
        }
    }
}

/// Collection of Git file-listing commands with their scopes
pub struct GitQueryCommands;

impl GitQueryCommands {
    /// Get all tracked files in the index (staged)
    pub fn ls_files_index() -> GitCommand {
        GitCommand::new(
            "git",
            vec!["ls-files".to_string()],
            GitView::Index,
            "Lists all tracked files in the index",
            "Validate file structure of committed/staged files",
        )
    }

    /// Get all files in the working directory (including untracked)
    pub fn ls_files_working() -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "ls-files".to_string(),
                "--others".to_string(),
                "--exclude-standard".to_string(),
            ],
            GitView::WorkingDirectory,
            "Lists untracked files in working directory",
            "Validate file structure including new files",
        )
    }

    /// Get all files in HEAD commit tree
    pub fn ls_tree_head() -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "ls-tree".to_string(),
                "-r".to_string(),
                "--name-only".to_string(),
                "HEAD".to_string(),
            ],
            GitView::CommitTree,
            "Lists all files and folders in HEAD commit",
            "Validate directory structure of committed state",
        )
    }

    /// Get modified files (unstaged changes)
    pub fn diff_working() -> GitCommand {
        GitCommand::new(
            "git",
            vec!["diff".to_string(), "--name-only".to_string()],
            GitView::WorkingDirectory,
            "Lists files changed between index and working tree",
            "Validate changes in working directory",
        )
    }

    /// Get staged files (index vs HEAD)
    pub fn diff_staged() -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "diff".to_string(),
                "--cached".to_string(),
                "--name-only".to_string(),
            ],
            GitView::Index,
            "Lists files changed between index and HEAD",
            "Validate staged changes",
        )
    }

    /// Get all changes (working + staged vs HEAD)
    pub fn diff_all() -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "diff".to_string(),
                "HEAD".to_string(),
                "--name-only".to_string(),
            ],
            GitView::Mixed,
            "Lists all files changed since last commit",
            "Validate all uncommitted changes",
        )
    }

    /// Get working tree status
    pub fn status_porcelain() -> GitCommand {
        GitCommand::new(
            "git",
            vec!["status".to_string(), "--porcelain".to_string()],
            GitView::Mixed,
            "Shows working tree state with paths and change type",
            "Get comprehensive file status",
        )
    }

    /// Get files changed in a specific commit
    pub fn show_commit(commit: &str) -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "show".to_string(),
                "--name-only".to_string(),
                commit.to_string(),
            ],
            GitView::CommitTree,
            "Lists files changed in a specific commit",
            "Validate changes in a specific commit",
        )
    }

    /// Get files changed in a commit (faster than show)
    pub fn diff_tree_commit(commit: &str) -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "diff-tree".to_string(),
                "--no-commit-id".to_string(),
                "--name-only".to_string(),
                "-r".to_string(),
                commit.to_string(),
            ],
            GitView::CommitTree,
            "Lists file changes in a commit (faster than show)",
            "Validate changes in a specific commit (optimized)",
        )
    }

    /// Search for files matching a pattern
    pub fn grep_files(pattern: &str) -> GitCommand {
        GitCommand::new(
            "git",
            vec![
                "grep".to_string(),
                "--name-only".to_string(),
                pattern.to_string(),
            ],
            GitView::Index,
            "Lists file paths with matches",
            "Find files matching a pattern",
        )
    }
}

/// Execute a Git command and return the output
pub fn execute_git_command(cmd: &GitCommand) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new(&cmd.command).args(&cmd.args).output()?;

    if !output.status.success() {
        return Err(format!(
            "Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let paths: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect();

    Ok(paths)
}

/// Get a summary of all available Git commands
pub fn get_git_commands_summary() -> HashMap<String, GitCommand> {
    let mut commands = HashMap::new();

    commands.insert(
        "ls_files_index".to_string(),
        GitQueryCommands::ls_files_index(),
    );
    commands.insert(
        "ls_files_working".to_string(),
        GitQueryCommands::ls_files_working(),
    );
    commands.insert("ls_tree_head".to_string(), GitQueryCommands::ls_tree_head());
    commands.insert("diff_working".to_string(), GitQueryCommands::diff_working());
    commands.insert("diff_staged".to_string(), GitQueryCommands::diff_staged());
    commands.insert("diff_all".to_string(), GitQueryCommands::diff_all());
    commands.insert(
        "status_porcelain".to_string(),
        GitQueryCommands::status_porcelain(),
    );

    commands
}

/// Git view matrix for reference
pub fn get_git_view_matrix() -> Vec<(&'static str, GitView, &'static str)> {
    vec![
        (
            "git ls-files",
            GitView::Index,
            "Shows all files currently tracked in the index",
        ),
        (
            "git ls-files --others",
            GitView::WorkingDirectory,
            "Shows untracked files",
        ),
        (
            "git ls-tree HEAD",
            GitView::CommitTree,
            "Lists files as stored in HEAD commit",
        ),
        (
            "git diff",
            GitView::WorkingDirectory,
            "Compares working changes vs index",
        ),
        (
            "git diff --cached",
            GitView::Index,
            "Compares staged changes vs last commit",
        ),
        (
            "git diff HEAD",
            GitView::Mixed,
            "Compares working + staged changes vs last commit",
        ),
        (
            "git status",
            GitView::Mixed,
            "Summarizes everything (committed, staged, unstaged, untracked)",
        ),
        (
            "git show <commit>",
            GitView::CommitTree,
            "Shows changes in a specific commit",
        ),
        (
            "git diff-tree <commit>",
            GitView::CommitTree,
            "Lists changed files in a specific commit",
        ),
    ]
}
