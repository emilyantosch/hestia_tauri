use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("The file watcher could not be found!")]
    WatcherNotFound,
    #[error("The thumbnail message handler could not be found!")]
    ThumbnailMessageHandlerNotFound,
    #[error("An internal error has occurred: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum StateErrorKind {
    WatcherNotFound(String),
    ThumbnailMessageHandlerNotFound(String),
    Internal(String),
}

impl Serialize for StateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::WatcherNotFound => StateErrorKind::WatcherNotFound(error_message),
            Self::ThumbnailMessageHandlerNotFound => {
                StateErrorKind::ThumbnailMessageHandlerNotFound(error_message)
            }
            Self::Internal(_) => StateErrorKind::Internal(error_message),
        };
        error_kind.serialize(serializer)
    }
}
