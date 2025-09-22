use std::sync::Arc;

use tauri::{command, State};

use crate::config::library::{Library, LibraryPathConfig};

#[command]
pub async fn get_paths(library: State<'_, Arc<Library>>) -> Result<Vec<LibraryPathConfig>, &str> {
    match library.library_config.as_ref() {
        Some(conf) => Ok(conf.library_paths.clone()),
        None => Err("There is no config defined!"),
    }
}
