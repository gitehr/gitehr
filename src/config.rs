use crate::error::{ConfigError, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;
use uuid::Uuid;

pub fn ensure_config(path: &Path) -> Result<()> {
    if path.exists() {
        let contents = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        if contents.contains("repo_id") && contents.contains("created_at") {
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| ConfigError::CreateDir {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let repo_id = Uuid::new_v4();
    let created_at = Utc::now().to_rfc3339();

    let contents = format!(
        r#"repo_id = "{repo_id}"
created_at = "{created_at}"
type = "organisation"

[journal]
enabled = true

[imaging]
enabled = true

[state]
enabled = true

Welcome to GitEHR! Welcome to the future!
"#
    );

    fs::write(path, contents).map_err(|source| ConfigError::Write {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(())
}
