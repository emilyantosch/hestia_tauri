use anyhow::{Context, Result};
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};

use crate::{
    config::library::Library,
    data::{thumbnails::ThumbnailGenerator, watched_folders::WatchedFolderTree},
    database::{thumbnail_repository, DatabaseManager, FileOperations, ThumbnailOperations},
    errors::{DbError, LibraryError, ScannerError},
    file_system::{
        DatabaseFileWatcherEventHandler, DirectoryScanner, FileWatcher, FileWatcherHandler,
        ThumbnailProcessor, ThumbnailProcessorHandler,
    },
    utils::canon_path::CanonPath,
};

use migration::{Migrator, MigratorTrait};

/// Unified application state containing all components
#[derive(Debug)]
pub struct AppState {
    pub library: Library,
    pub database_manager: Arc<DatabaseManager>,
    pub file_operations: FileOperations,
    pub directory_scanner: DirectoryScanner,
    pub file_watcher_handler: Option<FileWatcherHandler>,
    pub thumbnail_processor_handler: Option<ThumbnailProcessorHandler>,
}

impl AppState {
    /// Create a new AppState with default components
    pub async fn new() -> Result<Self> {
        // Initialize database manager with default settings
        let database_manager = Arc::new(DatabaseManager::new_sqlite_default().await?);
        // Test database connection
        database_manager.test_connection().await?;
        // Create file operations with database connection
        let file_operations = FileOperations::new(Arc::clone(&database_manager));

        // Create directory scanner - we need to create another FileOperations for it
        let file_operations_for_scanner = FileOperations::new(Arc::clone(&database_manager));
        let directory_scanner = DirectoryScanner::new(Arc::new(file_operations_for_scanner));

        // Load last library or create new one
        let library = Library::last_or_new();

        Ok(Self {
            library,
            database_manager,
            file_operations,
            directory_scanner,
            file_watcher_handler: None,
            thumbnail_processor_handler: None,
        })
    }

    async fn create_new_thumbnail_engine(
        &mut self,
        database_manager: Arc<DatabaseManager>,
    ) -> Result<()> {
        if let Some(ref handler) = self.thumbnail_processor_handler {
            info!("Shutting down Thumbnail Engine!");
            handler.shutdown().await?;
        }
        info!("Creating new thumbnail engine");
        // Create the thumbnail processor, which (I hope) is library-agnostic
        info!("Creating new thumbnail processor!");
        let (thumbnail_processor_handler, thumbnail_msg_receiver) =
            ThumbnailProcessorHandler::new();
        let thumbnail_generator = ThumbnailGenerator::new();
        let thumbnail_repository = ThumbnailOperations::new(Arc::clone(&database_manager));
        let thumbnail_processor = ThumbnailProcessor::new(
            thumbnail_msg_receiver,
            Arc::new(thumbnail_repository),
            Arc::new(thumbnail_generator),
        );

        tokio::spawn(async move {
            if let Err(e) = thumbnail_processor.run().await {
                error!("Thumbnail Engine failed to start properly! Reason: {e}");
            }
        });
        self.thumbnail_processor_handler = Some(thumbnail_processor_handler);
        Ok(())
    }

    /// Switch to a new library and update all dependent components
    pub async fn switch_library(&mut self, library: Library) -> Result<()> {
        info!("Switching to library: {:?}", library.library_config);

        // Update library
        self.library = library;

        // Get new database path
        let db_path = self.library.get_canon_database_path()?;
        info!("New database path: {:?}", db_path);

        // Update database connection
        self.update_database_connection(db_path).await?;

        // Recreate dependent components with new database connection
        self.reinitialize_components().await?;

        info!("Successfully switched library");
        Ok(())
    }

    /// Update database connection to point to the new library's database
    async fn update_database_connection(&mut self, db_path: CanonPath) -> Result<()> {
        // For now, create a new DatabaseManager with the new path
        // TODO: Add proper connection switching to DatabaseManager
        let connection_string = format!("sqlite:///{}", db_path.as_str()?);
        info!("Updating database connection to: {}", connection_string);

        // Create new database settings for the library database
        let sqlite_config = crate::config::database::SqliteConfig {
            con_string: connection_string,
            create_if_missing: true,
            connection_timeout_ms: 30000,
            journal_mode: sea_orm::sqlx::sqlite::SqliteJournalMode::Wal,
            synchronous: sea_orm::sqlx::sqlite::SqliteSynchronous::Normal,
        };
        let settings = crate::config::database::DatabaseSettings {
            db_type: crate::config::database::DatabaseType::Sqlite,
            sqlite_config: Some(sqlite_config),
            postgres_config: None,
        };

        // Create new database manager
        self.database_manager = Arc::new(DatabaseManager::new(settings).await?);

        // Test the new connection
        self.database_manager.test_connection().await?;

        Ok(())
    }

