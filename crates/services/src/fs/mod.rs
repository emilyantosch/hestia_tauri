pub mod scanner;
pub mod watcher;

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WatcherError {
    #[error("The path {path} is not being watched and cannot be unwatched!")]
    PathNotWatched { path: String },
    #[error("The operation could not be completed because of an IO failure! Cause: {0}")]
    Io(#[from] std::io::Error),
    #[error("The path to be watcher or that interacts with the watcher could not be found!")]
    PathNotFound,
    #[error("The event emitted by the watcher is not recognized!")]
    InvalidEventType,
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum FileError {
    #[error("Generic file error: {path}")]
    GenericError { path: String },
    #[error("Input/Output error: {path}")]
    Io { path: String },
    #[error("Path could not be found: {path}")]
    PathNotFoundError { path: String },
}

impl PartialEq for FileError {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CanonPath {
    path: PathBuf,
}

//FIXME: Turn this into try_from as this may fail
impl From<PathBuf> for CanonPath {
    fn from(path: PathBuf) -> CanonPath {
        let p = match path.canonicalize() {
            Ok(path) => path,
            Err(_) => PathBuf::new(),
        };
        CanonPath { path: p }
    }
}

impl From<CanonPath> for PathBuf {
    fn from(path: CanonPath) -> PathBuf {
        path.path
    }
}

impl AsRef<Path> for CanonPath {
    fn as_ref(&self) -> &Path {
        &self.path.as_path()
    }
}

impl CanonPath {
    pub fn try_exists(&self) -> Result<bool> {
        Ok(self.path.try_exists()?)
    }

    pub fn as_str(&self) -> Result<&str> {
        match self.path.to_str() {
            Some(str) => Ok(str),
            None => Err(FileError::PathNotFoundError {
                path: self.path.to_string_lossy().to_string(),
            })?,
        }
    }
}

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum ScannerError {
    #[error("The scan of path {path} failed!")]
    PathScanFailedError { path: String },
    #[error("The scan of path {path} failed!")]
    DatabaseState { path: String },
}
