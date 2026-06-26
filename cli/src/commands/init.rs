// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

use super::scaffold;

/// Initialise a standalone single-subject repository in the current directory.
///
/// Superseded by the Store-first model (see spec/adr/0005): the front door is
/// `gitehr store init`. This is retained only until context detection lands and
/// the test suite moves onto stores, then it is removed.
pub fn run() -> Result<()> {
    scaffold::scaffold_cwd(&scaffold::new_subject_id())?;
    println!("Initialized empty GitEHR repository");
    Ok(())
}
