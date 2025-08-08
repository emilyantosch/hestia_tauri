use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use async_recursion::async_recursion;
use tokio::fs;

use crate::config::scanner::ScanConfig;
use crate::database::{FileInfo, FileMetadata, FileOperations};
use crate::errors::{AppError, DbError};
use crate::file_system::FileHash;

/// Types of synchronization operations
#[derive(Debug, Clone)]
pub enum SyncOperation {
    /// Insert a new file into the database
    Insert(FileInfo),
    /// Update an existing file in the database
    Update(FileInfo),
    /// Delete a file from the database (file no longer exists)
    Delete(PathBuf),
}

/// Report of synchronization operations
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub files_scanned: usize,
    pub files_inserted: usize,
    pub files_updated: usize,
    pub files_deleted: usize,
    pub files_skipped: usize,
    pub errors: Vec<String>,
    pub duration: std::time::Duration,
}

impl SyncReport {
    pub fn new() -> Self {
        Self {
            files_scanned: 0,
            files_inserted: 0,
            files_updated: 0,
            files_deleted: 0,
            files_skipped: 0,
            errors: Vec::new(),
            duration: std::time::Duration::from_secs(0),
        }
    }

    pub fn total_operations(&self) -> usize {
        self.files_inserted + self.files_updated + self.files_deleted
    }
}

/// Directory scanner that synchronizes filesystem state with database
pub struct DirectoryScanner {
    file_operations: Arc<FileOperations>,
    config: ScanConfig,
}

impl DirectoryScanner {
    /// Create a new directory scanner
    pub fn new(file_operations: Arc<FileOperations>) -> Self {
        Self {
            file_operations,
            config: ScanConfig::default(),
        }
    }

    /// Create a new directory scanner with custom configuration
    pub fn new_with_config(file_operations: Arc<FileOperations>, config: ScanConfig) -> Self {
        Self {
            file_operations,
            config,
        }
    }

    /// Synchronize a directory with the database
    pub async fn sync_directory(&self, dir_path: &Path) -> Result<SyncReport, AppError> {
        let start_time = Instant::now();
        let mut report = SyncReport::new();

        println!("Starting directory sync for: {}", dir_path.display());

        // 1. Get current database state
        let db_state = match self.file_operations.get_directory_state(dir_path).await {
            Ok(state) => state,
            Err(e) => {
                let error_msg = format!("Failed to get database state: {:?}", e);
                report.errors.push(error_msg.clone());
                return Err(AppError::from(e));
            }
        };

        println!("Found {} files in database", db_state.len());

        // 2. Scan filesystem
        let fs_files = match self.scan_filesystem_recursive(dir_path).await {
            Ok(files) => files,
            Err(e) => {
                let error_msg = format!("Failed to scan filesystem: {:?}", e);
                report.errors.push(error_msg);
                return Err(e);
            }
        };

        report.files_scanned = fs_files.len();
        println!("Found {} files in filesystem", fs_files.len());

        // 3. Calculate sync operations
        let operations = self.calculate_sync_operations(db_state, fs_files);
        println!("Calculated {} operations to perform", operations.len());

        // 4. Execute operations in batches
        let mut insert_batch = Vec::new();
        let mut delete_batch = Vec::new();

        for operation in operations {
            match operation {
                SyncOperation::Insert(file_info) => {
                    insert_batch.push(file_info);
                    if insert_batch.len() >= self.config.batch_size {
                        self.execute_insert_batch(&mut insert_batch, &mut report)
                            .await;
                    }
                }
                SyncOperation::Update(file_info) => {
                    insert_batch.push(file_info); // Upsert handles both insert and update
                    if insert_batch.len() >= self.config.batch_size {
                        self.execute_insert_batch(&mut insert_batch, &mut report)
                            .await;
                    }
                }
                SyncOperation::Delete(path) => {
                    delete_batch.push(path);
                    if delete_batch.len() >= self.config.batch_size {
                        self.execute_delete_batch(&mut delete_batch, &mut report)
                            .await;
                    }
                }
            }
        }

        // Execute remaining batches
        if !insert_batch.is_empty() {
            self.execute_insert_batch(&mut insert_batch, &mut report)
                .await;
        }
        if !delete_batch.is_empty() {
            self.execute_delete_batch(&mut delete_batch, &mut report)
                .await;
        }

        report.duration = start_time.elapsed();

        println!("Directory sync completed in {:?}", report.duration);
        println!(
            "Results: {} inserted, {} updated, {} deleted, {} errors",
            report.files_inserted,
            report.files_updated,
            report.files_deleted,
            report.errors.len()
        );

        Ok(report)
    }

