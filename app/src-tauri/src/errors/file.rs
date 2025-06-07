use std::path::PathBuf;

#[derive(Debug)]
pub struct FileError {
    pub kind: crate::errors::FileErrorKind,
    pub paths: Option<Vec<PathBuf>>,
}

#[derive(Debug)]
pub enum FileErrorKind {
    GenericError(String),
    Io(std::io::Error),
    FileIdExtractionError(String),
    HashError,
    PathNotFoundError,
    FileHashError(String),
    MaxFilesWatchError,
    InvalidConfigError,
    WatchNotFoundError,
}

impl From<std::io::Error> for FileError {
    fn from(other: std::io::Error) -> Self {
        FileError {
            kind: FileErrorKind::FileIdExtractionError(other.kind().to_string()),
            paths: None,
        }
    }
}

impl From<notify::Error> for FileError {
    fn from(other: notify::Error) -> Self {
        let error_kind = match other.kind {
            notify::ErrorKind::Generic(x) => FileErrorKind::GenericError(x),
            notify::ErrorKind::Io(x) => FileErrorKind::Io(x),
            notify::ErrorKind::PathNotFound => FileErrorKind::PathNotFoundError,
            notify::ErrorKind::InvalidConfig(_) => FileErrorKind::InvalidConfigError,
            notify::ErrorKind::MaxFilesWatch => FileErrorKind::MaxFilesWatchError,
            notify::ErrorKind::WatchNotFound => FileErrorKind::WatchNotFoundError,
        };

        FileError {
            kind: error_kind,
            paths: Some(other.paths),
        }
    }
}

impl FileError {
    pub fn new(kind: FileErrorKind, paths: Option<Vec<PathBuf>>) -> Self {
        FileError { kind, paths }
    }
}

impl PartialEq<notify::ErrorKind> for FileErrorKind {
    fn eq(&self, other: &notify::ErrorKind) -> bool {
        match (self, other) {
            (FileErrorKind::GenericError(_), notify::ErrorKind::Generic(_)) => true,
            (FileErrorKind::Io(_), notify::ErrorKind::Io(_)) => true,
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

// impl PartialEq for FileErrorKind {
//     fn eq(&self, other: &Self) -> bool {
//         matches!((self, other),
//             (FileErrorKind::GenericError(_), FileErrorKind::GenericError(_)),
//             (FileErrorKind::Io(_), FileErrorKind(_)),
//             (FileErrorKind::PathNotFoundError, FileErrorKind::PathNotFoundError),
//             (FileErrorKind::InvalidConfigError, FileErrorKind::InvalidConfigError),
//             (FileErrorKind::MaxFilesWatchError, FileErrorKind::MaxFilesWatchError),
//             (FileErrorKind::WatchNotFoundError, FileErrorKind::WatchNotFoundError),
//             _ => false,
//         )
//     }
// }
