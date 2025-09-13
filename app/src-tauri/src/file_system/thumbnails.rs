use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;
use thiserror::Error;
use entity::thumbnails;
use sea_orm::{Set, ActiveValue};
use chrono::Utc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThumbnailSize {
    Small,
    Medium,
    Large,
}

impl ThumbnailSize {
    pub const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Small => (128, 128),
            Self::Medium => (256, 256),
            Self::Large => (512, 512),
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    pub const fn all() -> [ThumbnailSize; 3] {
        [Self::Small, Self::Medium, Self::Large]
    }
}

impl fmt::Display for ThumbnailSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<ThumbnailSize> for String {
    fn from(size: ThumbnailSize) -> Self {
        size.as_str().to_string()
    }
}

impl TryFrom<&str> for ThumbnailSize {
    type Error = ThumbnailError;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "small" => Ok(ThumbnailSize::Small),
            "medium" => Ok(ThumbnailSize::Medium),
            "large" => Ok(ThumbnailSize::Large),
            _ => Err(ThumbnailError::GenerationFailed {
                reason: format!("Invalid thumbnail size: {}", value),
            }),
        }
    }
}

impl TryFrom<String> for ThumbnailSize {
    type Error = ThumbnailError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

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

pub type ThumbnailResult<T> = std::result::Result<T, ThumbnailError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thumbnail {
    pub size: ThumbnailSize,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub file_size: usize,
}

impl Thumbnail {
    pub fn new(size: ThumbnailSize, data: Vec<u8>, mime_type: String) -> Self {
        let file_size = data.len();
        Self {
            size,
            data,
            mime_type,
            file_size,
        }
    }
    
    pub fn with_image_data(size: ThumbnailSize, data: Vec<u8>) -> Self {
        Self::new(size, data, "image/png".to_string())
    }
    
    pub fn size(&self) -> ThumbnailSize {
        self.size
    }
    
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }
    
    pub fn file_size(&self) -> usize {
        self.file_size
    }
    
    pub fn dimensions(&self) -> (u32, u32) {
        self.size.dimensions()
    }
    
    /// Converts to SeaORM ActiveModel for database insertion
    pub fn to_active_model(self, file_id: i32) -> thumbnails::ActiveModel {
        let now = Utc::now();
        
        thumbnails::ActiveModel {
            id: ActiveValue::NotSet,
            file_id: Set(file_id),
            size: Set(self.size.to_string()),
            data: Set(self.data),
            mime_type: Set(self.mime_type),
            file_size: Set(self.file_size as i32),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
    }
    
    /// Creates thumbnail from SeaORM Model
    pub fn from_model(model: thumbnails::Model) -> Result<Self, ThumbnailError> {
        let size = ThumbnailSize::try_from(model.size)?;
        
        Ok(Self {
            size,
            data: model.data,
            mime_type: model.mime_type,
            file_size: model.file_size as usize,
        })
    }
    
    /// Returns true if this is an image thumbnail (not a file icon)
    pub fn is_image(&self) -> bool {
        self.mime_type == "image/png" || self.mime_type.starts_with("image/")
    }
}

pub struct ThumbnailGenerator {
    filter_type: FilterType,
}

impl ThumbnailGenerator {
    pub fn new() -> Self {
        Self {
            filter_type: FilterType::Lanczos3, // High quality resizing
        }
    }

    pub fn with_filter(filter_type: FilterType) -> Self {
        Self { filter_type }
    }

    pub fn generate_image_thumbnail(
        &self,
        image_data: &[u8],
        size: ThumbnailSize,
    ) -> ThumbnailResult<Thumbnail> {
        let img = image::load_from_memory(image_data).context("Failed to decode image")?;

        let (target_width, target_height) = size.dimensions();

        // Preserve aspect ratio by using thumbnail() instead of resize()
        let thumbnail = img.thumbnail(target_width, target_height);

        // Encode as PNG for consistent output
        let mut output = Vec::new();
        thumbnail
            .write_to(&mut std::io::Cursor::new(&mut output), ImageFormat::Png)
            .context("Failed to encode thumbnail")?;

        Ok(Thumbnail::with_image_data(size, output))
    }

