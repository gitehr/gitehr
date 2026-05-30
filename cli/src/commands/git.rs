use anyhow::Result;
use std::process::Command;

/// Execute a git command with the given arguments
fn run_git_command(args: &[&str]) -> Result<()> {
    let output = Command::new("git").args(args).output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("Git binary not found. Please install git to use this feature.")
        } else {
            anyhow::anyhow!("Failed to execute git command: {}", e)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr.trim());
    }

    Ok(())
}

/// Initialize a new git repository
pub fn git_init() -> Result<()> {
    run_git_command(&["init"])
}

/// Stage a file for commit
pub fn git_add(file_path: &str) -> Result<()> {
    run_git_command(&["add", file_path])
}

/// Create a commit with the given message
pub fn git_commit(message: &str) -> Result<()> {
    run_git_command(&["commit", "-m", message])
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
