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
    if !path.exists() {
        return Ok(T::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read state file {}", path.display()))?;
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

fn refuse_symlinked_path(path: &Path) -> Result<()> {
    for candidate in path
        .ancestors()
        .take_while(|candidate| *candidate != Path::new(""))
    {
        if candidate
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            anyhow::bail!("Refusing to write through symlink {}", candidate.display());
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
    if let Some(rest) = content.strip_prefix("---\n") {
        let end = rest.find("\n---")?;
        return Some((&rest[..end], &rest[end + 4..]));
    }
    let rest = content.strip_prefix("---\r\n")?;
    let end = rest.find("\r\n---")?;
    Some((&rest[..end], &rest[end + 5..]))
}