    pub fn generate_from_file_path(
        &self,
        file_path: &Path,
        size: ThumbnailSize,
    ) -> ThumbnailResult<Thumbnail> {
        // Check if file exists first
        if !file_path.exists() {
            return Err(ThumbnailError::FileNotFound {
                path: file_path.display().to_string(),
            });
        }

        // Detect file type first
        let file_data = std::fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let mime_type = infer::get(&file_data)
            .map(|kind| kind.mime_type())
            .unwrap_or("application/octet-stream");

        match mime_type {
            mime if mime.starts_with("image/") => self.generate_image_thumbnail(&file_data, size),
            _ => self.generate_file_icon(mime_type, size),
        }
    }

    fn generate_file_icon(
        &self,
        mime_type: &str,
        size: ThumbnailSize,
    ) -> ThumbnailResult<Thumbnail> {
        let (width, height) = size.dimensions();
        let mut img = ImageBuffer::new(width, height);

        // Generate themed background based on file type
        let bg_color = self.get_file_type_color(mime_type);

        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = bg_color;
        }

        // TODO: Add file type icon/text overlay in future iteration
        let mut output = Vec::new();
        img.save_with_format(&mut std::io::Cursor::new(&mut output), ImageFormat::Png)
            .context("Failed to encode file icon")?;

        Ok(Thumbnail::new(size, output, "image/png".to_string()))
    }

    fn get_file_type_color(&self, mime_type: &str) -> Rgba<u8> {
        match mime_type {
            mime if mime.starts_with("text/") => Rgba([74, 144, 226, 255]), // Blue
            mime if mime.starts_with("application/pdf") => Rgba([231, 76, 60, 255]), // Red
            mime if mime.starts_with("video/") => Rgba([155, 89, 182, 255]), // Purple
            mime if mime.starts_with("audio/") => Rgba([46, 204, 113, 255]), // Green
            _ => Rgba([149, 165, 166, 255]),                                // Gray for unknown
        }
    }
}

