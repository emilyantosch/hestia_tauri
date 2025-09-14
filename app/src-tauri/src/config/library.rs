use serde::{Deserialize, Serialize};
use std::{
    fs::read_to_string,
    fs::File,
    io::{Read, Write},
    time::Duration,
};
use tracing::{error, info};

use crate::{
    errors::{FileError, LibraryError, LibraryErrorKind},
    utils::{self, canon_path::CanonPath},
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Library {
    pub share_path: Option<std::path::PathBuf>,
    pub library_config: Option<LibraryConfig>,
}

impl Drop for Library {
    fn drop(&mut self) {
        let _ = self.save_last();
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct LibraryConfig {
    pub name: String,
    pub color: utils::decorations::Color,
    pub icon: utils::decorations::Icon,
    pub library_paths: Vec<LibraryPathConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LibraryPathConfig {
    pub name: Option<String>,
    pub path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
struct LastLibrary {
    path: Option<PathBuf>,
}

impl Default for LibraryPathConfig {
    fn default() -> Self {
        LibraryPathConfig {
            name: Some(String::from("")),
            path: Some(PathBuf::new().join("")),
        }
    }
}

impl Default for LibraryConfig {
    fn default() -> Self {
        LibraryConfig {
            name: "Library".to_string(),
            color: utils::decorations::Color::default(),
            icon: utils::decorations::Icon::default(),
            library_paths: vec![LibraryPathConfig::default()],
        }
    }
}

impl Library {
    pub fn new() -> Library {
        Library {
            share_path: None,
            library_config: None,
        }
    }

    pub fn save_last(&self) -> std::result::Result<(), LibraryError> {
        match self.share_path.as_ref() {
            Some(path) => {
                let last_library = LastLibrary {
                    path: Some(path.to_owned()),
                };

                let last_library_toml: String = toml::to_string(&last_library).map_err(|e| {
                    LibraryError::with_source(
                        LibraryErrorKind::Io,
                        "The conversion to TOML format of last library failed".to_string(),
                        Some(Box::new(e)),
                    )
                })?;
                {
                    let mut file: File = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(path)?;
                    file.write_all(last_library_toml.as_bytes())?;
                    file.flush()?;
                    file.sync_all()?;
                }
            }
            None => return Ok(()),
        };
        Ok(())
    }

    /// Return the library from the previous run, may return LibraryError if there is none
    pub fn last() -> Result<Library, LibraryError> {
        let last_path = Self::create_or_validate_data_directory()?.join("hestia/last_lib.toml");
        let mut last_content = String::new();
        {
            let mut file = std::fs::OpenOptions::new().read(true).open(&last_path)?;
            file.read_to_string(&mut last_content)?;
        }
        let last_lib_path: LastLibrary = toml::from_str(&last_content).map_err(|e| {
            LibraryError::with_source(
                LibraryErrorKind::Io,
                "Could not find last library, creating new one...".to_string(),
                Some(Box::new(e)),
            )
        })?;
        let share_path = match last_lib_path.path {
            Some(path) => path,
            None => {
                return Err(LibraryError::new(
                    LibraryErrorKind::LastLibraryNotFound,
                    "Last library path was empty, creating new library!".to_string(),
                ))
            }
        };
        Self::new().switch_or_create_lib(&share_path)
    }

    pub fn last_or_new() -> Library {
        match Self::last() {
            Ok(lib) => lib,
            Err(_) => {
                info!("Could not find old library, executing prompt for new one!");
                Self::new()
            }
        }
    }

    pub fn create_or_validate_data_directory() -> Result<PathBuf, LibraryError> {
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
            std::fs::create_dir_all(&datahome).map_err(|e| {
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

    pub fn get_canon_database_path(&self) -> Result<CanonPath, LibraryError> {
        let db_path = match self.share_path.as_ref() {
            Some(path) => path.join("db.sqlite"),
            None => {
                return Err(LibraryError::new(
                    LibraryErrorKind::Io,
                    "No db path found at!".to_string(),
                ));
            }
        };
        Ok(CanonPath::from(db_path))
    }
    /// Save the config into a file on the disk that is specified in the share path of the Library
    /// Returns:
    ///     - Ok(true): The save to disk was a success
    ///     - Ok(false): The save to disk was a success, but the file had to be created
    ///     - Err: The save to disk failed either because share path was not set or the write to the file failed
    pub fn save_config(&self) -> Result<bool, LibraryError> {
        info!("Save config started!");
        let config_path = match self.share_path.as_ref() {
            Some(path) => {
                std::fs::create_dir_all(path)?;
                path.join("config.toml")
            }
            None => {
                return Err(LibraryError::new(
                    LibraryErrorKind::Io,
                    "No config path found at!".to_string(),
                ));
            }
        };
        info!("Config Path {config_path:#?} extracted!");
        Self::open_or_create_database_file(self.share_path.as_ref().unwrap())?;
        self._save_config(config_path)
    }

    fn _save_config(&self, config_path: PathBuf) -> Result<bool, LibraryError> {
        match std::fs::exists(&config_path) {
            Ok(true) => {
                match self.library_config.as_ref() {
                    Some(lib) => {
                        {
                            info!("Config Path {config_path:#?} exists!");
                            let mut file = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&config_path)
                                .map_err(|e| {
                                    LibraryError::with_source(
                                        LibraryErrorKind::Io,
                                        "The opening of the file failed!".to_string(),
                                        Some(Box::new(e)),
                                    )
                                })?;
                            info!("Config file opened!");

                            let content = toml::to_string(lib).map_err(|e| {
                                LibraryError::with_source(
                        LibraryErrorKind::Io,
                        format!("An error occurred while trying to parse interal library to toml at {config_path:#?}"),
                        Some(Box::new(e)),
                            )
                            })?;
                            info!("Parsed version: {:#?}", content);
                            file.write_all(content.as_bytes())?;
                            file.flush()?;
                            file.sync_all()?;
                        }
                        println!("Trying to read file");
                        let file_contents = read_to_string(config_path)?;
                        println!("Written version: {file_contents:#?}");
                    }
                    None => {
                        return Err(LibraryError::new(
                            LibraryErrorKind::InvalidSharePath,
                            format!("Library Config not found"),
                        ))
                    }
                }
                Ok(true)
            }
            Ok(false) => {
                match self.library_config.as_ref() {
                    Some(lib) => {
                        {
                            info!("Config Path {config_path:#?} does not exist!");
                            let mut file = std::fs::OpenOptions::new()
                                .create(true)
                                .write(true)
                                .truncate(true)
                                .open(&config_path)
                                .map_err(|e| {
                                    LibraryError::with_source(
                                        LibraryErrorKind::Io,
                                        "The opening of the file failed!".to_string(),
                                        Some(Box::new(e)),
                                    )
                                })?;
                            info!("Config file opened!");

                            let content = toml::to_string(lib).map_err(|e| {
                                LibraryError::with_source(
                        LibraryErrorKind::Io,
                        format!("An error occurred while trying to parse interal library to toml at {config_path:#?}"),
                        Some(Box::new(e)),
                            )
                            })?;
                            info!("Parsed version: {:#?}", content);
                            file.write_all(content.as_bytes())?;
                            file.flush()?;
                            file.sync_all()?;
                        }
                        println!("Trying to read file");
                        std::thread::sleep(Duration::from_secs(2));
                        let file_contents = read_to_string(config_path)?;
                        println!("Written version: {file_contents:#?}");
                    }
                    None => {
                        return Err(LibraryError::new(
                            LibraryErrorKind::InvalidSharePath,
                            format!("Library Config not found"),
                        ))
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
    pub fn load_config(&self) -> Result<(), FileError> {
        Ok(())
    }

    pub fn switch_or_create_lib(
        self,
        share_path: &std::path::PathBuf,
    ) -> Result<Library, LibraryError> {
        let lib = self._switch_or_create_lib(share_path)?;
        Ok(lib)
    }

    pub fn delete(self) -> Result<(), LibraryError> {
        self._delete()?;
        Ok(())
    }

    pub fn list_libraries() -> Result<Vec<String>, LibraryError> {
        let share_path = Library::create_or_validate_data_directory()?.join("hestia");
        info!("The share path to list libraries {share_path:#?}");

        let libraries = std::fs::read_dir(&share_path)?;

        let collected_library = libraries
            .filter_map(Result::ok)
            .map(|v| v.path().to_string_lossy().to_string())
            .collect();
        info!("List of libraries: {collected_library:#?}");
        Ok(collected_library)
        // Ok(std::fs::read_dir(&share_path)?
        //     .filter_map(Result::ok)
        //     .map(|v| v.path().to_string_lossy().to_string())
        //     .collect())
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

    pub fn _switch_or_create_lib(
        mut self,
        share_path: &std::path::PathBuf,
    ) -> Result<Library, LibraryError> {
        info!("Trying to validate data home directory:");
        println!("Trying to validate data home directory:");
        let datahome = Library::create_or_validate_data_directory()?;
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
            Ok(false) => std::fs::create_dir_all(share_path)?,
            Err(e) => {
                return Err(LibraryError::with_source(
                    LibraryErrorKind::InvalidSharePath,
                    format!("The path {share_path:#?} is not a directory or could not be found"),
                    Some(Box::new(e)),
                ));
            }
        };

        let config_path = Library::open_or_create_config_file(share_path)?;
        let _ = Library::open_or_create_database_file(share_path)?;

        info!("Created or found library file at {config_path:#?}");
        println!("Created or found library file at {config_path:#?}");
        let config_file = match std::fs::read(&config_path) {
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

        let config_toml: LibraryConfig = toml::from_str(config_file_content)
            .map_err(|e| {
                LibraryError::with_source(
                    LibraryErrorKind::ConfigCreationError,
                    format!("Couldn't parse {config_path:#?} into TOML format"),
                    Some(Box::new(e)),
                )
            })
            .unwrap_or_default();

        self.share_path = Some(share_path.to_owned());
        self.library_config = Some(config_toml);
        Ok(self)
    }

    fn _delete(&self) -> Result<(), LibraryError> {
        if let Some(path) = self.share_path.as_deref() {
            match std::fs::exists(path) {
                Ok(true) => std::fs::remove_dir_all(path).map_err(|e| {
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

    //TODO: Refactor this into being a struct method and not static
    fn open_or_create_config_file(share_path: &Path) -> Result<PathBuf, LibraryError> {
        let config_path = share_path.join("config.toml");
        let content = toml::to_string(&LibraryConfig::default()).map_err(|e| {
            LibraryError::with_source(
                LibraryErrorKind::Io,
                format!("An error occurred while trying to parse toml at {config_path:#?}"),
                Some(Box::new(e)),
            )
        })?;
        println!("{content:#?}");
        match std::fs::exists(&config_path) {
            Ok(true) => (),
            Ok(false) => {
                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(false)
                    .open(&config_path)?;
                file.write_all(content.as_bytes())?;
                file.flush()?;
                file.sync_all()?;
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

    //TODO: Refactor this into being a struct method and not static
    fn open_or_create_database_file(share_path: &Path) -> Result<PathBuf, LibraryError> {
        println!("Trying to open share path database");
        let db_path = share_path.join("db.sqlite");
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&db_path)?;
        Ok(db_path)
    }
}
