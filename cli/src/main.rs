// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;

mod commands;
mod utils;

fn main() -> Result<()> {
    commands::run()
}
