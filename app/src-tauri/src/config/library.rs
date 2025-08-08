use serde::Deserialize;
use std::{fmt::write, fs::File, str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::errors::{FileError, FileErrorKind};
use std::path::{Path, PathBuf};

pub struct Library {
    pub share_path: Arc<Option<std::path::PathBuf>>,
    pub library_config: Arc<Mutex<Option<LibraryConfig>>>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryConfig {
    pub library_paths: Option<LibraryPathConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryPathConfig {
    name: Option<String>,
    path: Option<PathBuf>,
}

impl Default for LibraryPathConfig {
    fn default() -> Self {
        LibraryPathConfig {
            name: Some(String::from("")),
            path: Some(PathBuf::new()),
        }
    }
}

impl Library {
    pub async fn new() -> Result<Library, FileError> {
        Ok(Library {
            share_path: Arc::new(None),
            library_config: Arc::new(Mutex::new(None)),
        })
    }

    async fn create_or_validate_data_directory() -> Result<PathBuf, FileError> {
        // Check whether datahome format is available on current OS
        let datahome = match dirs::data_dir() {
            Some(dir) => dir,
            None => {
                return Err(FileError::new(
                    FileErrorKind::InvalidConfigError,
                    "There is no know format of the data directory on this OS!".to_string(),
                    None,
                ));
            }
        };
        // If format is available, but data dir does not exist, create it and await
        if !datahome.try_exists().is_ok_and(|x| x) {
            tokio::fs::create_dir_all(&datahome).await.map_err(|e| {
                FileError::with_source(
                    FileErrorKind::Io,
                    format!(
                        "Local data directory at {datahome:#?} could neither be found nor created"
                    ),
                    e,
                    None,
                )
            })?;
        }
        Ok(datahome)
    }

    //TODO: There still needs to be a method to save to disk.
    pub async fn save_config(&self) -> Result<(), FileError> {
        Ok(())
    }

    //TODO: There still needs to be a method to load from disk.
    pub async fn load_config(&self) -> Result<(), FileError> {
        Ok(())
    }

    pub async fn switch_or_create_lib(
        &mut self,
        share_path: &std::path::PathBuf,
    ) -> Result<(), FileError> {
        info!("Trying to validate data home directory:");
        println!("Trying to validate data home directory:");
        let datahome = Library::create_or_validate_data_directory().await?;
        println!("Data home directory has been verified successfully");
        if !share_path.starts_with(&datahome) {
            return Err(FileError::new(
                FileErrorKind::PathNotFoundError,
                format!("Path {share_path:#?} does not start with correct datahome {datahome:#?}"),
                Some(vec![share_path.to_owned(), datahome]),
            ));
        }
        println!("Share path starts with data home");
        match share_path.try_exists() {
            Ok(true) => (),
            Ok(false) => tokio::fs::create_dir_all(&share_path).await?,
            Err(e) => {
                return Err(FileError::with_source(
                    FileErrorKind::PathNotFoundError,
                    format!("The path {share_path:#?} is not a directory or could not be found"),
                    e,
                    Some(vec![share_path.to_owned()]),
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
                return Err(FileError::with_source(
                    FileErrorKind::PathNotFoundError,
                    format!("Library Config {config_path:#?} cannot be loaded!"),
                    e,
                    Some(vec![config_path]),
                ))
            }
        };

        let config_file_content = match str::from_utf8(&config_file) {
            Ok(x) => x,
            Err(e) => {
                return Err(FileError::with_source(
                    FileErrorKind::Io,
                    format!("The file {config_path:#?} contains non utf-8 characters"),
                    e,
                    Some(vec![config_path]),
                ));
            }
        };

        let config_toml: LibraryConfig = toml::from_str(config_file_content).map_err(|e| {
            FileError::with_source(
                FileErrorKind::Io,
                format!("Couldn't parse {config_path:#?} into TOML format"),
                e,
                Some(vec![config_path]),
            )
        })?;

        self.share_path = Arc::new(Some(share_path.to_owned()));
        self.library_config = Arc::new(Mutex::new(Some(config_toml)));
        Ok(())
    }

    pub async fn delete(self) -> Result<(), FileError> {
        if let Some(path) = self.share_path.as_deref() {
            match tokio::fs::try_exists(path).await {
                Ok(true) => tokio::fs::remove_dir_all(path).await.map_err(|e| {
                    FileError::with_source(
                        FileErrorKind::Io,
                        format!("Failed to delete directory {path:#?}: {e:#?}"),
                        e,
                        Some(vec![path.into()]),
                    )
                })?,
                Ok(false) => {
                    return Err(FileError::new(
                        FileErrorKind::PathNotFoundError,
                        format!("Could not find {path:#?}"),
                        Some(vec![path.into()]),
                    ))
                }
                Err(e) => {
                    return Err(FileError::with_source(
                        FileErrorKind::Io,
                        format!("Error occurred while trying to path {path:#?}"),
                        e,
                        Some(vec![path.into()]),
                    ))
                }
            }
        }
        Ok(())
    }

    async fn open_or_create_config_file(share_path: &Path) -> Result<PathBuf, FileError> {
        let config_path = share_path.join("config.toml");
        tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&config_path)
            .await?;
        Ok(config_path)
    }

    async fn open_or_create_database_file(share_path: &Path) -> Result<PathBuf, FileError> {
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
