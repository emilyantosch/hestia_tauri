use crate::config::library::{LibraryConfig, LibraryPathConfig};
use crate::errors::{FileError, FileErrorKind};

#[tokio::test]
async fn check_delete_library() {
    use crate::config::library::Library;

    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new()
        .await
        .unwrap()
        .switch_or_create_lib(&path)
        .await
        .unwrap();

    let _ = lib.delete().await;
    assert!(tokio::fs::try_exists(path).await.is_ok_and(|v| !v));
}

#[tokio::test]
async fn check_library_creation_successful() {
    use crate::config::library::Library;

    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new()
        .await
        .unwrap()
        .switch_or_create_lib(&path)
        .await
        .unwrap();
    let lib_config = lib.library_config.clone();
    let lib_path = lib_config.lock().await;
    let lib_paths = &lib_path.as_ref().unwrap().library_paths;
    println!("{lib_paths:#?}");

    dbg!(&lib_paths);

    assert!(lib_paths.is_none());
    let _ = lib.delete().await;
}

#[tokio::test]
async fn check_library_default_values() {
    use crate::config::library::Library;

    let mut path = dirs::data_dir().unwrap().to_path_buf();
    path.push("hestia/test_lib/");
    let lib = Library::new()
        .await
        .unwrap()
        .switch_or_create_lib(&path)
        .await
        .unwrap();
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
    let lib = Library::new()
        .await
        .unwrap()
        .switch_or_create_lib(&path)
        .await
        .unwrap();
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
    let test_lib = Library::new()
        .await
        .unwrap()
        .switch_or_create_lib(&path)
        .await
        .unwrap();
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
