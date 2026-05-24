use crate::SerializableError;
use thiserror::Error;

#[derive(Debug, Error, SerializableError)]
pub enum ThumbnailServiceError {
    /// The decoding of the image has failed (most likely in engine)
    #[error("Image decoding failed: {0}")]
    ImageDecode(#[from] image::ImageError),

    #[error("The thumbnail is not in the database. Is it already generated?")]
    ThumbnailNotFound,

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported file type: {mime_type}")]
    UnsupportedFileType { mime_type: String },

    #[error("Unsupported Thumbnail size requested: {requested_size}")]
    UnsupportedThumbnailSize { requested_size: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Thumbnail generation failed: {reason}")]
    GenerationFailed { reason: String },

    #[error("File ID for thumbnail generation not provided")]
    FileIdNotProvided,
}
