use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use serial_test::serial;
use sha2::Digest;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

use gitehr::commands::journal::{create_journal_entry, get_latest_journal_entry};

fn setup() -> tempfile::TempDir {
    tempdir().unwrap()
}

#[test]
#[serial]
fn test_create_journal_entry() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    let content = "Test entry";
    let parent_hash = Some("test_hash".to_string());

    create_journal_entry(content, parent_hash.clone())?;

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
    let entry: gitehr::commands::journal::JournalEntry = serde_yml::from_str(yaml_content)?;

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
#[serial]
fn test_get_latest_journal_entry() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    create_journal_entry("First entry", None)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    create_journal_entry("Second entry", None)?;

    let latest = get_latest_journal_entry()?;
    assert!(latest.is_some());
    let (filename, hash) = latest.unwrap();

    let content = fs::read_to_string(Path::new("journal").join(&filename))?;
    assert!(content.contains("Second entry"));

    let calculated_hash = format!("{:x}", sha2::Sha256::digest(content.as_bytes()));
    assert_eq!(
        hash, calculated_hash,
        "File hash does not match expected hash"
    );

    Ok(())
}

#[test]
#[serial]
fn test_parent_entry_linking() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    create_journal_entry("First entry", None)?;

    let latest = get_latest_journal_entry()?.unwrap();
    let (first_filename, first_hash) = latest;

    std::thread::sleep(std::time::Duration::from_millis(5));

    create_journal_entry("Second entry", Some(first_hash))?;

    let entries: Vec<_> = fs::read_dir("journal")?.collect();
    let mut entries: Vec<_> = entries.into_iter().map(|e| e.unwrap()).collect();
    entries.sort_by_key(|e| e.file_name());
    let second_entry = &entries[1];

    let content = fs::read_to_string(second_entry.path())?;
    let yaml_content = content
        .split("---")
        .nth(1)
        .expect("No YAML front matter found");
    let entry: gitehr::commands::journal::JournalEntry = serde_yml::from_str(yaml_content)?;

    assert_eq!(
        entry.parent_entry,
        Some(first_filename),
        "Parent entry filename should match first entry"
    );

    Ok(())
}

#[test]
#[serial]
fn test_timestamp_ordering() -> Result<()> {
    let temp_dir = setup();
    std::env::set_current_dir(&temp_dir)?;
    fs::create_dir("journal")?;

    for i in 0..5 {
        create_journal_entry(&format!("Entry {}", i), None)?;
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
