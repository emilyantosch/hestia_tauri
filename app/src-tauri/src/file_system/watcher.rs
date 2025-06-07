use crate::errors::{FileError, FileErrorKind};
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::file_system::FileId;

#[derive(Debug)]
pub struct FileEvent {
    event: DebouncedEvent,
    paths: Vec<PathBuf>,
    kind: EventKind,
    file_id: FileId,
}

type RawEventReceiver = Option<
    Arc<Mutex<tokio::sync::mpsc::Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<Error>>>>>,
>;

#[derive(Debug)]
pub struct FileWatcher {
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    raw_event_receiver: RawEventReceiver,
    processed_event_sender: Option<Sender<FileEvent>>,
    pub processed_event_receiver: Arc<Mutex<Option<tokio::sync::mpsc::Receiver<FileEvent>>>>,
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
        })
    }

    pub async fn watch(&mut self, path: &Path) -> Result<(), FileError> {
        if !path.exists() {
            let error_path = vec![path.to_path_buf()];
            return Err(FileError::new(
                FileErrorKind::PathNotFoundError,
                Some(error_path),
            ));
        }
        println!("Watching path: {:?}", path);

        if let Some(watcher) = self.watcher.as_mut() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            println!("Watcher ready! {:?}", watcher);

            let r_rx_clone = Arc::clone(self.raw_event_receiver.as_ref().unwrap());

            let p_tx_clone = self
                .processed_event_sender
                .as_ref()
                .expect("Processed event handler has not been initialized")
                .clone();

            tokio::spawn(async move {
                println!("Spawned thread! Raw Receiver: {:?}", r_rx_clone);
                while let Some(res) = r_rx_clone.lock().await.recv().await {
                    println!("Received events!");
                    match res {
                        Ok(events) => {
                            Self::to_file_event_and_send(events, &p_tx_clone).await;
                        }
                        Err(e) => {
                            println!("errors: {:?}", e);
                        }
                    }
                }
            });
        }

        Ok(())
    }

    async fn to_file_event_and_send(
        events: Vec<DebouncedEvent>,
        processed_event_tx: &Sender<FileEvent>,
    ) {
        for event in events {
            let kind = event.kind;
            let paths = event.paths.to_owned();
            let file_id = match kind {
                EventKind::Create(CreateKind::File)
                | EventKind::Modify(_)
                | EventKind::Remove(RemoveKind::File) => {
                    if !paths.is_empty() {
                        match FileId::extract(event.paths[0].as_path()).await {
                            Ok(file_id) => file_id,
                            Err(e) => {
                                println!("Error occurred: {:?}", e);
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
                }
                EventKind::Create(CreateKind::Folder) => {
                    if !paths.is_empty() {
                        match FileId::extract(event.paths[0].as_path()).await {
                            Ok(file_id) => file_id,
                            Err(e) => continue,
                        }
                    } else {
                        continue;
                    }
                }
                EventKind::Remove(RemoveKind::Folder) => {
                    if !paths.is_empty() {
                        match FileId::extract(event.paths[0].as_path()).await {
                            Ok(file_id) => file_id,
                            Err(e) => continue,
                        }
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };

            let file_event = FileEvent::new(event, kind, file_id, paths);
            println!("Constructed FileEvent from Raw Stream");

            if let Err(e) = processed_event_tx.send(file_event).await {
                println!("Error sending processed event into channel: {:?}", e);
            } else {
                println!("Sending processed event successful")
            }
        }
    }
}

impl FileEvent {
    fn new(event: DebouncedEvent, kind: EventKind, file_id: FileId, paths: Vec<PathBuf>) -> Self {
        FileEvent {
            event,
            paths,
            kind,
            file_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Import items from the parent module
    use notify::event::{ModifyKind, RemoveKind, RenameMode}; // Added RenameMode
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir; // For creating temporary directories for tests
    use tokio::sync::mpsc::error::TryRecvError; // Added for try_recv

    // Helper function to create a FileWatcher instance for tests
    async fn create_test_watcher() -> FileWatcher {
        let mut watcher = FileWatcher::new().await.expect("Failed to create watcher");
        watcher.init_watcher().await;
        watcher
    }

    #[tokio::test]
    async fn on_create_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("test_file.txt");

        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path())
            .await
            .expect("Failed to watch temp directory");

        // Give the watcher a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Create a file to trigger an event
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();
        drop(file); // Close the file

        // Wait for the event to be processed
        // The debouncer is set to 2 seconds, plus some buffer
        tokio::time::sleep(Duration::from_secs(3)).await;

        let mut receiver_lock = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx) = *receiver_lock {
            match tokio::time::timeout(Duration::from_secs(1), rx.recv()).await {
                Ok(Some(event)) => {
                    assert!(matches!(event.kind, EventKind::Create(CreateKind::File)));
                    assert_eq!(event.paths, vec![file_path.clone()]);
                    // Further assertions on file_id if necessary,
                    // e.g. check if it's not default or matches expected hash
                    let expected_file_id = FileId::extract(&file_path).await.unwrap();
                    assert_eq!(event.file_id, expected_file_id);
                }
                Ok(None) => panic!("Processed event channel closed prematurely"),
                Err(_) => panic!("Timeout waiting for processed event for create"),
            }
        } else {
            panic!("Processed event receiver was not initialized for create");
        }
    }

    #[tokio::test]
    async fn on_modify_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("mod_test_file.txt");

        // 1. Create and write initial content
        let mut file = fs::File::create(&file_path).expect("Failed to create test file");
        writeln!(file, "Initial content").expect("Failed to write initial content");
        drop(file); // Ensure file is closed

        // 2. Create and start watcher
        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path())
            .await
            .expect("Failed to watch temp directory");

        // Give watcher time to pick up initial create event
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        // 3. Drain initial create events
        let mut receiver_lock_drain = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_drain) = *receiver_lock_drain {
            println!("Draining initial events...");
            let mut drained_count = 0;
            loop {
                match rx_drain.try_recv() {
                    Ok(event) => {
                        drained_count += 1;
                        println!("Drained event: {:?}", event);
                    }
                    Err(TryRecvError::Empty) => {
                        println!(
                            "No more initial events to drain. Drained {} events.",
                            drained_count
                        );
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("Event channel disconnected while draining initial events");
                    }
                }
            }
        } else {
            panic!("Processed event receiver was not initialized for draining");
        }
        drop(receiver_lock_drain); // Release lock

        // 4. Modify the file
        fs::write(&file_path, "New content").expect("Failed to write new content to file");
        println!("File modified: {:?}", file_path);

        // 5. Wait for the modify event to be processed
        // Debouncer is 2 seconds, wait a bit longer.
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 6. Receive and assert the modify event
        let mut receiver_lock_modify = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_modify) = *receiver_lock_modify {
            match tokio::time::timeout(Duration::from_secs(2), rx_modify.recv()).await {
                Ok(Some(event)) => {
                    println!("Received event after modify: {:?}", event);
                    // Assert event kind is Modify (any sub-kind of Modify is fine)
                    assert!(
                        matches!(event.kind, EventKind::Modify(_)),
                        "Event kind was not Modify: {:?}",
                        event.kind
                    );
                    // Assert path is correct
                    assert_eq!(event.paths, vec![file_path.clone()]);
                    // Assert FileId is correct
                    let expected_file_id = FileId::extract(&file_path)
                        .await
                        .expect("Failed to extract FileId for modified file");
                    assert_eq!(event.file_id, expected_file_id);
                }
                Ok(None) => {
                    panic!("Processed event channel closed prematurely before modify event")
                }
                Err(_) => panic!("Timeout waiting for processed modify event"),
            }
        } else {
            panic!("Processed event receiver was not initialized for modify event");
        }
    }

    #[tokio::test]
    async fn on_delete_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("del_test_file.txt");

        // 1. Create the file
        fs::File::create(&file_path).expect("Failed to create test file for deletion test");
        // Content doesn't matter, ensure it's closed by dropping the File object implicitly.

        // 2. Create and start watcher
        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path())
            .await
            .expect("Failed to watch temp directory for deletion test");

        // 3. Wait and drain initial create events
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        let mut receiver_lock_drain = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_drain) = *receiver_lock_drain {
            println!("Draining initial events for deletion test...");
            let mut drained_count = 0;
            loop {
                match rx_drain.try_recv() {
                    Ok(event) => {
                        drained_count += 1;
                        println!("Drained event (delete test): {:?}", event);
                    }
                    Err(TryRecvError::Empty) => {
                        println!(
                            "No more initial events to drain (delete test). Drained {} events.",
                            drained_count
                        );
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("Event channel disconnected while draining initial events (delete test)");
                    }
                }
            }
        } else {
            panic!("Processed event receiver was not initialized for draining (delete test)");
        }
        drop(receiver_lock_drain); // Release lock

        // 4. Delete the file
        fs::remove_file(&file_path).expect("Failed to delete test file");
        println!("File deleted: {:?}", file_path);

        // 5. Wait for the delete event to be processed
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        // 6. Receive and assert the delete event
        let mut receiver_lock_delete = watcher.processed_event_receiver.lock().await;
        println!("Processed Event Receiver has been acquired!");
        if let Some(ref mut rx_delete) = *receiver_lock_delete {
            match tokio::time::timeout(Duration::from_secs(2), rx_delete.recv()).await {
                Ok(Some(event)) => {
                    println!("Received event after delete: {:?}", event);
                    // Assert event kind is RemoveKind::File
                    assert!(
                        matches!(event.kind, EventKind::Remove(RemoveKind::File)),
                        "Event kind was not Remove(File): {:?}",
                        event.kind
                    );
                    // Assert path is correct
                    assert_eq!(event.paths, vec![file_path.clone()]);
                    // Assert FileId is correct (should be from_path as file is deleted)
                    let expected_file_id = FileId::extract(&file_path).await.unwrap();
                    assert_eq!(
                        event.file_id, expected_file_id,
                        "FileId did not match expected from_path ID"
                    );
                }
                Ok(None) => {
                    panic!("Processed event channel closed prematurely before delete event")
                }
                Err(_) => panic!("Timeout waiting for processed delete event"),
            }
        } else {
            panic!("Processed event receiver was not initialized for delete event");
        }
    }

    #[tokio::test]
    async fn on_rename_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let old_file_path = tmp_dir.path().join("rename_old.txt");
        let new_file_path = tmp_dir.path().join("rename_new.txt");

        // 1. Create the file at the old path
        fs::File::create(&old_file_path)
            .expect("Failed to create test file for rename test (old_file_path)");

        // 2. Create and start watcher
        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path())
            .await
            .expect("Failed to watch temp directory for rename test");

        // 3. Wait and drain initial create events
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        let mut receiver_lock_drain = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_drain) = *receiver_lock_drain {
            println!("Draining initial events for rename test...");
            let mut drained_count = 0;
            loop {
                match rx_drain.try_recv() {
                    Ok(event) => {
                        drained_count += 1;
                        println!("Drained event (rename test): {:?}", event);
                    }
                    Err(TryRecvError::Empty) => {
                        println!(
                            "No more initial events to drain (rename test). Drained {} events.",
                            drained_count
                        );
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("Event channel disconnected while draining initial events (rename test)");
                    }
                }
            }
        } else {
            panic!("Processed event receiver was not initialized for draining (rename test)");
        }
        drop(receiver_lock_drain);

        // 4. Rename the file
        fs::rename(&old_file_path, &new_file_path).expect("Failed to rename test file");
        println!(
            "File renamed from {:?} to {:?}",
            old_file_path, new_file_path
        );

        // 5. Wait for the rename event to be processed
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        // 6. Receive and assert the rename event
        let mut receiver_lock_rename = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_rename) = *receiver_lock_rename {
            match tokio::time::timeout(Duration::from_secs(2), rx_rename.recv()).await {
                Ok(Some(event)) => {
                    println!("Received event after rename: {:?}", event);
                    // Assert event kind is ModifyKind::Name(RenameMode::Both)
                    assert!(
                        matches!(
                            event.kind,
                            EventKind::Modify(ModifyKind::Name(RenameMode::Both))
                        ),
                        "Event kind was not Modify(Name(Both)): {:?}",
                        event.kind
                    );
                    // Assert paths are correct (old and new)
                    // The order of paths in the event can sometimes vary by platform or notify backend.
                    // We should check that both paths are present and the order.
                    // The current implementation of `notify-debouncer-full` seems to provide [from, to]
                    assert_eq!(
                        event.paths,
                        vec![old_file_path.clone(), new_file_path.clone()],
                        "Event paths did not match expected [old_path, new_path]"
                    );
                    // Assert FileId is based on the new path
                    let expected_file_id = FileId::extract(&new_file_path).await.unwrap();
                    assert_eq!(
                        event.file_id, expected_file_id,
                        "FileId did not match expected from_path for new_file_path"
                    );
                }
                Ok(None) => {
                    panic!("Processed event channel closed prematurely before rename event")
                }
                Err(_) => panic!("Timeout waiting for processed rename event"),
            }
        } else {
            panic!("Processed event receiver was not initialized for rename event");
        }
    }

    #[tokio::test]
    async fn on_folder_create_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let folder_path = tmp_dir.path().join("new_test_folder");

        // 1. Create and start watcher
        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path()) // Watch the base directory
            .await
            .expect("Failed to watch temp directory for folder creation test");

        // 2. Give watcher a moment to initialize
        tokio::time::sleep(Duration::from_millis(100)).await;

        // 3. Create the new folder
        fs::create_dir(&folder_path).expect("Failed to create test folder");
        println!("Folder created: {:?}", folder_path);

        // 4. Wait for the folder creation event to be processed
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        // 5. Receive and assert the folder creation event
        let mut receiver_lock_folder_create = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_folder_create) = *receiver_lock_folder_create {
            match tokio::time::timeout(Duration::from_secs(2), rx_folder_create.recv()).await {
                Ok(Some(event)) => {
                    println!("Received event after folder create: {:?}", event);
                    // Assert event kind is CreateKind::Folder
                    assert!(
                        matches!(event.kind, EventKind::Create(CreateKind::Folder)),
                        "Event kind was not Create(Folder): {:?}",
                        event.kind
                    );
                    // Assert path is correct
                    assert_eq!(event.paths, vec![folder_path.clone()]);
                    // Assert oileId is based on the folder path
                    let expected_file_id = FileId::extract(&folder_path).await.unwrap();
                    assert_eq!(
                        event.file_id, expected_file_id,
                        "FileId did not match expected from_path for new_folder_path"
                    );
                }
                Ok(None) => {
                    panic!("Processed event channel closed prematurely before folder create event")
                }
                Err(_) => panic!("Timeout waiting for processed folder create event"),
            }
        } else {
            panic!("Processed event receiver was not initialized for folder create event");
        }
    }

    #[tokio::test]
    async fn on_folder_delete_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let folder_path = tmp_dir.path().join("del_test_folder");

        // 1. Create the folder
        fs::create_dir(&folder_path).expect("Failed to create test folder for delete test");

        // 2. Create and start watcher
        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path()) // Watch the base directory
            .await
            .expect("Failed to watch temp directory for folder delete test");

        // 3. Wait and drain initial folder creation events
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        let mut receiver_lock_drain = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_drain) = *receiver_lock_drain {
            println!("Draining initial events for folder delete test...");
            let mut drained_count = 0;
            loop {
                match rx_drain.try_recv() {
                    Ok(event) => {
                        drained_count += 1;
                        println!("Drained event (folder delete test): {:?}", event);
                    }
                    Err(TryRecvError::Empty) => {
                        println!("No more initial events to drain (folder delete test). Drained {} events.", drained_count);
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("Event channel disconnected while draining initial events (folder delete test)");
                    }
                }
            }
        } else {
            panic!(
                "Processed event receiver was not initialized for draining (folder delete test)"
            );
        }
        drop(receiver_lock_drain); // Release lock

        // 4. Delete the folder
        fs::remove_dir(&folder_path).expect("Failed to delete test folder");
        println!("Folder deleted: {:?}", folder_path);

        // 5. Wait for the folder delete event to be processed
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        // 6. Receive and assert the folder delete event
        let mut receiver_lock_folder_delete = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_folder_delete) = *receiver_lock_folder_delete {
            match tokio::time::timeout(Duration::from_secs(2), rx_folder_delete.recv()).await {
                Ok(Some(event)) => {
                    println!("Received event after folder delete: {:?}", event);
                    // Assert event kind is RemoveKind::Folder
                    assert!(
                        matches!(event.kind, EventKind::Remove(RemoveKind::Folder)),
                        "Event kind was not Remove(Folder): {:?}",
                        event.kind
                    );
                    // Assert path is correct
                    assert_eq!(event.paths, vec![folder_path.clone()]);
                    // Assert FileId is based on the folder path
                    let expected_file_id = FileId::extract(&folder_path).await.unwrap();
                    assert_eq!(
                        event.file_id, expected_file_id,
                        "FileId did not match expected from_path for deleted_folder_path"
                    );
                }
                Ok(None) => {
                    panic!("Processed event channel closed prematurely before folder delete event")
                }
                Err(_) => panic!("Timeout waiting for processed folder delete event"),
            }
        } else {
            panic!("Processed event receiver was not initialized for folder delete event");
        }
    }

    #[tokio::test]
    async fn on_folder_rename_emit_correct_event() {
        let tmp_dir = tempdir().unwrap();
        let old_folder_path = tmp_dir.path().join("rename_old_folder");
        let new_folder_path = tmp_dir.path().join("rename_new_folder");

        // 1. Create the folder at the old path
        fs::create_dir(&old_folder_path)
            .expect("Failed to create test folder for rename test (old_folder_path)");

        // 2. Create and start watcher
        let mut watcher = create_test_watcher().await;
        watcher
            .watch(tmp_dir.path()) // Watch the base directory
            .await
            .expect("Failed to watch temp directory for folder rename test");

        // 3. Wait and drain initial folder creation events
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        let mut receiver_lock_drain = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_drain) = *receiver_lock_drain {
            println!("Draining initial events for folder rename test...");
            let mut drained_count = 0;
            loop {
                match rx_drain.try_recv() {
                    Ok(event) => {
                        drained_count += 1;
                        println!("Drained event (folder rename test): {:?}", event);
                    }
                    Err(TryRecvError::Empty) => {
                        println!("No more initial events to drain (folder rename test). Drained {} events.", drained_count);
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        panic!("Event channel disconnected while draining initial events (folder rename test)");
                    }
                }
            }
        } else {
            panic!(
                "Processed event receiver was not initialized for draining (folder rename test)"
            );
        }
        drop(receiver_lock_drain); // Release lock

        // 4. Rename the folder
        fs::rename(&old_folder_path, &new_folder_path).expect("Failed to rename test folder");
        println!(
            "Folder renamed from {:?} to {:?}",
            old_folder_path, new_folder_path
        );

        // 5. Wait for the folder rename event to be processed
        tokio::time::sleep(Duration::from_secs(3)).await; // Debouncer is 2s

        // 6. Receive and assert the folder rename event
        let mut receiver_lock_folder_rename = watcher.processed_event_receiver.lock().await;
        if let Some(ref mut rx_folder_rename) = *receiver_lock_folder_rename {
            match tokio::time::timeout(Duration::from_secs(2), rx_folder_rename.recv()).await {
                Ok(Some(event)) => {
                    println!("Received event after folder rename: {:?}", event);
                    // Assert event kind is ModifyKind::Name(RenameMode::Both)
                    assert!(
                        matches!(
                            event.kind,
                            EventKind::Modify(ModifyKind::Name(RenameMode::Both))
                        ),
                        "Event kind was not Modify(Name(Both)) for folder rename: {:?}",
                        event.kind
                    );
                    // Assert paths are correct (old and new)
                    assert_eq!(
                        event.paths,
                        vec![old_folder_path.clone(), new_folder_path.clone()],
                        "Event paths did not match expected [old_folder_path, new_folder_path]"
                    );
                    // Assert FileId is based on the new folder path
                    let expected_file_id = FileId::extract(&new_folder_path).await.unwrap();
                    assert_eq!(
                        event.file_id, expected_file_id,
                        "FileId did not match expected from_path for new_folder_path"
                    );
                }
                Ok(None) => {
                    panic!("Processed event channel closed prematurely before folder rename event")
                }
                Err(_) => panic!("Timeout waiting for processed folder rename event"),
            }
        } else {
            panic!("Processed event receiver was not initialized for folder rename event");
        }
    }

    #[tokio::test]
    async fn on_watch_non_existent_path_return_error() {
        // 1. Create a temporary directory (base for non-existent path)
        let tmp_dir = tempdir().unwrap();

        // 2. Define a non-existent path
        let non_existent_path = tmp_dir.path().join("this_path_should_not_exist");

        // 3. Get an initialized FileWatcher instance
        // Note: create_test_watcher also calls init_watcher()
        let mut watcher = create_test_watcher().await;

        // 4. Call watcher.watch() on the non-existent path
        let result = watcher.watch(&non_existent_path).await;
        println!("Watch result for non-existent path: {:?}", result);

        // 5. Assert that the result is an Err
        assert!(
            result.is_err(),
            "Watching a non-existent path should return an error."
        );
        // 6. Assert that the kind of the notify::Error is notify::ErrorKind::PathNotFound
        if let Err(err) = result {
            assert_eq!(
                err.kind,
                FileErrorKind::PathNotFoundError,
                "Error kind should be PathNotFound for a non-existent watch path."
            );
        } else {
            // This branch should not be reached if the previous assert passed
            panic!("Result was not an Err, contrary to assertion.");
        }
    }
}
