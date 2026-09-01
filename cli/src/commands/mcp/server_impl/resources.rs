// SPDX-FileCopyrightText: 2026 Marcus Baw and Baw Medical Ltd
// SPDX-License-Identifier: AGPL-3.0-or-later

//! MCP Resource Handlers
//!
//! Resources provide read-only access to GitEHR repository data.

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::commands::document::{DOCUMENT_ROOTS, MANIFEST_FILENAME};

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

const REPO_URI_PREFIX: &str = "gitehr://repo/";

/// Accept only a bare filename: a single normal path component.
///
/// Filenames arrive from MCP resource URIs and tool arguments; anything
/// with separators, `..`, or a root component would escape the repository.
pub(super) fn safe_filename(name: &str) -> anyhow::Result<&str> {
    let mut components = std::path::Path::new(name).components();
    match (components.next(), components.next()) {
        (Some(std::path::Component::Normal(_)), None) if !name.contains('\\') => Ok(name),
        _ => Err(anyhow::anyhow!("Invalid filename: {name}")),
    }
}

/// Best-effort MIME type from a Document's file extension. Falls back to a
/// generic binary type - the original bytes are always returned unmodified
/// as a base64 blob regardless of what this guesses.
fn guess_mime_type(filename: &str) -> &'static str {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "pdf" => "application/pdf",
        "json" => "application/json",
        "md" | "txt" => "text/plain",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "dcm" => "application/dicom",
        "xml" => "application/xml",
        "csv" => "text/csv",
        _ => "application/octet-stream",
    }
}

