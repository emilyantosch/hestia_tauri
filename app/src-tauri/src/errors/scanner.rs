use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScannerError {
    #[error("The scan of path {path} failed!")]
    PathScanFailedError { path: String },
}
