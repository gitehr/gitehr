// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result, bail};
use std::io::{IsTerminal, Read};
use uuid::Uuid;

/// Add a journal entry. The body comes from, in order of precedence:
/// `--file <path>` (or `--file -` for stdin), the inline `text` argument, piped
/// stdin, or - on a terminal with none of those - your `$EDITOR`. The entry is
/// written to `journal/` and git-committed immediately; entries are append-only.
pub fn run(text: Option<String>, file: Option<String>) -> Result<()> {
    let body = match (file, text) {
        (Some(f), _) => read_source(&f)?,
        (None, Some(t)) => t,
        (None, None) => {
            if std::io::stdin().is_terminal() {
                from_editor()?
            } else {
                read_source("-")?
            }
        }
    };

    let body = body.trim();
    if body.is_empty() {
        bail!("Aborting: the journal entry is empty.");
    }
    super::create_journal_entry(body)
}

/// Read from a file path, or from stdin when the path is `-`.
fn read_source(path: &str) -> Result<String> {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read the journal entry from stdin")?;
        Ok(buf)
    } else {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read the journal entry from {path}"))
    }
}

/// Open the user's editor on an ephemeral temp file and return what they wrote.
fn from_editor() -> Result<String> {
    let path = std::env::temp_dir().join(format!("gitehr-journal-{}.md", Uuid::new_v4()));
    std::fs::write(&path, "")?;

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());
    let status = std::process::Command::new(&editor).arg(&path).status()?;

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    let _ = std::fs::remove_file(&path);

    if !status.success() {
        bail!("Editor exited with a non-zero status; the entry was not recorded.");
    }
    Ok(content)
}
