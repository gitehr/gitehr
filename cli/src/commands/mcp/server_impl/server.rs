// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Server Implementation

use super::prompts::PromptHandler;
use super::protocol::{McpError, McpMethod, McpRequest, McpRequestId, McpResponse};
use super::resources::ResourceHandler;
use super::tools::ToolHandler;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, error, info};

const MAX_REQUEST_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InitializationState {
    NotStarted,
    AwaitingInitialized,
    Ready,
}

enum ReadLine {
    Eof,
    Line,
    TooLarge,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    protocol_version: String,
    capabilities: serde_json::Map<String, serde_json::Value>,
    client_info: ClientInfo,
}

#[derive(Deserialize)]
struct ClientInfo {
    name: String,
    version: String,
}

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
    prompt_handler: PromptHandler,
    initialization_state: InitializationState,
}

impl McpServer {
    pub fn new(config: ServerConfig) -> Self {
        let resource_handler = ResourceHandler::new(config.repo_path.clone());
        let tool_handler = ToolHandler::new(config.repo_path.clone());
        let prompt_handler = PromptHandler::new();

        Self {
            config,
            resource_handler,
            tool_handler,
            prompt_handler,
            initialization_state: InitializationState::NotStarted,
        }
    }

    /// Handle an MCP request, returning no response for notifications.
    pub async fn handle_request(&mut self, request: McpRequest) -> Option<McpResponse> {
        if request.jsonrpc != "2.0" {
            return Some(McpResponse::error(
                None,
                McpError::invalid_request("'jsonrpc' must be '2.0'"),
            ));
        }

        let method = McpMethod::from_str(&request.method);
        let id = match &request.id {
            McpRequestId::Missing => {
                match method {
                    McpMethod::InitializedNotification
                        if request
                            .params
                            .as_ref()
                            .is_none_or(serde_json::Value::is_object) =>
                    {
                        self.handle_initialized()
                    }
                    McpMethod::InitializedNotification => {
                        error!("Ignoring malformed initialized notification")
                    }
                    McpMethod::Unknown(_) => debug!("Ignoring unknown notification"),
                    _ => error!("Ignoring request method sent as a notification"),
                }
                return None;
            }
            McpRequestId::Value(id)
                if matches!(id, serde_json::Value::String(_))
                    || id.as_i64().is_some()
                    || id.as_u64().is_some() =>
            {
                id.clone()
            }
            McpRequestId::Value(_) => {
                return Some(McpResponse::error(
                    None,
                    McpError::invalid_request("Request id must be a string or integer"),
                ));
            }
        };
        if request
            .params
            .as_ref()
            .is_some_and(|params| !params.is_object())
        {
            return Some(McpResponse::error(
                Some(id),
                McpError::invalid_request("Request params must be an object"),
            ));
        }

        let result = match method {
            McpMethod::Initialize => self.handle_initialize(&request).await,
            McpMethod::InitializedNotification => Err(McpError::invalid_request(
                "notifications/initialized must not include an id",
            )),
            McpMethod::ResourcesList => self.handle_resources_list(&request).await,
            McpMethod::ResourcesRead => self.handle_resources_read(&request).await,
            McpMethod::ToolsList => self.handle_tools_list(&request).await,
            McpMethod::ToolsCall => self.handle_tools_call(&request).await,
            McpMethod::PromptsList => self.handle_prompts_list(&request).await,
            McpMethod::PromptsGet => self.handle_prompts_get(&request).await,
            McpMethod::Unknown(method_name) => Err(McpError::method_not_found(method_name)),
        };

        Some(match result {
            Ok(result) => McpResponse::success(id, result),
            Err(error) => {
                error!(code = error.code, "MCP request failed");
                McpResponse::error(Some(id), error)
            }
        })
    }

