use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntryInfo {
    pub filename: String,
    pub timestamp: String,
    pub parent_entry: Option<String>,
    pub author: Option<String>,
    pub content_preview: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StateFileInfo {
    pub name: String,
    pub content: String,
    pub last_modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoStatusInfo {
    pub is_gitehr_repo: bool,
    pub gitehr_version: Option<String>,
    pub journal_entry_count: usize,
    pub state_files: Vec<String>,
    pub is_encrypted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContributorInfo {
    pub id: String,
    pub name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub enabled: bool,
    pub active: bool,
}

fn with_repo_dir<T, F>(repo_path: &str, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let original_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    std::env::set_current_dir(repo_path).map_err(|e| format!("Failed to change to repo: {}", e))?;
    let result = f();
    let _ = std::env::set_current_dir(original_dir);
    result
}

#[tauri::command]
fn get_current_dir() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn is_gitehr_repo(path: String) -> bool {
    PathBuf::from(&path).join(".gitehr").exists()
}

#[tauri::command]
fn get_status(repo_path: String) -> Result<RepoStatusInfo, String> {
    with_repo_dir(&repo_path, || {
        let status = gitehr::commands::status::RepoStatus::gather().map_err(|e| e.to_string())?;

        Ok(RepoStatusInfo {
            is_gitehr_repo: status.is_gitehr_repo,
            gitehr_version: status.gitehr_version,
            journal_entry_count: status.journal_entry_count,
            state_files: status.state_files,
            is_encrypted: status.is_encrypted,
        })
    })
}

#[tauri::command]
fn get_journal_entries(
    repo_path: String,
    limit: Option<usize>,
    offset: Option<usize>,
    reverse: Option<bool>,
) -> Result<Vec<JournalEntryInfo>, String> {
    with_repo_dir(&repo_path, || {
        let journal_dir = PathBuf::from("journal");
        if !journal_dir.exists() {
            return Ok(vec![]);
        }

        let mut entries: Vec<_> = std::fs::read_dir(&journal_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name.contains('T') && name.contains('-') && name.ends_with(".md"))
                    .unwrap_or(false)
            })
            .collect();

        entries.sort();

        if reverse.unwrap_or(false) {
            entries.reverse();
        }

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(50);

        let entries_to_show: Vec<_> = entries.into_iter().skip(offset).take(limit).collect();

        let mut result = Vec::new();
        for path in entries_to_show {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let parts: Vec<&str> = content.splitn(3, "---").collect();
                if parts.len() >= 3 {
                    let yaml_content = parts[1].trim();
                    let body_content = parts[2].trim();

                    if let Ok(metadata) =
                        serde_yml::from_str::<gitehr::commands::journal::JournalEntry>(yaml_content)
                    {
                        let preview: String = body_content
                            .chars()
                            .take(200)
                            .collect::<String>()
                            .replace('\n', " ");

                        result.push(JournalEntryInfo {
                            filename: path
                                .file_name()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            timestamp: metadata.timestamp.to_rfc3339(),
                            parent_entry: metadata.parent_entry,
                            author: metadata.author,
                            content_preview: preview,
                        });
                    }
                }
            }
        }

        Ok(result)
    })
}

#[tauri::command]
fn get_state_files(repo_path: String) -> Result<Vec<StateFileInfo>, String> {
    with_repo_dir(&repo_path, || {
        let files = gitehr::commands::state::list_state_files().map_err(|e| e.to_string())?;

        Ok(files
            .into_iter()
            .map(|f| StateFileInfo {
                name: f.name,
                content: f.content,
                last_modified: f.last_modified,
            })
            .collect())
    })
}

#[tauri::command]
fn get_state_file(repo_path: String, filename: String) -> Result<StateFileInfo, String> {
    with_repo_dir(&repo_path, || {
        let file =
            gitehr::commands::state::view_state_file(&filename).map_err(|e| e.to_string())?;

        Ok(StateFileInfo {
            name: file.name,
            content: file.content,
            last_modified: file.last_modified,
        })
    })
}

#[tauri::command]
fn update_state_file(repo_path: String, filename: String, content: String) -> Result<(), String> {
    with_repo_dir(&repo_path, || {
        gitehr::commands::state::update_state_file(&filename, &content).map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn add_journal_entry(repo_path: String, content: String) -> Result<String, String> {
    with_repo_dir(&repo_path, || {
        let latest =
            gitehr::commands::journal::get_latest_journal_entry().map_err(|e| e.to_string())?;
        let parent_hash = latest.map(|(_, hash)| hash);

        gitehr::commands::journal::create_journal_entry(&content, parent_hash)
            .map_err(|e| e.to_string())?;

        Ok("Journal entry created".to_string())
    })
}

#[tauri::command]
fn verify_journal(repo_path: String) -> Result<String, String> {
    with_repo_dir(&repo_path, || {
        gitehr::commands::verify::verify_journal()
            .map(|_| "Journal verification successful".to_string())
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn get_contributors(repo_path: String) -> Result<Vec<ContributorInfo>, String> {
    with_repo_dir(&repo_path, || {
        let config_path = PathBuf::from(".gitehr/contributors.json");
        if !config_path.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let config: gitehr::commands::contributor::ContributorConfig =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        Ok(config
            .contributors
            .into_values()
            .map(|c| ContributorInfo {
                id: c.id,
                name: c.name,
                role: c.role,
                email: c.email,
                enabled: c.enabled,
                active: c.active,
            })
            .collect())
    })
}

#[tauri::command]
fn get_current_contributor(repo_path: String) -> Result<Option<String>, String> {
    with_repo_dir(&repo_path, || {
        Ok(gitehr::commands::contributor::get_current_contributor())
    })
}

#[tauri::command]
fn activate_contributor(repo_path: String, contributor_id: String) -> Result<(), String> {
    with_repo_dir(&repo_path, || {
        gitehr::commands::contributor::activate_contributor(&contributor_id)
            .map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn init_repo(path: String) -> Result<String, String> {
    let output = std::process::Command::new("gitehr")
        .arg("init")
        .current_dir(&path)
        .output()
        .map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                "GitEHR CLI not found. Install gitehr or ensure it is in PATH.".to_string()
            } else {
                format!("Failed to execute gitehr binary: {}", e)
            }
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    const APP_ICON: &[u8] = include_bytes!("../icons/icon.png");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_current_dir,
            is_gitehr_repo,
            get_status,
            get_journal_entries,
            get_state_files,
            get_state_file,
            update_state_file,
            add_journal_entry,
            verify_journal,
            get_contributors,
            get_current_contributor,
            activate_contributor,
            init_repo,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                let icon = tauri::image::Image::from_bytes(APP_ICON)?;
                window.set_icon(icon)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
