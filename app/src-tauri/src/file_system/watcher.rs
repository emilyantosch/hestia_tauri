use entity::files;
use notify::event::{CreateKind, EventKind, ModifyKind, RemoveKind, RenameMode};
use notify::{Error, RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{
    new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, RecommendedCache,
};
use sea_orm::ActiveValue::Set;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

use crate::file_system::FileId;

#[derive(Debug)]
pub struct FileWatcher {
    watcher: Option<Debouncer<RecommendedWatcher, RecommendedCache>>,
    receiver: Option<Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<Error>>>>,
}

impl FileWatcher {
    pub async fn init_watcher(&mut self) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let rt = tokio::runtime::Handle::current();

        let debouncer = new_debouncer(
            Duration::from_secs(2),
            None,
            move |result: DebounceEventResult| {
                let tx = tx.clone();

                println!("Calling by notify => {:?}", &result);
                rt.spawn(async move {
                    if let Err(e) = tx.send(result).await {
                        println!("Error sending event result: {:?}", e);
                    };
                });
            },
        );

        match debouncer {
            Ok(watcher) => {
                println!("Init of FileWatcher completed successfully!");
                self.watcher = Some(watcher);
                self.receiver = Some(rx);
            }
            Err(e) => println!("{:?}", e),
        };
    }
    pub async fn new() -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            watcher: None,
            receiver: None,
        })
    }

    pub async fn watch(&mut self, path: &PathBuf) -> notify::Result<()> {
        println!("Does path exist?");
        if !path.exists() {
            panic!();
        }
        println!("Watching path: {:?}", path);

        if let Some(watcher) = self.watcher.as_mut() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            println!("Watcher ready! {:?}", watcher);

            if let Some(mut rx) = self.receiver.take() {
                println!("RX taken out of Option: {:?}", rx);
                tokio::spawn(async move {
                    println!("Spawned thread! Receiver: {:?}", rx);
                    while let Some(res) = rx.recv().await {
                        println!("Received events!");
                        match res {
                            Ok(events) => {
                                Self::to_database(events).await;
                            }
                            Err(e) => {
                                println!("errors: {:?}", e);
                            }
                        }
                    }
                });
            }
        }
        Ok(())
    }

    pub async fn to_database(events: Vec<DebouncedEvent>) {
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
