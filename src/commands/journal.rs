use crate::error::{JournalError, Result};
use crate::fs_layout::FsLayout;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

// TODO: Need to add registration number for authors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextInfo {
    // TODO: Need to think of a better way to handle this. Consider using a enum with different types of context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeItem {
    pub system: String,
    pub code: String,
    pub display: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalCodes {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnoses: Vec<CodeItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedures: Vec<CodeItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalLinks {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_imaging: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryHeader {
    pub gitehr_event_id: String,
    pub gitehr_patient_id: String,
    pub recorded_at: DateTime<Utc>,
    pub author: AuthorInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<JournalLinks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codes: Option<JournalCodes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_of: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JournalEntryInput {
    pub patient_id: String,
    pub recorded_at: DateTime<Utc>,
    pub author: AuthorInfo,
    pub body: String,
    pub event_id: Option<String>,
    pub context: Option<ContextInfo>,
    pub links: Option<JournalLinks>,
    pub codes: Option<JournalCodes>,
    pub correction_of: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LatestJournalEntry {
    pub filename: String,
    pub hash: String,
    pub path: PathBuf,
}

impl Default for JournalEntryInput {
    fn default() -> Self {
        Self {
            patient_id: String::new(),
            recorded_at: Utc::now(),
            author: AuthorInfo {
                id: String::new(),
                name: String::new(),
                role: None,
            },
            body: String::new(),
            event_id: None,
            context: None,
            links: None,
            codes: None,
            correction_of: None,
        }
    }
}

pub fn create_journal_entry(input: JournalEntryInput) -> Result<PathBuf> {
    create_journal_entry_with_layout(&FsLayout::new(), input)
}

pub fn create_journal_entry_with_layout(
    layout: &FsLayout,
    input: JournalEntryInput,
) -> Result<PathBuf> {
    if input.patient_id.is_empty() {
        return Err(JournalError::MissingPatientId.into());
    }
    if input.author.id.is_empty() || input.author.name.is_empty() {
        return Err(JournalError::MissingAuthor.into());
    }

    layout.ensure_structure()?;

    let patient_dir = layout.patient_journal_dir(&input.patient_id);
    if !patient_dir.exists() {
        fs::create_dir_all(&patient_dir).map_err(|source| JournalError::CreatePatientDir {
            path: patient_dir.clone(),
            source,
        })?;
    }

    let event_id = input
        .event_id
        .clone()
        .unwrap_or_else(|| format!("c{}", Uuid::new_v4().simple()));

    let filename = format!(
        "{}--{}.md",
        input.recorded_at.format("%Y-%m-%dT%H-%M-%SZ"),
        event_id
    );
    let path = patient_dir.join(&filename);

    let header = JournalEntryHeader {
        gitehr_event_id: event_id,
        gitehr_patient_id: input.patient_id,
        recorded_at: input.recorded_at,
        author: input.author,
        context: input.context,
        links: input.links,
        codes: input.codes,
        correction_of: input.correction_of,
    };

    let yaml = serde_yaml::to_string(&header).map_err(JournalError::InvalidHeader)?;
    let file_content = format!("---\n{}---\n\n{}", yaml, input.body);

    fs::write(&path, file_content).map_err(|source| JournalError::WriteEntry {
        path: path.clone(),
        source,
    })?;

    Ok(path)
}

pub fn get_latest_journal_entry(patient_id: &str) -> Result<Option<LatestJournalEntry>> {
    get_latest_journal_entry_for_patient(&FsLayout::new(), patient_id)
}

pub fn get_latest_journal_entry_for_patient(
    layout: &FsLayout,
    patient_id: &str,
) -> Result<Option<LatestJournalEntry>> {
    let patient_dir = layout.patient_journal_dir(patient_id);
    if !patient_dir.exists() {
        return Ok(None);
    }

    let mut entries = Vec::new();
    let read_dir = fs::read_dir(&patient_dir).map_err(|source| JournalError::ReadEntry {
        path: patient_dir.clone(),
        source,
    })?;
    for entry in read_dir {
        let entry = entry.map_err(|source| JournalError::ReadEntry {
            path: patient_dir.clone(),
            source,
        })?;
        entries.push(entry);
    }
    entries.sort_by_key(|e| e.file_name());

    if let Some(latest) = entries.last() {
        let entry_path = latest.path();
        let content =
            fs::read_to_string(&entry_path).map_err(|source| JournalError::ReadEntry {
                path: entry_path.clone(),
                source,
            })?;
        let hash = format!("{:x}", Sha256::digest(content.as_bytes()));
        Ok(Some(LatestJournalEntry {
            filename: latest.file_name().to_string_lossy().to_string(),
            hash,
            path: entry_path,
        }))
    } else {
        Ok(None)
    }
}
