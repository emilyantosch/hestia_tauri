use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileSystemFolder {
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub structure_hash: String,
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