impl Default for ThumbnailGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_size_dimensions() {
        assert_eq!(ThumbnailSize::Small.dimensions(), (128, 128));
        assert_eq!(ThumbnailSize::Medium.dimensions(), (256, 256));
        assert_eq!(ThumbnailSize::Large.dimensions(), (512, 512));
    }

    #[test]
    fn test_thumbnail_size_as_str() {
        assert_eq!(ThumbnailSize::Small.as_str(), "small");
        assert_eq!(ThumbnailSize::Medium.as_str(), "medium");
        assert_eq!(ThumbnailSize::Large.as_str(), "large");
    }

    #[test]
    fn test_thumbnail_size_all() {
        let all_sizes = ThumbnailSize::all();
        assert_eq!(all_sizes.len(), 3);
        assert!(all_sizes.contains(&ThumbnailSize::Small));
        assert!(all_sizes.contains(&ThumbnailSize::Medium));
        assert!(all_sizes.contains(&ThumbnailSize::Large));
    }

    #[test]
    fn test_thumbnail_generator_creation() {
        let generator = ThumbnailGenerator::new();
        assert_eq!(generator.filter_type, FilterType::Lanczos3);

        let generator_default = ThumbnailGenerator::default();
        assert_eq!(generator_default.filter_type, FilterType::Lanczos3);

        let generator_custom = ThumbnailGenerator::with_filter(FilterType::Nearest);
        assert_eq!(generator_custom.filter_type, FilterType::Nearest);
    }

    #[test]
    fn test_thumbnail_struct() {
        let data = vec![1, 2, 3, 4, 5];
        let mime_type = "image/png".to_string();
        let size = ThumbnailSize::Medium;

        let thumbnail = Thumbnail::new(size, data.clone(), mime_type.clone());
        
        assert_eq!(thumbnail.size(), size);
        assert_eq!(thumbnail.data(), &data[..]);
        assert_eq!(thumbnail.mime_type(), &mime_type);
        assert_eq!(thumbnail.file_size(), data.len());
        assert_eq!(thumbnail.dimensions(), (256, 256));
    }

    #[test]
    fn test_thumbnail_with_image_data() {
        let data = vec![10, 20, 30];
        let size = ThumbnailSize::Large;

        let thumbnail = Thumbnail::with_image_data(size, data.clone());
        
        assert_eq!(thumbnail.size(), size);
        assert_eq!(thumbnail.data(), &data[..]);
        assert_eq!(thumbnail.mime_type(), "image/png");
        assert_eq!(thumbnail.file_size(), data.len());
        assert_eq!(thumbnail.dimensions(), (512, 512));
    }

    #[test]
    fn test_thumbnail_seaorm_conversion() {
        use chrono::Utc;
        
        let original_data = vec![1, 2, 3, 4, 5];
        let size = ThumbnailSize::Small;
        let mime_type = "image/jpeg".to_string();
        let file_size = original_data.len();
        let file_id = 42;

        // Create thumbnail
        let thumbnail = Thumbnail::new(size, original_data.clone(), mime_type.clone());
        
        // Test to_active_model
        let active_model = thumbnail.clone().to_active_model(file_id);
        assert_eq!(active_model.file_id.clone().unwrap(), file_id);
        assert_eq!(active_model.size.clone().unwrap(), "small");
        assert_eq!(active_model.data.clone().unwrap(), original_data);
        assert_eq!(active_model.mime_type.clone().unwrap(), mime_type);
        assert_eq!(active_model.file_size.clone().unwrap(), file_size as i32);
        
        // Create a mock Model for testing from_model
        let now = Utc::now();
        let model = thumbnails::Model {
            id: 1,
            file_id,
            size: "small".to_string(),
            data: original_data.clone(),
            mime_type: mime_type.clone(),
            file_size: file_size as i32,
            created_at: now.into(),
            updated_at: now.into(),
        };
        
        // Test from_model
        let restored_thumbnail = Thumbnail::from_model(model).unwrap();
        assert_eq!(restored_thumbnail.size(), size);
        assert_eq!(restored_thumbnail.data(), &original_data[..]);
        assert_eq!(restored_thumbnail.mime_type(), &mime_type);
        assert_eq!(restored_thumbnail.file_size(), file_size);
    }

    #[test]
    fn test_thumbnail_size_conversions() {
        // Test TryFrom<&str>
        assert_eq!(ThumbnailSize::try_from("small").unwrap(), ThumbnailSize::Small);
        assert_eq!(ThumbnailSize::try_from("medium").unwrap(), ThumbnailSize::Medium);
        assert_eq!(ThumbnailSize::try_from("large").unwrap(), ThumbnailSize::Large);
        assert!(ThumbnailSize::try_from("invalid").is_err());
        
        // Test TryFrom<String>
        assert_eq!(ThumbnailSize::try_from("small".to_string()).unwrap(), ThumbnailSize::Small);
        assert!(ThumbnailSize::try_from("invalid".to_string()).is_err());
        
        // Test From<ThumbnailSize> for String
        let size_str: String = ThumbnailSize::Medium.into();
        assert_eq!(size_str, "medium");
    }

    #[test]
    fn test_thumbnail_is_image() {
        let png_thumbnail = Thumbnail::with_image_data(ThumbnailSize::Small, vec![1, 2, 3]);
        assert!(png_thumbnail.is_image());
        
        let jpeg_thumbnail = Thumbnail::new(ThumbnailSize::Medium, vec![1, 2, 3], "image/jpeg".to_string());
        assert!(jpeg_thumbnail.is_image());
        
        let pdf_thumbnail = Thumbnail::new(ThumbnailSize::Large, vec![1, 2, 3], "application/pdf".to_string());
        assert!(!pdf_thumbnail.is_image());
    }
}