    /// Scan filesystem recursively and return file information
    async fn scan_filesystem_recursive(&self, dir_path: &Path) -> Result<Vec<FileInfo>, AppError> {
        let mut files = Vec::new();
        self.scan_directory_impl(dir_path, &mut files).await?;
        Ok(files)
    }

    /// Recursive implementation of directory scanning
    #[async_recursion]
    async fn scan_directory_impl(
        &self,
        dir_path: &Path,
        files: &mut Vec<FileInfo>,
    ) -> Result<(), AppError> {
        let mut entries = match fs::read_dir(dir_path).await {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read directory {}: {}", dir_path.display(), e);
                return Ok(()); // Continue with other directories
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if path.is_dir() {
                // Check if directory should be ignored
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    if self
                        .config
                        .ignore_directories
                        .contains(&dir_name.to_string())
                    {
                        continue;
                    }
                }

                // Recurse into subdirectory if configured
                if self.config.recursive {
                    self.scan_directory_impl(&path, files).await?;
                }
            } else if path.is_file() {
                // Check if file should be ignored
                if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                    let ext_with_dot = format!(".{}", extension);
                    if self.config.ignore_extensions.contains(&ext_with_dot) {
                        continue;
                    }
                }

                // Check file size if limit is set
                if let Some(max_size) = self.config.max_file_size {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        if metadata.len() > max_size {
                            continue;
                        }
                    }
                }

                // Process the file
                match self.create_file_info(&path).await {
                    Ok(file_info) => files.push(file_info),
                    Err(e) => {
                        eprintln!("Failed to process file {}: {:?}", path.display(), e);
                        // Continue with other files
                    }
                }
            }
        }

        Ok(())
    }

    /// Create FileInfo from a filesystem path
    async fn create_file_info(&self, path: &Path) -> Result<FileInfo, AppError> {
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| AppError::Categorized {
                kind: crate::errors::AppErrorKind::FileError,
                message: format!("Failed to get file metadata: {}", e),
                source: Some(Box::new(e)),
            })?;

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
        let file_type_name = self.detect_file_type(path);

        Ok(FileInfo {
            path: path.to_path_buf(),
            name,
            content_hash: content_hash_str,
            identity_hash: identity_hash_str,
            file_type_name,
            file_system_id: None, // Will be set during database operations
        })
    }

    /// Calculate what operations need to be performed
    fn calculate_sync_operations(
        &self,
        db_state: HashMap<PathBuf, FileMetadata>,
        fs_files: Vec<FileInfo>,
    ) -> Vec<SyncOperation> {
        let mut operations = Vec::new();
        let mut processed_paths = std::collections::HashSet::new();

        // Check filesystem files against database
        for fs_file in fs_files {
            processed_paths.insert(fs_file.path.clone());

            match db_state.get(&fs_file.path) {
                Some(db_metadata) => {
                    // File exists in database, check if it needs updating
                    if db_metadata.content_hash != fs_file.content_hash
                        || db_metadata.identity_hash != fs_file.identity_hash
                    {
                        operations.push(SyncOperation::Update(fs_file));
                    }
                    // If hashes match, no operation needed
                }
                None => {
                    // File doesn't exist in database, insert it
                    operations.push(SyncOperation::Insert(fs_file));
                }
            }
        }

        // Check for files in database that no longer exist in filesystem
        for (db_path, _) in db_state {
            if !processed_paths.contains(&db_path) {
                operations.push(SyncOperation::Delete(db_path));
            }
        }

        operations
    }

    /// Execute a batch of insert/update operations
    async fn execute_insert_batch(&self, batch: &mut Vec<FileInfo>, report: &mut SyncReport) {
        if batch.is_empty() {
            return;
        }

        match self.file_operations.batch_upsert_files(batch.clone()).await {
            Ok(count) => {
                report.files_inserted += count; // Note: this includes both inserts and updates
                println!("Successfully processed batch of {} files", count);
            }
            Err(e) => {
                let error_msg = format!("Failed to execute insert batch: {:?}", e);
                report.errors.push(error_msg);
                eprintln!("Batch insert failed: {:?}", e);
            }
        }

        batch.clear();
    }

    /// Execute a batch of delete operations
    async fn execute_delete_batch(&self, batch: &mut Vec<PathBuf>, report: &mut SyncReport) {
        if batch.is_empty() {
            return;
        }

        match self.file_operations.batch_delete_files(batch.clone()).await {
            Ok(count) => {
                report.files_deleted += count;
                println!("Successfully deleted {} files from database", count);
            }
            Err(e) => {
                let error_msg = format!("Failed to execute delete batch: {:?}", e);
                report.errors.push(error_msg);
                eprintln!("Batch delete failed: {:?}", e);
            }
        }

        batch.clear();
    }

    /// Detect file type based on file extension (reusing logic from FileOperations)
    fn detect_file_type(&self, file_path: &Path) -> String {
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
