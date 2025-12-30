use entity::folders;
use serde::{Deserialize, Serialize};

/// Folder information for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub parent_folder_id: Option<i32>,
    pub content_hash: String,
    pub identity_hash: String,
    pub structure_hash: String,
    pub file_system_id: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<folders::Model> for FolderInfo {
    fn from(folder: folders::Model) -> Self {
        Self {
            id: folder.id,
            name: folder.name,
            path: folder.path,
            parent_folder_id: folder.parent_folder_id,
            content_hash: folder.content_hash,
            identity_hash: folder.identity_hash,
            structure_hash: folder.structure_hash,
            file_system_id: folder.file_system_id,
            created_at: folder.created_at.to_string(),
            updated_at: folder.updated_at.to_string(),
        }
    }
}
