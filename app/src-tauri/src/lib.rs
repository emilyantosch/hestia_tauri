mod config;
mod database;
mod errors;
mod file_system;
mod tests;

use std::sync::Arc;

use crate::database::{DatabaseManager, FileOperations};
use crate::errors::AppError;
use crate::file_system::{FileWatcher, DirectoryScanner};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DatabaseManager>,
}

pub struct AppBuilder {}

impl AppBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn build(self) -> Result<App, AppError> {
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
    pub async fn run(self) -> Result<(), AppError> {
        // Create shared file operations with database connection
        let file_operations = Arc::new(FileOperations::new(self.state.database.get_connection()));
        
        // Preload file type cache for better performance
        if let Err(e) = file_operations.preload_file_type_cache().await {
            eprintln!("Warning: Failed to preload file type cache: {:?}", e);
        }
        
        let watch_directory = PathBuf::from("/home/emmi/projects/projects/hestia_tauri/test_vault/");
        
        // === INITIAL DIRECTORY SCAN ===
        println!("Starting initial directory scan...");
        let scanner = DirectoryScanner::new(file_operations.clone());
        
        match scanner.sync_directory(&watch_directory).await {
            Ok(report) => {
                println!("Initial scan completed successfully!");
                println!("  - Files scanned: {}", report.files_scanned);
                println!("  - Files inserted: {}", report.files_inserted);
                println!("  - Files updated: {}", report.files_updated);
                println!("  - Files deleted: {}", report.files_deleted);
                println!("  - Total operations: {}", report.total_operations());
                println!("  - Duration: {:?}", report.duration);
                
                if !report.errors.is_empty() {
                    println!("  - Errors encountered:");
                    for error in &report.errors {
                        println!("    • {}", error);
                    }
                }
            }
            Err(e) => {
                eprintln!("Initial directory scan failed: {:?}", e);
                eprintln!("Continuing with file watcher anyway...");
            }
        }
        
        // === START FILE WATCHER ===
        println!("Starting real-time file watcher...");
        let mut watcher = FileWatcher::new_with_database(file_operations.clone())
            .await
            .unwrap();
        watcher.init_watcher().await;
        
        // Watch the test vault directory
        watcher
            .watch(&watch_directory)
            .await
            .unwrap();
            
        println!("File watcher started! Monitoring: {}", watch_directory.display());
        println!("Application is now running. Press Ctrl+C to stop.");
        
        // Keep the application running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            
            // Optional: Periodic status report
            // You could add periodic re-scans here if needed
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize the async runtime for database operations
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    rt.block_on(async {
        // Initialize the app with database
        match AppBuilder::new().build().await {
            Ok(app) => {
                println!("App initialized successfully with database connection");

                // Start the file watching system in a background task
                let app_clone = app.clone();
                tokio::spawn(async move {
                    if let Err(e) = app_clone.run().await {
                        eprintln!("Error running file watcher: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to initialize app: {:?}", e);
            }
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
