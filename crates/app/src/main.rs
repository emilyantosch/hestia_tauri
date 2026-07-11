// Prevents an additional console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use library::library::Library;

fn main() -> Result<()> {
    hestia_tauri_lib::falliable_main(Library::last_or_new())
}
