// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Store/repo working-context resolution (see spec/adr/0005).
//!
//! GitEHR finds its working context the way git finds `.git/`: by walking up
//! from the current directory. Repo-level commands resolve a subject repo
//! (`.gitehr/`); store-level commands resolve the Store root (`gitehr-mpi.json`).
//! When run at a Store root with exactly one subject, repo-level commands
//! auto-target it, so a lone self-hoster never has to `cd` into the subject.

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

const REPO_MARKER: &str = ".gitehr";
const STORE_MARKER: &str = "gitehr-mpi.json";

/// Nearest ancestor of the cwd (inclusive) that contains `marker`.
fn find_up(marker: &str) -> Result<Option<PathBuf>> {
    let mut dir = std::env::current_dir()?;
    loop {
        if dir.join(marker).exists() {
            return Ok(Some(dir));
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

/// Resolve the Store root for a store-level command.
pub fn resolve_store_root() -> Result<PathBuf> {
    match find_up(STORE_MARKER)? {
        Some(root) => Ok(root),
        None => configured_store_root()?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not inside a GitEHR Store (no {STORE_MARKER} found). Run `gitehr store init` to create one, or set one with `gitehr config set-store <path>`."
            )
        }),
    }
}

/// Resolve the subject repo for a repo-level command: the nearest `.gitehr/`
/// ancestor, or - at a Store root with exactly one subject - that subject.
pub fn resolve_repo_root() -> Result<PathBuf> {
    if let Some(repo) = find_up(REPO_MARKER)? {
        return Ok(repo);
    }
    let store = match find_up(STORE_MARKER)? {
        Some(store) => Some(store),
        None => configured_store_root()?,
    };

    if let Some(store) = store {
        let subjects = subjects(&store)?;
        return match subjects.as_slice() {
            [(_, path)] => Ok(path.clone()),
            [] => bail!("This Store has no subjects yet. Add one with `gitehr store add [name]`."),
            many => bail!(
                "You are at a GitEHR Store with {} subjects. cd into one (e.g. `cd {}`), or add a new one with `gitehr store add`.",
                many.len(),
                many[0].0
            ),
        };
    }
    bail!(
        "Not a GitEHR repository or Store. Run `gitehr store init` to create one, or set one with `gitehr config set-store <path>`."
    )
}

fn configured_store_root() -> Result<Option<PathBuf>> {
    let Some(store) = crate::config::configured_store_path()? else {
        return Ok(None);
    };

    if store.join(STORE_MARKER).exists() {
        Ok(Some(store))
    } else {
        bail!(
            "Configured GitEHR Store path {} does not contain {STORE_MARKER}. Update it with `gitehr config set-store <path>` or override it with {}.",
            store.display(),
            crate::config::STORE_PATH_ENV
        )
    }
}

/// `(name, absolute path)` for each subject in the Store's MPI (parsed loosely so
/// this module does not depend on the store command's structs).
fn subjects(store: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mpi: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(store.join(STORE_MARKER))?)?;
    let mut out = Vec::new();
    if let Some(arr) = mpi.get("patients").and_then(|v| v.as_array()) {
        for p in arr {
            if let Some(rp) = p.get("repo_path").and_then(|v| v.as_str()) {
                out.push((rp.to_string(), store.join(rp)));
            }
        }
    }
    Ok(out)
}
