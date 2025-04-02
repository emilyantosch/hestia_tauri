// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_system;
use std::path::PathBuf;

use crate::file_system::FileWatcher;

#[tokio::main]
async fn main() {
    let mut watcher = FileWatcher::new().await.unwrap();
    watcher.init_watcher().await;
    watcher
        .watch(&PathBuf::from(
            "/Users/emmi/projects/projects/hestia/test_vault/",
        ))
        .await
        .unwrap();
    hestia_tauri_lib::run()
}
