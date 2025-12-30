use crate::file_system::FileHash;
use crate::{data::file, errors::FileError};
use anyhow::{Context, Result};
use entity::files;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct File {
    pub id: Option<i32>,
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_type_name: String,
    pub file_system_id: Option<i32>,
}

impl From<files::Model> for File {
    fn from(value: files::Model) -> Self {
        let path = PathBuf::from(&value.path);
        let file_type_name = File::detect_file_type(path.as_path());
        File {
            id: Some(value.id),
            path,
            name: value.name,
            content_hash: value.content_hash,
            identity_hash: value.identity_hash,
            file_type_name,
            file_system_id: Some(value.file_system_id),
        }
    }
}

impl File {
    /// Create FileInfo from a filesystem path
    pub async fn create_file_info_from_path(path: &Path) -> Result<File> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Calculate file hash using sophisticated algorithm
        let file_hash = FileHash::hash(path).await?;
        let content_hash_str = format!("{:?}", file_hash.content_hash);
        let identity_hash_str = format!("{:?}", file_hash.identity_hash);

        // Detect file type
        let file_type_name = Self::detect_file_type(path);

        Ok(File {
            id: None,
            path: path.to_path_buf(),
            name,
            content_hash: content_hash_str,
            identity_hash: identity_hash_str,
            file_type_name,
            file_system_id: None, // Will be set during database operations
        })
    }

    fn detect_file_type(file_path: &Path) -> String {
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => {
                let ext_lower = ext.to_lowercase();
                match ext_lower.as_str() {
                    // Document types
                    "md" | "markdown" => "markdown",
                    "txt" => "text",
                    "pdf" => "pdf",
                    "doc" | "docx" => "document",
                    "xls" | "xlsx" => "spreadsheet",
                    "ppt" | "pptx" => "presentation",

                    // Image types
                    "jpg" | "jpeg" => "image_jpeg",
                    "png" => "image_png",
                    "gif" => "image_gif",
                    "svg" => "image_svg",
                    "webp" => "image_webp",
                    "bmp" => "image_bmp",

                    // Video types
                    "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" => "video",

                    // Audio types
                    "mp3" | "wav" | "flac" | "ogg" | "aac" => "audio",

                    // Code types
                    "rs" => "rust",
                    "js" | "ts" => "javascript",
                    "py" => "python",
                    "java" => "java",
                    "cpp" | "cc" | "cxx" => "cpp",
                    "c" => "c",
                    "h" | "hpp" => "header",
                    "html" | "htm" => "html",
                    "css" => "css",
                    "json" => "json",
                    "xml" => "xml",
                    "yaml" | "yml" => "yaml",
                    "toml" => "toml",

                    // Archive types
                    "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "archive",

                    // Default
                    _ => {
                        return format!("ext_{}", ext_lower);
                    }
                }
                .to_string()
            }
            None => {
                // Check if it's a directory
                if file_path.is_dir() {
                    "directory".to_string()
                } else {
                    "unknown".to_string()
                }
            }
        }
    }
}
