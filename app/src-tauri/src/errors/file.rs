use std::path::PathBuf;
use thiserror::Error;

use crate::errors::{HashError, HashErrorKind};

// #[derive(Debug, Error)]
// pub struct FileError {
//     pub kind: crate::errors::FileErrorKind,
//     pub message: String,
//     #[source]
//     pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
//     pub paths: Option<Vec<PathBuf>>,
// }

#[derive(Debug, Error)]
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
