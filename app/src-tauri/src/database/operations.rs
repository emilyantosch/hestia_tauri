use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, QueryFilter, Set, TransactionTrait,
};
use tokio::sync::RwLock;

use entity::{file_system_identifier, file_types, files, folders, prelude::*};

use crate::database::DatabaseManager;
use crate::errors::{DbError, DbErrorKind};
use crate::file_system::{FileEvent, FolderEvent};

/// File information for bulk operations
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_type_name: String,
    pub file_system_id: Option<i32>,
}

/// Database file metadata for comparison
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub id: i32,
    pub path: PathBuf,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_system_id: i32,
    pub updated_at: chrono::NaiveDateTime,
}

/// Database operations for file management with caching and bulk operations
pub struct FileOperations {
    database_manager: Arc<DatabaseManager>,
    file_type_cache: Arc<RwLock<HashMap<String, i32>>>,
}

impl FileOperations {
    pub fn new(database_manager: Arc<DatabaseManager>) -> Self {
        Self {
            database_manager,
            file_type_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create_parent_folder_id<C: ConnectionTrait>(
        &self,
        folder_path: &PathBuf,
        transaction: &C,
    ) -> Result<i32, DbError> {
        let folder_with_pfi = Folders::find()
            .filter(folders::Column::Path.eq(folder_path.to_string_lossy().to_string()))
            .one(transaction)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to find parent folder id due to db error".to_string(),
                    e,
                )
            })?;

        let parent_folder_id = if let Some(existing) = folder_with_pfi {
            existing.parent_folder_id
        } else {
            let parent_folder = match folder_path.parent() {
                Some(path) => path,
                None => {
                    return Err(DbError { kind: DbErrorKind::QueryError, message: "This is already the root folder of the system and therefore has no parent folder".to_string(), source: None})
                }
            };

            match Folders::find()
                .filter(folders::Column::Path.eq(parent_folder.to_string_lossy().to_string()))
                .one(transaction)
                .await
                .map_err(|e| {
                    DbError::with_source(
                        DbErrorKind::QueryError,
                        "Failed to find parent folder".to_string(),
                        e,
                    )
                })? {
                Some(model) => model.id,
                None => {
                    return Err(DbError::new(
                        DbErrorKind::QueryError,
                        "Failed to find model from database".to_string(),
                    ))
                }
            }
        };

