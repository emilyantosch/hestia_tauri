use tauri::State;
use tokio::sync::Mutex;

use crate::{config::app::AppState, errors::ThumbnailError};

#[tauri::command]
pub async fn get_thumbnails_for_filter(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<(), ThumbnailError> {
    {
        let mut state = app_state.lock().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn generate_missing_thumbnails_for_library(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<(), ThumbnailError> {
    {
        let mut state = app_state.lock().await;
    }
    Ok(())
}
