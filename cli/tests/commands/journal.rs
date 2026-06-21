use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use serial_test::serial;
use std::fs;
use tempfile::tempdir;

use gitehr::commands::journal::{create_journal_entry, parsed_entries, sorted_entries};

fn setup_with_git() -> Result<tempfile::TempDir> {
    let temp_dir = tempdir()?;
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;
    // Initialize git repository
    std::process::Command::new("git").args(["init"]).output()?;
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .output()?;
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .output()?;
    Ok(temp_dir)
}

#[test]
#[serial]
fn test_create_journal_entry() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    let content = "Test entry";
    create_journal_entry(content)?;

    let entries: Vec<_> = fs::read_dir("journal")?.collect();
    assert_eq!(entries.len(), 1, "Expected one journal entry");

    let mut entries: Vec<_> = entries.into_iter().map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    let entry = &entries[0];
    let file_type = entry.file_type()?;
    assert!(file_type.is_file(), "Journal entry should be a file");
    let entry_path = entry.path();
    let filename = entry_path.file_name().unwrap().to_string_lossy();

    let re = Regex::new(r"^\d{8}T\d{6}\.\d{3}Z-[0-9a-f-]{36}\.md$").unwrap();
    assert!(
        re.is_match(&filename),
        "Filename should match expected format"
    );

    let file_content = fs::read_to_string(&entry_path)?;

    let yaml_content = file_content
        .split("---")
        .nth(1)
        .expect("No YAML front matter found");
    let entry: gitehr::commands::journal::JournalEntry = serde_yaml_ng::from_str(yaml_content)?;

    assert!(
        entry.timestamp <= Utc::now(),
        "Timestamp should be in the past"
    );
    assert!(file_content.contains(content), "Entry content not found");

    Ok(())
}

#[test]
#[serial]
fn test_entries_sorted_newest_first() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    create_journal_entry("First entry")?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    create_journal_entry("Second entry")?;

    let sorted = sorted_entries(false)?;
    assert_eq!(sorted.len(), 2, "Expected two entries");

    // sorted_entries returns newest-first, so the most recent is index 0.
    let newest_path = std::path::Path::new("journal").join(&sorted[0]);
    let newest_content = fs::read_to_string(&newest_path)?;
    assert!(
        newest_content.contains("Second entry"),
        "Newest entry should be the most recently created"
    );

    let oldest_path = std::path::Path::new("journal").join(&sorted[1]);
    let oldest_content = fs::read_to_string(&oldest_path)?;
    assert!(
        oldest_content.contains("First entry"),
        "Oldest entry should be the first created"
    );

    Ok(())
}

#[test]
#[serial]
fn test_parsed_entries_reads_back_content() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    create_journal_entry("First entry")?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    create_journal_entry("Second entry")?;

    // parsed_entries returns oldest-first.
    let parsed = parsed_entries()?;
    assert_eq!(parsed.len(), 2, "Expected two parsed entries");
    assert_eq!(parsed[0].content, "First entry");
    assert_eq!(parsed[1].content, "Second entry");

    // Timestamps are recorded and ordered.
    assert!(
        parsed[0].metadata.timestamp <= parsed[1].metadata.timestamp,
        "Entries parsed oldest-first should have non-decreasing timestamps"
    );

    Ok(())
}

#[test]
#[serial]
fn test_timestamp_uniqueness_and_ordering() -> Result<()> {
    let _temp_dir = setup_with_git()?;

    for i in 0..5 {
        create_journal_entry(&format!("Entry {}", i))?;
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    let mut timestamps = Vec::new();
    for entry in fs::read_dir("journal")? {
        let filename = entry?.file_name().into_string().unwrap();
        let timestamp = filename.split('-').next().unwrap();
        timestamps.push(timestamp.to_string());
    }

    assert_eq!(timestamps.len(), 5, "Should have exactly 5 entries");

    let unique_timestamps: std::collections::HashSet<_> = timestamps.iter().cloned().collect();
    assert_eq!(
        timestamps.len(),
        unique_timestamps.len(),
        "All timestamps should be unique"
    );

    Ok(())
}
