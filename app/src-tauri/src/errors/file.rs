use serde::{Deserialize, Serialize};
use thiserror::Error;

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
