// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use chrono::Utc;
use regex::Regex;
use serial_test::serial;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

use gitehr::commands::journal::{
    approve_mcp_draft, create_journal_entry, create_mcp_draft_entry, mcp_drafts, parsed_entries,
    reject_mcp_draft, sorted_entries,
};

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

    let sorted = sorted_entries()?;
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

// ── ADR-0007: MCP drafts ──────────────────────────────────────────────────────

#[test]
#[serial]
fn mcp_draft_full_lifecycle() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    fs::create_dir_all("journal")?;
    fs::create_dir_all(".gitehr")?;
    fs::write(
        ".gitehr/contributors.json",
        r#"{"contributors":{},"current_contributor":null}"#,
    )?;
    Command::new("git").args(["init"]).output()?;
    Command::new("git")
        .args(["config", "user.name", "Dr Jones"])
        .output()?;
    Command::new("git")
        .args(["config", "user.email", "jones@example.org"])
        .output()?;
    Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .output()?;

    // Write a draft (as the MCP tool would).
    let filename = create_mcp_draft_entry(
        Path::new("."),
        "AI-drafted note",
        Some("assistant-x".into()),
    )?;

    // Draft exists, is marked, and nothing is committed.
    let drafts = mcp_drafts(Path::new("."))?;
    assert_eq!(drafts.len(), 1, "one pending draft");
    assert!(fs::read_to_string(&filename)?.contains("mcp_draft: true"));
    let log = Command::new("git").args(["log", "--oneline"]).output()?;
    assert_eq!(
        String::from_utf8_lossy(&log.stdout).lines().count(),
        0,
        "draft must not be committed"
    );

    // Approve: marker stripped, staged, committed, committer is the human.
    approve_mcp_draft(Path::new("."), &filename)?;
    let committed = fs::read_to_string(&filename)?;
    assert!(
        !committed.contains("mcp_draft"),
        "approval strips the marker"
    );
    let log = Command::new("git").args(["log", "--oneline"]).output()?;
    assert_eq!(
        String::from_utf8_lossy(&log.stdout).lines().count(),
        1,
        "approval commits"
    );
    assert!(mcp_drafts(Path::new("."))?.is_empty(), "no drafts remain");

    Ok(())
}

#[test]
#[serial]
fn mcp_draft_reject_deletes_the_file() -> Result<()> {
    let temp = tempdir().unwrap();
    std::env::set_current_dir(&temp)?;

    fs::create_dir_all("journal")?;
    fs::create_dir_all(".gitehr")?;
    Command::new("git").args(["init"]).output()?;

    let filename = create_mcp_draft_entry(Path::new("."), "bad draft", None)?;
    reject_mcp_draft(Path::new("."), &filename)?;

    assert!(!Path::new(&filename).exists());
    assert!(mcp_drafts(Path::new("."))?.is_empty());

    // Refusing to delete a committed entry: write a normal entry, then reject it.
    create_journal_entry("real entry")?;
    let entries = parsed_entries()?;
    let committed_name = &entries[0].filename;
    assert!(
        reject_mcp_draft(Path::new("."), committed_name).is_err(),
        "reject must refuse non-draft entries"
    );
    assert!(Path::new("journal").join(committed_name).exists());
    Ok(())
}
