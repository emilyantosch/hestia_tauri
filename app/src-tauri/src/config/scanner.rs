/// Configuration for directory scanning
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Maximum number of files to process in a single batch
    pub batch_size: usize,
    /// Whether to scan subdirectories recursively
    pub recursive: bool,
    /// File extensions to ignore (e.g., [".tmp", ".log"])
    pub ignore_extensions: Vec<String>,
    /// Directory names to ignore (e.g., [".git", "node_modules"])
    pub ignore_directories: Vec<String>,
    /// Maximum file size to process (in bytes)
    pub max_file_size: Option<u64>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            recursive: true,
            ignore_extensions: vec![
                ".tmp".to_string(),
                ".log".to_string(),
                ".bak".to_string(),
                ".swp".to_string(),
            ],
            ignore_directories: vec![
                ".git".to_string(),
                ".svn".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".DS_Store".to_string(),
            ],
            max_file_size: Some(100 * 1024 * 1024), // 100 MB
        }
    }
}