        Ok(parent_folder_id)
    }

    pub async fn find_folder_by_id(&self, fsi_id: i32) -> Result<Option<folders::Model>, DbError> {
        let connection = self.database_manager.get_connection();
        let folder = Folders::find()
            .filter(folders::Column::Id.eq(fsi_id))
            .one(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    format!("Could not find folder with id: {}", fsi_id),
                    e,
                )
            })?;

        Ok(folder)
    }

    pub async fn find_root_folders(&self) -> Result<Vec<folders::Model>, DbError> {
        let connection = self.database_manager.get_connection();
        let root_folders = Folders::find()
            .filter(folders::Column::ParentFolderId.is_null())
            .all(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Could not find root_folders with id: {}".to_string(),
                    e,
                )
            })?;

        Ok(root_folders)
    }

    pub async fn find_subfolders_of_folder(
        &self,
        folder_id: i32,
    ) -> Result<Vec<folders::Model>, DbError> {
        let connection = self.database_manager.get_connection();
        let subfolders = Folders::find()
            .filter(folders::Column::ParentFolderId.eq(folder_id))
            .all(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    format!(
                        "Could not find subfolders for folder with id: {}",
                        folder_id
                    ),
                    e,
                )
            })?;

        Ok(subfolders)
    }

    pub async fn upsert_folder_from_event(
        &self,
        event: &FolderEvent,
    ) -> Result<folders::Model, DbError> {
        let connection = self.database_manager.get_connection();
        let transaction = connection.begin().await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::TransactionError,
                "Failed to start database transaction".to_string(),
                e,
            )
        })?;

        let folder_path = match event.paths.last() {
            Some(path) => path,
            None => {
                return Err(DbError::new(
                    DbErrorKind::ConfigurationError,
                    "The last of the paths could not be extracted. Does the path exist?"
                        .to_string(),
                ))
            }
        };

        let folder_name = folder_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let path_str = folder_path.to_string_lossy().to_string();

        // Get or create file type
        let file_type_id = self
            .get_or_create_file_type(&folder_path, &transaction)
            .await?;

        // Get proper content and identity hashes from the FileHash struct
        let content_hash_str = format!("{:?}", event.hash.content_hash);
        let identity_hash_str = format!("{:?}", event.hash.identity_hash);
        let structure_hash_str = format!("{:?}", event.hash.structure_hash);

        // Get file system identifier
        let file_system_id = self
            .get_or_create_file_system_identifier(&folder_path, &transaction)
            .await?;

        let parent_folder_id = self
            .get_or_create_parent_folder_id(&folder_path, &transaction)
            .await?;

        //  What we actually wanna do is check if the file exists by fsi and/or hash.
        let folder_with_fsi = Folders::find()
            .filter(folders::Column::FileSystemId.eq(file_system_id))
            .one(&transaction)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to find any files".to_string(),
                    e,
                )
            })?;

        let folder_model = if let Some(existing) = folder_with_fsi {
            // Update existing file
            let mut active_model = existing.into_active_model();
            active_model.name = Set(folder_name);
            active_model.path = Set(path_str);
            active_model.content_hash = Set(content_hash_str);
            active_model.identity_hash = Set(identity_hash_str);
            active_model.structure_hash = Set(structure_hash_str);
            active_model.parent_folder_id = Set(parent_folder_id);
            active_model.file_system_id = Set(file_system_id);
            active_model.updated_at = Set(chrono::Local::now().naive_local());

            active_model.update(&transaction).await.map_err(|e| {
                DbError::with_source(
                    DbErrorKind::UpdateError,
                    "Failed to update file record".to_string(),
                    e,
                )
            })?
        } else {
            // Insert new file
            let new_file = folders::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                name: Set(folder_name),
                path: Set(path_str),
                parent_folder_id: Set(parent_folder_id),
                content_hash: Set(content_hash_str),
                identity_hash: Set(identity_hash_str),
                structure_hash: Set(structure_hash_str),
                file_system_id: Set(file_system_id),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            };

            new_file.insert(&transaction).await.map_err(|e| {
                DbError::with_source(
                    DbErrorKind::InsertError,
                    "Failed to insert file record".to_string(),
                    e,
                )
            })?
        };

        transaction.commit().await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::TransactionError,
                "Failed to commit transaction".to_string(),
                e,
            )
        })?;

        Ok(folder_model)
    }
    /// Insert or update a file in the database based on FileEvent
    pub async fn upsert_file_from_event(&self, event: &FileEvent) -> Result<files::Model, DbError> {
        let connection = self.database_manager.get_connection();
        let transaction = connection.begin().await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::TransactionError,
                "Failed to start database transaction".to_string(),
                e,
            )
        })?;

        // Extract file information from the event
        let file_path = match event.paths.last() {
            Some(path) => path,
            None => {
                return Err(DbError::new(
                    DbErrorKind::ConfigurationError,
                    String::from(
                        "The last of the paths could not be extracted, no paths were provided.",
                    ),
                ));
            }
        };
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let path_str = file_path.to_string_lossy().to_string();

        // Get or create file type
        let file_type_id = self
            .get_or_create_file_type(&file_path, &transaction)
            .await?;

        // Get proper content and identity hashes from the FileHash struct
        let content_hash_str = format!("{:?}", event.hash.content_hash);
        let identity_hash_str = format!("{:?}", event.hash.identity_hash);

        // Get file system identifier
        let file_system_id = self
            .get_or_create_file_system_identifier(&file_path, &transaction)
            .await?;

        //  What we actually wanna do is check if the file exists by fsi and/or hash.
        let file_with_fsi = Files::find()
            .filter(files::Column::FileSystemId.eq(file_system_id))
            .one(&transaction)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to find any files".to_string(),
                    e,
                )
            })?;

        let file_model = if let Some(existing) = file_with_fsi {
            // Update existing file
            let mut active_model = existing.into_active_model();
            active_model.name = Set(file_name);
            active_model.path = Set(path_str);
            active_model.content_hash = Set(content_hash_str);
            active_model.identity_hash = Set(identity_hash_str);
            active_model.file_type_id = Set(file_type_id);
            active_model.file_system_id = Set(file_system_id);
            active_model.updated_at = Set(chrono::Local::now().naive_local());

            active_model.update(&transaction).await.map_err(|e| {
                DbError::with_source(
                    DbErrorKind::UpdateError,
                    "Failed to update file record".to_string(),
                    e,
                )
            })?
        } else {
            // Insert new file
            let new_file = files::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                name: Set(file_name),
                path: Set(path_str),
                content_hash: Set(content_hash_str),
                identity_hash: Set(identity_hash_str),
                file_type_id: Set(file_type_id),
                file_system_id: Set(file_system_id),
                created_at: Set(chrono::Utc::now().naive_utc()),
                updated_at: Set(chrono::Utc::now().naive_utc()),
            };

            new_file.insert(&transaction).await.map_err(|e| {
                DbError::with_source(
                    DbErrorKind::InsertError,
                    "Failed to insert file record".to_string(),
                    e,
                )
            })?
        };

        transaction.commit().await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::TransactionError,
                "Failed to commit transaction".to_string(),
                e,
            )
        })?;
        Ok(file_model)
    }

    /// Delete a file record from the database
    pub async fn delete_file_by_path(&self, file_path: &Path) -> Result<bool, DbError> {
        let path_str = file_path.to_string_lossy().to_string();
        let connection = self.database_manager.get_connection();

        let result = Files::delete_many()
            .filter(files::Column::Path.eq(&path_str))
            .exec(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::DeleteError,
                    "Failed to delete file record".to_string(),
                    e,
                )
            })?;

        Ok(result.rows_affected > 0)
    }

    /// Delete a file record from the database
    pub async fn delete_folder_by_path(&self, folder_path: &Path) -> Result<bool, DbError> {
        let path_str = folder_path.to_string_lossy().to_string();
        let connection = self.database_manager.get_connection();

        let result = Folders::delete_many()
            .filter(folders::Column::Path.eq(&path_str))
            .exec(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::DeleteError,
                    "Failed to delete folder record".to_string(),
                    e,
                )
            })?;

        Ok(result.rows_affected > 0)
    }

    /// Get or create a file type based on file extension
    async fn get_or_create_file_type<C>(
        &self,
        file_path: &Path,
        connection: &C,
    ) -> Result<i32, DbError>
    where
        C: ConnectionTrait,
    {
        let file_type_name = Self::detect_file_type(file_path);

        // Check if file type already exists
        if let Some(existing_type) = FileTypes::find()
            .filter(file_types::Column::Name.eq(&file_type_name))
            .one(connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to query file type".to_string(),
                    e,
                )
            })?
        {
            return Ok(existing_type.id);
        }

        // Create new file type
        let new_file_type = file_types::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            name: Set(file_type_name),
        };

        let created_type = new_file_type.insert(connection).await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::InsertError,
                "Failed to insert file type".to_string(),
                e,
            )
        })?;

        Ok(created_type.id)
    }

    /// Detect file type based on file extension
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

    /// Get file by path
    pub async fn get_file_by_path(
        &self,
        file_path: &Path,
    ) -> Result<Option<files::Model>, DbError> {
        let path_str = file_path.to_string_lossy().to_string();
        let connection = self.database_manager.get_connection();

        Files::find()
            .filter(files::Column::Path.eq(&path_str))
            .one(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to query file by path".to_string(),
                    e,
                )
            })
    }

    /// Get all files in a directory
    pub async fn get_files_in_directory(
        &self,
        dir_path: &Path,
    ) -> Result<Vec<files::Model>, DbError> {
        let dir_str = dir_path.to_string_lossy().to_string();
        let pattern = format!("{}%", dir_str);
        let connection = self.database_manager.get_connection();

        Files::find()
            .filter(files::Column::Path.like(&pattern))
            .all(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to query files in directory".to_string(),
                    e,
                )
            })
    }

    // === BULK OPERATIONS FOR SCANNER ===

    /// Get directory state as a map for efficient comparison
    pub async fn get_directory_state(
        &self,
        dir_path: &Path,
    ) -> Result<HashMap<PathBuf, FileMetadata>, DbError> {
        let files = self.get_files_in_directory(dir_path).await?;

        let mut state = HashMap::new();
        for file in files {
            let metadata = FileMetadata {
                id: file.id,
                path: PathBuf::from(&file.path),
                content_hash: file.content_hash,
                identity_hash: file.identity_hash,
                file_system_id: file.file_system_id,
                updated_at: file.updated_at,
            };
            state.insert(PathBuf::from(file.path), metadata);
        }

        Ok(state)
    }

    /// Get file hashes as a map for quick comparison
    pub async fn get_file_hashes_map(
        &self,
        dir_path: &Path,
    ) -> Result<HashMap<PathBuf, (String, String)>, DbError> {
        let files = self.get_files_in_directory(dir_path).await?;

        let mut hashes = HashMap::new();
        for file in files {
            hashes.insert(
                PathBuf::from(file.path),
                (file.content_hash, file.identity_hash),
            );
        }

        Ok(hashes)
    }

    /// Batch insert/update files with transaction
    pub async fn batch_upsert_files(&self, files: Vec<FileInfo>) -> Result<usize, DbError> {
        if files.is_empty() {
            return Ok(0);
        }

        let connection = self.database_manager.get_connection();
        let transaction = connection.begin().await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::TransactionError,
                "Failed to start batch upsert transaction".to_string(),
                e,
            )
        })?;

        let mut processed = 0;

        for file_info in files {
            // Get or create file type (with caching)
            let file_type_id = self
                .get_or_create_file_type_cached(&file_info.file_type_name, &transaction)
                .await?;

            // Get file system identifier
            let file_system_id = if let Some(fsi_id) = file_info.file_system_id {
                fsi_id
            } else {
                self.get_or_create_file_system_identifier_with_connection(
                    &file_info.path,
                    &transaction,
                )
                .await?
            };

            // Check if file exists
            let existing_file = Files::find()
                .filter(files::Column::Path.eq(&file_info.path.to_string_lossy().to_string()))
                .one(&transaction)
                .await
                .map_err(|e| {
                    DbError::with_source(
                        DbErrorKind::QueryError,
                        "Failed to check existing file in batch upsert".to_string(),
                        e,
                    )
                })?;

            if let Some(existing) = existing_file {
                // Update existing file
                let mut active_model = existing.into_active_model();
                active_model.name = Set(file_info.name);
                active_model.content_hash = Set(file_info.content_hash);
                active_model.identity_hash = Set(file_info.identity_hash);
                active_model.file_type_id = Set(file_type_id);
                active_model.file_system_id = Set(file_system_id);
                active_model.updated_at = Set(chrono::Utc::now().naive_utc());

                active_model.update(&transaction).await.map_err(|e| {
                    DbError::with_source(
                        DbErrorKind::UpdateError,
                        "Failed to update file in batch upsert".to_string(),
                        e,
                    )
                })?;
            } else {
                // Insert new file
                let new_file = files::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    name: Set(file_info.name),
                    path: Set(file_info.path.to_string_lossy().to_string()),
                    content_hash: Set(file_info.content_hash),
                    identity_hash: Set(file_info.identity_hash),
                    file_type_id: Set(file_type_id),
                    file_system_id: Set(file_system_id),
                    created_at: Set(chrono::Utc::now().naive_utc()),
                    updated_at: Set(chrono::Utc::now().naive_utc()),
                };

                new_file.insert(&transaction).await.map_err(|e| {
                    DbError::with_source(
                        DbErrorKind::InsertError,
                        "Failed to insert file in batch upsert".to_string(),
                        e,
                    )
                })?;
            }

            processed += 1;
        }

        transaction.commit().await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::TransactionError,
                "Failed to commit batch upsert transaction".to_string(),
                e,
            )
        })?;

        Ok(processed)
    }

    /// Batch delete files by paths
    pub async fn batch_delete_files(&self, paths: Vec<PathBuf>) -> Result<usize, DbError> {
        if paths.is_empty() {
            return Ok(0);
        }

        let path_strings: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let connection = self.database_manager.get_connection();
        let result = Files::delete_many()
            .filter(files::Column::Path.is_in(path_strings))
            .exec(&*connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::DeleteError,
                    "Failed to batch delete files".to_string(),
                    e,
                )
            })?;

        Ok(result.rows_affected as usize)
    }

    /// Clear file type cache (useful for testing or cache invalidation)
    pub async fn clear_file_type_cache(&self) {
        let mut cache = self.file_type_cache.write().await;
        cache.clear();
    }

    /// Get or create file type with caching
    async fn get_or_create_file_type_cached<C>(
        &self,
        file_type_name: &str,
        connection: &C,
    ) -> Result<i32, DbError>
    where
        C: ConnectionTrait,
    {
        // Check cache first
        {
            let cache = self.file_type_cache.read().await;
            if let Some(&type_id) = cache.get(file_type_name) {
                return Ok(type_id);
            }
        }

        // Not in cache, get or create from database
        let type_id = self
            .get_or_create_file_type_by_name(file_type_name, connection)
            .await?;

        // Update cache
        {
            let mut cache = self.file_type_cache.write().await;
            cache.insert(file_type_name.to_string(), type_id);
        }

        Ok(type_id)
    }

    /// Get or create file type by name (without path inference)
    async fn get_or_create_file_type_by_name<C>(
        &self,
        file_type_name: &str,
        connection: &C,
    ) -> Result<i32, DbError>
    where
        C: ConnectionTrait,
    {
        // Check if file type already exists
        if let Some(existing_type) = FileTypes::find()
            .filter(file_types::Column::Name.eq(file_type_name))
            .one(connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to query file type by name".to_string(),
                    e,
                )
            })?
        {
            return Ok(existing_type.id);
        }

        // Create new file type
        let new_file_type = file_types::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            name: Set(file_type_name.to_string()),
        };

        let created_type = new_file_type.insert(connection).await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::InsertError,
                "Failed to insert file type by name".to_string(),
                e,
            )
        })?;

        Ok(created_type.id)
    }

    /// Get or create file system identifier based on file metadata
    pub async fn get_or_create_file_system_identifier<C: ConnectionTrait>(
        &self,
        file_path: &Path,
        transaction: &C,
    ) -> Result<i32, DbError> {
        use std::os::unix::fs::MetadataExt;

        let metadata = std::fs::metadata(file_path).map_err(|e| {
            DbError::with_source(
                DbErrorKind::QueryError,
                format!("Failed to get file metadata for {}", file_path.display()),
                e,
            )
        })?;

        let inode = metadata.ino() as i32;
        let device_num = metadata.dev() as i32;
        let index_num = metadata.ino() as i32;
        let volume_serial_num = 0; // Unix systems don't have volume serial numbers

        // Check if file system identifier already exists
        let existing_fsi = file_system_identifier::Entity::find()
            .filter(file_system_identifier::Column::Inode.eq(inode))
            .filter(file_system_identifier::Column::DeviceNum.eq(device_num))
            .one(transaction)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to query file system identifier".to_string(),
                    e,
                )
            })?;

        if let Some(fsi) = existing_fsi {
            return Ok(fsi.id);
        }

        // Create new file system identifier
        let new_fsi = file_system_identifier::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            inode: Set(inode),
            device_num: Set(device_num),
            index_num: Set(index_num),
            volume_serial_num: Set(volume_serial_num),
        };

        let created_fsi = new_fsi.insert(transaction).await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::InsertError,
                "Failed to insert file system identifier".to_string(),
                e,
            )
        })?;

        Ok(created_fsi.id)
    }

    /// Get or create file system identifier based on file metadata (transaction-aware)
    pub async fn get_or_create_file_system_identifier_with_connection<C>(
        &self,
        file_path: &Path,
        connection: &C,
    ) -> Result<i32, DbError>
    where
        C: ConnectionTrait,
    {
        use std::os::unix::fs::MetadataExt;

        let metadata = std::fs::metadata(file_path).map_err(|e| {
            DbError::with_source(
                DbErrorKind::QueryError,
                format!("Failed to get file metadata for {}", file_path.display()),
                e,
            )
        })?;

        let inode = metadata.ino() as i32;
        let device_num = metadata.dev() as i32;
        let index_num = metadata.ino() as i32;
        let volume_serial_num = 0; // Unix systems don't have volume serial numbers

        // Check if file system identifier already exists
        let existing_fsi = file_system_identifier::Entity::find()
            .filter(file_system_identifier::Column::Inode.eq(inode))
            .filter(file_system_identifier::Column::DeviceNum.eq(device_num))
            .one(connection)
            .await
            .map_err(|e| {
                DbError::with_source(
                    DbErrorKind::QueryError,
                    "Failed to query file system identifier".to_string(),
                    e,
                )
            })?;

        if let Some(fsi) = existing_fsi {
            return Ok(fsi.id);
        }

        // Create new file system identifier
        let new_fsi = file_system_identifier::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            inode: Set(inode),
            device_num: Set(device_num),
            index_num: Set(index_num),
            volume_serial_num: Set(volume_serial_num),
        };

        let created_fsi = new_fsi.insert(connection).await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::InsertError,
                "Failed to insert file system identifier".to_string(),
                e,
            )
        })?;

        Ok(created_fsi.id)
    }

    /// Preload common file types into cache
    pub async fn preload_file_type_cache(&self) -> Result<(), DbError> {
        let connection = self.database_manager.get_connection();
        let all_types = FileTypes::find().all(&*connection).await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::QueryError,
                "Failed to preload file types".to_string(),
                e,
            )
        })?;

        let mut cache = self.file_type_cache.write().await;
        for file_type in all_types {
            cache.insert(file_type.name, file_type.id);
        }

        Ok(())
    }
}
