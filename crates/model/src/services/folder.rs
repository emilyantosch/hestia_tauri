use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use hash::hash::FolderHash;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct FileSystemFolder {
    pub id: Option<i32>,
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub structure_hash: String,
    pub file_system_id: Option<i32>,
    pub parent_folder_id: Option<i32>,
}

impl FileSystemFolder {
    pub async fn create_folder_info(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .with_context(|| format!("path {} has no valid folder name", path.display()))?
            .to_string();
        let folder_hash = FolderHash::hash(path).await?;

        Ok(Self {
            id: None,
            path: path.to_path_buf(),
            name,
            content_hash: folder_hash.content_hash.to_string(),
            identity_hash: folder_hash.identity_hash.to_string(),
            structure_hash: folder_hash.structure_hash.to_string(),
            file_system_id: None,
            parent_folder_id: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PersistedFolder {
    pub id: i32,
    pub file_system_id: i32,
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub structure_hash: String,
    pub parent_folder_id: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}
