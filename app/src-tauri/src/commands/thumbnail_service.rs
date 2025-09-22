use tauri::State;
use tokio::sync::Mutex;

use crate::{
    config::app::AppState,
    errors::{StateError, ThumbnailError},
};

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
) -> Result<(), StateError> {
    {
        let state = app_state.lock().await;
        match state.thumbnail_processor_handler.as_ref() {
            Some(handler) => {
                tracing::info!("Trying to queue missing files!");
                handler.queue_missing_files().await?;
            }
            None => {
                tracing::error!("The thumbnail message handler could not be found");
                return Err(StateError::ThumbnailMessageHandlerNotFound);
            }
        }
    }
    Ok(())
}
