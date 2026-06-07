use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("A save named \"{0}\" already exists")]
    DuplicateName(String),
    #[error("Save \"{0}\" not found")]
    SaveNotFound(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    pub name: String,
    pub path: String,
    pub repo_path: String,
    pub remote_repo_path: String,
}

struct SaveState {
    saves: Mutex<Vec<Save>>,
    data_dir: PathBuf,
}

fn saves_file_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join("saves.json")
}

fn load_saves(data_dir: &PathBuf) -> Result<Vec<Save>, AppError> {
    let path = saves_file_path(data_dir);
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
        Ok(serde_json::from_str(&content).unwrap_or_else(|_| vec![]))
    } else {
        // Ensure the data directory exists
        fs::create_dir_all(data_dir)?;
        Ok(vec![])
    }
}

fn save_saves(data_dir: &PathBuf, saves: &[Save]) -> Result<(), AppError> {
    let path = saves_file_path(data_dir);
    fs::create_dir_all(data_dir)?;
    let content = serde_json::to_string_pretty(saves)?;
    fs::write(&path, content)?;
    Ok(())
}

#[tauri::command]
fn list_saves(state: tauri::State<SaveState>) -> Vec<Save> {
    state.saves.lock().unwrap().clone()
}

#[tauri::command]
fn add_save(
    state: tauri::State<SaveState>,
    name: String,
    path: String,
    repo_path: String,
    remote_repo_path: String,
) -> Result<Save, AppError> {
    let mut saves = state.saves.lock().unwrap();

    // Check for duplicate name
    if saves.iter().any(|s| s.name == name) {
        return Err(AppError::DuplicateName(name));
    }

    let save = Save {
        name,
        path,
        repo_path,
        remote_repo_path,
    };
    saves.push(save.clone());
    save_saves(&state.data_dir, &saves)?;
    Ok(save)
}

#[tauri::command]
fn delete_save(state: tauri::State<SaveState>, name: String) -> Result<(), AppError> {
    let mut saves = state.saves.lock().unwrap();
    let len_before = saves.len();
    saves.retain(|s| s.name != name);
    if saves.len() == len_before {
        return Err(AppError::SaveNotFound(name));
    }
    save_saves(&state.data_dir, &saves)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");

            let saves = load_saves(&data_dir)?;

            app.manage(SaveState {
                saves: Mutex::new(saves),
                data_dir,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_saves, add_save, delete_save])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