impl ResourceHandler {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    /// List all available resources
    pub fn list_resources(&self) -> anyhow::Result<ResourcesList> {
        let resources = vec![
            Resource {
                uri: "gitehr://repo/journal".to_string(),
                name: "Journal Entries".to_string(),
                description: Some("Chronological clinical notes and entries".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gitehr://repo/state".to_string(),
                name: "Current Clinical State".to_string(),
                description: Some("Active problems, medications, allergies".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gitehr://repo/status".to_string(),
                name: "Repository Status".to_string(),
                description: Some("Repository metadata and status".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gitehr://repo/documents".to_string(),
                name: "Documents".to_string(),
                description: Some(
                    "Non-imaging clinical Documents (reports, correspondence, results)".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "gitehr://repo/imaging".to_string(),
                name: "Imaging".to_string(),
                description: Some("Imaging Documents and studies".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ];

        Ok(ResourcesList { resources })
    }

    /// Read a specific resource by URI
    pub fn read_resource(&self, uri: &str) -> anyhow::Result<ResourcesRead> {
        let rest = uri
            .strip_prefix(REPO_URI_PREFIX)
            .ok_or_else(|| anyhow::anyhow!("Unknown resource URI: {}", uri))?;

        match rest {
            "journal" => self.read_journal(),
            "state" => self.read_state(),
            "status" => self.read_status(),
            "documents" => self.read_document_root("documents"),
            "imaging" => self.read_document_root("imaging"),
            _ => {
                if let Some(entry_id) = rest.strip_prefix("journal/") {
                    self.read_journal_entry(entry_id)
                } else if let Some(filename) = rest.strip_prefix("state/") {
                    self.read_state_file(filename)
                } else if let Some(name) = rest.strip_prefix("documents/") {
                    self.read_document_item("documents", name)
                } else if let Some(name) = rest.strip_prefix("imaging/") {
                    self.read_document_item("imaging", name)
                } else {
                    Err(anyhow::anyhow!("Unknown resource URI: {}", uri))
                }
            }
        }
    }

    fn read_journal(&self) -> anyhow::Result<ResourcesRead> {
        let journal_dir = self.repo_path.join("journal");
        let mut entries = vec![];

        if journal_dir.exists() {
            for entry in std::fs::read_dir(&journal_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md")
                    && let Some(filename) = path.file_name().and_then(|s| s.to_str())
                {
                    entries.push(filename.to_string());
                }
            }
        }

        entries.sort();

        let content = ResourceContent::Text {
            text: serde_json::to_string_pretty(&entries)?,
        };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: "gitehr://repo/journal".to_string(),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    fn read_journal_entry(&self, entry_id: &str) -> anyhow::Result<ResourcesRead> {
        let entry_id = safe_filename(entry_id)?;
        let entry_path = self.repo_path.join("journal").join(entry_id);

        if !entry_path.exists() {
            return Err(anyhow::anyhow!("Journal entry not found: {}", entry_id));
        }

        let content_text = std::fs::read_to_string(&entry_path)?;
        let content = ResourceContent::Text { text: content_text };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!("gitehr://repo/journal/{}", entry_id),
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
                if path.is_file()
                    && let Some(filename) = path.file_name().and_then(|s| s.to_str())
                    && filename != "README.md"
                {
                    files.push(filename.to_string());
                }
            }
        }

        files.sort();

        let content = ResourceContent::Text {
            text: serde_json::to_string_pretty(&files)?,
        };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: "gitehr://repo/state".to_string(),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    fn read_state_file(&self, filename: &str) -> anyhow::Result<ResourcesRead> {
        let filename = safe_filename(filename)?;
        let file_path = self.repo_path.join("state").join(filename);

        if !file_path.exists() {
            return Err(anyhow::anyhow!("State file not found: {}", filename));
        }

        let content_text = std::fs::read_to_string(&file_path)?;
        let content = ResourceContent::Text { text: content_text };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!("gitehr://repo/state/{}", filename),
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
                uri: "gitehr://repo/status".to_string(),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    /// List the Documents (files or directory studies) under a Document root.
    /// `root` is always a literal `"documents"` or `"imaging"` from
    /// `read_resource`'s match arms.
    fn read_document_root(&self, root: &str) -> anyhow::Result<ResourcesRead> {
        debug_assert!(DOCUMENT_ROOTS.contains(&root));
        let root_dir = self.repo_path.join(root);
        let mut entries = vec![];

        // Same symlink discipline as transport create (R77): refuse to read
        // through a symlink planted in the Documents tree. Also skip
        // symlinked entries when listing, so the listing cannot advertise a
        // link that the item read will refuse.
        if root_dir.exists() {
            for entry in std::fs::read_dir(&root_dir)? {
                let entry = entry?;
                if entry.file_type().map(|t| t.is_symlink()).unwrap_or(true) {
                    continue;
                }
                if let Some(name) = entry.file_name().to_str()
                    && name != "README.md"
                {
                    entries.push(name.to_string());
                }
            }
        }

        entries.sort();

        let content = ResourceContent::Text {
            text: serde_json::to_string_pretty(&entries)?,
        };

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri: format!("gitehr://repo/{}", root),
                mime_type: Some("application/json".to_string()),
                content,
            }],
        })
    }

    /// Read a single Document under a root. A directory Document (a
    /// multi-file study) has no single byte stream to return, so its
    /// manifest is returned instead - the manifest hashes every file in the
    /// study and is itself the Document's anchoring content (ADR-0003).
    fn read_document_item(&self, root: &str, name: &str) -> anyhow::Result<ResourcesRead> {
        debug_assert!(DOCUMENT_ROOTS.contains(&root));
        let name = safe_filename(name)?;
        let path = self.repo_path.join(root).join(name);
        let uri = format!("gitehr://repo/{}/{}", root, name);

        if path.is_dir() {
            let manifest_path = path.join(MANIFEST_FILENAME);
            let text = std::fs::read_to_string(&manifest_path)
                .map_err(|_| anyhow::anyhow!("Manifest not found for study Document: {}", name))?;

            return Ok(ResourcesRead {
                contents: vec![ResourceReadContent {
                    uri,
                    mime_type: Some("application/json".to_string()),
                    content: ResourceContent::Text { text },
                }],
            });
        }

        if !path.is_file() {
            return Err(anyhow::anyhow!("Document not found: {}/{}", root, name));
        }

        // Same symlink discipline as transport create (R77): Documents are
        // repo-local content, and a symlink planted in a received or cloned
        // repo must not become an arbitrary-file-read through the MCP
        // surface. Regular files only; symlinks are refused.
        if std::fs::symlink_metadata(&path)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false)
        {
            return Err(anyhow::anyhow!(
                "Refusing to read '{}': it is a symlink, which could point outside the repository.",
                path.display()
            ));
        }

        let bytes = std::fs::read(&path)?;
        let blob = base64::engine::general_purpose::STANDARD.encode(bytes);

        Ok(ResourcesRead {
            contents: vec![ResourceReadContent {
                uri,
                mime_type: Some(guess_mime_type(name).to_string()),
                content: ResourceContent::Blob { blob },
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

    #[test]
    fn test_read_document_item_rejects_traversal() {
        let handler = ResourceHandler::new(PathBuf::from("."));
        for name in ["../evil.txt", "/tmp/evil.txt", "a/b.txt", "..", "c\\d.txt"] {
            let err = handler.read_document_item("documents", name).unwrap_err();
            assert!(
                err.to_string().contains("Invalid filename"),
                "expected rejection for {name:?}"
            );
        }
    }

    #[test]
    fn test_read_document_item_missing_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let handler = ResourceHandler::new(dir.path().to_path_buf());
        let err = handler
            .read_document_item("documents", "nope.pdf")
            .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_read_document_root_lists_files_and_skips_readme() {
        let dir = tempfile::tempdir().unwrap();
        let documents_dir = dir.path().join("documents");
        std::fs::create_dir_all(&documents_dir).unwrap();
        std::fs::write(documents_dir.join("README.md"), "layout notes").unwrap();
        std::fs::write(documents_dir.join("2026-01-01-report-abcd1234.pdf"), b"pdf").unwrap();

        let handler = ResourceHandler::new(dir.path().to_path_buf());
        let read = handler.read_resource("gitehr://repo/documents").unwrap();

        let ResourceContent::Text { text } = &read.contents[0].content else {
            panic!("expected text content");
        };
        let entries: Vec<String> = serde_json::from_str(text).unwrap();
        assert_eq!(entries, vec!["2026-01-01-report-abcd1234.pdf"]);
    }

    #[test]
    fn test_read_document_item_returns_base64_blob_of_original_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let imaging_dir = dir.path().join("imaging");
        std::fs::create_dir_all(&imaging_dir).unwrap();
        let original = b"not really a jpeg but bytes are bytes";
        std::fs::write(imaging_dir.join("scan.jpg"), original).unwrap();

        let handler = ResourceHandler::new(dir.path().to_path_buf());
        let read = handler
            .read_resource("gitehr://repo/imaging/scan.jpg")
            .unwrap();

        let content = &read.contents[0];
        assert_eq!(content.mime_type.as_deref(), Some("image/jpeg"));
        let ResourceContent::Blob { blob } = &content.content else {
            panic!("expected blob content");
        };
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(blob)
            .unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_read_document_item_directory_returns_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let study_dir = dir
            .path()
            .join("imaging")
            .join("2026-01-01-ct-head-abcd1234");
        std::fs::create_dir_all(&study_dir).unwrap();
        let manifest = r#"{"files":{"scan.dcm":"deadbeef"}}"#;
        std::fs::write(study_dir.join(MANIFEST_FILENAME), manifest).unwrap();

        let handler = ResourceHandler::new(dir.path().to_path_buf());
        let read = handler
            .read_resource("gitehr://repo/imaging/2026-01-01-ct-head-abcd1234")
            .unwrap();

        let content = &read.contents[0];
        assert_eq!(content.mime_type.as_deref(), Some("application/json"));
        let ResourceContent::Text { text } = &content.content else {
            panic!("expected text content");
        };
        assert_eq!(text, manifest);
    }
}
