mod config;
mod database;
mod errors;
mod file_system;
mod tests;

use std::sync::Arc;

use crate::database::manager::DatabaseManager;
use crate::errors::AppError;
use crate::file_system::FileWatcher;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<DatabaseManager>,
}

pub struct AppBuilder {}

pub struct App {
    pub state: AppState,
}

impl App {
    pub async fn run(self) -> Result<(), AppError> {
        let mut watcher = FileWatcher::new().await.unwrap();
        watcher.init_watcher().await;
        watcher
            .watch(&PathBuf::from(
                "/home/emmi/projects/projects/hestia_tauri/test_vault/",
            ))
            .await
            .unwrap();
        Ok(())
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
