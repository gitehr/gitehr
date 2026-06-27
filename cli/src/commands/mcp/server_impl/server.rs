// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Server Implementation

use super::protocol::{McpError, McpMethod, McpRequest, McpResponse};
use super::resources::ResourceHandler;
use super::tools::ToolHandler;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub repo_path: PathBuf,
    pub server_name: String,
    pub server_version: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from("."),
            server_name: "gitehr".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// MCP Server
pub struct McpServer {
    config: ServerConfig,
    resource_handler: ResourceHandler,
    tool_handler: ToolHandler,
    initialized: bool,
}

impl McpServer {
    pub fn new(config: ServerConfig) -> Self {
        let resource_handler = ResourceHandler::new(config.repo_path.clone());
        let tool_handler = ToolHandler::new(config.repo_path.clone());

        Self {
            config,
            resource_handler,
            tool_handler,
            initialized: false,
        }
    }

    /// Handle an MCP request
    pub async fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        debug!("Handling request: {:?}", request.method);

        let method = McpMethod::from_str(&request.method);

        let result = match method {
            McpMethod::Initialize => self.handle_initialize(&request).await,
            McpMethod::ResourcesList => self.handle_resources_list(&request).await,
            McpMethod::ResourcesRead => self.handle_resources_read(&request).await,
            McpMethod::ToolsList => self.handle_tools_list(&request).await,
            McpMethod::ToolsCall => self.handle_tools_call(&request).await,
            McpMethod::PromptsList => self.handle_prompts_list(&request).await,
            McpMethod::PromptsGet => self.handle_prompts_get(&request).await,
            McpMethod::Unknown(method_name) => {
                error!("Unknown method: {}", method_name);
                Err(McpError::method_not_found(method_name))
            }
        };

        match result {
            Ok(result) => McpResponse::success(request.id, result),
            Err(error) => {
                error!("Request error: {}", error);
                McpResponse::error(request.id, error)
            }
        }
    }

    async fn handle_initialize(
        &mut self,
        _request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        info!("Initializing MCP server");

        self.initialized = true;

        Ok(serde_json::json!({
            "protocolVersion": super::super::MCP_VERSION,
            "capabilities": {
                "resources": {},
                "tools": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": self.config.server_name,
                "version": self.config.server_version
            }
        }))
    }

    async fn handle_resources_list(
        &self,
        _request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }

        let resources = self
            .resource_handler
            .list_resources()
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        serde_json::to_value(resources).map_err(|e| McpError::internal_error(e.to_string()))
    }

    async fn handle_resources_read(
        &self,
        request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }

        let params = request
            .params
            .as_ref()
            .ok_or_else(|| McpError::invalid_params("Missing params"))?;

        let uri = params
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing 'uri' parameter"))?;

        let content = self
            .resource_handler
            .read_resource(uri)
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        serde_json::to_value(content).map_err(|e| McpError::internal_error(e.to_string()))
    }

    async fn handle_tools_list(
        &self,
        _request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }

        let tools = self
            .tool_handler
            .list_tools()
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        serde_json::to_value(tools).map_err(|e| McpError::internal_error(e.to_string()))
    }

    async fn handle_tools_call(&self, request: &McpRequest) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }

        let params = request
            .params
            .as_ref()
            .ok_or_else(|| McpError::invalid_params("Missing params"))?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::invalid_params("Missing 'name' parameter"))?;

        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let result = self
            .tool_handler
            .call_tool(name, arguments)
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        serde_json::to_value(result).map_err(|e| McpError::internal_error(e.to_string()))
    }

    async fn handle_prompts_list(
        &self,
        _request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }

        // Placeholder for prompts - will implement later
        Ok(serde_json::json!({
            "prompts": []
        }))
    }

    async fn handle_prompts_get(
        &self,
        _request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        if !self.initialized {
            return Err(McpError::invalid_request("Server not initialized"));
        }

        // Placeholder for prompts - will implement later
        Err(McpError::method_not_found(
            "prompts/get not yet implemented",
        ))
    }

    /// Run the server on stdio (for local MCP clients)
    pub async fn run_stdio(&mut self) -> anyhow::Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        info!("Starting MCP server on stdio");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                debug!("EOF on stdin, shutting down");
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            debug!("Received: {}", trimmed);

            let request: McpRequest = match serde_json::from_str(trimmed) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    let error_response = McpResponse::error(
                        None,
                        McpError::parse_error(format!("Invalid JSON: {}", e)),
                    );
                    let response_json = serde_json::to_string(&error_response)?;
                    stdout.write_all(response_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            let response_json = serde_json::to_string(&response)?;

            debug!("Sending: {}", response_json);

            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        info!("MCP server shutdown complete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initialize() {
        let config = ServerConfig::default();
        let mut server = McpServer::new(config);

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.result.is_some());
        assert!(server.initialized);
    }

    #[tokio::test]
    async fn test_resources_list_before_init() {
        let config = ServerConfig::default();
        let mut server = McpServer::new(config);

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "resources/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await;
        assert!(response.error.is_some());
    }
}