    async fn handle_initialize(
        &mut self,
        request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        if self.initialization_state != InitializationState::NotStarted {
            return Err(McpError::invalid_request("Server already initialized"));
        }

        let params: InitializeParams = serde_json::from_value(
            request
                .params
                .clone()
                .ok_or_else(|| McpError::invalid_params("Missing initialize params"))?,
        )
        .map_err(|error| McpError::invalid_params(format!("Invalid initialize params: {error}")))?;
        if params.protocol_version.trim().is_empty()
            || params.client_info.name.trim().is_empty()
            || params.client_info.version.trim().is_empty()
        {
            return Err(McpError::invalid_params(
                "Initialize protocolVersion and clientInfo fields must not be empty",
            ));
        }
        let _client_capabilities = params.capabilities;

        info!("Initializing MCP server");

        self.initialization_state = InitializationState::AwaitingInitialized;

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

    fn handle_initialized(&mut self) {
        match self.initialization_state {
            InitializationState::AwaitingInitialized => {
                self.initialization_state = InitializationState::Ready;
                info!("MCP client initialization complete");
            }
            InitializationState::Ready => debug!("Ignoring duplicate initialized notification"),
            InitializationState::NotStarted => {
                error!("Ignoring initialized notification before initialize request")
            }
        }
    }

    fn ensure_initialized(&self) -> Result<(), McpError> {
        if self.initialization_state != InitializationState::Ready {
            return Err(McpError::invalid_request("Server not initialized"));
        }
        Ok(())
    }

    async fn handle_resources_list(
        &self,
        _request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        self.ensure_initialized()?;

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
        self.ensure_initialized()?;

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
        self.ensure_initialized()?;

        let tools = self
            .tool_handler
            .list_tools()
            .map_err(|e| McpError::internal_error(e.to_string()))?;

        serde_json::to_value(tools).map_err(|e| McpError::internal_error(e.to_string()))
    }

    async fn handle_tools_call(&self, request: &McpRequest) -> Result<serde_json::Value, McpError> {
        self.ensure_initialized()?;

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
        self.ensure_initialized()?;

        let prompts = self.prompt_handler.list_prompts();

        serde_json::to_value(prompts).map_err(|e| McpError::internal_error(e.to_string()))
    }

    async fn handle_prompts_get(
        &self,
        request: &McpRequest,
    ) -> Result<serde_json::Value, McpError> {
        self.ensure_initialized()?;

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
            .prompt_handler
            .get_prompt(name, &arguments)
            .map_err(|e| McpError::invalid_params(e.to_string()))?;

        serde_json::to_value(result).map_err(|e| McpError::internal_error(e.to_string()))
    }

    /// Run the server on stdio (for local MCP clients)
    pub async fn run_stdio(&mut self) -> anyhow::Result<()> {
        info!("Starting MCP server on stdio");

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = Vec::new();

        loop {
            match read_bounded_line(&mut reader, &mut line).await? {
                ReadLine::Eof => {
                    debug!("EOF on stdin, shutting down");
                    break;
                }
                ReadLine::TooLarge => {
                    error!("Rejected MCP request larger than {MAX_REQUEST_BYTES} bytes");
                    write_response(
                        &mut stdout,
                        &McpResponse::error(
                            None,
                            McpError::parse_error(format!(
                                "Request exceeds {MAX_REQUEST_BYTES} byte limit"
                            )),
                        ),
                    )
                    .await?;
                    continue;
                }
                ReadLine::Line => {}
            }

            let text = match std::str::from_utf8(&line) {
                Ok(text) => text,
                Err(error) => {
                    write_response(
                        &mut stdout,
                        &McpResponse::error(
                            None,
                            McpError::parse_error(format!("Request is not valid UTF-8: {error}")),
                        ),
                    )
                    .await?;
                    continue;
                }
            };
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let value: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(value) => value,
                Err(_) => {
                    error!("Failed to parse MCP JSON");
                    write_response(
                        &mut stdout,
                        &McpResponse::error(None, McpError::parse_error("Invalid JSON")),
                    )
                    .await?;
                    continue;
                }
            };
            let request: McpRequest = match serde_json::from_value(value) {
                Ok(request) => request,
                Err(_) => {
                    error!("Rejected invalid JSON-RPC request");
                    write_response(
                        &mut stdout,
                        &McpResponse::error(
                            None,
                            McpError::invalid_request("Invalid JSON-RPC request"),
                        ),
                    )
                    .await?;
                    continue;
                }
            };

            debug!(
                method = McpMethod::from_str(&request.method).log_name(),
                has_id = !request.id.is_missing(),
                request_bytes = line.len(),
                "Received MCP message"
            );
            if let Some(response) = self.handle_request(request).await {
                write_response(&mut stdout, &response).await?;
            }
        }

        info!("MCP server shutdown complete");
        Ok(())
    }
}

