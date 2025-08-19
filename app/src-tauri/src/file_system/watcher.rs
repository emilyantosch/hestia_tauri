use crate::database::FileOperations;
use crate::errors::{AppError, AppErrorKind, DbError, FileError, FileErrorKind};
use crate::file_system::{FileHash, FolderHash};
use crate::utils::canon_path::CanonPath;
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tracing::{info, warn};

#[derive(Debug)]
pub struct FSEvent {
    pub file_event: Option<FileEvent>,
    pub folder_event: Option<FolderEvent>,
}

impl From<FileEvent> for FSEvent {
    fn from(file_event: FileEvent) -> Self {
        FSEvent {
            file_event: Some(file_event),
            folder_event: None,
        }
    }
}

impl From<FolderEvent> for FSEvent {
    fn from(folder_event: FolderEvent) -> Self {
        FSEvent {
            file_event: None,
            folder_event: Some(folder_event),
        }
    }
}

#[derive(Debug)]
pub struct FileEvent {
    pub event: DebouncedEvent,
    pub paths: Vec<PathBuf>,
    pub kind: EventKind,
    pub hash: FileHash,
}

#[derive(Debug)]
pub struct FolderEvent {
    pub event: DebouncedEvent,
    pub paths: Vec<PathBuf>,
    pub kind: EventKind,
    pub hash: FolderHash,
}

type RawEventReceiver = Option<
    Arc<Mutex<tokio::sync::mpsc::Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<Error>>>>>,
>;

pub struct FileWatcherHandler {
    pub sender: mpsc::UnboundedSender<FileWatcherMessage>,
}

pub enum FileWatcherMessage {
    WatchPath(CanonPath),
    UnwatchPath(CanonPath),
}

pub struct FileWatcher {
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    db_operations: Option<Arc<FileOperations>>,
    pub message_receiver: mpsc::UnboundedReceiver<FileWatcherMessage>,
    watched_paths: Option<HashSet<PathBuf>>,
}

