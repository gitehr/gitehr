use anyhow::{bail, Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Content-addressed file storage for binary attachments.
/// Implements VPR's approach: files are stored by their SHA-256 hash to ensure:
/// 1. Deduplication - identical files stored only once
/// 2. Integrity - hash serves as checksum
/// 3. Immutability - content cannot be changed without changing hash
///
/// Directory structure:
/// .gitehr/
///   attachments/
///     ab/
///       cd/
///         abcdef1234567890... (actual file)
///     metadata/
///       abcdef1234567890....json (metadata about the file)

const ATTACHMENTS_DIR: &str = ".gitehr/attachments";
const METADATA_DIR: &str = ".gitehr/attachments/metadata";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AttachmentMetadata {
    /// SHA-256 hash of the file content
    pub hash: String,
    /// Original filename when attached
    pub original_filename: String,
    /// MIME type (if detectable)
    pub mime_type: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Timestamp when attached
    pub attached_at: String,
    /// Optional description/note
    pub description: Option<String>,
    /// Optional tags for categorization
    pub tags: Vec<String>,
}

/// Add a file to the attachment store
pub fn add_attachment(
    file_path: &Path,
    description: Option<String>,
    tags: Vec<String>,
) -> Result<String> {
    // Ensure we're in a gitehr repository
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository. Run 'gitehr init' first.");
    }

    // Read and hash the file
    let mut file = fs::File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    hasher.update(&buffer);
    let hash = format!("{:x}", hasher.finalize());

    // Create attachments directory structure
    let attachments_dir = Path::new(ATTACHMENTS_DIR);
    let metadata_dir = Path::new(METADATA_DIR);
    fs::create_dir_all(attachments_dir).context("Failed to create attachments directory")?;
    fs::create_dir_all(metadata_dir).context("Failed to create metadata directory")?;

    // Store file using content-addressed path: ab/cd/abcdef...
    let hash_dir1 = &hash[0..2];
    let hash_dir2 = &hash[2..4];
    let storage_dir = attachments_dir.join(hash_dir1).join(hash_dir2);
    fs::create_dir_all(&storage_dir).context("Failed to create storage directory")?;

    let storage_path = storage_dir.join(&hash);

    // Only copy if not already present (deduplication)
    if !storage_path.exists() {
        fs::write(&storage_path, &buffer).context("Failed to write attachment to storage")?;
    }

    // Detect MIME type (basic detection)
    let mime_type = detect_mime_type(file_path, &buffer);

    // Create metadata
    let metadata = AttachmentMetadata {
        hash: hash.clone(),
        original_filename: file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        mime_type,
        size: buffer.len() as u64,
        attached_at: Utc::now().to_rfc3339(),
        description,
        tags,
    };

    // Store metadata
    let metadata_path = metadata_dir.join(format!("{}.json", hash));
    let metadata_json =
        serde_json::to_string_pretty(&metadata).context("Failed to serialize metadata")?;
    fs::write(&metadata_path, metadata_json).context("Failed to write metadata")?;

    Ok(hash)
}

/// Get an attachment by hash and copy it to a destination
pub fn get_attachment(hash: &str, dest_path: &Path) -> Result<()> {
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository.");
    }

    let storage_path = get_storage_path(hash)?;

    if !storage_path.exists() {
        bail!("Attachment not found: {}", hash);
    }

    fs::copy(&storage_path, dest_path)
        .with_context(|| format!("Failed to copy attachment to {}", dest_path.display()))?;

    Ok(())
}

/// Get attachment metadata
pub fn get_metadata(hash: &str) -> Result<AttachmentMetadata> {
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository.");
    }

    let metadata_path = Path::new(METADATA_DIR).join(format!("{}.json", hash));

    if !metadata_path.exists() {
        bail!("Metadata not found for attachment: {}", hash);
    }

    let metadata_json = fs::read_to_string(&metadata_path).context("Failed to read metadata")?;

    let metadata: AttachmentMetadata =
        serde_json::from_str(&metadata_json).context("Failed to parse metadata")?;

    Ok(metadata)
}

/// List all attachments
pub fn list_attachments() -> Result<Vec<AttachmentMetadata>> {
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository.");
    }

    let metadata_dir = Path::new(METADATA_DIR);
    if !metadata_dir.exists() {
        return Ok(Vec::new());
    }

    let mut attachments = Vec::new();

    for entry in fs::read_dir(metadata_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let metadata_json = fs::read_to_string(&path)?;
            if let Ok(metadata) = serde_json::from_str::<AttachmentMetadata>(&metadata_json) {
                attachments.push(metadata);
            }
        }
    }

    // Sort by attached_at timestamp (newest first)
    attachments.sort_by(|a, b| b.attached_at.cmp(&a.attached_at));

    Ok(attachments)
}

/// Verify attachment integrity (check hash matches content)
pub fn verify_attachment(hash: &str) -> Result<bool> {
    if !Path::new(".gitehr").exists() {
        bail!("Not in a gitehr repository.");
    }

    let storage_path = get_storage_path(hash)?;

    if !storage_path.exists() {
        bail!("Attachment not found: {}", hash);
    }

    let content = fs::read(&storage_path).context("Failed to read attachment")?;

    let mut hasher = Sha256::new();
    hasher.update(&content);
    let computed_hash = format!("{:x}", hasher.finalize());

    Ok(computed_hash == hash)
}

// Helper functions

fn get_storage_path(hash: &str) -> Result<PathBuf> {
    if hash.len() < 4 {
        bail!("Invalid hash: too short");
    }

    let hash_dir1 = &hash[0..2];
    let hash_dir2 = &hash[2..4];

    Ok(Path::new(ATTACHMENTS_DIR)
        .join(hash_dir1)
        .join(hash_dir2)
        .join(hash))
}

