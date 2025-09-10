#![allow(dead_code)]
use async_recursion::async_recursion;
use sea_orm::{ConnectionTrait, DatabaseConnection, PaginatorTrait};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{command, State};

use tracing::info;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};

use super::file_operations::FileInfo;
use crate::data::folder_info::FolderInfo;
use crate::data::watched_folders::WatchedFolderTree;
use crate::{config::app::AppState, errors::DbError};

use entity::{files, folders, prelude::*};

/// Get all folder and subfolders being watched for the frontend to display
#[tauri::command]
pub async fn get_watched_folders(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<HashMap<String, WatchedFolderTree>, DbError> {
    info!("Getting watched folders");
    {
        let state = app_state.lock().await;
        state.get_watched_folders_map().await
    }
}

/// Folder content summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSummary {
    pub folder: FolderInfo,
    pub total_files: u64,
    pub total_subfolders: u64,
    pub total_size: u64,
    pub file_types: Vec<String>,
}

/// Get all folders
#[command]
pub async fn get_all_folders(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    match folders::Entity::find().all(&*connection).await {
        Ok(folders) => Ok(folders.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to get folders: {}", e)),
    }
}

/// Get folder by ID
#[command]
pub async fn get_folder_by_id(
    app_state: State<'_, Mutex<AppState>>,
    folder_id: i32,
) -> Result<Option<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    match folders::Entity::find_by_id(folder_id)
        .one(&*connection)
        .await
    {
        Ok(Some(folder)) => Ok(Some(folder.into())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get folder: {}", e)),
    }
}

/// Get folder by path
#[command]
pub async fn get_folder_by_path(
    app_state: State<'_, Mutex<AppState>>,
    folder_path: String,
) -> Result<Option<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    match folders::Entity::find()
        .filter(folders::Column::Path.eq(&folder_path))
        .one(&*connection)
        .await
    {
        Ok(Some(folder)) => Ok(Some(folder.into())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get folder by path: {}", e)),
    }
}

/// Get root folders (folders with no parent)
#[command]
pub async fn get_root_folders(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    match folders::Entity::find()
        .filter(folders::Column::ParentFolderId.is_null())
        .all(&*connection)
        .await
    {
        Ok(folders) => Ok(folders.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to get root folders: {}", e)),
    }
}

/// Get subfolders of a folder
#[command]
pub async fn get_subfolders(
    app_state: State<'_, Mutex<AppState>>,
    parent_folder_id: i32,
) -> Result<Vec<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    match folders::Entity::find()
        .filter(folders::Column::ParentFolderId.eq(parent_folder_id))
        .all(&*connection)
        .await
    {
        Ok(folders) => Ok(folders.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to get subfolders: {}", e)),
    }
}

/// Get files in a folder
#[command]
pub async fn get_files_in_folder(
    app_state: State<'_, Mutex<AppState>>,
    folder_path: String,
) -> Result<Vec<FileInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    // Get files that are directly in this folder path
    let pattern = format!("{}%", folder_path);
    match files::Entity::find()
        .filter(files::Column::Path.like(&pattern))
        .all(&*connection)
        .await
    {
        Ok(files) => {
            // Filter to only include direct children (not nested files)
            let folder_path_buf = PathBuf::from(&folder_path);
            let direct_files: Vec<FileInfo> = files
                .into_iter()
                .filter(|file| {
                    let file_path = PathBuf::from(&file.path);
                    if let Some(parent) = file_path.parent() {
                        parent == folder_path_buf
                    } else {
                        false
                    }
                })
                .map(|f| f.into())
                .collect();
            Ok(direct_files)
        }
        Err(e) => Err(format!("Failed to get files in folder: {}", e)),
    }
}

/// Get folder summary with statistics
#[command]
pub async fn get_folder_summary(
    app_state: State<'_, Mutex<AppState>>,
    folder_path: String,
) -> Result<FolderSummary, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    // Get folder info
    let folder = match folders::Entity::find()
        .filter(folders::Column::Path.eq(&folder_path))
        .one(&*connection)
        .await
    {
        Ok(Some(folder)) => folder,
        Ok(None) => return Err("Folder not found".to_string()),
        Err(e) => return Err(format!("Failed to get folder: {}", e)),
    };

    // Count files in folder and subfolders
    let pattern = format!("{}%", folder_path);
    let total_files = match files::Entity::find()
        .filter(files::Column::Path.like(&pattern))
        .count(&*connection)
        .await
    {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count files: {}", e)),
    };

    // Count subfolders
    let total_subfolders = match folders::Entity::find()
        .filter(folders::Column::Path.like(&pattern))
        .filter(folders::Column::Id.ne(folder.id))
        .count(&*connection)
        .await
    {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count subfolders: {}", e)),
    };

    // Get file types in this folder
    let files_in_folder = match files::Entity::find()
        .filter(files::Column::Path.like(&pattern))
        .all(&*connection)
        .await
    {
        Ok(files) => files,
        Err(e) => return Err(format!("Failed to get files for types: {}", e)),
    };

    // Calculate total size and collect file types
    let mut total_size = 0u64;
    let mut file_types = std::collections::HashSet::new();

    for file in files_in_folder {
        // Try to get file size from filesystem
        if let Ok(metadata) = std::fs::metadata(&file.path) {
            total_size += metadata.len();
        }

        // Get file type
        let file_path = PathBuf::from(&file.path);
        if let Some(extension) = file_path.extension() {
            if let Some(ext_str) = extension.to_str() {
                file_types.insert(ext_str.to_lowercase());
            }
        }
    }

    let file_types: Vec<String> = file_types.into_iter().collect();

    Ok(FolderSummary {
        folder: folder.into(),
        total_files,
        total_subfolders,
        total_size,
        file_types,
    })
}