impl FileWatcher {
    pub async fn init_watcher(&mut self) -> Result<(), AppError> {
        let (r_tx, mut r_rx) = tokio::sync::mpsc::channel(100);
        let rt = tokio::runtime::Handle::current();
        let (p_tx, mut p_rx) = tokio::sync::mpsc::channel::<FSEvent>(100);

        //NOTE: This might need to get added back in if I need to see certain events happening in
        //testing
        //self.processed_event_receiver = Arc::new(Mutex::new(Some(p_rx)));

        let debouncer = new_debouncer(
            Duration::from_secs(2),
            None,
            move |result: DebounceEventResult| {
                let r_tx_clone = r_tx.clone();
                rt.spawn(async move {
                    if let Err(e) = r_tx_clone.send(result).await {
                        println!("Error sending event result: {:?}", e);
                    };
                });
            },
        );

        tokio::spawn(async move {
            while let Some(res) = r_rx.recv().await {
                match res {
                    Ok(events) => {
                        for event in events {
                            if let Err(e) = to_file_or_folder_event_and_send(event, &p_tx).await {
                                eprintln!("Failed to process event: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("errors: {:?}", e);
                    }
                }
            }
        });
        let db_ops_clone = self.db_operations.clone();

        tokio::spawn(async move {
            while let Some(event) = p_rx.recv().await {
                println!("Event received from processed sender!");
                if let Err(e) = Self::to_database(event, &db_ops_clone).await {
                    eprintln!("Failed to store event to database: {:?}", e);
                }
            }
        });

        match debouncer {
            Ok(watcher) => {
                println!("Init of FileWatcher completed successfully!");
                self.watcher = Some(watcher);
            }
            Err(e) => println!("{:?}", e),
        };
        Ok(())
    }

    pub async fn new(
        message_receiver: mpsc::UnboundedReceiver<FileWatcherMessage>,
    ) -> std::result::Result<FileWatcher, Box<dyn std::error::Error>> {
        Ok(Self {
            watcher: None,
            db_operations: None,
            message_receiver,
            watched_paths: None,
        })
    }

    pub async fn new_with_database(
        db_operations: Arc<FileOperations>,
        message_receiver: mpsc::UnboundedReceiver<FileWatcherMessage>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            watcher: None,
            db_operations: Some(db_operations),
            message_receiver,
            watched_paths: None,
        })
    }

    pub async fn run(mut self) -> Result<(), AppError> {
        self.init_watcher().await?;
        while let Some(res) = self.message_receiver.recv().await {
            match res {
                FileWatcherMessage::WatchPath(path) => {
                    self.watch(path.try_into()?).await?;
                }
                FileWatcherMessage::UnwatchPath(path) => {
                    self.unwatch(path.try_into()?).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn unwatch(&mut self, path: PathBuf) -> Result<(), AppError> {
        match self.watched_paths.as_mut() {
            Some(paths) => {
                if paths.remove(&path) {
                    return Ok(());
                }
                return Err(FileError::new(
                    FileErrorKind::WatchNotFoundError,
                    format!("The path {path:#?} is not being watched and cannot be unwatched"),
                    Some(vec![path.to_owned()]),
                )
                .into());
            }
            None => {
                return Err(FileError::new(
                    FileErrorKind::PathNotFoundError,
                    format!("There is no path being watched, can't remove path from watch list."),
                    None,
                )
                .into());
            }
        }
    }

    //FIXME: This function works. However, the configuration of the paths will need to be
    //controlled, since each library/configuration will need a root folder. Each folder path that
    //is watched by the watcher is not added to the database and therefore needs to get added
    //separately. This is true for each folder added to watcher, but also changes based on the
    //library that is currently being looked at. I assume we want to use different db files for
    //different vault configs.
    pub async fn watch(&mut self, path: PathBuf) -> Result<(), AppError> {
        match path.try_exists() {
            Ok(true) => (),
            Ok(false) => {
                let error_path = vec![path.to_path_buf()];
                return Err(FileError::new(
                    FileErrorKind::PathNotFoundError,
                    format!("Path could not be found: {:?}", path),
                    Some(error_path),
                )
                .into());
            }
            Err(e) => {
                let error_path = vec![path.to_path_buf()];
                return Err(FileError::with_source(
                    FileErrorKind::PathNotFoundError,
                    format!("Path could not be found: {path:?}"),
                    e,
                    Some(error_path),
                )
                .into());
            }
        }
        if let Some(watcher) = self.watcher.as_mut() {
            watcher
                .watch(&path, RecursiveMode::Recursive)
                .map_err(|e| {
                    FileError::with_source(
                        FileErrorKind::WatchNotFoundError,
                        format!("Failed to watch directory: {:?}", e.to_string()),
                        e,
                        Some(vec![path.to_owned()]),
                    )
                })?;
        }
        match self.watched_paths.as_mut() {
            Some(paths) => {
                if !paths.insert(path.to_owned()) {
                    warn!("Trying to add the same path twice to the watch list. No change to the watch list committed");
                }
            }
            None => {
                let mut hs = HashSet::new();
                hs.insert(path.to_owned());
                self.watched_paths = Some(hs);
            }
        }
        info!("Watching path: {path:#?}");

        Ok(())
    }

    async fn to_database(
        event: FSEvent,
        db_operations: &Option<Arc<FileOperations>>,
    ) -> Result<(), AppError> {
        if let Some(db_ops) = db_operations {
            if let Some(file_event) = event.file_event {
                match file_event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        // File was created or modified, insert/update in database
                        match db_ops.upsert_file_from_event(&file_event).await {
                            Ok(file_model) => {
                                println!(
                                    "Successfully stored file: {} (ID: {})",
                                    file_model.path, file_model.id
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to upsert file: {:?}", e);
                                return Err(e).map_err(|e| {
                                    AppError::with_source(
                                        AppErrorKind::FileError,
                                        "Failed to upsert file".to_string(),
                                        Some(Box::new(e)),
                                    )
                                });
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        // File was deleted, remove from database
                        for path in &file_event.paths {
                            match db_ops.delete_file_by_path(path).await {
                                Ok(deleted) => {
                                    if deleted {
                                        println!(
                                            "Successfully removed file from database: {}",
                                            path.display()
                                        );
                                    } else {
                                        println!(
                                            "File not found in database (already removed?): {}",
                                            path.display()
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to delete file from database: {:?}", e);
                                    return Err(e).map_err(|e| {
                                        AppError::with_source(
                                            AppErrorKind::FileError,
                                            "Failed to upsert file".to_string(),
                                            Some(Box::new(e)),
                                        )
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // Other event types (e.g., access) - we might not need to handle these
                        println!("Ignoring event type: {:?}", file_event.kind);
                    }
                }
            } else if let Some(folder_event) = event.folder_event {
                match folder_event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) => {
                        // File was created or modified, insert/update in database
                        match db_ops.upsert_folder_from_event(&folder_event).await {
                            Ok(folder_model) => {
                                println!(
                                    "Successfully stored file: {} (ID: {})",
                                    folder_model.path, folder_model.id
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to upsert file: {:?}", e);
                                return Err(e).map_err(|e| {
                                    AppError::with_source(
                                        AppErrorKind::FileError,
                                        "Failed to upsert file".to_string(),
                                        Some(Box::new(e)),
                                    )
                                });
                            }
                        }
                    }
                    EventKind::Remove(_) => {
                        // File was deleted, remove from database
                        for path in &folder_event.paths {
                            match db_ops.delete_folder_by_path(path).await {
                                Ok(deleted) => {
                                    if deleted {
                                        println!(
                                            "Successfully removed file from database: {}",
                                            path.display()
                                        );
                                    } else {
                                        println!(
                                            "File not found in database (already removed?): {}",
                                            path.display()
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to delete file from database: {:?}", e);
                                    return Err(e).map_err(|e| {
                                        AppError::with_source(
                                            AppErrorKind::FileError,
                                            "Failed to upsert file".to_string(),
                                            Some(Box::new(e)),
                                        )
                                    });
                                }
                            }
                        }
                    }
                    _ => {
                        // Other event types (e.g., access) - we might not need to handle these
                        println!("Ignoring event type: {:?}", folder_event.kind);
                    }
                }
            } else {
                let e = FileError::new(
                    FileErrorKind::GenericError,
                    "Could not extract FolderEvent or FileEvent".to_string(),
                    None,
                );
                return Err(e).map_err(|e| {
                    AppError::with_source(
                        AppErrorKind::FileError,
                        "Failed to upsert file".to_string(),
                        Some(Box::new(e)),
                    )
                });
            }
        } else {
            println!("No database operations configured, skipping database storage");
        }
        Ok(())
    }
}

async fn to_file_or_folder_event_and_send(
    event: DebouncedEvent,
    processed_event_tx: &Sender<FSEvent>,
) -> Result<(), AppError> {
    let path = match event.paths.last() {
        Some(path) => path,
        None => {
            return Err(AppError::Categorized {
                kind: AppErrorKind::FileError,
                message: "Could not extract last paths. Were paths even provided?".to_string(),
                source: None,
            })
        }
    };

    if path.is_dir() {
        to_folder_event_and_send(event, processed_event_tx).await?;
    } else {
        to_file_event_and_send(event, processed_event_tx).await?;
    }

    Ok(())
}

async fn to_file_event_and_send(
    event: DebouncedEvent,
    processed_event_tx: &Sender<FSEvent>,
) -> Result<(), AppError> {
    let kind = event.kind;
    let paths = event.paths.to_owned();
    println!("{:?}", paths);

    let hash = FileHash::hash(match &paths.last() {
        Some(x) => x,
        None => {
            return Err(AppError::Categorized {
                kind: AppErrorKind::FileError,
                message: String::from(
                    "Error while trying to extract last path: There is no path to be extracted.",
                ),
                source: None,
            });
        }
    })
    .await?;

    let file_event = FileEvent::new(event, kind, paths, hash);
    println!("Constructed FileEvent from Raw Stream");

    if let Err(e) = processed_event_tx.send(file_event.into()).await {
        println!("Error sending processed event into channel: {:?}", e);
    } else {
        println!("Sending processed event successful")
    }
    Ok(())
}

async fn to_folder_event_and_send(
    event: DebouncedEvent,
    processed_event_tx: &Sender<FSEvent>,
) -> Result<(), AppError> {
    let kind = event.kind;
    let paths = event.paths.to_owned();
    println!("{:?}", paths);

    let hash = FolderHash::hash(match &paths.last() {
        Some(x) => x,
        None => {
            return Err(AppError::Categorized {
                kind: AppErrorKind::FileError,
                message: String::from(
                    "Error while trying to extract last path: There is no path to be extracted.",
                ),
                source: None,
            });
        }
    })
    .await?;

    let folder_event = FolderEvent::new(event, kind, paths, hash);
    println!("Constructed FileEvent from Raw Stream");

    if let Err(e) = processed_event_tx.send(folder_event.into()).await {
        println!("Error sending processed event into channel: {:?}", e);
    } else {
        println!("Sending processed event successful")
    }
    Ok(())
}

impl FolderEvent {
    fn new(event: DebouncedEvent, kind: EventKind, paths: Vec<PathBuf>, hash: FolderHash) -> Self {
        FolderEvent {
            event,
            paths,
            kind,
            hash,
        }
    }
}
impl FileEvent {
    fn new(event: DebouncedEvent, kind: EventKind, paths: Vec<PathBuf>, hash: FileHash) -> Self {
        FileEvent {
            event,
            paths,
            kind,
            hash,
        }
    }
}
