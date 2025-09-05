mod commands;
mod config;
mod database;
mod errors;
mod file_system;
mod tests;
mod utils;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::app::AppState;
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing first
    init_tracing();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            info!("Initializing Tauri application with unified AppState");

            #[cfg(target_os = "linux")]
            {
                std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
                std::env::set_var("GTK_DEBUG", "interactive");
            }

            // Initialize unified app state asynchronously
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            let app_state = rt.block_on(async {
                match AppState::new().await {
                    Ok(state) => state,
                    Err(e) => {
                        error!("Failed to initialize AppState: {:?}", e);
                        panic!("Cannot continue without proper app state initialization");
                    }
                }
            });

            info!("AppState initialized successfully");

            // Manage the unified state
            app.manage(Mutex::new(app_state));

            info!("Unified AppState managed as application state");
            Ok(())
        })
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
            commands::library_management::list_available_library,
            commands::library_management::initialize_library_workspace,
            //Utils
            commands::util::check_health,
            commands::util::select_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
