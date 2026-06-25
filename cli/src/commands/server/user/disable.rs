// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

pub fn run(id: &str) -> Result<()> {
    crate::commands::contributor::disable_contributor(id)
}
