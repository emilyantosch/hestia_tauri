use rfd::AsyncFileDialog;
///This IPC endpoint returns a template string to the frontend to check for connection
#[tauri::command]
pub fn check_health() -> String {
    "Hello from Rust".to_string()
}

///This IPC selects a folder on the computer
#[tauri::command]
pub async fn select_folder() -> Result<Option<String>, String> {
    let folder = AsyncFileDialog::new()
        .set_title("Select library folder")
        .pick_folder()
        .await;

    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}
