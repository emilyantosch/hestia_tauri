#![allow(dead_code)]

use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tracing::info;

use crate::config::app::AppState;
use crate::config::library::{Library, LibraryConfig, LibraryPathConfig};
use crate::data::commands::watched_folders::WatchedFolderTree;
use crate::errors::{LibraryError, LibraryErrorKind};
use crate::file_system::FileWatcherMessage;
use crate::utils;
use tauri::State;

#[tauri::command]
pub async fn get_library_paths(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<Vec<LibraryPathConfig>, &str> {
    let state = app_state.lock().await;
    match state.library.library_config.as_ref() {
        Some(conf) => Ok(conf.library_paths.clone()),
        None => Err("There is no config defined!"),
    }
}

///This IPC endpoint lists all libraries in the datahome directory
#[tauri::command]
pub async fn list_available_library() -> Result<Vec<String>, LibraryError> {
    info!("Trying to fetch list of libraries");
    let library_list = Library::list_libraries()?;
    info!("Library List is {library_list:#?}");
    Ok(library_list)
}

#[tauri::command]
pub async fn select_library(
    path: String,
    app_state: State<'_, Mutex<AppState>>,
) -> Result<String, LibraryError> {
    let library_path = PathBuf::from(&path);

    if !library_path.exists() || !library_path.is_dir() {
        return Err(LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            format!("The path {library_path:#?} is invalid!"),
        ));
    }

    // Create the new library and switch to it
    let new_library = Library::new().switch_or_create_lib(&library_path)?;

    {
        let mut state = app_state.lock().await;
        state.switch_library(new_library).await?;
    }

    Ok(path)
}

///This IPC endpoint creates a new library in the data directory with at least one path config and
///a name
#[tauri::command]
pub async fn create_new_library(
    app_state: State<'_, Mutex<AppState>>,
    name: String,
    path: String,
) -> Result<String, LibraryError> {
    //Extract the share path from the name of the library
    info!("Trying to create new library!");
    let share_path = Library::create_or_validate_data_directory()?
        .join("hestia")
        .join(&name);

    match std::fs::exists(&share_path) {
        Ok(true) => {
            return Err(LibraryError::new(
                LibraryErrorKind::InvalidSharePath,
                "The share path already exists, creation aborted".to_string(),
            ))
        }
        Ok(false) => (),
        Err(e) => {
            return Err(LibraryError::with_source(
                LibraryErrorKind::InvalidSharePath,
                "Existance of the share path could not be verified".to_string(),
                Some(Box::new(e)),
            ))
        }
    }
    //Parse path from string
    let path = PathBuf::from(&path);
    if !path.exists() || !path.is_dir() {
        return Err(LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            "The library path is not valid!".to_string(),
        ));
    }
    let file_name = path
        .file_name()
        .unwrap_or(OsStr::new("Folder"))
        .to_string_lossy()
        .to_string();

    // Create a new library config
    let new_config = LibraryConfig {
        library_paths: vec![LibraryPathConfig {
            name: Some(file_name.clone()),
            path: Some(path.clone()),
        }],
        name: name.clone(),
        color: utils::decorations::Color::default(),
        icon: utils::decorations::Icon::default(),
    };

    // Create the new library and switch to it using AppState
    info!("Creating new library: {name:#?} at path: {path:#?}");
    let mut new_library = Library::new();
    new_library.share_path = Some(share_path.clone());
    new_library.library_config = Some(new_config.clone());
    new_library.save_config()?;

    {
        let mut state = app_state.lock().await;
        state.switch_library(new_library).await?;
    }

    info!("Successfully created and switched to new library");
    Ok(share_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn initialize_library_workspace(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<(), LibraryError> {
    info!("Initializing library workspace");

    {
        let mut state = app_state.lock().await;

        info!("Running migrations...");
        // Run database migrations
        state.run_migrations().await?;

        info!("Trying to upsert root folders!");
        // Upsert root folders
        state.upsert_root_folders().await?;

        info!("Scanning library directories");
        // Perform initial directory scan
        state.scan_library_directories().await?;

        //Create FileWatcher, set handle to state
        state.create_file_watcher().await?;

        match state.file_watcher_handler.as_ref() {
            Some(handler) => {
                if let Some(paths_config) = state.library.library_config.as_ref() {
                    for path_config in paths_config.library_paths.clone() {
                        if let Some(single_path) = path_config.path {
                            let _ = handler
                                .sender
                                .send(FileWatcherMessage::WatchPath(single_path.into()));
                        }
                    }
                }
            }
            None => {
                return Err(LibraryError::new(
                    LibraryErrorKind::ConfigCreationError,
                    "The file watcher has not been created correctly".to_string(),
                ))
            }
        }
    }

    info!("Library workspace initialization completed successfully");
    Ok(())
}
