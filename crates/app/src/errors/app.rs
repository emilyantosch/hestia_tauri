use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("The file watcher could not be found!")]
    WatcherNotFound,
    #[error("An internal error has occurred: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum AppErrorKind {
    WatcherNotFound(String),
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::WatcherNotFound => AppErrorKind::WatcherNotFound(error_message),
            Self::Internal(_) => AppErrorKind::Internal(error_message),
        };
        error_kind.serialize(serializer)
    }
}
