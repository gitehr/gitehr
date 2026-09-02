// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Tool Handlers
//!
//! Tools allow write operations on GitEHR repositories.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::commands::{contributor, journal};

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
                        },
                        "author": {
                            "type": "string",
                            "description": "Optional contributor ID (defaults to the active contributor)"
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
            "search_repository" => self.search_repository(arguments),
            _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
        }
    }

    fn add_journal_entry(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        let content = content.trim();
        if content.is_empty() {
            return Err(anyhow::anyhow!("'content' must not be empty"));
        }

        let journal_dir = self.repo_path.join("journal");
        if !journal_dir.exists() {
            return Err(anyhow::anyhow!("Journal directory not found"));
        }

        let author = arguments
            .get("author")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| contributor::get_current_contributor_at(&self.repo_path));

        let filename =
            journal::create_journal_entry_at(&self.repo_path, content, Vec::new(), author)?;

        Ok(ToolResult {
            content: vec![ToolContent::Text {
                text: format!("Created journal entry: {}", filename),
            }],
            is_error: Some(false),
        })
    }

    fn update_state(&self, arguments: serde_json::Value) -> anyhow::Result<ToolResult> {
        let filename = arguments
            .get("filename")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'filename' parameter"))?;
        let filename = super::resources::safe_filename(filename)?;

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
                if path.extension().and_then(|s| s.to_str()) == Some("md")
                    && let Ok(content) = std::fs::read_to_string(&path)
                    && content.to_lowercase().contains(&query.to_lowercase())
                    && let Some(filename) = path.file_name().and_then(|s| s.to_str())
                {
                    results.push(format!("journal/{}", filename));
                }
            }
        }

        // Search state
        let state_dir = self.repo_path.join("state");
        if state_dir.exists() {
            for entry in std::fs::read_dir(&state_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file()
                    && let Ok(content) = std::fs::read_to_string(&path)
                    && content.to_lowercase().contains(&query.to_lowercase())
                    && let Some(filename) = path.file_name().and_then(|s| s.to_str())
                {
                    results.push(format!("state/{}", filename));
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
    fn test_unknown_tool_is_transport_error() {
        let handler = ToolHandler::new(PathBuf::from("."));
        let err = handler
            .call_tool("clincalc_nonesuch", serde_json::json!({}))
            .unwrap_err();
        assert!(err.to_string().contains("Unknown tool"));
    }

    #[test]
    fn test_update_state_rejects_traversal() {
        let handler = ToolHandler::new(PathBuf::from("."));
        for filename in ["../evil.txt", "/tmp/evil.txt", "a/b.txt", "..", "c\\d.txt"] {
            let err = handler
                .call_tool(
                    "update_state",
                    serde_json::json!({"filename": filename, "content": "x"}),
                )
                .unwrap_err();
            assert!(
                err.to_string().contains("Invalid filename"),
                "expected rejection for {filename:?}"
            );
        }
    }

    fn init_git_repo(dir: &std::path::Path) {
        for args in [
            vec!["init"],
            vec!["config", "user.name", "Test User"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "commit.gpgsign", "false"],
        ] {
            std::process::Command::new("git")
                .current_dir(dir)
                .args(&args)
                .output()
                .unwrap();
        }
    }

    #[test]
    fn test_add_journal_entry_rejects_missing_content() {
        let handler = ToolHandler::new(PathBuf::from("."));
        let err = handler
            .call_tool("add_journal_entry", serde_json::json!({}))
            .unwrap_err();
        assert!(err.to_string().contains("Missing 'content'"));
    }

    #[test]
    fn test_add_journal_entry_rejects_empty_content() {
        let handler = ToolHandler::new(PathBuf::from("."));
        let err = handler
            .call_tool("add_journal_entry", serde_json::json!({"content": "   "}))
            .unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn test_add_journal_entry_writes_and_commits_at_repo_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("journal")).unwrap();
        init_git_repo(dir.path());

        let handler = ToolHandler::new(dir.path().to_path_buf());
        let result = handler
            .call_tool(
                "add_journal_entry",
                serde_json::json!({"content": "Test entry"}),
            )
            .unwrap();

        let ToolContent::Text { text } = &result.content[0];
        assert!(text.starts_with("Created journal entry: journal/"));

        let entries: Vec<_> = std::fs::read_dir(dir.path().join("journal"))
            .unwrap()
            .collect();
        assert_eq!(entries.len(), 1, "expected one journal entry on disk");

        let log = std::process::Command::new("git")
            .current_dir(dir.path())
            .args(["log", "--oneline"])
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&log.stdout).lines().count(),
            1,
            "expected the entry to be committed"
        );
    }

    #[test]
    fn test_add_journal_entry_defaults_author_from_repo_contributors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("journal")).unwrap();
        std::fs::create_dir(dir.path().join(".gitehr")).unwrap();
        std::fs::write(
            dir.path().join(".gitehr/contributors.json"),
            r#"{"contributors":{},"current_contributor":"dr-jones"}"#,
        )
        .unwrap();
        init_git_repo(dir.path());

        let handler = ToolHandler::new(dir.path().to_path_buf());
        handler
            .call_tool(
                "add_journal_entry",
                serde_json::json!({"content": "Test entry"}),
            )
            .unwrap();

        let entry_path = std::fs::read_dir(dir.path().join("journal"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let content = std::fs::read_to_string(entry_path).unwrap();
        assert!(content.contains("dr-jones"));
    }

    #[test]
    fn test_add_journal_entry_missing_journal_dir_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let handler = ToolHandler::new(dir.path().to_path_buf());
        let err = handler
            .call_tool(
                "add_journal_entry",
                serde_json::json!({"content": "Test entry"}),
            )
            .unwrap_err();
        assert!(err.to_string().contains("Journal directory not found"));
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
