use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub struct FileError {
    pub kind: crate::errors::FileErrorKind,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    pub paths: Option<Vec<PathBuf>>,
}

#[derive(Debug)]
pub enum FileErrorKind {
    GenericError,
    Io,
    FileIdExtractionError,
    HashError,
    PathNotFoundError,
    FileHashError(String),
    MaxFilesWatchError,
    InvalidConfigError,
    WatchNotFoundError,
}

impl std::fmt::Display for FileErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for FileError {
    fn from(other: std::io::Error) -> Self {
        FileError {
            kind: FileErrorKind::FileIdExtractionError,
            message: format!("An IO error occured: {:?}", other),
            source: Some(Box::new(other)),
            paths: None,
        }
    }
}

impl From<notify::Error> for FileError {
    fn from(other: notify::Error) -> Self {
        let error_kind = match other.kind {
            notify::ErrorKind::Generic(_) => FileErrorKind::GenericError,
            notify::ErrorKind::Io(_) => FileErrorKind::Io,
            notify::ErrorKind::PathNotFound => FileErrorKind::PathNotFoundError,
            notify::ErrorKind::InvalidConfig(_) => FileErrorKind::InvalidConfigError,
            notify::ErrorKind::MaxFilesWatch => FileErrorKind::MaxFilesWatchError,
            notify::ErrorKind::WatchNotFound => FileErrorKind::WatchNotFoundError,
        };

        FileError {
            kind: error_kind,
            message: format!("Notify has encontered an error!"),
            paths: Some(other.paths.clone()),
            source: Some(Box::new(other)),
        }
    }
}

impl FileError {
    pub fn new(kind: FileErrorKind, message: String, paths: Option<Vec<PathBuf>>) -> Self {
        FileError {
            kind,
            message,
            source: None,
            paths,
        }
    }
    pub fn with_source<E>(
        kind: FileErrorKind,
        message: String,
        source: E,
        paths: Option<Vec<PathBuf>>,
    ) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        FileError {
            kind,
            message,
            source: Some(Box::new(source)),
            paths,
        }
    }
}

impl PartialEq<notify::ErrorKind> for FileErrorKind {
    fn eq(&self, other: &notify::ErrorKind) -> bool {
        match (self, other) {
            (FileErrorKind::GenericError, notify::ErrorKind::Generic(_)) => true,
            (FileErrorKind::Io, notify::ErrorKind::Io(_)) => true,
            (FileErrorKind::PathNotFoundError, notify::ErrorKind::PathNotFound) => true,
            (FileErrorKind::InvalidConfigError, notify::ErrorKind::InvalidConfig(_)) => true,
            (FileErrorKind::MaxFilesWatchError, notify::ErrorKind::MaxFilesWatch) => true,
            (FileErrorKind::WatchNotFoundError, notify::ErrorKind::WatchNotFound) => true,
            _ => false,
        }
    }
}

impl PartialEq for FileErrorKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}
