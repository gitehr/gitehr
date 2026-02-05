use anyhow::Result;
use serial_test::serial;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::state::{list_state_files, update_state_file, view_state_file};

fn setup() -> tempfile::TempDir {
    let temp_dir = tempdir().unwrap();
    let _ = std::env::set_current_dir(&temp_dir);
    fs::create_dir_all(".gitehr").ok();
    fs::create_dir_all("state").ok();
    temp_dir
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
