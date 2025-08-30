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
pub async fn list_available_library() -> Result<Vec<String>, LibraryError> {
    // For now, we'll prompt the user to select where to create the library
    // This could be enhanced to create a default location
    info!("Trying to fetch list of libraries");
    let library_list = Library::list_libraries()?;
    info!("Library List is {library_list:#?}");
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
        let mut l: Library = Library::new().switch_or_create_lib(&library_path)?;
        library_lock.share_path = l.share_path.take();
        library_lock.library_config = l.library_config.take();
    }
    Ok(path)
}

#[command]
pub async fn create_new_library(
    name: String,
    path: String,
    library: tauri::State<'_, Mutex<Library>>,
) -> Result<String, LibraryError> {
    let share_path = Library::create_or_validate_data_directory()?
        .join("hestia")
        .join(&name);
    let path = PathBuf::from(&path);

    if !path.exists() || !path.is_dir() {
        return Err(LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            "The library path is not valid!".to_string(),
        ));
    }

    // Create a new library config
    let new_config = LibraryConfig {
        library_paths: vec![LibraryPathConfig {
            name: Some(name.clone()),
            path: Some(path.clone()),
        }],
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
