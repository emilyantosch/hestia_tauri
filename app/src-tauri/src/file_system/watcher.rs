use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::file_system::FileId;

#[derive(Debug)]
pub struct FileEvent {
    event: DebouncedEvent,
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
    pub processed_event_receiver: Option<Arc<Mutex<tokio::sync::mpsc::Receiver<FileEvent>>>>,
}

impl FileWatcher {
    pub async fn init_watcher(&mut self) {
        let (r_tx, r_rx) = tokio::sync::mpsc::channel(100);
        let rt = tokio::runtime::Handle::current();
        let (p_tx, p_rx) = tokio::sync::mpsc::channel::<FileEvent>(100);
        let r_rx_arc = Arc::new(Mutex::new(r_rx));

        self.processed_event_receiver = Some(Arc::new(Mutex::new(p_rx)));
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
            processed_event_receiver: None,
        })
    }

    pub async fn watch(&mut self, path: &Path) -> notify::Result<()> {
        if !path.exists() {
            return Err(notify::Error::path_not_found().add_path(path.to_path_buf()));
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
                            Self::to_file_event_and_send(events, p_tx_clone).await;
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

    pub async fn to_file_event_and_send(
        events: Vec<DebouncedEvent>,
        processed_event_tx: Sender<FileEvent>,
    ) {
        for event in events {
            println!("Event unwrapped!");
            match event.kind {
                EventKind::Create(CreateKind::File) => {
                    let file_id = FileId::extract(event.paths[0].as_path()).await.unwrap();

                    println!(
                        "File created at path: {:?}, with ID {:?}",
                        event.paths[0], file_id
                    );
                }
                EventKind::Create(CreateKind::Folder) => println!("Folder was created!"),
                EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                    println!("Rename happened!")
                }
                EventKind::Remove(RemoveKind::File) => println!("Rename happened!"),
                EventKind::Remove(RemoveKind::Folder) => println!("Rename happened!"),
                _ => println!("Something else happened!"),
            }
        }
    }
}

impl FileEvent {
    fn new(event: DebouncedEvent, kind: EventKind, file_id: FileId) -> Self {
        FileEvent {
            event,
            kind,
            file_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FileWatcher;
    use std::path::Path;

    #[tokio::test]
    async fn on_create_emit_correct_event() {
        let path = Path::new("./../../../../test_vault/");
        let mut watcher = FileWatcher::new().await.unwrap();
        watcher.init_watcher().await;
        watcher.watch(path).await.unwrap();
    }
}
