// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Internal MCP server implementation for `gitehr mcp serve`.

mod audit;
mod prompts;
mod protocol;
mod resources;
mod security;
mod server;
mod tools;

pub use security::ensure_not_encrypted;
pub use server::{McpServer, ServerConfig};
