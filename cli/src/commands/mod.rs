// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod completions;
pub mod contributor;
pub mod decrypt;
pub mod document;
pub mod encrypt;
mod git;
pub mod gui;
pub mod init;
pub mod journal;
pub mod mcp;
pub mod plugin;
pub mod remote;
#[cfg(feature = "server")]
pub mod server;
pub mod state;
pub mod status;
pub mod transport;
pub mod upgrade;
pub mod upgrade_binary;
pub mod version;
