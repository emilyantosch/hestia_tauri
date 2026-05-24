use errors_derive::SerializableError;
use thiserror::Error;

#[derive(Error, Debug, SerializableError)]
pub enum WatcherError {
    #[error("The path {path} is not being watched and cannot be unwatched!")]
    PathNotWatched { path: String },
    #[error("The operation could not be completed because of an IO failure! Cause: {0}")]
    Io(#[from] std::io::Error),
    #[error("The path to be watcher or that interacts with the watcher could not be found!")]
    PathNotFound,
    #[error("The event emitted by the watcher is not recognized!")]
    InvalidEventType,
}

#[derive(Debug, Error, SerializableError)]
pub enum FileError {
    #[error("Generic file error: {path}")]
    GenericError { path: String },
    #[error("Input/Output error: {path}")]
    Io { path: String },
    #[error("Path could not be found: {path}")]
    PathNotFoundError { path: String },
}

#[derive(Debug, Error, SerializableError)]
pub enum ScannerError {
    #[error("The scan of path {path} failed!")]
    PathScanFailedError { path: String },
    #[error("The scan of path {path} failed!")]
    DatabaseState { path: String },
}
