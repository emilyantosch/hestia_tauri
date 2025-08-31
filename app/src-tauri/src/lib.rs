mod commands;
mod config;
mod database;
mod errors;
mod file_system;
mod tests;
mod utils;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::library::Library;
use crate::database::{DatabaseManager, FileOperations};
use crate::errors::AppError;
use crate::file_system::{
    DatabaseFileWatcherEventHandler, DirectoryScanner, FileWatcher, FileWatcherHandler,
    FileWatcherMessage,
};
use std::path::PathBuf;
use tauri::Manager;
use tokio::sync::mpsc::{self, UnboundedSender};
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DatabaseManager>,
}

pub struct AppBuilder {}

impl AppBuilder {
    pub fn new() -> Self {
        Self {}
    }

    fn init_tracing() {
        let filter = std::env::var("RUST_LOG")
            .map(|_| EnvFilter::from_default_env())
            .unwrap_or_else(|_| EnvFilter::new("info"));
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
    }

    pub async fn build(self) -> Result<App, AppError> {
        //Init the tracing
        Self::init_tracing();
        // Initialize database with default SQLite configuration
        let database_manager = DatabaseManager::new_sqlite_default().await?;

        // Test the database connection
        database_manager.test_connection().await?;

        let state = AppState {
            database: Arc::new(database_manager),
        };

        Ok(App { state })
    }
}

#[derive(Clone)]
pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn run(self) -> Result<FileWatcherHandler, AppError> {
        // Create shared file operations with database connection
        let file_operations = Arc::new(FileOperations::new(self.state.database.clone()));
        let fw_event_handler = Arc::new(DatabaseFileWatcherEventHandler {
            db_operations: Some(file_operations.clone()),
        });
        let (fw_sender, fw_receiver) = mpsc::unbounded_channel();

        // Preload file type cache for better performance
        if let Err(e) = file_operations.preload_file_type_cache().await {
            warn!("Failed to preload file type cache: {:?}", e);
        }

        let watch_directory =
            PathBuf::from("/home/emmi/projects/projects/hestia_tauri/test_vault/");
        let mut watch_directory = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .to_path_buf();
        watch_directory.push(std::path::Path::new("test_vault"));
        // === INITIAL DIRECTORY SCAN ===
        info!("Starting initial directory scan...");
        let scanner = DirectoryScanner::new(file_operations.clone());

        match scanner.sync_directory(&watch_directory).await {
            Ok(report) => {
                info!("Initial scan completed successfully!");
                info!("  - Files scanned: {}", report.files_scanned);
                info!("  - Files inserted: {}", report.files_inserted);
                info!("  - Files updated: {}", report.files_updated);
                info!("  - Files deleted: {}", report.files_deleted);
                info!("  - Total operations: {}", report.total_operations());
                info!("  - Duration: {:?}", report.duration);

                if !report.errors.is_empty() {
                    info!("  - Errors encountered:");
                    for error in &report.errors {
                        info!("    • {}", error);
                    }
                }
            }
            Err(e) => {
                error!("Initial directory scan failed: {:?}", e);
                error!("Continuing with file watcher anyway...");
            }
        }

        // === START FILE WATCHER ===
        info!("Starting real-time file watcher...");
        let watcher = FileWatcher::new_with_handler(fw_event_handler, fw_receiver)
            .await
            .unwrap();
        tokio::spawn(async move {
            match watcher.run().await {
                Ok(()) => (),
                Err(e) => {
                    return Err(AppError::with_source(
                        errors::AppErrorKind::FileError,
                        format!("The watcher has failed due to an previous error!"),
                        Some(Box::new(e)),
                    ))
                }
            }
            Ok(())
        });

        Ok(FileWatcherHandler { sender: fw_sender })
        // Keep the application running
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the async runtime for database operations
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let mut fw_handler: Option<FileWatcherHandler> = None;

    rt.block_on(async {
        // Initialize the app with database
        match AppBuilder::new().build().await {
            Ok(app) => {
                info!("App initialized successfully with database connection");

                // Start the file watching system in a background task
                match app.run().await {
                    Ok(handler) => {
                        fw_handler = Some(handler);
                        info!("The app has been initialized successfully!");
                    }
                    Err(e) => {
                        error!("Err(e) to initialize app: {:?}", e);
                    }
                };
            }
            Err(e) => {
                error!("Failed to initialize app: {:?}", e);
            }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            // Initialize database manager
            info!("Hello!");
            println!("Hello!");
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            let db_manager = rt
                .block_on(async { DatabaseManager::new_sqlite_default().await })
                .expect("Failed to initialize database manager");

            #[cfg(target_os = "linux")]
            {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
                std::env::set_var("GTK_DEBUG", "interactive");
            }
            // Test the database connection
            rt.block_on(async { db_manager.test_connection().await })
                .expect("Database connection test failed");

            // Create file operations with database connection
            let file_operations = Arc::new(FileOperations::new(Arc::new(db_manager)));
            let file_scanner = Arc::new(DirectoryScanner::new(Arc::clone(&file_operations)));
            let library = Mutex::new(Library::last_or_new());

            // Preload file type cache for better performance
            rt.block_on(async {
                if let Err(e) = file_operations.preload_file_type_cache().await {
                    error!("Warning: Failed to preload file type cache: {:?}", e);
                }
            });
            // Manage the file operations as application state
            app.manage(file_operations);
            app.manage(file_scanner);
            app.manage(library);
            app.manage(fw_handler);

            info!("FileOperations initialized and managed as application state");
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // File operations
            commands::file_operations::scan_directory,
            commands::file_operations::get_file_by_path,
            commands::file_operations::get_files_in_directory,
            commands::file_operations::delete_file_by_path,
            commands::file_operations::get_file_metadata,
            commands::file_operations::file_exists_in_database,
            // Tag management
            commands::tag_management::create_tag,
            commands::tag_management::get_all_tags,
            commands::tag_management::get_tag_by_id,
            commands::tag_management::get_tag_by_name,
            commands::tag_management::update_tag,
            commands::tag_management::delete_tag,
            commands::tag_management::add_tag_to_file,
            commands::tag_management::remove_tag_from_file,
            commands::tag_management::get_tags_for_file,
            commands::tag_management::get_files_for_tag,
            commands::tag_management::get_all_file_tag_relationships,
            commands::tag_management::search_tags_by_name,
            // Database queries
            commands::database_queries::search_files,
            commands::database_queries::get_files_with_details,
            commands::database_queries::get_database_stats,
            commands::database_queries::search_files_by_tags,
            commands::database_queries::find_duplicate_files,
            commands::database_queries::get_untagged_files,
            commands::database_queries::get_recent_files,
            commands::database_queries::get_recently_updated_files,
            // Folder management
            commands::folder_management::get_all_folders,
            commands::folder_management::get_folder_by_id,
            commands::folder_management::get_folder_by_path,
            commands::folder_management::get_root_folders,
            commands::folder_management::get_subfolders,
            commands::folder_management::get_files_in_folder,
            commands::folder_management::get_folder_tree,
            commands::folder_management::get_folder_summary,
            commands::folder_management::search_folders_by_name,
            commands::folder_management::get_folder_path_hierarchy,
            commands::folder_management::delete_empty_folders,
            commands::folder_management::get_folder_statistics,
            // Library management
            commands::library_management::get_library_paths,
            commands::library_management::select_library,
            commands::library_management::create_new_library,
            commands::library_management::select_library_folder,
            commands::library_management::list_available_library,
            //Utils
            commands::util::check_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
