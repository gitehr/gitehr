// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! GitEHR Model Context Protocol (MCP) Server
//!
//! This crate implements the Model Context Protocol specification,
//! exposing GitEHR repositories as MCP servers for LLM integration.

pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;

pub use protocol::{McpError, McpRequest, McpResponse, McpResult};
pub use server::{McpServer, ServerConfig};

/// MCP protocol version
pub const MCP_VERSION: &str = "2024-11-05";

/// Initialize tracing for MCP server
pub fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}