    /// Reinitialize all components that depend on the database
    async fn reinitialize_components(&mut self) -> Result<()> {
        info!("Reinitializing components with new database connection");

        // Recreate file operations with new database manager
        self.file_operations = FileOperations::new(Arc::clone(&self.database_manager));

        // Preload file type cache for better performance
        if let Err(e) = self.file_operations.preload_file_type_cache().await {
            tracing::warn!("Warning: Failed to preload file type cache: {:?}", e);
        }

        // Recreate directory scanner with new file operations
        let file_operations_for_scanner = FileOperations::new(Arc::clone(&self.database_manager));
        self.directory_scanner = DirectoryScanner::new(Arc::new(file_operations_for_scanner));

        self.create_new_thumbnail_engine(self.database_manager.clone())
            .await?;

        info!("Successfully reinitialized components");
        Ok(())
    }

    /// Run database migrations for the current library
    pub async fn run_migrations(&self) -> Result<()> {
        info!("Running database migrations");

        let db_connection = self.database_manager.get_connection();

        Migrator::up(db_connection.as_ref(), None).await?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    pub async fn get_watched_folders_map(&self) -> Result<HashMap<String, WatchedFolderTree>> {
        self.file_operations.get_watched_folder_map().await
    }
    /// Get library paths for scanning
    pub fn get_library_paths(&self) -> Vec<std::path::PathBuf> {
        match &self.library.library_config {
            Some(config) => config
                .library_paths
                .iter()
                .filter_map(|path_config| path_config.path.clone())
                .collect(),
            None => Vec::new(),
        }
    }

    pub async fn upsert_root_folders(&self) -> Result<()> {
        let library_paths = self.get_library_paths();

        if library_paths.is_empty() {
            info!("No library paths to upsert");
            return Ok(());
        }

        self.file_operations
            .upsert_root_folders(library_paths)
            .await?;

        Ok(())
    }
    /// Perform initial directory scan for all library paths
    pub async fn scan_library_directories(&self) -> Result<()> {
        let library_paths = self.get_library_paths();

        if library_paths.is_empty() {
            info!("No library paths to scan");
            return Ok(());
        }

        info!(
            "Starting initial directory scan for {} paths",
            library_paths.len()
        );

        for path in library_paths {
            match self.directory_scanner.sync_directory(&path).await {
                Ok(report) => {
                    info!(
                        r"Scanned {}:
                        {} files scanned,
                        {} folders scanned,
                        {} files inserted,
                        {} folders inserted,
                        {} files updated,
                        {} folders updated,
                        {} files deleted,
                        {} folders deleted",
                        path.display(),
                        report.files_scanned,
                        report.folders_scanned,
                        report.files_inserted,
                        report.folders_inserted,
                        report.files_updated,
                        report.folders_updated,
                        report.files_deleted,
                        report.folders_deleted
                    );
                }
                Err(e) => {
                    error!("Failed to scan directory {}: {:?}", path.display(), e);
                    return Err(ScannerError::PathScanFailedError {
                        path: path.to_string_lossy().to_string(),
                    })?;
                }
            }
        }

        info!("Initial directory scan completed");
        Ok(())
    }

    /// Set the file watcher handler
    pub fn set_file_watcher_handler(&mut self, handler: FileWatcherHandler) {
        self.file_watcher_handler = Some(handler);
    }

    pub async fn create_file_watcher(&mut self) -> Result<()> {
        let (fw_sender, fw_receiver) = tokio::sync::mpsc::unbounded_channel();

        self.set_file_watcher_handler(FileWatcherHandler { sender: fw_sender });

        let fw_file_operations = FileOperations::new(Arc::clone(&self.database_manager));
        let fw_event_handler = DatabaseFileWatcherEventHandler {
            db_operations: fw_file_operations,
        };
        tokio::spawn(async move {
            if let Err(e) = FileWatcher::new(fw_receiver)
                .run(Box::new(fw_event_handler))
                .await
            {
                error!("FileWatcher could not be created due to {e:#?}!")
            }
        });
        Ok(())
    }
}

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("The file watcher could not be found!")]
    WatcherNotFound,
    #[error("The thumbnail message handler could not be found!")]
    ThumbnailMessageHandlerNotFound,
    #[error("An internal error has occurred: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum StateErrorKind {
    WatcherNotFound(String),
    ThumbnailMessageHandlerNotFound(String),
    Internal(String),
}

impl Serialize for StateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::WatcherNotFound => StateErrorKind::WatcherNotFound(error_message),
            Self::ThumbnailMessageHandlerNotFound => {
                StateErrorKind::ThumbnailMessageHandlerNotFound(error_message)
            }
            Self::Internal(_) => StateErrorKind::Internal(error_message),
        };
        error_kind.serialize(serializer)
    }
}
