use std::fmt;

use thiserror::Error;

use crate::errors::{DbError, FileError, HashError};

impl PartialEq for AppErrorKind {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Debug, Clone)]
pub enum AppErrorKind {
    FileError,
    DbError,
    HashError,
}

impl std::fmt::Display for AppErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{kind}: {message}")]
    Categorized {
        kind: AppErrorKind,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<HashError> for AppError {
    fn from(hash_error: HashError) -> AppError {
        AppError::Categorized {
            kind: AppErrorKind::HashError,
            message: hash_error.message.clone(),
            source: Some(Box::new(hash_error)),
        }
    }
}

impl From<FileError> for AppError {
    fn from(file_error: FileError) -> AppError {
        AppError::Categorized {
            kind: AppErrorKind::FileError,
            message: file_error.message.clone(),
            source: Some(Box::new(file_error)),
        }
    }
}

impl From<DbError> for AppError {
    fn from(db_error: DbError) -> AppError {
        AppError::Categorized {
            kind: AppErrorKind::DbError,
            message: db_error.message.clone(),
            source: Some(Box::new(db_error)),
        }
    }
}
