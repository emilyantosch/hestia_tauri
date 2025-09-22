use std::path::PathBuf;

use crate::config::library::{LibraryConfig, LibraryPathConfig};
use crate::errors::{
    AppError, AppErrorKind, FileError, FileErrorKind, LibraryError, LibraryErrorKind,
};
use tempfile::TempDir;
use tokio::sync::Notify;
use tracing::info;

#[test]
fn check_delete_library() -> Result<(), LibraryError> {
    use crate::config::library::Library;
    use tempfile::TempDir;
    use tracing::{error, info};

    let path = dirs::data_dir().ok_or_else(|| {
        LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            "Could not find data directory".to_string(),
        )
    })?;
    let test_path = TempDir::new_in(path)?;
    let lib = Library::new().switch_or_create_lib(&test_path.path().to_path_buf())?;
    info!("Found or created library to be deleted!");

    //Assert that path exists
    assert!(
        std::fs::exists(&test_path)?,
        "Library Path should exist before deletion"
    );

    //Delete Library
    if let Err(e) = lib.delete() {
        error!("The lib could not get deleted due to {e:#?}");
    }

    let still_exists = std::fs::exists(test_path)?;
    assert!(!still_exists);
    Ok(())
}

#[test]
fn check_library_creation_successful() -> Result<(), LibraryError> {
    use crate::config::library::Library;

    let path = dirs::data_dir().ok_or_else(|| {
        LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            "Could not find data directory".to_string(),
        )
    })?;
    let test_path = TempDir::new_in(path)?;
    let lib = Library::new().switch_or_create_lib(&test_path.path().to_path_buf())?;
    match lib.library_config.as_ref() {
        Some(conf) => {
            assert_eq!(conf.library_paths, vec![LibraryPathConfig::default()]);
        }
        None => {
            return Err(LibraryError::new(
                LibraryErrorKind::ConfigCreationError,
                "Initialization of config failed!".to_string(),
            ))
        }
    }
    lib.delete()?;
    Ok(())
}

#[test]
fn check_library_default_values() -> Result<(), LibraryError> {
    use crate::config::library::Library;
    use tempfile::TempDir;

    let path = dirs::data_dir().ok_or_else(|| {
        LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            "Could not find data directory".to_string(),
        )
    })?;
    let test_path = TempDir::new_in(path)?;
    let lib = Library::new().switch_or_create_lib(&test_path.path().to_path_buf())?;
    if let Some(lib_config) = lib.library_config.as_ref() {
        assert_eq!(lib_config, &LibraryConfig::default());
    }
    lib.delete()?;
    Ok(())
}

#[test]
fn check_library_write_save_and_retrieve() -> Result<(), LibraryError> {
    use crate::config::library::Library;
    println!("Start of test");
    info!("Start of test");
    let path = dirs::data_dir().ok_or_else(|| {
        LibraryError::new(
            LibraryErrorKind::InvalidSharePath,
            "Could not find data directory".to_string(),
        )
    })?;
    let test_path = TempDir::new_in(path)?;
    info!("Data home configured");
    let mut lib = Library::new().switch_or_create_lib(&test_path.path().to_path_buf())?;
    let lbc = vec![
        LibraryPathConfig::default(),
        LibraryPathConfig {
            name: Some("Hello".to_string()),
            path: Some(PathBuf::new().join("home/emmi/Documents/")),
        },
    ];
    if let Some(lib_config) = lib.library_config.as_mut() {
        lib_config.library_paths.push(lbc[1].clone());
    }
    info!("Pushed lib paths, config is now: {lib:#?}");
    println!("Pushed lib paths, config is now: {lib:#?}");
    lib.save_config()?;
    let test_lib = Library::new().switch_or_create_lib(&test_path.path().to_path_buf())?;
    if let Some(lib_config) = test_lib.library_config.as_ref() {
        assert_eq!(lib_config.library_paths, lbc);
    }
    lib.delete()?;
    Ok(())
}
