use crate::file_system::FolderHash;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Folder {
    pub id: Option<i32>,
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub structure_hash: String,
    pub file_system_id: Option<i32>,
    pub parent_folder_id: Option<i32>,
}

impl Folder {
    /// Create FolderInfo from a filesystem path
    pub async fn create_folder_info(path: &Path) -> Result<Folder> {
        let metadata = std::fs::metadata(path)?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Calculate file hash using sophisticated algorithm
        let folder_hash = FolderHash::hash(path).await?;
        let content_hash_str = format!("{:?}", folder_hash.content_hash);
        let identity_hash_str = format!("{:?}", folder_hash.identity_hash);
        let structure_hash_str = format!("{:?}", folder_hash.structure_hash);

        Ok(Folder {
            id: None,
            path: path.to_path_buf(),
            name,
            content_hash: content_hash_str,
            identity_hash: identity_hash_str,
            structure_hash: structure_hash_str,
            file_system_id: None,   // Will be set during database operations
            parent_folder_id: None, // Will be set during database operations
        })
    }
}
