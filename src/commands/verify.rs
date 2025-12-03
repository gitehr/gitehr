use crate::commands::journal::JournalEntryHeader;
use crate::error::{Result, VerifyError};
use crate::fs_layout::FsLayout;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn verify_journal() -> Result<()> {
    let layout = FsLayout::new();
    let patients_dir = layout.journal_patients_dir();
    if !patients_dir.exists() {
        return Err(VerifyError::MissingJournal.into());
    }

    let mut total_entries = 0usize;

    let patients_iter = fs::read_dir(&patients_dir).map_err(VerifyError::Io)?;
    for patient_dir in patients_iter {
        let patient_dir = patient_dir.map_err(VerifyError::Io)?;
        if !patient_dir.file_type().map_err(VerifyError::Io)?.is_dir() {
            continue;
        }

        let patient_id = patient_dir.file_name().to_string_lossy().to_string();
        let mut entries = Vec::new();
        let dir_iter = fs::read_dir(patient_dir.path()).map_err(VerifyError::Io)?;
        for entry in dir_iter {
            entries.push(entry.map_err(VerifyError::Io)?);
        }
        entries.sort_by_key(|e| e.file_name());

        let mut seen_events: HashMap<String, PathBuf> = HashMap::new();

        for entry in entries {
            total_entries += 1;
            let filename = entry.file_name().to_string_lossy().to_string();
            let entry_path = entry.path();
            let content = fs::read_to_string(&entry_path).map_err(VerifyError::Io)?;

            let yaml_content =
                content
                    .split("---")
                    .nth(1)
                    .ok_or_else(|| VerifyError::InvalidEntry {
                        entry_file: filename.clone(),
                        reason: "Missing YAML front matter".to_string(),
                    })?;

            let header: JournalEntryHeader =
                serde_yaml::from_str(yaml_content).map_err(|e| VerifyError::InvalidEntry {
                    entry_file: filename.clone(),
                    reason: format!("Invalid YAML: {e}"),
                })?;

            if header.gitehr_patient_id != patient_id {
                return Err(VerifyError::PatientMismatch {
                    entry_file: filename,
                    expected: patient_id,
                    found: header.gitehr_patient_id,
                }
                .into());
            }

            if let Some(correction) = &header.correction_of {
                if !seen_events.contains_key(correction) {
                    return Err(VerifyError::MissingCorrectionTarget {
                        entry_file: filename.clone(),
                        correction_of: correction.clone(),
                    }
                    .into());
                }
            }

            seen_events.insert(header.gitehr_event_id.clone(), entry_path);
        }
    }

    println!(
        "Journal verification successful: {} entries verified",
        total_entries
    );
    Ok(())
}
