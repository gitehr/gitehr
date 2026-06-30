// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod allergies;
pub mod config;
pub mod contributor;
pub mod demographics;
pub mod document;
pub mod encrypt;
pub mod journal;
pub mod mcp;
#[cfg(unix)]
pub mod plugin;
pub mod remote;
pub mod state;
pub mod status;
pub mod store;
pub mod transport;
pub mod upgrade;
pub mod vaccinations;
