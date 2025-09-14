use thiserror::Error;
#[derive(Error, Debug)]
pub enum ThumbnailError {
    #[error("Image decoding failed: {0}")]
    ImageDecode(#[from] image::ImageError),

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Unsupported file type: {mime_type}")]
    UnsupportedFileType { mime_type: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Thumbnail generation failed: {reason}")]
    GenerationFailed { reason: String },
}
