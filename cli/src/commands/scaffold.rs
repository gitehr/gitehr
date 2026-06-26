// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shared repo-scaffolding for the Store-first model (see spec/adr/0005).
//!
//! Every subject (a patient, a family member, a pet) is a GitEHR repo. Each gets
//! a stable canonical id - a UUIDv7 (time-ordered) encoded in Crockford base32 -
//! that lives in the MPI and inside the repo (`.gitehr/ID`). The on-disk
//! directory is a friendly slug when a name is given, else the canonical id, so a
//! lone self-hoster gets `rex/` while a 400k-patient store needs no human naming.

use anyhow::{Context, Result};
use fs_extra::dir::{self, CopyOptions};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::git;

/// Mint a new canonical subject id: a UUIDv7 in Crockford base32 (26 chars,
/// sortable, no ambiguous characters).
pub fn new_subject_id() -> String {
    crockford_encode(Uuid::now_v7().as_bytes())
}

/// Create a new subject repo under `parent`. With a `name`, the directory is a
/// de-duplicated slug of it; without one, it is the canonical id. Returns the
/// chosen directory name and the canonical id.
pub fn create_subject_repo(parent: &Path, name: Option<&str>) -> Result<(String, String)> {
    let canonical_id = new_subject_id();
    let dir_name = match name {
        Some(n) => unique_dir(parent, &slugify(n)),
        None => canonical_id.clone(),
    };
    let repo_dir = parent.join(&dir_name);
    std::fs::create_dir(&repo_dir)
        .with_context(|| format!("Failed to create subject directory {}", repo_dir.display()))?;

    // Scaffold reuses the cwd-relative logic; cd in, build, then restore cwd so
    // the caller (e.g. `store init`) keeps writing the MPI at the Store root.
    let prev = std::env::current_dir()?;
    std::env::set_current_dir(&repo_dir)?;
    let result = scaffold_cwd(&canonical_id);
    std::env::set_current_dir(&prev)?;
    result?;

    Ok((dir_name, canonical_id))
}

/// Scaffold a GitEHR repo in the current directory: `.gitehr/` (with the bundled
/// binary, version, and canonical id), a git repo, and the template folders.
pub fn scaffold_cwd(canonical_id: &str) -> Result<()> {
    let gitehr_dir = PathBuf::from(".gitehr");
    if gitehr_dir.exists() {
        anyhow::bail!("GitEHR repository already exists in this directory");
    }

    let template_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("folder-structure");
    if !template_path.exists() {
        anyhow::bail!("Could not find template directory");
    }

    std::fs::create_dir(".gitehr")?;
    git::git_init()?;
    std::fs::write(".gitehr/GITEHR_VERSION", env!("CARGO_PKG_VERSION"))?;
    std::fs::write(".gitehr/ID", canonical_id)?;
    copy_binary_to_repo()?;

    for entry in std::fs::read_dir(&template_path)? {
        let entry = entry?;
        let target_name = entry.file_name();
        if entry.file_type()?.is_dir() {
            dir::copy(entry.path(), ".", &CopyOptions::new())?;
        } else {
            fs_extra::file::copy(
                entry.path(),
                target_name,
                &fs_extra::file::CopyOptions::new(),
            )?;
        }
    }

    Ok(())
}

fn copy_binary_to_repo() -> Result<()> {
    let source = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;
    let dest = PathBuf::from(".gitehr/gitehr");
    std::fs::copy(&source, &dest)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }
    Ok(())
}

/// Turn a user-supplied name into a filesystem-safe slug (lowercase, ASCII
/// alphanumerics, single hyphens between runs).
fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut pending_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(c.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        slug.push_str("subject");
    }
    slug
}

/// First free `base`, `base-2`, `base-3`, ... under `parent`.
fn unique_dir(parent: &Path, base: &str) -> String {
    if !parent.join(base).exists() {
        return base.to_string();
    }
    let mut n = 2u32;
    loop {
        let candidate = format!("{base}-{n}");
        if !parent.join(&candidate).exists() {
            return candidate;
        }
        n += 1;
    }
}

/// Crockford base32 (no I/L/O/U) of a byte slice; 16 bytes -> 26 chars.
fn crockford_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let mut out = String::with_capacity(bytes.len() * 8 / 5 + 1);
    let mut buffer: u64 = 0;
    let mut bits: u8 = 0;
    for &byte in bytes {
        buffer = (buffer << 8) | byte as u64;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(ALPHABET[((buffer >> bits) & 0x1f) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(ALPHABET[((buffer << (5 - bits)) & 0x1f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_handles_names_and_pets() {
        assert_eq!(slugify("Rex"), "rex");
        assert_eq!(slugify("Mrs Smith"), "mrs-smith");
        assert_eq!(slugify("Fluffy the Cat!"), "fluffy-the-cat");
        assert_eq!(slugify("  --  "), "subject");
    }

    #[test]
    fn subject_ids_are_unique_26_char_crockford() {
        let a = new_subject_id();
        let b = new_subject_id();
        assert_ne!(a, b);
        assert_eq!(a.len(), 26);
        assert!(
            a.bytes()
                .all(|c| b"0123456789ABCDEFGHJKMNPQRSTVWXYZ".contains(&c))
        );
    }
}
