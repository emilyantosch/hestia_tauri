use serde::{Deserialize, Serialize};
use std::{fmt::write, fs::File, str::FromStr, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{io::AsyncWriteExt, sync::Mutex};
use tracing::{error, info};

use crate::errors::{FileError, FileErrorKind, LibraryError, LibraryErrorKind};
use std::path::{Path, PathBuf};

pub struct Library {
    pub share_path: Option<std::path::PathBuf>,
    pub library_config: Arc<Mutex<Option<LibraryConfig>>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LibraryConfig {
    pub library_paths: Option<Vec<LibraryPathConfig>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LibraryPathConfig {
    name: Option<String>,
    path: Option<PathBuf>,
}

impl Default for LibraryPathConfig {
    fn default() -> Self {
        LibraryPathConfig {
            name: Some(String::from("Test")),
            path: Some(PathBuf::new().join("/home/emmi/Downloads/")),
        }
    }
}

impl Default for LibraryConfig {
    fn default() -> Self {
        LibraryConfig {
            library_paths: Some(vec![LibraryPathConfig::default()]),
        }
    }
}

impl Library {
    pub fn new() -> Library {
        Library {
            share_path: None,
            library_config: Arc::new(Mutex::new(None)),
        }
    }

    async fn create_or_validate_data_directory() -> Result<PathBuf, LibraryError> {
        // Check whether datahome format is available on current OS
        let datahome = match dirs::data_dir() {
            Some(dir) => dir,
            None => {
                return Err(LibraryError::new(
                    LibraryErrorKind::InvalidSharePath,
                    "There is no know format of the data directory on this OS!".to_string(),
                ));
            }
        };
        // If format is available, but data dir does not exist, create it and await
        if !datahome.try_exists().is_ok_and(|x| x) {
            tokio::fs::create_dir_all(&datahome).await.map_err(|e| {
                LibraryError::with_source(
                    LibraryErrorKind::Io,
                    format!(
                        "Local data directory at {datahome:#?} could neither be found nor created"
                    ),
                    Some(Box::new(e)),
                )
            })?;
        }
        Ok(datahome)
    }

    /// Save the config into a file on the disk that is specified in the share path of the Library
    /// Returns:
    ///     - Ok(true): The save to disk was a success
    ///     - Ok(false): The save to disk was a success, but the file had to be created
    ///     - Err: The save to disk failed either because share path was not set or the write to the file failed
    pub async fn save_config(&self) -> Result<bool, LibraryError> {
        let config_path = match self.share_path.as_ref() {
            Some(path) => path.join("config.toml"),
            None => {
                return Err(LibraryError::new(
                    LibraryErrorKind::Io,
                    format!("No config path found at!"),
                ));
            }
        };

        match tokio::fs::try_exists(&config_path).await {
            Ok(true) => {
                let mut file = tokio::fs::OpenOptions::new()
                    .create(false)
                    .write(true)
                    .truncate(true)
                    .open(&config_path)
                    .await?;
                {
                    let lib_lock = self.library_config.lock().await;
                    match lib_lock.as_ref() {
                        Some(lib) => {
                            let paths = lib.library_paths.as_ref().unwrap();
                            let content = toml::to_string(paths).map_err(|e| {
                                LibraryError::with_source(
                        LibraryErrorKind::Io,
                        format!("An error occurred while trying to parse toml at {config_path:#?}"),
                        Some(Box::new(e)),
                            )
                            })?;
                            file.write_all(content.as_bytes()).await?;
                        }
                        None => {
                            return Err(LibraryError::new(
                                LibraryErrorKind::InvalidSharePath,
                                format!("Library Config not found"),
                            ))
                        }
                    }
                }
                Ok(true)
            }
            Ok(false) => {
                let mut file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(false)
                    .open(&config_path)
                    .await?;
                {
                    let lib_lock = self.library_config.lock().await;
                    match lib_lock.as_ref() {
                        Some(lib) => {
                            let paths = lib.library_paths.as_ref().unwrap();
                            let content = toml::to_string(paths).map_err(|e| {
                                LibraryError::with_source(
                        LibraryErrorKind::Io,
                        format!("An error occurred while trying to parse toml at {config_path:#?}"),
                        Some(Box::new(e)),
                            )
                            })?;
                            file.write_all(content.as_bytes()).await?;
                        }
                        None => {
                            return Err(LibraryError::new(
                                LibraryErrorKind::InvalidSharePath,
                                format!("Library Config not found"),
                            ))
                        }
                    }
                }
                Ok(false)
            }
            Err(e) => Err(LibraryError::with_source(
                LibraryErrorKind::Io,
                format!("An error occurred while trying to look for {config_path:#?}"),
                Some(Box::new(e)),
            )),
        }
    }

    //TODO: There still needs to be a method to load from disk.
    pub async fn load_config(&self) -> Result<(), FileError> {
        Ok(())
    }

    pub async fn switch_or_create_lib(
        self,
        share_path: &std::path::PathBuf,
    ) -> Result<Library, LibraryError> {
        let lib = self._switch_or_create_lib(share_path).await?;

        if let Err(e) = lib.await_path_exists(Duration::from_secs(5)).await {
            error!("Path creation failed and/or timed out due to: {e:#?}");
        }

        Ok(lib)
    }

    pub async fn delete(self) -> Result<(), LibraryError> {
        self._delete().await?;

        if let Err(e) = self.await_path_deleted(Duration::from_secs(5)).await {
            error!("Failed to delete library or library deletion timedout due to: {e:#?}");
            return Err(LibraryError::with_source(
                LibraryErrorKind::DeletionTimeout,
                format!("Failed to delete library or library deletion timedout due to: {e:#?}"),
                Some(Box::new(e)),
            ));
        }

        Ok(())
    }

    async fn await_path_deleted(self, timeout: Duration) -> Result<(), LibraryError> {
        let now = std::time::Instant::now();
        loop {
            match tokio::fs::try_exists(self.share_path.as_ref().ok_or(LibraryError::new(
                LibraryErrorKind::Io,
                "There is no share path provided.".to_string(),
            ))?)
            .await
            {
                Ok(false) => return Ok(()),
                Ok(true) if now.elapsed() > timeout => {
                    return Err(LibraryError::new(
                        LibraryErrorKind::DeletionTimeout,
                        "The deletion of the library timed out!".to_string(),
                    ))
                }
                Ok(true) => tokio::task::yield_now().await,
                Err(e) if now.elapsed() > timeout => {
                    return Err(LibraryError::with_source(
                        LibraryErrorKind::CreationTimeout,
                        "The creation of the library timed out and path failed the verify!"
                            .to_string(),
                        Some(Box::new(e)),
                    ))
                }
                Err(_) => tokio::task::yield_now().await,
            }
        }
    }

    async fn await_path_exists(&self, timeout: Duration) -> Result<(), LibraryError> {
        let now = std::time::Instant::now();
        loop {
            match tokio::fs::try_exists(self.share_path.as_ref().ok_or(LibraryError::new(
                LibraryErrorKind::Io,
                "There is no share path provided.".to_string(),
            ))?)
            .await
            {
                Ok(true) => return Ok(()),
                Ok(false) if now.elapsed() > timeout => {
                    return Err(LibraryError::new(
                        LibraryErrorKind::CreationTimeout,
                        "The creation of the library timed out!".to_string(),
                    ))
                }
                Ok(false) => tokio::task::yield_now().await,
                Err(e) if now.elapsed() > timeout => {
                    return Err(LibraryError::with_source(
                        LibraryErrorKind::CreationTimeout,
                        "The creation of the library timed out and path failed the verify!"
                            .to_string(),
                        Some(Box::new(e)),
                    ))
                }
                Err(_) => tokio::task::yield_now().await,
            }
        }
    }

    pub async fn _switch_or_create_lib(
        mut self,
        share_path: &std::path::PathBuf,
    ) -> Result<Library, LibraryError> {
        info!("Trying to validate data home directory:");
        println!("Trying to validate data home directory:");
        let datahome = Library::create_or_validate_data_directory().await?;
        println!("Data home directory has been verified successfully");
        if !share_path.starts_with(&datahome) {
            return Err(LibraryError::new(
                LibraryErrorKind::InvalidSharePath,
                format!("Path {share_path:#?} does not start with correct datahome {datahome:#?}"),
            ));
        }
        println!("Share path starts with data home");
        match share_path.try_exists() {
            Ok(true) => (),
            Ok(false) => tokio::fs::create_dir_all(&share_path).await?,
            Err(e) => {
                return Err(LibraryError::with_source(
                    LibraryErrorKind::InvalidSharePath,
                    format!("The path {share_path:#?} is not a directory or could not be found"),
                    Some(Box::new(e)),
                ));
            }
        };

        let config_path = Library::open_or_create_config_file(share_path).await?;
        let _ = Library::open_or_create_database_file(share_path).await?;

        info!("Created or found library file at {config_path:#?}");
        println!("Created or found library file at {config_path:#?}");
        let config_file = match tokio::fs::read(&config_path).await {
            Ok(x) => x,
            Err(e) => {
                return Err(LibraryError::with_source(
                    LibraryErrorKind::ConfigCreationError,
                    format!("Library Config {config_path:#?} cannot be loaded!"),
                    Some(Box::new(e)),
                ))
            }
        };

        let config_file_content = match str::from_utf8(&config_file) {
            Ok(x) => x,
            Err(e) => {
                return Err(LibraryError::with_source(
                    LibraryErrorKind::ConfigCreationError,
                    format!("The file {config_path:#?} contains non utf-8 characters"),
                    Some(Box::new(e)),
                ));
            }
        };

        let config_toml: LibraryConfig = toml::from_str(config_file_content).map_err(|e| {
            LibraryError::with_source(
                LibraryErrorKind::ConfigCreationError,
                format!("Couldn't parse {config_path:#?} into TOML format"),
                Some(Box::new(e)),
            )
        })?;

        self.share_path = Some(share_path.to_owned());
        self.library_config = Arc::new(Mutex::new(Some(config_toml)));
        Ok(self)
    }

    pub async fn _delete(&self) -> Result<(), LibraryError> {
        if let Some(path) = self.share_path.as_deref() {
            match tokio::fs::try_exists(path).await {
                Ok(true) => tokio::fs::remove_dir_all(path).await.map_err(|e| {
                    LibraryError::with_source(
                        LibraryErrorKind::Io,
                        format!("Failed to delete directory {path:#?}: {e:#?}"),
                        Some(Box::new(e)),
                    )
                })?,
                Ok(false) => {
                    return Err(LibraryError::new(
                        LibraryErrorKind::InvalidSharePath,
                        format!("Could not find {path:#?}"),
                    ))
                }
                Err(e) => {
                    return Err(LibraryError::with_source(
                        LibraryErrorKind::Io,
                        format!("Error occurred while trying to path {path:#?}"),
                        Some(Box::new(e)),
                    ))
                }
            }
        }
        Ok(())
    }

    async fn open_or_create_config_file(share_path: &Path) -> Result<PathBuf, LibraryError> {
        let config_path = share_path.join("config.toml");
        let content = toml::to_string(&LibraryConfig::default()).map_err(|e| {
            LibraryError::with_source(
                LibraryErrorKind::Io,
                format!("An error occurred while trying to parse toml at {config_path:#?}"),
                Some(Box::new(e)),
            )
        })?;
        println!("{content:#?}");
        match tokio::fs::try_exists(&config_path).await {
            Ok(true) => (),
            Ok(false) => {
                let mut file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(false)
                    .open(&config_path)
                    .await?;
                file.write_all(content.as_bytes()).await?;
            }
            Err(e) => {
                return Err(LibraryError::with_source(
                    LibraryErrorKind::Io,
                    format!("An error occurred while trying to look for {config_path:#?}"),
                    Some(Box::new(e)),
                ))
            }
        }
        Ok(config_path)
    }

    async fn open_or_create_database_file(share_path: &Path) -> Result<PathBuf, LibraryError> {
        println!("Trying to open share path database");
        let db_path = share_path.join("db.sqlite");
        tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&db_path)
            .await?;
        Ok(db_path)
    }
}
