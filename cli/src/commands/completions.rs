// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::Command;
use clap_complete::{Shell, generate};
use std::io;

pub fn run(shell: Shell, cmd: &mut Command) {
    generate(shell, cmd, "gitehr", &mut io::stdout());
}
