// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::{git, journal};

pub fn ensure_gitehr_repository() -> Result<()> {
    if !Path::new(".gitehr").exists() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }
    Ok(())
}

pub fn state_path(filename: &str) -> PathBuf {
    PathBuf::from("state").join(filename)
}

pub fn read_front_matter<T>(filename: &str) -> Result<T>
where
    T: DeserializeOwned + Default,
{
    let path = state_path(filename);
    refuse_symlinked_path(&path)?;
    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(T::default()),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to read state file {}", path.display()));
        }
    };
    if content.trim().is_empty() {
        return Ok(T::default());
    }

    let yaml = extract_front_matter(&content).unwrap_or(content.as_str());
    if yaml.trim().is_empty() {
        return Ok(T::default());
    }

    serde_yaml_ng::from_str(yaml)
        .with_context(|| format!("Failed to parse YAML front matter in {}", path.display()))
}

pub fn write_front_matter<T>(filename: &str, value: &T) -> Result<PathBuf>
where
    T: Serialize,
{
    let path = state_path(filename);
    refuse_symlinked_path(&path)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml = serde_yaml_ng::to_string(value)?;
    let body = match fs::read_to_string(&path) {
        Ok(content) => markdown_body(&content)
            .map(str::to_owned)
            .unwrap_or_default(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to read state file {}", path.display()));
        }
    };
    let content = format!("---\n{yaml}---{body}");
    write_atomic(&path, content.as_bytes())?;
    Ok(path)
}

/// Persist typed state and its audit narrative as one isolated Git commit.
/// On failure, restore the prior state and remove the uncommitted journal file.
pub(crate) fn write_with_journal<T>(
    filename: &str,
    value: &T,
    journal_body: &str,
    pristine_untracked_contents: &[&[u8]],
) -> Result<()>
where
    T: Serialize,
{
    let state_path = state_path(filename);
    let state_name = state_path.to_string_lossy().into_owned();
    refuse_symlinked_path(&state_path)?;
    let original = match fs::read(&state_path) {
        Ok(content) => Some(content),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Failed to read state file {}", state_path.display()));
        }
    };
    let status = git::git_path_status(&state_name)?;
    let is_pristine_untracked = status.starts_with("?? ")
        && original
            .as_deref()
            .is_some_and(|content| pristine_untracked_contents.contains(&content));
    if !status.is_empty() && !is_pristine_untracked {
        anyhow::bail!(
            "Refusing to update {} while it has uncommitted changes",
            state_path.display()
        );
    }

    let mut journal_name = None;
    let mut state_written = false;
    let result = (|| {
        write_front_matter(filename, value)?;
        state_written = true;
        let name = journal::write_journal_entry(journal_body)?;
        journal_name = Some(name.clone());
        git::git_add(&state_name)?;
        git::git_add(&name)?;
        git::git_commit_paths(
            &format!("Journal entry: {name}"),
            &[state_name.as_str(), name.as_str()],
        )?;
        println!("Created journal entry: {name}");
        Ok(())
    })();

    if let Err(error) = result {
        let mut rollback_errors = Vec::new();
        let paths = journal_name
            .as_deref()
            .map(|name| vec![state_name.as_str(), name])
            .unwrap_or_else(|| vec![state_name.as_str()]);
        if let Err(rollback_error) = git::git_unstage_paths(&paths) {
            rollback_errors.push(format!("unstage failed: {rollback_error}"));
        }
        if state_written {
            let restore_result = match original.as_deref() {
                Some(content) => write_atomic(&state_path, content),
                None => remove_if_present(&state_path),
            };
            if let Err(rollback_error) = restore_result {
                rollback_errors.push(format!("state restore failed: {rollback_error}"));
            }
        }
        if let Some(name) = journal_name.as_deref()
            && let Err(rollback_error) = remove_if_present(Path::new(name))
        {
            rollback_errors.push(format!("journal cleanup failed: {rollback_error}"));
        }
        if rollback_errors.is_empty() {
            return Err(error);
        }
        anyhow::bail!(
            "{error}\nRollback incomplete: {}",
            rollback_errors.join("; ")
        );
    }

    Ok(())
}

fn write_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("Failed to create temporary file for {}", path.display()))?;
    // Preserve Unix mode bits; replacement files inherit access controls from the directory.
    if let Ok(metadata) = fs::metadata(path) {
        temporary
            .as_file()
            .set_permissions(metadata.permissions())?;
    }
    temporary.write_all(content)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("Failed to write state file {}", path.display()))?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("Failed to remove {}", path.display())),
    }
}

pub(crate) fn refuse_symlinked_path(path: &Path) -> Result<()> {
    for candidate in path
        .ancestors()
        .take_while(|candidate| *candidate != Path::new(""))
    {
        match candidate.symlink_metadata() {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                anyhow::bail!("Refusing to access through symlink {}", candidate.display());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("Failed to inspect path {}", candidate.display()));
            }
        }
    }
    Ok(())
}

fn extract_front_matter(content: &str) -> Option<&str> {
    split_front_matter(content).map(|(yaml, _)| yaml)
}

fn markdown_body(content: &str) -> Option<&str> {
    split_front_matter(content).map(|(_, body)| body)
}

