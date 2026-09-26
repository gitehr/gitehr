// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::tempdir;

use gitehr::commands::state::{list_state_files, update_state_file, view_state_file};

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    fs::create_dir_all("state").ok();
    temp_dir
}

fn git(args: &[&str]) -> Result<()> {
    let output = Command::new("git").args(args).output()?;
    if !output.status.success() {
        anyhow::bail!(
            "git {:?}: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

#[test]
#[serial]
fn test_list_state_files_empty() -> Result<()> {
    let _temp_dir = setup();

    let files = list_state_files()?;

    assert_eq!(files.len(), 0, "No files should exist initially");

    Ok(())
}

#[test]
#[serial]
fn test_update_state_file() -> Result<()> {
    let _temp_dir = setup();

    let filename = "test_state.txt";
    let content = "test content here";

    update_state_file(filename, content)?;

    let file_path = Path::new("state").join(filename);
    assert!(file_path.exists(), "File should be created");

    let saved_content = fs::read_to_string(file_path)?;
    assert_eq!(saved_content, content, "Content should match");

    Ok(())
}

#[test]
#[serial]
fn test_view_state_file() -> Result<()> {
    let _temp_dir = setup();

    let filename = "view_test.txt";
    let content = "file content for viewing";

    update_state_file(filename, content)?;

    let state_file = view_state_file(filename)?;

    assert_eq!(state_file.name, filename);
    assert_eq!(state_file.content, content);
    assert!(state_file.last_modified.is_some());

    Ok(())
}

#[test]
#[serial]
fn test_view_nonexistent_state_file() -> Result<()> {
    let _temp_dir = setup();

    let result = view_state_file("nonexistent.txt");

    assert!(result.is_err(), "Should fail for nonexistent file");

    Ok(())
}

#[test]
#[serial]
fn test_list_state_files_multiple() -> Result<()> {
    let _temp_dir = setup();

    update_state_file("file1.txt", "content1")?;
    update_state_file("file2.txt", "content2")?;
    update_state_file("file3.txt", "content3")?;

    let files = list_state_files()?;

    assert_eq!(files.len(), 3, "Should list all created files");

    let names: Vec<_> = files.iter().map(|f| &f.name).collect();
    assert!(names.contains(&&"file1.txt".to_string()));
    assert!(names.contains(&&"file2.txt".to_string()));
    assert!(names.contains(&&"file3.txt".to_string()));

    Ok(())
}

#[test]
#[serial]
fn test_list_state_files_excludes_readme() -> Result<()> {
    let _temp_dir = setup();

    fs::write("state/README.md", "This is readme")?;
    update_state_file("actual_file.txt", "content")?;

    let files = list_state_files()?;

    assert_eq!(files.len(), 1, "README.md should be excluded");
    assert_eq!(files[0].name, "actual_file.txt");

    Ok(())
}

#[test]
#[serial]
fn test_update_state_file_overwrites() -> Result<()> {
    let _temp_dir = setup();

    let filename = "overwrite_test.txt";
    update_state_file(filename, "original content")?;
    update_state_file(filename, "new content")?;

    let state_file = view_state_file(filename)?;
    assert_eq!(
        state_file.content, "new content",
        "Content should be updated"
    );

    Ok(())
}

#[test]
#[serial]
fn test_state_files_sorted_alphabetically() -> Result<()> {
    let _temp_dir = setup();

    update_state_file("zebra.txt", "z")?;
    update_state_file("apple.txt", "a")?;
    update_state_file("banana.txt", "b")?;

    let files = list_state_files()?;

    assert_eq!(files[0].name, "apple.txt");
    assert_eq!(files[1].name, "banana.txt");
    assert_eq!(files[2].name, "zebra.txt");

    Ok(())
}

#[test]
#[serial]
fn test_state_files_have_modification_time() -> Result<()> {
    let _temp_dir = setup();

    update_state_file("time_test.txt", "content")?;

    let state_file = view_state_file("time_test.txt")?;

    assert!(
        state_file.last_modified.is_some(),
        "Should have modification time"
    );

    let mod_time = state_file.last_modified.unwrap();
    assert!(mod_time.contains("T"), "Time format should be ISO 8601");
    assert!(mod_time.contains("Z"), "Time should be in UTC");

    Ok(())
}

#[test]
#[serial]
fn test_update_state_file_creates_subdirectories() -> Result<()> {
    let _temp_dir = setup();

    update_state_file("calculations/feverpain-latest.json", "{\"result\":3}")?;

    let file_path = Path::new("state")
        .join("calculations")
        .join("feverpain-latest.json");
    assert!(file_path.exists(), "Nested file should be created");
    assert_eq!(fs::read_to_string(file_path)?, "{\"result\":3}");

    Ok(())
}

#[test]
#[serial]
fn test_nested_state_files_are_listed_and_readable() -> Result<()> {
    let _temp_dir = setup();

    update_state_file("calculations/feverpain-latest.json", "{\"result\":3}")?;

    let files = list_state_files()?;
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].name, "calculations/feverpain-latest.json");
    assert_eq!(
        view_state_file("calculations/feverpain-latest.json")?.content,
        "{\"result\":3}"
    );

    Ok(())
}

#[test]
#[serial]
fn test_state_set_reads_content_from_stdin() -> Result<()> {
    let _temp_dir = setup();

    let mut child = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args([
            "state",
            "set",
            "calculations/feverpain-latest.json",
            "--file",
            "-",
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .expect("child stdin was piped")
        .write_all(b"{\"result\":3}")?;
    let output = child.wait_with_output()?;

    assert!(output.status.success());
    assert_eq!(
        fs::read_to_string("state/calculations/feverpain-latest.json")?,
        "{\"result\":3}"
    );

    Ok(())
}

#[test]
#[serial]
fn test_state_set_commit_leaves_unrelated_staged_changes_untouched() -> Result<()> {
    let _temp_dir = setup();
    git(&["init"])?;
    git(&["config", "user.email", "test@example.com"])?;
    git(&["config", "user.name", "Test User"])?;
    git(&["config", "commit.gpgsign", "false"])?;
    fs::write("unrelated.txt", "keep staged")?;
    git(&["add", "unrelated.txt"])?;

    let mut child = Command::new(env!("CARGO_BIN_EXE_gitehr"))
        .args([
            "state",
            "set",
            "calculations/feverpain-latest.json",
            "--file",
            "-",
            "--commit",
        ])
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .expect("child stdin was piped")
        .write_all(b"{\"result\":3}")?;
    assert!(child.wait()?.success());

    let staged = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()?;
    assert_eq!(String::from_utf8(staged.stdout)?.trim(), "unrelated.txt");
    let committed = Command::new("git")
        .args(["show", "--format=", "--name-only", "HEAD"])
        .output()?;
    assert_eq!(
        String::from_utf8(committed.stdout)?.trim(),
        "state/calculations/feverpain-latest.json"
    );

    Ok(())
}

#[test]
#[serial]
fn test_update_state_file_rejects_path_traversal() -> Result<()> {
    let _temp_dir = setup();

    let result = update_state_file("../escape.txt", "content");
    assert!(result.is_err(), "Should reject a filename with '..'");

    let result = update_state_file("calculations/../../escape.txt", "content");
    assert!(result.is_err(), "Should reject a nested filename with '..'");

    assert!(!Path::new("escape.txt").exists());

    fs::write("outside.txt", "outside")?;
    assert!(view_state_file("../outside.txt").is_err());

    Ok(())
}

#[cfg(unix)]
#[test]
#[serial]
fn test_nested_state_files_refuse_symlinked_directories() -> Result<()> {
    use std::os::unix::fs::symlink;

    let _temp_dir = setup();
    fs::create_dir("outside")?;
    symlink("../outside", "state/calculations")?;

    assert!(update_state_file("calculations/feverpain-latest.json", "{}").is_err());
    assert!(!Path::new("outside/feverpain-latest.json").exists());

    Ok(())
}
