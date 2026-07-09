use std::path::PathBuf;

use notify::EventKind;
use notify_debouncer_full::DebouncedEvent;

#[derive(Debug)]
pub struct FileEvent {
    pub event: DebouncedEvent,
    pub paths: Vec<PathBuf>,
    pub kind: EventKind,
    pub hash: Option<FileHash>,
}

#[derive(Debug)]
pub struct FolderEvent {
    pub event: DebouncedEvent,
    pub paths: Vec<PathBuf>,
    pub kind: EventKind,
    pub hash: Option<FolderHash>,
}