fn detect_mime_type(path: &Path, content: &[u8]) -> Option<String> {
    // Basic MIME type detection based on file extension and magic bytes

    // Check extension first
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let mime = match ext.to_lowercase().as_str() {
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "dcm" => "application/dicom",
            "xml" => "application/xml",
            "json" => "application/json",
            "txt" => "text/plain",
            "md" => "text/markdown",
            _ => return None,
        };
        return Some(mime.to_string());
    }

    // Check magic bytes for common formats
    if content.len() >= 4 {
        if &content[0..4] == b"%PDF" {
            return Some("application/pdf".to_string());
        }
        if &content[0..4] == [0x89, 0x50, 0x4E, 0x47] {
            return Some("image/png".to_string());
        }
        if content.len() >= 2 && &content[0..2] == [0xFF, 0xD8] {
            return Some("image/jpeg".to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(&temp).unwrap();
        fs::create_dir(".gitehr").unwrap();
        temp
    }

    #[test]
    #[serial]
    fn test_add_attachment() {
        let _temp = setup();

        // Create a test file
        let test_file = Path::new("test.txt");
        fs::write(test_file, b"Hello, World!").unwrap();

        let hash = add_attachment(
            test_file,
            Some("Test file".to_string()),
            vec!["test".to_string()],
        )
        .unwrap();

        assert_eq!(hash.len(), 64); // SHA-256 is 64 hex characters

        // Verify file was stored
        let storage_path = get_storage_path(&hash).unwrap();
        assert!(storage_path.exists());

        // Verify metadata was created
        let metadata_path = Path::new(METADATA_DIR).join(format!("{}.json", hash));
        assert!(metadata_path.exists());
    }

    #[test]
    #[serial]
    fn test_deduplication() {
        let _temp = setup();

        // Create two identical files
        fs::write("file1.txt", b"Same content").unwrap();
        fs::write("file2.txt", b"Same content").unwrap();

        let hash1 = add_attachment(Path::new("file1.txt"), None, vec![]).unwrap();
        let hash2 = add_attachment(Path::new("file2.txt"), None, vec![]).unwrap();

        // Both should have same hash
        assert_eq!(hash1, hash2);

        // Only one copy should exist in storage
        let storage_path = get_storage_path(&hash1).unwrap();
        assert!(storage_path.exists());
    }

    #[test]
    #[serial]
    fn test_get_attachment() {
        let _temp = setup();

        let content = b"Test content for retrieval";
        fs::write("original.txt", content).unwrap();

        let hash = add_attachment(Path::new("original.txt"), None, vec![]).unwrap();

        // Retrieve to different path
        let retrieved_path = Path::new("retrieved.txt");
        get_attachment(&hash, retrieved_path).unwrap();

        let retrieved_content = fs::read(retrieved_path).unwrap();
        assert_eq!(retrieved_content, content);
    }

    #[test]
    #[serial]
    fn test_get_metadata() {
        let _temp = setup();

        fs::write("test.pdf", b"%PDFtest").unwrap();

        let hash = add_attachment(
            Path::new("test.pdf"),
            Some("A test PDF".to_string()),
            vec!["document".to_string(), "test".to_string()],
        )
        .unwrap();

        let metadata = get_metadata(&hash).unwrap();

        assert_eq!(metadata.hash, hash);
        assert_eq!(metadata.original_filename, "test.pdf");
        assert_eq!(metadata.mime_type, Some("application/pdf".to_string()));
        assert_eq!(metadata.description, Some("A test PDF".to_string()));
        assert_eq!(metadata.tags, vec!["document", "test"]);
    }

    #[test]
    #[serial]
    fn test_list_attachments() {
        let _temp = setup();

        fs::write("file1.txt", b"File 1").unwrap();
        fs::write("file2.txt", b"File 2").unwrap();

        add_attachment(Path::new("file1.txt"), None, vec![]).unwrap();
        add_attachment(Path::new("file2.txt"), None, vec![]).unwrap();

        let attachments = list_attachments().unwrap();
        assert_eq!(attachments.len(), 2);
    }

    #[test]
    #[serial]
    fn test_verify_attachment() {
        let _temp = setup();

        fs::write("test.txt", b"Verify me").unwrap();
        let hash = add_attachment(Path::new("test.txt"), None, vec![]).unwrap();

        assert!(verify_attachment(&hash).unwrap());
    }

    #[test]
    #[serial]
    fn test_verify_corrupted_attachment() {
        let _temp = setup();

        fs::write("test.txt", b"Original content").unwrap();
        let hash = add_attachment(Path::new("test.txt"), None, vec![]).unwrap();

        // Corrupt the stored file
        let storage_path = get_storage_path(&hash).unwrap();
        fs::write(&storage_path, b"Corrupted!").unwrap();

        assert!(!verify_attachment(&hash).unwrap());
    }

    #[test]
    #[serial]
    fn test_mime_detection_pdf() {
        let _temp = setup();

        fs::write("doc.pdf", b"%PDF-1.4").unwrap();
        let hash = add_attachment(Path::new("doc.pdf"), None, vec![]).unwrap();

        let metadata = get_metadata(&hash).unwrap();
        assert_eq!(metadata.mime_type, Some("application/pdf".to_string()));
    }

    #[test]
    #[serial]
    fn test_attachment_fails_without_gitehr() {
        let temp = TempDir::new().unwrap();
        env::set_current_dir(&temp).unwrap();

        fs::write("test.txt", b"Test").unwrap();

        let result = add_attachment(Path::new("test.txt"), None, vec![]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a gitehr repository"));
    }
}
