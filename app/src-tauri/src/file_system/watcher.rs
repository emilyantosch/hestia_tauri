use crate::database::FileOperations;
use crate::errors::{AppError, AppErrorKind, DbError, FileError, FileErrorKind};
use crate::file_system::FileHash;
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};
use std::fmt::format;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::file_system::FileId;

#[derive(Debug)]
pub struct FileEvent {
    pub event: DebouncedEvent,
    pub paths: Vec<PathBuf>,
    pub kind: EventKind,
    pub hash: FileHash,
}

type RawEventReceiver = Option<
    Arc<Mutex<tokio::sync::mpsc::Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<Error>>>>>,
>;
pub struct FileWatcher {
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    raw_event_receiver: RawEventReceiver,
    processed_event_sender: Option<Sender<FileEvent>>,
    pub processed_event_receiver: Arc<Mutex<Option<tokio::sync::mpsc::Receiver<FileEvent>>>>,
    db_operations: Option<Arc<FileOperations>>,
}

impl FileWatcher {
    pub async fn init_watcher(&mut self) {
        let (r_tx, r_rx) = tokio::sync::mpsc::channel(100);
        let rt = tokio::runtime::Handle::current();
        let (p_tx, p_rx) = tokio::sync::mpsc::channel::<FileEvent>(100);
        let r_rx_arc = Arc::new(Mutex::new(r_rx));

        self.processed_event_receiver = Arc::new(Mutex::new(Some(p_rx)));
        self.processed_event_sender = Some(p_tx);

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

        match debouncer {
            Ok(watcher) => {
                println!("Init of FileWatcher completed successfully!");
                self.watcher = Some(watcher);
                self.raw_event_receiver = Some(r_rx_arc);
            }
            Err(e) => println!("{:?}", e),
        };
    }

    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            watcher: None,
            raw_event_receiver: None,
            processed_event_sender: None,
            processed_event_receiver: Arc::new(Mutex::new(None)),
            db_operations: None,
        })
    }

    pub async fn new_with_database(
        db_operations: Arc<FileOperations>,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            watcher: None,
            raw_event_receiver: None,
            processed_event_sender: None,
            processed_event_receiver: Arc::new(Mutex::new(None)),
            db_operations: Some(db_operations),
        })
    }

    pub async fn watch(&mut self, path: &Path) -> Result<(), AppError> {
        if !path.exists() {
            let error_path = vec![path.to_path_buf()];
            return Err(FileError::new(
                FileErrorKind::PathNotFoundError,
                format!("Path could not be found: {:?}", path),
                Some(error_path),
            )
            .into());
        }
        println!("Watching path: {:?}", path);

        if let Some(watcher) = self.watcher.as_mut() {
            watcher.watch(path, RecursiveMode::Recursive).map_err(|e| {
                FileError::with_source(
                    FileErrorKind::WatchNotFoundError,
                    format!("Failed to watch directory: {:?}", e.to_string()),
                    e,
                    Some(vec![path.into()]),
                )
            })?;
            println!("Watcher ready! {:?}", watcher);

            let r_rx_clone = Arc::clone(self.raw_event_receiver.as_ref().unwrap());

            let p_tx_clone = self
                .processed_event_sender
                .as_ref()
                .expect("Processed event handler has not been initialized")
                .clone();

            let p_rx_clone = Arc::clone(&self.processed_event_receiver);

            tokio::spawn(async move {
                while let Some(res) = r_rx_clone.lock().await.recv().await {
                    match res {
                        Ok(events) => {
                            for event in events {
                                if let Err(e) =
                                    Self::to_file_event_and_send(event, &p_tx_clone).await
                                {
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
                while let Some(event) = p_rx_clone.lock().await.as_mut().unwrap().recv().await {
                    println!("Event received from processed sender!");
                    if let Err(e) = Self::to_database(event, &db_ops_clone).await {
                        eprintln!("Failed to store event to database: {:?}", e);
                    }
                }
            });
        }

        Ok(())
    }

    async fn to_database(
        event: FileEvent,
        db_operations: &Option<Arc<FileOperations>>,
    ) -> Result<(), DbError> {
        if let Some(db_ops) = db_operations {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    // File was created or modified, insert/update in database
                    match db_ops.upsert_file_from_event(&event).await {
                        Ok(file_model) => {
                            println!(
                                "Successfully stored file: {} (ID: {})",
                                file_model.path, file_model.id
                            );
                        }
                        Err(e) => {
                            eprintln!("Failed to upsert file: {:?}", e);
                            return Err(e);
                        }
                    }
                }
                EventKind::Remove(_) => {
                    // File was deleted, remove from database
                    for path in &event.paths {
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
                                return Err(e);
                            }
                        }
                    }
                }
                _ => {
                    // Other event types (e.g., access) - we might not need to handle these
                    println!("Ignoring event type: {:?}", event.kind);
                }
            }
        } else {
            println!("No database operations configured, skipping database storage");
        }
        Ok(())
    }

    async fn to_file_event_and_send(
        event: DebouncedEvent,
        processed_event_tx: &Sender<FileEvent>,
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

        if let Err(e) = processed_event_tx.send(file_event).await {
            println!("Error sending processed event into channel: {:?}", e);
        } else {
            println!("Sending processed event successful")
        }
        Ok(())
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
