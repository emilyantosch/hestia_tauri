#[tauri::command]
pub fn check_health() -> String {
    "Hello from Rust".to_string()
}
