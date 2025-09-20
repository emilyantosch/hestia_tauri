use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum ScannerError {
    #[error("The scan of path {path} failed!")]
    PathScanFailedError { path: String },
    #[error("The scan of path {path} failed!")]
    DatabaseState { path: String },
}