/// Search folders by name pattern
#[command]
pub async fn search_folders_by_name(
    app_state: State<'_, Mutex<AppState>>,
    search_pattern: String,
) -> Result<Vec<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    let pattern = format!("%{}%", search_pattern);

    match folders::Entity::find()
        .filter(folders::Column::Name.like(&pattern))
        .all(&*connection)
        .await
    {
        Ok(folders) => Ok(folders.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to search folders: {}", e)),
    }
}

/// Get folder path hierarchy (breadcrumb)
#[command]
pub async fn get_folder_path_hierarchy(
    app_state: State<'_, Mutex<AppState>>,
    folder_id: i32,
) -> Result<Vec<FolderInfo>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    let mut hierarchy = Vec::new();
    let mut current_folder_id = Some(folder_id);

    // Traverse up the folder hierarchy
    while let Some(folder_id) = current_folder_id {
        match folders::Entity::find_by_id(folder_id)
            .one(&*connection)
            .await
        {
            Ok(Some(folder)) => {
                current_folder_id = folder.parent_folder_id;
                hierarchy.push(folder.into());
            }
            Ok(None) => break,
            Err(e) => return Err(format!("Failed to get folder in hierarchy: {}", e)),
        }
    }

    // Reverse to get path from root to target folder
    hierarchy.reverse();
    Ok(hierarchy)
}

/// Delete empty folders
#[command]
pub async fn delete_empty_folders(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<Vec<String>, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    // Get all folders
    let all_folders = match folders::Entity::find().all(&*connection).await {
        Ok(folders) => folders,
        Err(e) => return Err(format!("Failed to get folders: {}", e)),
    };

    let mut deleted_folders = Vec::new();

    for folder in all_folders {
        // Check if folder has any files
        let file_count = match files::Entity::find()
            .filter(files::Column::Path.like(&format!("{}%", folder.path)))
            .count(&*connection)
            .await
        {
            Ok(count) => count,
            Err(_) => continue,
        };

        // Check if folder has any subfolders
        let subfolder_count = match folders::Entity::find()
            .filter(folders::Column::ParentFolderId.eq(folder.id))
            .count(&*connection)
            .await
        {
            Ok(count) => count,
            Err(_) => continue,
        };

        // If folder is empty, delete it
        if file_count == 0 && subfolder_count == 0 {
            match folders::Entity::delete_by_id(folder.id)
                .exec(&*connection)
                .await
            {
                Ok(_) => deleted_folders.push(folder.path),
                Err(_) => continue,
            }
        }
    }

    Ok(deleted_folders)
}

/// Get folder statistics
#[command]
pub async fn get_folder_statistics(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<serde_json::Value, String> {
    let connection = {
        let state = app_state.lock().await;
        state.database_manager.get_connection()
    };

    let total_folders = match folders::Entity::find().count(&*connection).await {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count folders: {}", e)),
    };

    let root_folders = match folders::Entity::find()
        .filter(folders::Column::ParentFolderId.is_null())
        .count(&*connection)
        .await
    {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count root folders: {}", e)),
    };

    let mut result = serde_json::Map::new();
    result.insert(
        "total_folders".to_string(),
        serde_json::Value::Number(total_folders.into()),
    );
    result.insert(
        "root_folders".to_string(),
        serde_json::Value::Number(root_folders.into()),
    );
    result.insert(
        "nested_folders".to_string(),
        serde_json::Value::Number((total_folders - root_folders).into()),
    );

    Ok(serde_json::Value::Object(result))
}
