use std::sync::Arc;
use tokio::sync::Mutex;

pub struct LibrarySettings {
    paths: Arc<Mutex<Vec<std::path::PathBuf>>>,
}
