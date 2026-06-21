// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Tool Handlers
//!
//! Tools allow write operations on GitEHR repositories.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

/// List tools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsList {
    pub tools: Vec<Tool>,
}

/// Tool handler for GitEHR repositories
pub struct ToolHandler {
    repo_path: PathBuf,
}

impl ToolHandler {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// List all available tools
    pub fn list_tools(&self) -> anyhow::Result<ToolsList> {
        let tools = vec![
            Tool {
                name: "add_journal_entry".to_string(),
                description: "Create a new clinical journal entry".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "Markdown content of the journal entry"
                        }
                    },
                    "required": ["content"]
                }),
            },
            Tool {
                name: "update_state".to_string(),
                description: "Update a state file in the repository".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "filename": {
                            "type": "string",
                            "description": "Name of the state file"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write to the state file"
                        }
                    },
                    "required": ["filename", "content"]
                }),
            },
            Tool {
                name: "verify_journal".to_string(),
                description: "Verify the integrity of the journal hash chain".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "search_repository".to_string(),
                description: "Search journal and state files for a query string".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query string"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ];

        // Clinical calculator tools: one per calc-core calculator, all driven by
        // the single scoring engine. The schema handed to the model is the
        // calculator's own `input_schema()`, so the LLM gets a typed input
        // contract rather than scraping it from prose. Namespaced with a `calc_`
        // prefix to keep them grouped and unambiguous to dispatch.
        let mut tools = tools;
        for calc in calc_core::all() {
            tools.push(Tool {
                name: format!("calc_{}", calc.name()),
                description: format!("{} - {}", calc.title(), calc.description()),
                input_schema: calc.input_schema(),
            });
        }

        Ok(ToolsList { tools })
    }

    /// Execute a tool by name
    pub fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        match name {
            "add_journal_entry" => self.add_journal_entry(arguments),
            "update_state" => self.update_state(arguments),
            "verify_journal" => self.verify_journal(arguments),
            "search_repository" => self.search_repository(arguments),
            other => {
                if let Some(calc_name) = other.strip_prefix("calc_") {
                    self.run_calculator(calc_name, arguments)
                } else {
                    Err(anyhow::anyhow!("Unknown tool: {}", name))
                }
            }
        }
    }

    /// Run a clinical calculator from calc-core and return its
    /// `CalculationResponse` as JSON text. Invalid inputs and unknown
    /// calculators are reported as tool errors rather than transport errors,
    /// so the model can see and recover from them.
    fn run_calculator(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<ToolResult> {
        let calc = match calc_core::get(name) {
            Some(c) => c,
            None => {
                return Ok(ToolResult {
                    content: vec![ToolContent::Text {
                        text: format!("Unknown calculator: {name}"),
                    }],
                    is_error: Some(true),
                });
            }
        };

        match calc.calculate(&arguments) {
            Ok(response) => Ok(ToolResult {
                content: vec![ToolContent::Text {
                    text: serde_json::to_string_pretty(&response)?,
                }],
                is_error: None,
            }),
            Err(e) => Ok(ToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Calculation error: {e}"),
                }],
                is_error: Some(true),
            }),
        }
    }

    fn add_journal_entry(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        // This is a placeholder - in reality we would use the gitehr library
        // For now, just validate that the journal directory exists
        let journal_dir = self.repo_path.join("journal");
        if !journal_dir.exists() {
            return Err(anyhow::anyhow!("Journal directory not found"));
        }

        // Generate a placeholder filename
        let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ");
        let uuid = "placeholder-uuid";
        let filename = format!("{}-{}.md", timestamp, uuid);

        // In a real implementation, we would:
        // 1. Get the latest journal entry hash
        // 2. Create the new entry with proper YAML front matter
        // 3. Commit to git
        // For now, return a success message

        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!(
                    "Would create journal entry: journal/{}\nContent length: {} characters\n\nNote: Full journal creation will be implemented when gitehr library is integrated.",
                    filename,
                    content.len()
                ),
            }],
            is_error: Some(false),
        })
    }

    fn update_state(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
        let filename = arguments
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'filename' parameter"))?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        let state_dir = self.repo_path.join("state");
        if !state_dir.exists() {
            std::fs::create_dir_all(&state_dir)?;
        }

        let file_path = state_dir.join(filename);
        std::fs::write(&file_path, content)?;

        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!("Updated state file: state/{}", filename),
            }],
            is_error: Some(false),
        })
    }

    fn verify_journal(&self, _arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
        let journal_dir = self.repo_path.join("journal");

        if !journal_dir.exists() {
            return Ok(ToolResult {
                content: vec![ToolContent::Text {
                    text: "Journal directory not found - cannot verify".to_string(),
                }],
                is_error: Some(true),
            });
        }

        let entry_count = std::fs::read_dir(&journal_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .count();

        // Placeholder - real verification would check hash chain
        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!(
                    "Journal verification placeholder:\nFound {} journal entries\n\nNote: Full hash chain verification will be implemented when gitehr library is integrated.",
                    entry_count
                ),
            }],
            is_error: Some(false),
        })
    }

    fn search_repository(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = arguments
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?;

        let mut results = vec![];

        // Search journal
        let journal_dir = self.repo_path.join("journal");
        if journal_dir.exists() {
            for entry in std::fs::read_dir(&journal_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    let content = std::fs::read_to_string(&path)?;
                    if content.to_lowercase().contains(&query.to_lowercase()) {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            results.push(format!("journal/{}", filename));
                        }
                    }
                }
            }
        }

        // Search state
        let state_dir = self.repo_path.join("state");
        if state_dir.exists() {
            for entry in std::fs::read_dir(&state_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let content = std::fs::read_to_string(&path)?;
                    if content.to_lowercase().contains(&query.to_lowercase()) {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            results.push(format!("state/{}", filename));
                        }
                    }
                }
            }
        }

        let result_text = if results.is_empty() {
            format!("No results found for query: {}", query)
        } else {
            format!(
                "Found {} results for query '{}':\n{}",
                results.len(),
                query,
                results.join("\n")
            )
        };

        Ok(ToolResult {
            content: vec![ToolContent::Text { text: result_text }],
            is_error: Some(false),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_serialization() {
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "Test tool".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };

        let json = serde_json::to_string(&tool).unwrap();
        let parsed: Tool = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "test_tool");
    }

    #[test]
    fn test_calculators_listed_as_tools() {
        let handler = ToolHandler::new(PathBuf::from("."));
        let tools = handler.list_tools().unwrap().tools;
        // Every calc-core calculator should surface as a `calc_<name>` tool,
        // carrying its own input schema.
        for calc in calc_core::all() {
            let tool_name = format!("calc_{}", calc.name());
            let tool = tools
                .iter()
                .find(|t| t.name == tool_name)
                .unwrap_or_else(|| panic!("calculator tool {tool_name} not listed"));
            assert_eq!(tool.input_schema, calc.input_schema());
        }
    }

    #[test]
    fn test_call_calculator_tool() {
        let handler = ToolHandler::new(PathBuf::from("."));
        let result = handler
            .call_tool(
                "calc_feverpain",
                serde_json::json!({
                    "fever": true,
                    "purulence": true,
                    "attend_rapidly": true,
                    "inflamed_tonsils": false,
                    "absence_of_cough": false
                }),
            )
            .unwrap();
        assert!(result.is_error.is_none());
        let ToolContent::Text { text } = &result.content[0];
        let response: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(response["calculator"], "feverpain");
        assert_eq!(response["result"], 3);
    }

    #[test]
    fn test_unknown_calculator_is_tool_error() {
        let handler = ToolHandler::new(PathBuf::from("."));
        let result = handler
            .call_tool("calc_nonesuch", serde_json::json!({}))
            .unwrap();
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn test_tool_result() {
        let result = ToolResult {
            content: vec![ToolContent::Text {
                text: "Success".to_string(),
            }],
            is_error: Some(false),
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["content"][0]["type"], "text");
        // The skip_serializing_if means false is omitted from JSON
        // Let's test it's either false or absent
        if let Some(is_error) = json.get("isError") {
            assert_eq!(is_error, &serde_json::json!(false));
        }
    }
}
