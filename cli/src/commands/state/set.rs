// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Context, Result};
use std::io::Read;

use super::{is_gitehr_repo, state_file_path, update_state_file};
use crate::commands::git;

pub fn run(filename: &str, content: Option<&str>, file: Option<&str>, commit: bool) -> Result<()> {
    if !is_gitehr_repo() {
        anyhow::bail!("Not a GitEHR repository (or not in the repository root).");
    }

    let content = match (content, file) {
        (Some(content), None) => content.to_owned(),
        (None, Some("-")) => {
            let mut content = String::new();
            std::io::stdin()
                .read_to_string(&mut content)
                .context("reading state content from stdin")?;
            content
        }
        (None, Some(path)) => std::fs::read_to_string(path)
            .with_context(|| format!("reading state content from {path}"))?,
        _ => anyhow::bail!("provide state content or --file <PATH>"),
    };

    let state_path = state_file_path(filename)?;
    let state_name = state_path.to_string_lossy().into_owned();
    if commit && !git::git_path_status(&state_name)?.is_empty() {
        anyhow::bail!("Refusing to update {state_name} while it has uncommitted changes");
    }

    update_state_file(filename, &content)?;
    if commit {
        git::git_add(&state_name)?;
        if let Err(error) =
            git::git_commit_paths(&format!("Update state file: {filename}"), &[&state_name])
        {
            let _ = git::git_unstage_paths(&[&state_name]);
            return Err(error);
        }
    }
    Ok(())
}
