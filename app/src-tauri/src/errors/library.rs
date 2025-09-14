use serde::Serialize;
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

#[derive(Debug, Serialize, Error)]
pub enum LibraryError {
    #[error("The creation of the library has timed out!")]
    CreationTimeout,
    #[error("The deletion of the library has timed out!")]
    DeletionTimeout,
    #[error("There has been an Input/Output issue!")]
    Io,
    #[error("The provided share path is invalid!")]
    InvalidSharePath,
    #[error("The config for the library could not be created!")]
    ConfigCreationError,
    #[error("There has been no last library detected, prompting for a new one...")]
    LastLibraryNotFound,
    #[error("The OS has no known configuration for a data home directory!")]
    DataHomeNotFoundError,
}
