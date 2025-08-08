use blake3::Hash;

use super::AppError;

#[derive(Debug)]
pub struct HashError {
    pub kind: HashErrorKind,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

#[derive(Debug)]
pub enum HashErrorKind {
    IoError,
    InvalidPathError,
    PermissionDeniedError,
}

impl HashError {
    pub fn new(kind: HashErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            source: None,
        }
    }

    pub fn with_source<E>(kind: HashErrorKind, message: String, source: E) -> Self
    where
        E: std::error::Error + Sync + Send + 'static,
    {
        Self {
            kind,
            message,
            source: Some(Box::new(source)),
        }
    }
}

impl std::fmt::Display for HashErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HashError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

impl From<std::io::Error> for HashError {
    fn from(other: std::io::Error) -> HashError {
        HashError::with_source(
            HashErrorKind::IoError,
            format!("An IO error occured: {:?}", other.to_string()),
            other,
        )
    }
}
