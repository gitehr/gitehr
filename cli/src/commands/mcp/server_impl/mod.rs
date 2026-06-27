// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Internal MCP server implementation for `gitehr mcp serve`.

mod protocol;
mod resources;
mod server;
mod tools;

pub use server::{McpServer, ServerConfig};
