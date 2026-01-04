use anyhow::{Context, Result};
use entity::files;
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
