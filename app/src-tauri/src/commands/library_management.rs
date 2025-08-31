use std::path::PathBuf;
use tokio::sync::Mutex;
use tracing::info;

use crate::errors::{LibraryError, LibraryErrorKind};


use crate::config::library::{Library, LibraryConfig, LibraryPathConfig};

#[tauri::command]
pub async fn get_library_paths(
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<Vec<LibraryPathConfig>, &str> {
    match library.lock().await.library_config.as_ref() {
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
        let mut l: Library = Library::new().switch_or_create_lib(&library_path)?;
        library_lock.share_path = l.share_path.take();
        library_lock.library_config = l.library_config.take();
    }
    Ok(path)
}

///This IPC endpoint creates a new library in the data directory with at least one path config and
///a name
#[tauri::command]
pub async fn create_new_library(
    name: String,
    path: String,
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<String, LibraryError> {
    //Extract the share path from the name of the library
    let share_path = Library::create_or_validate_data_directory()?
        .join("hestia")
        .join(&name);
    //TODO: Check whether the share_path already exists. If so, abort creation.
    match std::fs::exists(share_path) {
    Ok(true) => {
        return Err(LibraryError::new(
                LibraryErrorKind::InvalidSharePath,
                "The share path already exists, creation aborted".to_string(),
            ))
        },
    Ok(false) => (),
    Err(e) => {
            return Err(LibraryError::with_source(LibraryErrorKind::InvalidSharePath, "Existance of the share path could not be verified".to_string(), Some(Box::new(e))))
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
    let file_name = path.file_name().unwrap_or("Folder").to_string_lossy().to_string();


    // Create a new library config
    let new_config = LibraryConfig {
        library_paths: vec![LibraryPathConfig {
            name: Some(file_name.clone()),
            path: Some(path.clone()),
        }],
        name: name.clone(),
        color: 
        icon: 
    };

    // Update the library state (you may need to adjust this based on your state management)
    // For now, this is a placeholder - you'll need to implement proper state persistence
    info!("Trying to set new library to name: {name:#?} and path: {path:#?}");
    {
        let mut l = library.lock().await;
        l.share_path = Some(share_path.clone());
        l.library_config = Some(new_config.clone());
        l.save_config()?;
    }
    info!("Library set to {library:#?}");
    Ok(share_path.to_string_lossy().to_string())
}
