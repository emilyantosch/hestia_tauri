use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum HashError {
    #[error("An IO error occurred while trying to hash file or folder!")]
    IoError,
    #[error("The path to be hashed is invalid!")]
    InvalidPathError,
    #[error("The operation could not be completed due to insufficient permissions!")]
    PermissionDeniedError,
}
