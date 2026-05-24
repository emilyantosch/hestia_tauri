use crate::SerializableError;
use thiserror::Error;

#[derive(Debug, Error, SerializableError)]
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
    #[error("The config for the library could not be created!")]
    ConfigCreationFailed,
    #[error("The config for the library could not be deleted! Reason: {error}")]
    ConfigDeletionError { error: String },
    #[error("There has been no last library detected, prompting for a new one...")]
    LastLibraryNotFound,
    #[error("The OS has no known configuration for a data home directory!")]
    DataHomeNotFoundError,
    #[error("An internal error occurred: {0}")]
    Internal(#[from] anyhow::Error),
}