fn split_front_matter(content: &str) -> Option<(&str, &str)> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n', ' ', '\t']) == "---" {
            // Keep the closing delimiter's whitespace and newline with the body.
            return Some((&rest[..offset], &rest[offset + 3..]));
        }
        offset += line.len();
    }
    None
}

/// Where an asserted fact came from (R60 Part 1, `spec/record-provenance-and-acquisition.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[value(rename_all = "kebab-case")]
pub enum SourceType {
    SelfReported,
    ClinicianAsserted,
    PortalExtracted,
    Sar,
    PaperTranscribed,
    Device,
    Inferred,
}

/// How strongly a fact is evidenced. An inference must never masquerade as a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum EvidenceLevel {
    Documented,
    Inferred,
    Assumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// The artifact that substantiates an assertion: a repository-relative path
/// under `documents/` or `imaging/`, pinned by the SHA-256 of its content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct DocumentRef {
    pub path: String,
    pub sha256: String,
}

impl DocumentRef {
    /// Resolves `path` against the current repository root and hashes the file.
    /// Only a regular file directly addressable under `documents/` or
    /// `imaging/` is accepted; traversal, symlinks and directories are refused.
    fn resolve(path: &str) -> Result<Self> {
        use sha2::{Digest, Sha256};
        use std::path::Component;

        let candidate = Path::new(path.trim());
        let mut components = candidate.components();
        let root_ok = matches!(
            components.next(),
            Some(Component::Normal(root)) if root == "documents" || root == "imaging"
        );
        if !root_ok
            || components.clone().next().is_none()
            || !components.all(|c| matches!(c, Component::Normal(_)))
        {
            anyhow::bail!(
                "--document-ref must be a relative path to a file under documents/ or imaging/"
            );
        }
        refuse_symlinked_path(candidate)?;
        let metadata = fs::metadata(candidate)
            .with_context(|| format!("Document {} not found", candidate.display()))?;
        if !metadata.is_file() {
            anyhow::bail!(
                "--document-ref {} is not a regular file (directory Documents are not supported)",
                candidate.display()
            );
        }
        let bytes = fs::read(candidate)
            .with_context(|| format!("Failed to read {}", candidate.display()))?;
        Ok(DocumentRef {
            path: candidate
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
            sha256: Sha256::digest(&bytes)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        })
    }
}

/// Optional, reusable metadata about an assertion.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, serde::Deserialize)]
pub struct Provenance {
    pub source_type: Option<SourceType>,
    pub source_detail: Option<String>,
    pub acquired_via: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_ref: Option<DocumentRef>,
    pub evidence_level: Option<EvidenceLevel>,
    pub confidence: Option<Confidence>,
}

impl Provenance {
    /// Builds a block from CLI inputs, or `None` when nothing was supplied.
    pub fn from_parts(
        source_type: Option<SourceType>,
        source_detail: Option<&str>,
        acquired_via: Option<&str>,
        document_ref: Option<&str>,
        evidence_level: Option<EvidenceLevel>,
        confidence: Option<Confidence>,
    ) -> Result<Option<Self>> {
        let clean = |v: Option<&str>| {
            v.map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
        };
        let source_detail = clean(source_detail);
        let acquired_via = clean(acquired_via);
        let document_ref = clean(document_ref);
        if source_type.is_none()
            && (source_detail.is_some()
                || acquired_via.is_some()
                || document_ref.is_some()
                || evidence_level.is_some()
                || confidence.is_some())
        {
            anyhow::bail!("--source-type is required with other provenance metadata");
        }
        let document_ref = document_ref
            .as_deref()
            .map(DocumentRef::resolve)
            .transpose()?;
        let provenance = Provenance {
            source_type,
            source_detail,
            acquired_via,
            document_ref,
            evidence_level,
            confidence,
        };
        Ok((provenance != Provenance::default()).then_some(provenance))
    }
}

#[cfg(test)]
mod tests {
    use super::{EvidenceLevel, Provenance, SourceType, split_front_matter};

    #[test]
    fn front_matter_requires_a_whole_delimiter_line() {
        for newline in ["\n", "\r\n"] {
            let yaml = format!("conditions: []{newline}---source: imported{newline}");
            let body = format!("{newline}{newline}Clinical notes{newline}");
            let content = format!("---{newline}{yaml}---{body}");
            assert_eq!(
                split_front_matter(&content),
                Some((yaml.as_str(), body.as_str()))
            );
            assert_eq!(split_front_matter(&format!("---{newline}{yaml}")), None);
            assert_eq!(
                split_front_matter(&format!("---{newline}---{body}")),
                Some(("", body.as_str()))
            );
            assert_eq!(
                split_front_matter(&format!("---{newline}{yaml}---")),
                Some((yaml.as_str(), ""))
            );
        }
        assert_eq!(split_front_matter("conditions: []\n"), None);
        assert_eq!(
            split_front_matter("---\nconditions: []\n---invalid\n"),
            None
        );
    }

    #[test]
    fn provenance_requires_a_source_type() {
        assert!(
            Provenance::from_parts(
                None,
                Some("Example GP Practice"),
                None,
                None,
                Some(EvidenceLevel::Documented),
                None,
            )
            .is_err()
        );
        assert_eq!(
            Provenance::from_parts(Some(SourceType::Sar), None, None, None, None, None).unwrap(),
            Some(Provenance {
                source_type: Some(SourceType::Sar),
                ..Default::default()
            })
        );
    }
}
