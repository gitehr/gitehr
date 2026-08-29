// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::{Result, bail};

use super::find_subject;

/// Print the repository path for a subject, by canonical id or friendly name.
pub fn run(subject: &str) -> Result<()> {
    let mpi = super::load_mpi()?;
    match find_subject(&mpi, subject) {
        Some(p) => println!("{}", p.repo_path),
        None => bail!("Subject '{subject}' not found in the MPI (tried both id and name)"),
    }
    Ok(())
}
