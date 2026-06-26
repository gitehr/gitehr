use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntryInfo {
    pub filename: String,
    pub timestamp: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiIdentifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiPatientInfo {
    pub patient_id: String,
    pub repo_path: String,
    pub status: String,
    pub merged_into: Option<String>,
    pub updated_at: String,
    pub identifiers: Vec<MpiIdentifier>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MpiInfo {
    pub version: u32,
    pub updated_at: String,
    pub patients: Vec<MpiPatientInfo>,
    #[serde(default)]
    pub store_root: String,
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

fn run_gitehr_in(path: &str, args: &[&str]) -> Result<String, String> {
    let cli = resolve_gitehr_cli();
    let output = Command::new(&cli)
        .args(args)
        .current_dir(path)
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
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("unrecognized subcommand 'store'") {
            return Err(format!(
                "The gitehr CLI at '{}' is too old and does not support `gitehr store`. \
                 Rebuild this checkout (`cargo build -p gitehr`) or put a current gitehr on PATH.",
                cli.display()
            ));
        }
        if stderr.is_empty() {
            Err(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(stderr)
        }
    }
}

fn resolve_gitehr_cli() -> PathBuf {
    if let Some(path) = std::env::var_os("GITEHR_CLI").map(PathBuf::from) {
        if path.exists() {
            return path;
        }
    }

    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let sibling = dir.join(exe_name("gitehr"));
            if sibling.exists() {
                return sibling;
            }
        }
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent);
    if let Some(repo_root) = repo_root {
        for profile in ["debug", "release"] {
            let candidate = repo_root.join("target").join(profile).join(exe_name("gitehr"));
            if candidate.exists() {
                return candidate;
            }
        }
    }

    PathBuf::from(exe_name("gitehr"))
}

fn exe_name(name: &str) -> String {
    #[cfg(windows)]
    {
        format!("{name}.exe")
    }
    #[cfg(not(windows))]
    {
        name.to_string()
    }
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
fn has_mpi(path: String) -> bool {
    PathBuf::from(&path).join("gitehr-mpi.json").exists()
}

#[tauri::command]
fn get_mpi(path: String) -> Result<MpiInfo, String> {
    let mpi_path = PathBuf::from(&path).join("gitehr-mpi.json");
    if !mpi_path.exists() {
        return Err("MPI not found in selected store root".to_string());
    }

    let content = std::fs::read_to_string(&mpi_path).map_err(|e| e.to_string())?;
    let mut mpi: MpiInfo = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    for patient in mpi.patients.iter_mut() {
        let abs = PathBuf::from(&path).join(&patient.repo_path);
        patient.repo_path = abs.to_string_lossy().to_string();
    }
    mpi.store_root = path;
    Ok(mpi)
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
        let mut entries =
            gitehr::commands::journal::parsed_entries().map_err(|e| e.to_string())?;

        if reverse.unwrap_or(false) {
            entries.reverse();
        }

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(50);

        Ok(entries
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|entry| JournalEntryInfo {
                filename: entry.filename,
                timestamp: entry.metadata.timestamp.to_rfc3339(),
                author: entry.metadata.author,
                content_preview: entry
                    .content
                    .chars()
                    .take(200)
                    .collect::<String>()
                    .replace('\n', " "),
            })
            .collect())
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
        gitehr::commands::journal::create_journal_entry(&content).map_err(|e| e.to_string())?;

        Ok("Journal entry created".to_string())
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
fn init_store_root(path: String, name: Option<String>) -> Result<String, String> {
    let trimmed = name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let args = match trimmed {
        Some(name) => vec!["store", "init", name],
        None => vec!["store", "init"],
    };

    run_gitehr_in(&path, &args)
}

#[tauri::command]
fn add_store_subject(path: String, name: Option<String>) -> Result<String, String> {
    let trimmed = name.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let args = match trimmed {
        Some(name) => vec!["store", "add", name],
        None => vec!["store", "add"],
    };

    run_gitehr_in(&path, &args)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    const APP_ICON: &[u8] = include_bytes!("../icons/icon.png");

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init());

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            get_current_dir,
            is_gitehr_repo,
            has_mpi,
            get_mpi,
            get_status,
            get_journal_entries,
            get_state_files,
            get_state_file,
            update_state_file,
            add_journal_entry,
            get_contributors,
            get_current_contributor,
            activate_contributor,
            init_store_root,
            add_store_subject,
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
