// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;

use super::server_impl::{McpServer, ServerConfig};

pub fn run(repo_path: Option<PathBuf>) -> Result<()> {
    super::init_tracing();

    let config = ServerConfig {
        repo_path: repo_path.unwrap_or_else(|| PathBuf::from(".")),
        server_name: "gitehr".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let mut server = McpServer::new(config);
        server.run_stdio().await
    })
}
