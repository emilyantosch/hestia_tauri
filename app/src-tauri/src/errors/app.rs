use std::fmt;

use thiserror::Error;

#[derive(Debug, Clone)]
enum AppErrorKind {
    FileError,
    DbError,
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
