use thiserror::Error;

#[derive(Error, Debug)]
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
