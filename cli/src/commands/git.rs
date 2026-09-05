// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::Path;
use std::process::{Command, Output};

/// Execute a git command with the given arguments in `dir`. Callers that
/// operate on the process's own working directory (the common case) pass
/// `.`; callers given an explicit repository path (like the MCP server,
/// which may run with a different cwd) pass that path instead, so the
/// command always targets the intended repository.
fn git_output_in(dir: &Path, args: &[&str]) -> Result<Output> {
    let output = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow::anyhow!("Git binary not found. Please install git to use this feature.")
            } else {
                anyhow::anyhow!("Failed to execute git command: {}", e)
            }
        })?;

    Ok(output)
}

fn run_git_command_in(dir: &Path, args: &[&str]) -> Result<()> {
    let output = git_output_in(dir, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr.trim());
    }

    Ok(())
}

/// Initialize a new git repository
pub fn git_init() -> Result<()> {
    run_git_command_in(Path::new("."), &["init"])
}

/// Stage a file for commit, relative to the process's current directory.
pub fn git_add(file_path: &str) -> Result<()> {
    git_add_in(Path::new("."), file_path)
}

/// Create a commit with the given message, in the process's current directory.
pub fn git_commit(message: &str) -> Result<()> {
    git_commit_in(Path::new("."), message)
}

/// Stage a file for commit in a specific repository directory.
pub fn git_add_in(repo_path: &Path, file_path: &str) -> Result<()> {
    // `--` stops git parsing a path that begins with `-` as an option.
    run_git_command_in(repo_path, &["add", "--", file_path])
}

/// Create a commit with the given message in a specific repository directory.
pub fn git_commit_in(repo_path: &Path, message: &str) -> Result<()> {
    run_git_command_in(repo_path, &["commit", "-m", message])
}

/// Return the porcelain status for one path, or an empty string when clean.
pub(crate) fn git_path_status(file_path: &str) -> Result<String> {
    let output = git_output_in(
        Path::new("."),
        &[
            "status",
            "--porcelain",
            "--untracked-files=all",
            "--",
            file_path,
        ],
    )?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr.trim());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Commit only the named paths, leaving unrelated staged work untouched.
pub(crate) fn git_commit_paths(message: &str, file_paths: &[&str]) -> Result<()> {
    let mut args = vec!["commit", "--only", "-m", message, "--"];
    args.extend_from_slice(file_paths);
    run_git_command_in(Path::new("."), &args)
}

/// Remove named paths from the index while preserving their working-tree files.
/// This also works in a newly initialised repository with no `HEAD` commit.
pub(crate) fn git_unstage_paths(file_paths: &[&str]) -> Result<()> {
    let has_head = git_output_in(Path::new("."), &["rev-parse", "--verify", "HEAD"])?
        .status
        .success();
    let mut args = if has_head {
        vec!["restore", "--staged", "--"]
    } else {
        vec!["rm", "--cached", "--quiet", "--ignore-unmatch", "--"]
    };
    args.extend_from_slice(file_paths);
    run_git_command_in(Path::new("."), &args)
}

/// Get the installed git version string
pub fn get_git_version() -> Option<String> {
    use std::process::Command;
    let output = Command::new("git").arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.trim().to_string())
}
