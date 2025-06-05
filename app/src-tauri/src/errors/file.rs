use std::path::PathBuf;

pub struct FileError {
    pub kind: crate::errors::FileErrorKind,
    pub paths: Option<Vec<PathBuf>>,
}

#[derive(Debug, Clone)]
pub enum FileErrorKind {
    FileIdExtractionError(String),
    PathNotFoundError(String),
    FileHashError(String),
    NotifyError(String),
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
        FileError {
            kind: FileErrorKind::NotifyError(other.kind),
            paths: Some(other.paths),
        }
    }
}

impl FileError {
    pub fn new(kind: FileErrorKind, paths: Option<Vec<PathBuf>>) -> Self {
        FileError { kind, paths }
    }
}
