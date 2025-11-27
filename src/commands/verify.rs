use crate::commands::journal::JournalEntry;
use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

/// Represents a verification error in the journal
#[derive(Debug)]
pub enum JournalVerificationError {
    ParentNotFound {
        entry_file: String,
        parent_hash: String,
    },
    BrokenChain {
        entry_file: String,
        reason: String,
    },
}

impl std::fmt::Display for JournalVerificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParentNotFound {
                entry_file,
                parent_hash,
            } => {
                write!(
                    f,
                    "Parent entry not found for {}: hash {}",
                    entry_file, parent_hash
                )
            }
            Self::BrokenChain { entry_file, reason } => {
                write!(f, "Chain broken at entry {}: {}", entry_file, reason)
            }
        }
    }
}

impl std::error::Error for JournalVerificationError {}

pub fn verify_journal() -> Result<()> {
    let journal_dir = PathBuf::from("journal");
    if !journal_dir.exists() {
        return Err(anyhow!("Journal directory not found"));
    }

    // Get all entries
    let mut entries: Vec<_> = fs::read_dir(&journal_dir)?.filter_map(|e| e.ok()).collect();

    // Sort by filename (which contains timestamp)
    entries.sort_by_key(|e| e.file_name());

    // Create a map of hashes to filenames for quick lookup
    let mut hash_map = std::collections::HashMap::new();
    for entry in &entries {
        let content = fs::read_to_string(entry.path())?;
        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        hash_map.insert(hash, entry.file_name().to_string_lossy().to_string());
    }

    // Verify each entry
    for entry in &entries {
        let filename = entry.file_name().to_string_lossy().to_string();
        let content = fs::read_to_string(entry.path())?;

        // Parse YAML front matter
        let yaml_content =
            content
                .split("---")
                .nth(1)
                .ok_or_else(|| JournalVerificationError::BrokenChain {
                    entry_file: filename.clone(),
                    reason: "Missing YAML front matter".to_string(),
                })?;

        let entry_data: JournalEntry = serde_yaml::from_str(yaml_content).map_err(|e| {
            JournalVerificationError::BrokenChain {
                entry_file: filename.clone(),
                reason: format!("Invalid YAML: {}", e),
            }
        })?;

        // For non-genesis entries, verify parent hash exists and matches
        if let Some(parent_hash) = entry_data.parent_hash {
            if !hash_map.contains_key(&parent_hash) {
                return Err(anyhow!(JournalVerificationError::ParentNotFound {
                    entry_file: filename,
                    parent_hash,
                }));
            }

            // Verify that parent_entry matches the filename we found
            if let Some(parent_entry) = entry_data.parent_entry {
                if hash_map[&parent_hash] != parent_entry {
                    return Err(anyhow!(JournalVerificationError::BrokenChain {
                        entry_file: filename,
                        reason: format!(
                            "Parent entry filename mismatch: expected {}, got {}",
                            parent_entry, hash_map[&parent_hash]
                        ),
                    }));
                }
            } else {
                return Err(anyhow!(JournalVerificationError::BrokenChain {
                    entry_file: filename,
                    reason: "Missing parent_entry in YAML".to_string(),
                }));
            }
        }
    }

    println!(
        "Journal verification successful: {} entries verified",
        entries.len()
    );
    Ok(())
}
