use serde::{Deserialize, Serialize};
use thiserror::Error;

// #[derive(Debug, Error)]
// pub struct LibraryError {
//     kind: LibraryErrorKind,
//     message: String,
//     #[source]
//     source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
// }

//
// impl serde::Serialize for LibraryError {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self.source.as_ref() {
//             Some(source) => {
//                 let mut s = serializer.serialize_struct("LibraryError", 3)?;
//                 s.serialize_field("LibraryErrorKind", &self.kind)?;
//                 s.serialize_field("Message", &self.message)?;
//                 s.serialize_field("Source", &source.to_string())?;
//                 s.end()
//             }
//             None => {
//                 let mut s = serializer.serialize_struct("LibraryError", 2)?;
//                 s.serialize_field("LibraryErrorKind", &self.kind)?;
//                 s.serialize_field("Message", &self.message)?;
//                 s.end()
//             }
//         }
//     }
// }

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("The creation of the library has timed out!")]
    CreationTimeout,
    #[error("The deletion of the library has timed out!")]
    DeletionTimeout,
    #[error("There has been an Input/Output issue!")]
    Io,
    #[error("The provided share path is invalid!")]
    InvalidSharePath,
    #[error("The config for the library could not be created! Reason: {error}")]
    ConfigCreationError { error: String },
    #[error("The config for the library could not be deleted! Reason: {error}")]
    ConfigDeletionError { error: String },
    #[error("There has been no last library detected, prompting for a new one...")]
    LastLibraryNotFound,
    #[error("The OS has no known configuration for a data home directory!")]
    DataHomeNotFoundError,
    #[error("The OS has no known configuration for a data home directory!")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum LibraryErrorKind {
    CreationTimeout(String),
    DeletionTimeout(String),
    Io(String),
    InvalidSharePath(String),
    ConfigCreationError(String),
    ConfigDeletionError(String),
    LastLibraryNotFound(String),
    DataHomeNotFoundError(String),
    Internal(String),
}

impl Serialize for LibraryError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::CreationTimeout => LibraryErrorKind::CreationTimeout(error_message),
            Self::DeletionTimeout => LibraryErrorKind::DeletionTimeout(error_message),
            Self::Io => LibraryErrorKind::Io(error_message),
            Self::InvalidSharePath => LibraryErrorKind::InvalidSharePath(error_message),
            Self::ConfigCreationError { error } => {
                LibraryErrorKind::ConfigCreationError(error_message)
            }
            Self::ConfigDeletionError { error } => {
                LibraryErrorKind::ConfigDeletionError(error_message)
            }
            Self::LastLibraryNotFound => LibraryErrorKind::LastLibraryNotFound(error_message),
            Self::DataHomeNotFoundError => LibraryErrorKind::DataHomeNotFoundError(error_message),
            Self::Internal(_) => LibraryErrorKind::Internal(error_message),
        };
        error_kind.serialize(serializer)
    }
}
