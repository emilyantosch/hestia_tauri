use crate::config::library::{LibraryConfig, LibraryPathConfig};
use crate::errors::{AppError, AppErrorKind, FileError, FileErrorKind};
use tokio::sync::Notify;

#[tokio::test]
async fn check_delete_library() -> Result<(), FileError> {
    use crate::config::library::Library;
    use std::time::Instant;
    use tracing::{error, info};

    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new().switch_or_create_lib(&path).await?;
    info!("Found or created library to be deleted!");

    //Assert that path exists
    assert!(
        tokio::fs::try_exists(&path).await?,
        "Library Path should exist before deletion"
    );

    //Delete Library
    if let Err(e) = lib.delete().await {
        error!("The lib could not get deleted due to {e:#?}");
    }
    let now = Instant::now();
    let timeout = std::time::Duration::from_secs(5);

    loop {
        match tokio::fs::try_exists(&path).await {
            Ok(false) => {
                info!("Library at {path:#?} deleted!");
                break;
            }
            Ok(true) => {
                if now.elapsed() > timeout {
                    return Err(FileError::new(
                        FileErrorKind::Io, 
                        format!("Path still exists, deletion timed out while trying to delete library at {path:#?}"), 
                        Some(vec![path])));
                }
                tokio::task::yield_now().await;
            }
            Err(e) => {
                return Err(FileError::with_source(
                    FileErrorKind::Io,
                    "Problem while trying to check for path".to_string(),
                    e,
                    None,
                ))
            }
        }
    }

    let is_deleted = tokio::fs::try_exists(path).await;
    assert!(is_deleted.is_ok_and(|v| !v));
    Ok(())
}

#[tokio::test]
async fn check_library_creation_successful() {
    use crate::config::library::Library;

    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new().switch_or_create_lib(&path).await.unwrap();
    let lib_config = lib.library_config.clone();
    let lib_path = lib_config.lock().await;
    let lib_paths = &lib_path.as_ref().unwrap().library_paths;
    assert!(lib_paths.is_some());
    let _ = lib.delete().await;
}

#[tokio::test]
async fn check_library_default_values() {
    use crate::config::library::Library;

    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new().switch_or_create_lib(&path).await.unwrap();
    {
        let lib_lock = lib.library_config.lock().await;
        let lib_config = lib_lock;
        assert_eq!(
            &lib_config.as_ref(),
            &Some(LibraryConfig::default()).as_ref()
        );
    }

    let _ = lib.delete().await;
}

#[tokio::test]
async fn check_library_write_save_and_retrieve() -> Result<(), FileError> {
    use crate::config::library::Library;

    let path = dirs::data_dir()
        .ok_or_else(|| {
            FileError::new(
                FileErrorKind::PathNotFoundError,
                "Could not find data directory".to_string(),
                None,
            )
        })?
        .join("hestia")
        .join("test_lib");
    let lib = Library::new().switch_or_create_lib(&path).await.unwrap();
    let lbc = vec![LibraryPathConfig::default(), LibraryPathConfig::default()];
    {
        let mut lib_lock = lib.library_config.lock().await;
        match lib_lock.as_mut() {
            Some(conf) => conf.library_paths.as_mut().unwrap().push(lbc[0].clone()),
            None => panic!("There is no config!"),
        };
    }
    {
        let mut lib_lock = lib.library_config.lock().await;
        match lib_lock.as_mut() {
            Some(conf) => conf.library_paths.as_mut().unwrap().push(lbc[1].clone()),
            None => panic!("There is no config!"),
        };
    }
    let _ = lib.save_config().await;
    let test_lib = Library::new().switch_or_create_lib(&path).await.unwrap();
    {
        let lib_lock = test_lib.library_config.lock().await;
        match lib_lock.as_ref() {
            Some(conf) => {
                assert_eq!(conf.library_paths.as_ref().unwrap(), &lbc)
            }
            None => panic!("There is no config!"),
        };
    }
    Ok(())
}

#[tokio::test]
async fn check_library_more_values_in_memory() {
    ()
}
