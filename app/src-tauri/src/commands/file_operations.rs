#![allow(dead_code)]
use std::path::PathBuf;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{State};

use crate::config::app::AppState;
use entity::files;

/// Response for file scanning operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub files_scanned: u64,
    pub files_inserted: u64,
    pub files_updated: u64,
    pub files_deleted: u64,
    pub duration: u64, // milliseconds
    pub errors: Vec<String>,
}

impl ScanReport {
    pub fn total_operations(&self) -> u64 {
        self.files_inserted + self.files_updated + self.files_deleted
    }
}

/// File information for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_type_id: i32,
    pub file_system_id: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<files::Model> for FileInfo {
    fn from(file: files::Model) -> Self {
        Self {
            id: file.id,
            name: file.name,
            path: file.path,
            content_hash: file.content_hash,
            identity_hash: file.identity_hash,
            file_type_id: file.file_type_id,
            file_system_id: file.file_system_id,
            created_at: file.created_at.to_string(),
            updated_at: file.updated_at.to_string(),
        }
    }
}
/// Scan a directory and sync files to database
#[tauri::comand]
pub async fn scan_directory(
    app_state: State<'_, Mutex<AppState>>,
    directory_path: String,
) -> Result<ScanReport, String> {
    let path = PathBuf::from(directory_path);

    if !path.exists() {
        return Err("Directory does not exist".to_string());
    }

    if !path.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let start_time = std::time::Instant::now();

    let result = {
        let state = app_state.lock().await;
        state.directory_scanner.sync_directory(&path).await
    };

    match result {
        Ok(report) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            Ok(ScanReport {
                files_scanned: report.files_scanned as u64,
                files_inserted: report.files_inserted as u64,
                files_updated: report.files_updated as u64,
                files_deleted: report.files_deleted as u64,
                duration: duration_ms,
                errors: report.errors,
            })
        }
        Err(e) => Err(format!("Directory scan failed: {e:#?}")),
    }
}

/// Get file information by path
#[tauri::comand]
pub async fn get_file_by_path(
    app_state: State<'_, Mutex<AppState>>,
    file_path: String,
) -> Result<Option<FileInfo>, String> {
    let path = PathBuf::from(file_path);

    let result = {
        let state = app_state.lock().await;
        state.file_operations.get_file_by_path(&path).await
    };

    match result {
        Ok(Some(file)) => Ok(Some(file.into())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get file: {e:#?}")),
    }
}

/// Get all files in a directory
#[tauri::comand]
pub async fn get_files_in_directory(
    app_state: State<'_, Mutex<AppState>>,
    directory_path: String,
) -> Result<Vec<FileInfo>, String> {
    let path = PathBuf::from(directory_path);

    let result = {
        let state = app_state.lock().await;
        state.file_operations.get_files_in_directory(&path).await
    };

    match result {
        Ok(files) => Ok(files.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to get files in directory: {e:#?}")),
    }
}

/// Delete a file record from the database
#[tauri::comand]
pub async fn delete_file_by_path(
    app_state: State<'_, Mutex<AppState>>,
    file_path: String,
) -> Result<bool, String> {
    let path = PathBuf::from(file_path);

    let result = {
        let state = app_state.lock().await;
        state.file_operations.delete_file_by_path(&path).await
    };

    match result {
        Ok(deleted) => Ok(deleted),
        Err(e) => Err(format!("Failed to delete file: {e:#?}")),
    }
}

/// Get file metadata (without database operations)
#[tauri::comand]
pub async fn get_file_metadata(file_path: String) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(file_path);

    if !path.exists() {
        return Err("File does not exist".to_string());
    }

    match std::fs::metadata(&path) {
        Ok(metadata) => {
            let mut result = serde_json::Map::new();
            result.insert(
                "path".to_string(),
                serde_json::Value::String(path.to_string_lossy().to_string()),
            );
            result.insert(
                "size".to_string(),
                serde_json::Value::Number(metadata.len().into()),
            );
            result.insert(
                "is_file".to_string(),
                serde_json::Value::Bool(metadata.is_file()),
            );
            result.insert(
                "is_dir".to_string(),
                serde_json::Value::Bool(metadata.is_dir()),
            );
            result.insert(
                "readonly".to_string(),
                serde_json::Value::Bool(metadata.permissions().readonly()),
            );

            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    result.insert(
                        "modified_timestamp".to_string(),
                        serde_json::Value::Number(duration.as_secs().into()),
                    );
                }
            }

            if let Ok(created) = metadata.created() {
                if let Ok(duration) = created.duration_since(std::time::UNIX_EPOCH) {
                    result.insert(
                        "created_timestamp".to_string(),
                        serde_json::Value::Number(duration.as_secs().into()),
                    );
                }
            }

            Ok(serde_json::Value::Object(result))
        }
        Err(e) => Err(format!("Failed to get file metadata: {e:#?}")),
    }
}

/// Check if a file exists in the database
#[tauri::comand]
pub async fn file_exists_in_database(
    app_state: State<'_, Mutex<AppState>>,
    file_path: String,
) -> Result<bool, String> {
    let path = PathBuf::from(file_path);

    let result = {
        let state = app_state.lock().await;
        state.file_operations.get_file_by_path(&path).await
    };

    match result {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Failed to check file existence: {e:#?}")),
    }
}

#[tauri::comand]
