// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Resource Handlers
//!
//! Resources provide read-only access to GitEHR repository data.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// MCP Resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "blob")]
    Blob { blob: String }, // base64 encoded
}

/// List resources response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesList {
    pub resources: Vec<Resource>,
}

/// Read resource response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesRead {
    pub contents: Vec<ResourceReadContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadContent {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(flatten)]
    pub content: ResourceContent,
}

/// Resource handler for GitEHR repositories
pub struct ResourceHandler {
    repo_path: PathBuf,
}

impl ResourceHandler {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// List all available resources
    pub fn list_resources(&self) -> anyhow::Result<ResourcesList> {
        let mut resources = vec![];

        // Journal resource
        resources.push(Resource {
            uri: format!("gitehr://repo/{}/journal", self.repo_path.display()),
            name: "Journal Entries".to_string(),
            description: Some("Chronological clinical notes and entries".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        // State resource
        resources.push(Resource {
            uri: format!("gitehr://repo/{}/state", self.repo_path.display()),
            name: "Current Clinical State".to_string(),
            description: Some("Active problems, medications, allergies".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        // Status resource
        resources.push(Resource {
            uri: format!("gitehr://repo/{}/status", self.repo_path.display()),
            name: "Repository Status".to_string(),
            description: Some("Repository metadata and status".to_string()),
            mime_type: Some("application/json".to_string()),
        });

        Ok(ResourcesList { resources })
    }

    /// Read a specific resource by URI
    pub fn read_resource(&self, uri: &str) -> anyhow::Result<ResourcesRead> {
        let parts: Vec<&str> = uri.split('/').collect();

        match parts.as_slice() {
            ["gitehr:", "", "repo", _repo, "journal"] => self.read_journal(),
            ["gitehr:", "", "repo", _repo, "journal", entry_id] => {
                self.read_journal_entry(entry_id)
            }
            ["gitehr:", "", "repo", _repo, "state"] => self.read_state(),
            ["gitehr:", "", "repo", _repo, "state", filename] => self.read_state_file(filename),
            ["gitehr:", "", "repo", _repo, "status"] => self.read_status(),
            _ => Err(anyhow::anyhow!("Unknown resource URI: {}", uri)),
        }
    }

    fn read_journal(&self) -> anyhow::Result<ResourcesRead> {
        let journal_dir = self.repo_path.join("journal");
        let mut entries = vec![];

        if journal_dir.exists() {
            for entry in std::fs::read_dir(&journal_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        entries.push(filename.to_string());
                    }
                }
            }
        }

        entries.sort();

        let content = ResourceContent::Text {
            text: serde_json::to_string_pretty(&entries)?,
        };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!("gitehr://repo/{}/journal", self.repo_path.display()),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    fn read_journal_entry(&self, entry_id: &str) -> anyhow::Result<ResourcesRead> {
        let entry_path = self.repo_path.join("journal").join(entry_id);

        if !entry_path.exists() {
            return Err(anyhow::anyhow!("Journal entry not found: {}", entry_id));
        }

        let content_text = std::fs::read_to_string(&entry_path)?;
        let content = ResourceContent::Text { text: content_text };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!(
                    "gitehr://repo/{}/journal/{}",
                    self.repo_path.display(),
                    entry_id
                ),
                mime_type: Some("text/markdown".to_string()),
                content,
            }],
        })
    }

    fn read_state(&self) -> anyhow::Result<ResourcesRead> {
        let state_dir = self.repo_path.join("state");
        let mut files = vec![];

        if state_dir.exists() {
            for entry in std::fs::read_dir(&state_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if filename != "README.md" {
                            files.push(filename.to_string());
                        }
                    }
                }
            }
        }

        files.sort();

        let content = ResourceContent::Text {
            text: serde_json::to_string_pretty(&files)?,
        };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!("gitehr://repo/{}/state", self.repo_path.display()),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    fn read_state_file(&self, filename: &str) -> anyhow::Result<ResourcesRead> {
        let file_path = self.repo_path.join("state").join(filename);

        if !file_path.exists() {
            return Err(anyhow::anyhow!("State file not found: {}", filename));
        }

        let content_text = std::fs::read_to_string(&file_path)?;
        let content = ResourceContent::Text { text: content_text };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!(
                    "gitehr://repo/{}/state/{}",
                    self.repo_path.display(),
                    filename
                ),
                mime_type: Some("text/plain".to_string()),
                content,
            }],
        })
    }

    fn read_status(&self) -> anyhow::Result<ResourcesRead> {
        let status = self.get_repo_status()?;
        let content = ResourceContent::Text {
            text: serde_json::to_string_pretty(&status)?,
        };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!("gitehr://repo/{}/status", self.repo_path.display()),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    fn get_repo_status(&self) -> anyhow::Result<serde_json::Value> {
        let version = std::fs::read_to_string(self.repo_path.join(".gitehr/GITEHR_VERSION"))
            .unwrap_or_else(|_| "unknown".to_string());

        let is_encrypted = self.repo_path.join(".gitehr/ENCRYPTED").exists();

        let journal_count = std::fs::read_dir(self.repo_path.join("journal"))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("md"))
            .count();

        let state_files: Vec<String> = std::fs::read_dir(self.repo_path.join("state"))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
            .filter(|name| name != "README.md")
            .collect();

        Ok(serde_json::json!({
            "version": version.trim(),
            "encrypted": is_encrypted,
            "journal_entry_count": journal_count,
            "state_files": state_files,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_resource_serialization() {
        let resource = Resource {
            uri: "gitehr://repo/test/journal".to_string(),
            name: "Journal".to_string(),
            description: Some("Test journal".to_string()),
            mime_type: Some("application/json".to_string()),
        };

        let json = serde_json::to_string(&resource).unwrap();
        let parsed: Resource = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.uri, "gitehr://repo/test/journal");
        assert_eq!(parsed.name, "Journal");
    }

    #[test]
    fn test_resource_content_text() {
        let content = ResourceContent::Text {
            text: "test content".to_string(),
        };

        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "test content");
    }
}
