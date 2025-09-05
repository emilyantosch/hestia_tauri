use serde::{ser::SerializeStruct, Serialize};
use thiserror::Error;

use crate::errors::FileError;

#[derive(Debug, Error)]
pub struct LibraryError {
    kind: LibraryErrorKind,
    message: String,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl std::fmt::Display for LibraryErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::Serialize for LibraryError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.source.as_ref() {
            Some(source) => {
                let mut s = serializer.serialize_struct("LibraryError", 3)?;
                s.serialize_field("LibraryErrorKind", &self.kind)?;
                s.serialize_field("Message", &self.message)?;
                s.serialize_field("Source", &source.to_string())?;
                s.end()
            }
            None => {
                let mut s = serializer.serialize_struct("LibraryError", 2)?;
                s.serialize_field("LibraryErrorKind", &self.kind)?;
                s.serialize_field("Message", &self.message)?;
                s.end()
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub enum LibraryErrorKind {
    CreationTimeout,
    DeletionTimeout,
    Io,
    InvalidSharePath,
    ConfigCreationError,
    LastLibraryNotFound,
}

impl From<FileError> for LibraryError {
    fn from(value: FileError) -> Self {
        LibraryError::new(LibraryErrorKind::Io, value.message)
    }
}

impl From<std::io::Error> for LibraryError {
    fn from(other: std::io::Error) -> Self {
        LibraryError {
            kind: LibraryErrorKind::Io,
            message: format!("An IO error occured: {other:#?}"),
            source: Some(Box::new(other)),
        }
    }
}
impl LibraryError {
    pub fn new(kind: LibraryErrorKind, message: String) -> LibraryError {
        LibraryError {
            kind,
            message,
            source: None,
        }
    }

    pub fn with_source(
        kind: LibraryErrorKind,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> LibraryError {
        LibraryError {
            kind,
            message,
            source,
        }
    }
}
