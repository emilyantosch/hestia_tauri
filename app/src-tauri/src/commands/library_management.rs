use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::errors::{LibraryError, LibraryErrorKind};

use rfd::AsyncFileDialog;
use tauri::{command, AppHandle, State};

use crate::config::library::{Library, LibraryConfig, LibraryPathConfig};

#[command]
pub async fn get_library_paths(
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<Vec<LibraryPathConfig>, &str> {
    match library.lock().await.library_config.as_ref() {
        Some(conf) => Ok(conf.library_paths.clone()),
        None => Err("There is no config defined!"),
    }
}

#[command]
pub async fn list_available_library(
    name: String,
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<Vec<String>, LibraryError> {
    // For now, we'll prompt the user to select where to create the library
    // This could be enhanced to create a default location
    let library_list = Library::list_libraries()?;
    Ok(library_list)
}

#[command]
pub async fn select_library_folder() -> Result<Option<String>, String> {
    let folder = AsyncFileDialog::new()
        .set_title("Select library folder")
        .pick_folder()
        .await;

    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[command]
pub async fn select_library(
    path: String,
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<String, LibraryError> {
    let library_path = PathBuf::from(&path);

    if !library_path.exists() || !library_path.is_dir() {
        return Err(LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            format!("The path {library_path:#?} is invalid!"),
        ));
    }
    {
        let mut library_lock = library.lock().await;
        let l = Library::new().switch_or_create_lib(&library_path)?;
        library_lock.share_path = l.share_path;
        library_lock.library_config = l.library_config;
    }
    Ok(path)
}

#[command]
pub async fn create_new_library(
    path: String,
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<String, LibraryError> {
    let library_path = PathBuf::from(&path);

    if !library_path.exists() || !library_path.is_dir() {
        return Err(LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            format!("The path {library_path:#?} is invalid!"),
        ));
    }

    // Extract folder name for the library name
    let library_name = library_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Create a new library config
    let new_config = LibraryConfig {
        library_paths: vec![LibraryPathConfig {
            name: Some(library_name.clone()),
            path: Some(library_path.clone()),
        }],
    };

    // Update the library state (you may need to adjust this based on your state management)
    // For now, this is a placeholder - you'll need to implement proper state persistence
    info!("Trying to set new library to name: {library_name:#?} and path: {library_path:#?}");
    {
        let mut l = library.lock().await;
        l.share_path = Some(library_path);
        l.library_config = Some(new_config.clone());
        l.save_config()?;
    }
    info!("Library set to {library:#?}");

    Ok(path)
}