async fn read_bounded_line<R>(reader: &mut R, line: &mut Vec<u8>) -> std::io::Result<ReadLine>
where
    R: AsyncBufRead + Unpin,
{
    line.clear();
    let mut oversized = false;

    loop {
        let available = reader.fill_buf().await?;
        if available.is_empty() {
            return Ok(if oversized {
                ReadLine::TooLarge
            } else if line.is_empty() {
                ReadLine::Eof
            } else {
                ReadLine::Line
            });
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |position| position + 1);
        if !oversized {
            if line.len().saturating_add(consumed) > MAX_REQUEST_BYTES {
                oversized = true;
                line.clear();
            } else {
                line.extend_from_slice(&available[..consumed]);
            }
        }
        reader.consume(consumed);

        if newline.is_some() {
            return Ok(if oversized {
                ReadLine::TooLarge
            } else {
                ReadLine::Line
            });
        }
    }
}

async fn write_response<W>(writer: &mut W, response: &McpResponse) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let response_json = serde_json::to_string(response)?;
    debug!(response_bytes = response_json.len(), "Sending MCP response");
    writer.write_all(response_json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::mcp::MCP_VERSION;

    #[tokio::test]
    async fn test_initialize() {
        let config = ServerConfig::default();
        let mut server = McpServer::new(config);

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Value(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": MCP_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            })),
        };

        let response = server.handle_request(request.clone()).await.unwrap();
        assert!(response.result.is_some());
        assert_eq!(
            server.initialization_state,
            InitializationState::AwaitingInitialized
        );

        let mut notification = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Missing,
            method: "notifications/initialized".to_string(),
            params: Some(serde_json::json!([])),
        };
        assert!(server.handle_request(notification.clone()).await.is_none());
        assert_eq!(
            server.initialization_state,
            InitializationState::AwaitingInitialized
        );

        notification.params = Some(serde_json::json!({}));
        assert!(server.handle_request(notification).await.is_none());
        assert_eq!(server.initialization_state, InitializationState::Ready);

        let duplicate = server.handle_request(request).await.unwrap();
        assert_eq!(
            duplicate.error.unwrap().code,
            super::super::protocol::error_codes::INVALID_REQUEST
        );
        assert_eq!(server.initialization_state, InitializationState::Ready);
    }

    #[tokio::test]
    async fn test_resources_list_before_init() {
        let config = ServerConfig::default();
        let mut server = McpServer::new(config);

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Value(serde_json::json!(1)),
            method: "resources/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await.unwrap();
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn explicit_null_id_receives_invalid_request_response() {
        let mut server = McpServer::new(ServerConfig::default());
        let request: McpRequest = serde_json::from_value(serde_json::json!({
            "jsonrpc": "2.0",
            "id": null,
            "method": "initialize"
        }))
        .unwrap();

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response.id, serde_json::Value::Null);
        assert_eq!(
            response.error.unwrap().code,
            super::super::protocol::error_codes::INVALID_REQUEST
        );
        assert_eq!(server.initialization_state, InitializationState::NotStarted);
    }

    #[tokio::test]
    async fn fractional_request_id_is_rejected() {
        let mut server = McpServer::new(ServerConfig::default());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Value(serde_json::json!(1.5)),
            method: "resources/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request).await.unwrap();
        assert!(response.id.is_null());
        assert_eq!(
            response.error.unwrap().code,
            super::super::protocol::error_codes::INVALID_REQUEST
        );
    }

    #[tokio::test]
    async fn initialize_requires_protocol_client_and_capabilities() {
        let mut server = McpServer::new(ServerConfig::default());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Value(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({})),
        };

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(
            response.error.unwrap().code,
            super::super::protocol::error_codes::INVALID_PARAMS
        );
        assert_eq!(server.initialization_state, InitializationState::NotStarted);
    }

    #[tokio::test]
    async fn request_params_must_be_an_object() {
        let mut server = McpServer::new(ServerConfig::default());
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Value(serde_json::json!(1)),
            method: "resources/list".to_string(),
            params: Some(serde_json::json!([])),
        };

        let response = server.handle_request(request).await.unwrap();
        assert_eq!(response.id, serde_json::json!(1));
        assert_eq!(
            response.error.unwrap().code,
            super::super::protocol::error_codes::INVALID_REQUEST
        );
    }

    #[tokio::test]
    async fn oversized_line_is_drained_before_the_next_message() {
        let mut input = vec![b'x'; MAX_REQUEST_BYTES + 1];
        input.extend_from_slice(b"\n{}\n");
        let mut reader = BufReader::new(input.as_slice());
        let mut line = Vec::new();

        assert!(matches!(
            read_bounded_line(&mut reader, &mut line).await.unwrap(),
            ReadLine::TooLarge
        ));
        assert!(matches!(
            read_bounded_line(&mut reader, &mut line).await.unwrap(),
            ReadLine::Line
        ));
        assert_eq!(line, b"{}\n");
    }
}
