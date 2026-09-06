// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP JSON-RPC 2.0 Protocol Implementation

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// A JSON-RPC request ID, preserving the distinction between an omitted ID and
/// an explicit null ID.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum McpRequestId {
    #[default]
    Missing,
    Value(serde_json::Value),
}

impl McpRequestId {
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

impl Serialize for McpRequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Missing => serializer.serialize_unit(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for McpRequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        serde_json::Value::deserialize(deserializer).map(Self::Value)
    }
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    #[serde(default, skip_serializing_if = "McpRequestId::is_missing")]
    pub id: McpRequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl fmt::Display for McpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MCP Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for McpError {}

/// Standard JSON-RPC error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

impl McpError {
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::PARSE_ERROR,
            message: message.into(),
            data: None,
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::INVALID_REQUEST,
            message: message.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: error_codes::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: error_codes::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }
}

impl McpResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<serde_json::Value>, error: McpError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.unwrap_or(serde_json::Value::Null),
            result: None,
            error: Some(error),
        }
    }
}

/// MCP Method types
#[derive(Debug, Clone, PartialEq)]
pub enum McpMethod {
    // Initialization
    Initialize,
    InitializedNotification,

    // Resource methods
    ResourcesList,
    ResourcesRead,

    // Tool methods
    ToolsList,
    ToolsCall,

    // Prompt methods
    PromptsList,
    PromptsGet,

    // Unknown method
    Unknown(String),
}

impl McpMethod {
    // Not `std::str::FromStr`: this mapping is infallible (unknown methods
    // become `Unknown`), so it has no error type to return.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "initialize" => McpMethod::Initialize,
            "notifications/initialized" => McpMethod::InitializedNotification,
            "resources/list" => McpMethod::ResourcesList,
            "resources/read" => McpMethod::ResourcesRead,
            "tools/list" => McpMethod::ToolsList,
            "tools/call" => McpMethod::ToolsCall,
            "prompts/list" => McpMethod::PromptsList,
            "prompts/get" => McpMethod::PromptsGet,
            _ => McpMethod::Unknown(s.to_string()),
        }
    }

    pub fn log_name(&self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::InitializedNotification => "notifications/initialized",
            Self::ResourcesList => "resources/list",
            Self::ResourcesRead => "resources/read",
            Self::ToolsList => "tools/list",
            Self::ToolsCall => "tools/call",
            Self::PromptsList => "prompts/list",
            Self::PromptsGet => "prompts/get",
            Self::Unknown(_) => "unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: McpRequestId::Value(serde_json::json!(1)),
            method: "resources/list".to_string(),
            params: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: McpRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.method, "resources/list");
        assert_eq!(parsed.id, McpRequestId::Value(serde_json::json!(1)));
    }

    #[test]
    fn test_request_deserialization_distinguishes_missing_and_null_ids() {
        let missing: McpRequest =
            serde_json::from_str(r#"{"jsonrpc":"2.0","method":"resources/list"}"#).unwrap();
        let null: McpRequest =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":null,"method":"resources/list"}"#)
                .unwrap();

        assert_eq!(missing.id, McpRequestId::Missing);
        assert_eq!(null.id, McpRequestId::Value(serde_json::Value::Null));
    }

    #[test]
    fn test_error_response() {
        let error = McpError::method_not_found("test_method");
        assert_eq!(error.code, error_codes::METHOD_NOT_FOUND);
        assert!(error.message.contains("test_method"));

        let response = serde_json::to_value(McpResponse::error(None, error)).unwrap();
        assert!(response.get("id").is_some());
        assert!(response["id"].is_null());
    }

    #[test]
    fn test_method_parsing() {
        assert_eq!(
            McpMethod::from_str("resources/list"),
            McpMethod::ResourcesList
        );
        assert_eq!(McpMethod::from_str("tools/call"), McpMethod::ToolsCall);
        assert_eq!(
            McpMethod::from_str("notifications/initialized"),
            McpMethod::InitializedNotification
        );
        assert!(matches!(
            McpMethod::from_str("unknown/method"),
            McpMethod::Unknown(_)
        ));
    }
}
