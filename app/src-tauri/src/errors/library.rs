use thiserror::Error;

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

#[derive(Debug)]
pub enum LibraryErrorKind {
    CreationTimeout,
    DeletionTimeout,
    Io,
    InvalidSharePath,
    ConfigCreationError,
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
