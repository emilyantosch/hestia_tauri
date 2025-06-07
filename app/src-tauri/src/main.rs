// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod errors;
mod file_system;
use crate::AppError;
use std::path::PathBuf;

use crate::errors::*;
use crate::file_system::FileWatcher;

#[tokio::main]
async fn main() {
    let mut watcher = FileWatcher::new().await.unwrap();
    watcher.init_watcher().await;
    watcher
        .watch(&PathBuf::from(
            "/home/emmi/projects/projects/hestia_tauri/test_vault/",
        ))
        .await
        .unwrap();
    hestia_tauri_lib::run()
}
