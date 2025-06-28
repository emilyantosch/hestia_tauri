// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod errors;
mod file_system;
use crate::AppError;
use std::path::PathBuf;

use crate::errors::*;
use crate::file_system::FileWatcher;

#[tokio::main]
async fn main() {
    hestia_tauri_lib::run()
}
