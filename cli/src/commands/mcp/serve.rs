// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use std::path::PathBuf;

pub fn run(repo_path: Option<PathBuf>) -> Result<()> {
    // Initialize tracing
    gitehr_mcp::init_tracing();

    // Determine repository path
    let path = repo_path
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    // Validate it's a GitEHR repository
    if !path.join(".gitehr").exists() {
        anyhow::bail!(
            "Not a GitEHR repository: {}\nRun 'gitehr init' to create a new repository.",
            path.display()
        );
    }

    tracing::info!("Starting GitEHR MCP server");
    tracing::info!("Repository: {}", path.display());

    // Create server configuration
    let config = gitehr_mcp::ServerConfig {
        repo_path: path,
        server_name: "gitehr-mcp".to_string(),
        server_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    // Create and run server
    let mut server = gitehr_mcp::McpServer::new(config);

    // Run with tokio runtime
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        server
            .run_stdio()
            .await
            .map_err(|e| anyhow::anyhow!("MCP server error: {}", e))
    })
}
