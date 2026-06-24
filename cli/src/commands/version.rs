// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::git;

pub fn run() {
    let gitehr_version = env!("CARGO_PKG_VERSION");
    println!("GitEHR: {}", gitehr_version);
    if let Some(git_version) = git::get_git_version() {
        let git_version_str = git_version.replace("git version ", "");
        println!("Git {}", git_version_str);
    } else {
        println!("Git (not found or not installed)");
    }
}
