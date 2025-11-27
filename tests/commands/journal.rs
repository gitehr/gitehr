use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use sha2::Digest;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::journal::{create_journal_entry, get_latest_journal_entry};

// Common test setup function
fn setup() -> tempfile::TempDir {
    tempdir().unwrap()
}

#[test]
fn test_create_journal_entry() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    let content = "Test entry";
    let parent_hash = Some("test_hash".to_string());

    create_journal_entry(content, parent_hash.clone())?;

    // Verify we have two files (genesis entry + our new entry)
    let entries: Vec<_> = fs::read_dir("journal")?.collect();
    assert_eq!(
        entries.len(),
        2,
        "Expected two journal entries (genesis + new entry)"
    );

    // Get the second (newest) file and verify it's a readable file
    let mut entries: Vec<_> = entries.into_iter().map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    let entry = &entries[1];
    let file_type = entry.file_type()?;
    assert!(file_type.is_file(), "Journal entry should be a file");
    let entry_path = entry.path();
    let filename = entry_path.file_name().unwrap().to_string_lossy();

    // Verify filename format (YYYYMMDDTHHMMSS.mmmZ-uuid.md)
    let re = Regex::new(r"^\d{8}T\d{6}\.\d{3}Z-[0-9a-f-]{36}\.md$").unwrap();
    assert!(re.is_match(&filename));

    // Read and verify content
    let file_content = fs::read_to_string(&entry_path)?;

    // The YAML front matter should be between the first two "---" markers
    let yaml_content = file_content
        .split("---")
        .nth(1)
        .expect("No YAML front matter found");
    let entry: gitehr::commands::journal::JournalEntry = serde_yaml::from_str(yaml_content)?;

    assert_eq!(
        entry.parent_hash,
        Some("test_hash".to_string()),
        "Parent hash doesn't match"
    );
    assert!(
        entry.parent_entry.is_none(),
        "Parent entry should be None for test_hash that doesn't exist"
    );
    assert!(
        entry.timestamp <= Utc::now(),
        "Timestamp should be in the past"
    );
    assert!(file_content.contains(content), "Entry content not found");

    Ok(())
}

#[test]
fn test_get_latest_journal_entry() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    // Create two entries with a small delay
    create_journal_entry("First entry", None)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    create_journal_entry("Second entry", None)?;

    // Get latest entry
    let latest = get_latest_journal_entry()?;
    assert!(latest.is_some());
    let (filename, hash) = latest.unwrap();

    // Verify it's the second entry
    let content = fs::read_to_string(Path::new("journal").join(&filename))?;
    assert!(content.contains("Second entry"));

    // Verify the hash matches the content
    let calculated_hash = format!("{:x}", sha2::Sha256::digest(content.as_bytes()));
    assert_eq!(
        hash, calculated_hash,
        "File hash does not match expected hash"
    );

    Ok(())
}

#[test]
fn test_parent_entry_linking() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    // Create first entry
    create_journal_entry("First entry", None)?;

    // Get its hash
    let latest = get_latest_journal_entry()?.unwrap();
    let (first_filename, first_hash) = latest;

    // Create second entry with first as parent
    create_journal_entry("Second entry", Some(first_hash))?;

    // Verify second entry's parent links
    let entries: Vec<_> = fs::read_dir("journal")?.collect();
    let mut entries: Vec<_> = entries.into_iter().map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    let second_entry = &entries[1];

    let content = fs::read_to_string(second_entry.path())?;
    let yaml_content = content
        .split("---")
        .nth(1)
        .expect("No YAML front matter found");
    let entry: gitehr::commands::journal::JournalEntry = serde_yaml::from_str(yaml_content)?;

    assert_eq!(
        entry.parent_entry,
        Some(first_filename),
        "Parent entry filename should match first entry"
    );

    Ok(())
}

#[test]
fn test_timestamp_ordering() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    // Create multiple entries in quick succession
    for i in 0..5 {
        create_journal_entry(&format!("Entry {}", i), None)?;
    }

    // Get all files and verify they have unique timestamps
    let mut timestamps = Vec::new();
    for entry in fs::read_dir("journal")? {
        let filename = entry?.file_name().into_string().unwrap();
        let timestamp = filename.split('-').next().unwrap();
        timestamps.push(timestamp.to_string());
    }

    // Verify all timestamps are unique
    let unique_timestamps: std::collections::HashSet<_> = timestamps.iter().cloned().collect();
    assert_eq!(timestamps.len(), unique_timestamps.len());

    Ok(())
}
