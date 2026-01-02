use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    fs::read_to_string,
    io::{Read, Write},
    time::Duration,
};
use tracing::{error, info};

use crate::{
    errors::{FileError, LibraryError},
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

    pub fn save_last(&self) -> Result<()> {
        match self.share_path.as_ref() {
            Some(path) => {
                let last_library = LastLibrary {
                    path: Some(path.to_owned()),
                };

                let last_library_toml: String = toml::to_string(&last_library)?;
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
    pub fn last() -> Result<Library> {
        let last_path = Self::create_or_validate_data_directory()?.join("hestia/last_lib.toml");
        let mut last_content = String::new();
        {
            let mut file = std::fs::OpenOptions::new().read(true).open(&last_path)?;
            file.read_to_string(&mut last_content)?;
        }
        let last_lib_path: LastLibrary = toml::from_str(&last_content)?;
        let share_path = match last_lib_path.path {
            Some(path) => path,
            None => return Err(LibraryError::InvalidSharePath)?,
        };
        Ok(Self::new().switch_or_create_lib(&share_path)?)
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

    pub fn create_or_validate_data_directory() -> Result<PathBuf> {
        // Check whether datahome format is available on current OS
        let datahome = match dirs::data_dir() {
            Some(dir) => dir,
            None => {
                return Err(LibraryError::DataHomeNotFoundError)?;
            }
        };
        // If format is available, but data dir does not exist, create it and await
        if !datahome.try_exists().is_ok_and(|x| x) {
            std::fs::create_dir_all(&datahome)?;
        }
        Ok(datahome)
    }

    pub fn get_canon_database_path(&self) -> Result<CanonPath> {
        let db_path = match self.share_path.as_ref() {
            Some(path) => path.join("db.sqlite"),
            None => {
                return Err(LibraryError::InvalidSharePath)?;
            }
        };
        Ok(CanonPath::from(db_path))
    }
    /// Save the config into a file on the disk that is specified in the share path of the Library
    /// Returns:
    ///     - Ok(true): The save to disk was a success
    ///     - Ok(false): The save to disk was a success, but the file had to be created
    ///     - Err: The save to disk failed either because share path was not set or the write to the file failed
    pub fn save_config(&self) -> Result<bool> {
        info!("Save config started!");
        let config_path = match self.share_path.as_ref() {
            Some(path) => {
                std::fs::create_dir_all(path)?;
                path.join("config.toml")
            }
            None => {
                return Err(LibraryError::InvalidSharePath)?;
            }
        };
        info!("Config Path {config_path:#?} extracted!");
        Self::open_or_create_database_file(self.share_path.as_ref().unwrap())?;
        Ok(self._save_config(config_path)?)
    }

    fn _save_config(&self, config_path: PathBuf) -> Result<bool> {
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
                                .open(&config_path)?;
                            info!("Config file opened!");

                            let content = toml::to_string(lib)?;
                            info!("Parsed version: {:#?}", content);
                            file.write_all(content.as_bytes())?;
                            file.flush()?;
                            file.sync_all()?;
                        }
                        println!("Trying to read file");
                        let file_contents = read_to_string(config_path)?;
                        println!("Written version: {file_contents:#?}");
                    }
                    None => return Err(LibraryError::InvalidSharePath)?,
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
                                .open(&config_path)?;
                            info!("Config file opened!");

                            let content = toml::to_string(lib)?;
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
                    None => return Err(LibraryError::InvalidSharePath)?,
                }
                Ok(false)
            }
            Err(e) => Err(LibraryError::Io)?,
        }
    }

    //TODO: There still needs to be a method to load from disk.
    pub fn load_config(&self) -> Result<(), FileError> {
        Ok(())
    }

    pub fn switch_or_create_lib(self, share_path: &std::path::PathBuf) -> Result<Library> {
        let lib = self._switch_or_create_lib(share_path)?;
        Ok(lib)
    }

    pub fn delete(self) -> Result<()> {
        self._delete()?;
        Ok(())
    }

    pub fn list_libraries() -> Result<Vec<String>> {
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

    pub fn _switch_or_create_lib(mut self, share_path: &std::path::PathBuf) -> Result<Library> {
        info!("Trying to validate data home directory:");
        let datahome = Library::create_or_validate_data_directory()?;
        info!("Data home directory has been verified successfully");
        if !share_path.starts_with(&datahome) {
            return Err(LibraryError::InvalidSharePath)?;
        }
        println!("Share path starts with data home");
        match share_path.try_exists() {
            Ok(true) => (),
            Ok(false) => std::fs::create_dir_all(share_path)?,
            Err(e) => return Err(e)?,
        };

        let config_path = Library::open_or_create_config_file(share_path)?;
        let _ = Library::open_or_create_database_file(share_path)?;

        info!("Created or found library file at {config_path:#?}");
        println!("Created or found library file at {config_path:#?}");
        let config_file = match std::fs::read(&config_path) {
            Ok(x) => x,
            Err(e) => {
                return Err(LibraryError::ConfigCreationError {
                    error: e.to_string(),
                })?;
            }
        };

        let config_file_content = match str::from_utf8(&config_file) {
            Ok(x) => x,
            Err(e) => {
                return Err(LibraryError::ConfigCreationError {
                    error: e.to_string(),
                })?;
            }
        };

        let config_toml: LibraryConfig = toml::from_str(config_file_content).unwrap_or_default();

        self.share_path = Some(share_path.to_owned());
        self.library_config = Some(config_toml);
        Ok(self)
    }

    fn _delete(&self) -> Result<()> {
        if let Some(path) = self.share_path.as_deref() {
            match std::fs::exists(path) {
                Ok(true) => std::fs::remove_dir_all(path)?,
                Ok(false) => return Err(LibraryError::InvalidSharePath)?,
                Err(e) => {
                    return Err(LibraryError::ConfigDeletionError {
                        error: e.to_string(),
                    })?;
                }
            }
        }
        Ok(())
    }

    //TODO: Refactor this into being a struct method and not static
    fn open_or_create_config_file(share_path: &Path) -> Result<PathBuf> {
        let config_path = share_path.join("config.toml");
        let content = toml::to_string(&LibraryConfig::default())?;
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
            Err(e) => return Err(LibraryError::Io)?,
        }
        Ok(config_path)
    }

    //TODO: Refactor this into being a struct method and not static
    fn open_or_create_database_file(share_path: &Path) -> Result<PathBuf> {
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
