use anyhow::{Context, Result};
use entity::files;
use hash::hash::FileHash;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileSystemFile {
    pub id: Option<i32>,
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_type_name: String,
    pub file_system_id: Option<i32>,
}

impl From<files::Model> for FileSystemFile {
    fn from(value: files::Model) -> Self {
        let path = PathBuf::from(&value.path);
        let file_type_name = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
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

impl FileSystemFile {
    pub async fn create_file_info_from_path(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .with_context(|| format!("path {} has no valid file name", path.display()))?
            .to_string();
        let file_hash = FileHash::hash(path).await?;

        Ok(Self {
            id: None,
            path: path.to_path_buf(),
            name,
            content_hash: file_hash.content_hash.to_string(),
            identity_hash: file_hash.identity_hash.to_string(),
            file_type_name: infer::get_from_path(path)?.map_or_else(
                || "unknown".to_string(),
                |kind| kind.mime_type().to_string(),
            ),
            file_system_id: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PersistedFile {
    pub id: i32,
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_system_id: i32,
}

impl From<files::Model> for PersistedFile {
    fn from(value: files::Model) -> Self {
        let path = PathBuf::from(&value.path);
        Self {
            id: value.id,
            path,
            name: value.name,
            content_hash: value.content_hash,
            identity_hash: value.identity_hash,
            file_system_id: value.file_system_id,
        }
    }
}
