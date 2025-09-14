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
    CreationTimeout,
    DeletionTimeout,
    Io,
    InvalidSharePath,
    ConfigCreationError,
    LastLibraryNotFound,
}
