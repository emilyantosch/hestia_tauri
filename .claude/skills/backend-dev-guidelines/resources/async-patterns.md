# Async Patterns with Tokio

## Overview

Asynchronous programming is fundamental to Rust backend development with Tauri. The `tokio` runtime provides powerful primitives for concurrent operations, background task spawning, and efficient I/O handling. This guide covers async patterns used throughout the Hestia codebase.

### Why Async?

Async code allows your application to:
- Handle multiple operations concurrently without blocking
- Spawn background tasks that run independently
- Manage I/O-bound operations efficiently
- Build responsive applications that don't freeze

### Key Concepts

- **`async fn`**: Functions that return a `Future` and can use `.await`
- **`.await`**: Suspend execution until a Future completes
- **`tokio::spawn`**: Spawn a new task on the runtime
- **`tokio::select!`**: Wait for multiple futures, proceeding with the first to complete
- **`tokio::join!`**: Wait for multiple futures to all complete
- **Channels**: Send messages between tasks (`mpsc`, `oneshot`)

### When to Use Async

✅ **Use async for**:
- I/O operations (file system, network, database)
- Background processing and long-running tasks
- Event handling and message passing
- Concurrent operations that can run in parallel

❌ **Avoid async for**:
- CPU-intensive computations (use `tokio::task::spawn_blocking` instead)
- Simple synchronous operations where async adds complexity
- Code that doesn't benefit from concurrency

---

## Async Fundamentals

### Basic Async Function

Every async function returns a `Future` that must be `.await`ed:

```rust
use anyhow::Result;

// Simple async function
async fn fetch_data() -> Result<String> {
    // Simulate async operation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok("data".to_string())
}

// Calling async functions
async fn process() -> Result<()> {
    let data = fetch_data().await?;
    println!("Received: {}", data);
    Ok(())
}
```

**Key Points**:
- `async fn` automatically returns `impl Future<Output = Result<String>>`
- `.await` suspends execution until the Future completes
- Use `?` operator with `.await` for error propagation

### Async Trait Implementations

Use `async-trait` crate for async trait methods:

```rust
use anyhow::Result;

#[async_trait::async_trait]
pub trait FileWatcherEventHandler: Send + Sync {
    async fn handle_event(&self, event: FSEvent) -> Result<()>;
}

pub struct DatabaseFileWatcherEventHandler {
    pub db_operations: FileOperations,
}

#[async_trait::async_trait]
impl FileWatcherEventHandler for DatabaseFileWatcherEventHandler {
    async fn handle_event(&self, event: FSEvent) -> Result<()> {
        FileWatcher::to_database(event, &self.db_operations).await
    }
}
```

**From**: `app/src-tauri/src/file_system/watcher.rs:43-57`

**Key Points**:
- `#[async_trait::async_trait]` macro enables async methods in traits
- Trait must be `Send + Sync` for cross-thread usage
- All implementers must also use `#[async_trait::async_trait]`

---

## Tokio Runtime

### Runtime Handle

Access the current runtime handle for spawning tasks:

```rust
use tokio::runtime::Handle;

let rt = tokio::runtime::Handle::current();
rt.spawn(async move {
    // Background task
});
```

**From**: `app/src-tauri/src/file_system/watcher.rs:118`

### Spawning Tasks with tokio::spawn

Spawn independent tasks that run concurrently:

```rust
use tokio::sync::mpsc;
use tracing::{error, info};

// Spawn a message processing task
tokio::spawn(async move {
    while let Some(res) = r_rx.recv().await {
        match res {
            Ok(events) => {
                for event in events {
                    if let Err(e) = process_event(event).await {
                        error!("Failed to process event: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Errors: {:?}", e);
            }
        }
    }
});
```

**From**: `app/src-tauri/src/file_system/watcher.rs:134-149`

**Key Points**:
- `tokio::spawn` returns `JoinHandle<T>` for the spawned task
- Spawned task must be `Send + 'static` (no borrowed references)
- Use `move` to transfer ownership into the spawned task
- Errors inside spawned tasks don't automatically propagate

---

## Example 1: Multi-Pipeline Event Processing

**Pattern**: Spawn multiple processing pipelines with channels

```rust
use tokio::sync::mpsc;
use anyhow::Result;

pub async fn init_watcher(
    &mut self,
    event_handler: Box<dyn FileWatcherEventHandler>,
) -> Result<()> {
    // Raw event channel (from file system watcher)
    let (r_tx, mut r_rx) = tokio::sync::mpsc::channel(100);

    // Processed event channel
    let (p_tx, mut p_rx) = tokio::sync::mpsc::channel::<FSEvent>(100);

    // Get runtime handle for spawning from sync context
    let rt = tokio::runtime::Handle::current();

    // Pipeline 1: Raw event processor
    tokio::spawn(async move {
        while let Some(res) = r_rx.recv().await {
            match res {
                Ok(events) => {
                    for event in events {
                        if let Err(e) = to_file_or_folder_event_and_send(event, &p_tx).await {
                            error!("Failed to process event: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Errors: {:?}", e);
                }
            }
        }
    });

    // Pipeline 2: Event handler (database storage)
    tokio::spawn(async move {
        while let Some(event) = p_rx.recv().await {
            if let Err(e) = event_handler.handle_event(event).await {
                error!("Failed to store event to database: {:?}", e);
            }
        }
    });

    Ok(())
}
```

**From**: `app/src-tauri/src/file_system/watcher.rs:113-167`

**Pattern Explanation**:
1. Create multiple bounded channels for different stages
2. Spawn independent tasks for each pipeline stage
3. First task processes raw events, sends to next stage
4. Second task handles processed events (e.g., database storage)
5. Errors are logged but don't crash the pipeline

**Use When**:
- Building event-driven systems
- Creating processing pipelines with multiple stages
- Separating concerns across async boundaries

---

## Example 2: Worker Pool Pattern

**Pattern**: Spawn multiple workers that process jobs from a shared queue

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run(mut self) -> Result<()> {
    info!("Starting ThumbnailProcessor with {} workers", self.config.worker_count);

    // Shared job queue protected by Mutex
    let job_queue = Arc::new(Mutex::new(Vec::new()));

    // Spawn worker tasks
    let mut worker_handles = Vec::new();
    for worker_id in 0..self.config.worker_count {
        let worker = ThumbnailWorker::new(
            worker_id,
            Arc::clone(&job_queue),
            Arc::clone(&self.repository),
            Arc::clone(&self.generator),
            Arc::clone(&self.stats),
            self.config.clone(),
        );

        let shutdown_signal = Arc::clone(&self.shutdown_signal);
        let handle = tokio::spawn(async move {
            worker.run(shutdown_signal).await;
        });
        worker_handles.push(handle);
    }

    // Main message processing loop
    while let Some(message) = self.message_receiver.recv().await {
        self.handle_message(message).await?;
    }

    // Wait for all workers to complete
    for handle in worker_handles {
        if let Err(e) = handle.await {
            error!("Worker task failed: {}", e);
        }
    }

    Ok(())
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:324-400`

**Pattern Explanation**:
1. Create `N` worker tasks based on CPU core count
2. Share job queue using `Arc<Mutex<Vec<Job>>>`
3. Each worker polls queue for pending jobs
4. Coordinator task receives messages and queues jobs
5. Graceful shutdown waits for all workers

**Use When**:
- Processing large batches of independent tasks
- CPU-bound work that benefits from parallelism
- Background job processing systems

---

## Example 3: tokio::select! for Concurrent Operations

**Pattern**: Wait for multiple futures, process whichever completes first

```rust
use tokio::sync::Notify;
use std::sync::Arc;

pub async fn run(self, shutdown_signal: Arc<Notify>) {
    info!("Worker {} started", self.worker_id);

    loop {
        tokio::select! {
            // Branch 1: Shutdown signal
            _ = shutdown_signal.notified() => {
                info!("Worker {} received shutdown signal", self.worker_id);
                break;
            }

            // Branch 2: Periodic job polling
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                if let Some(job) = self.get_next_job().await {
                    if let Err(e) = self.process_job(job).await {
                        error!("Worker {} failed to process job: {}", self.worker_id, e);
                    }
                }
            }
        }
    }

    info!("Worker {} stopped", self.worker_id);
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:134-154`

**Pattern Explanation**:
1. `tokio::select!` races multiple async operations
2. First branch to complete executes its code block
3. Other branches are cancelled
4. Useful for shutdown signals + work loops

**Use When**:
- Implementing cancellable operations
- Handling timeouts alongside work
- Building event-driven state machines
- Graceful shutdown patterns

### Stats Updater with select!

```rust
async fn spawn_stats_updater(&self) -> tokio::task::JoinHandle<()> {
    let stats = Arc::clone(&self.stats);
    let shutdown_signal = Arc::clone(&self.shutdown_signal);

    tokio::spawn(async move {
        let mut last_completed = 0u64;
        let mut last_update = Instant::now();

        loop {
            tokio::select! {
                _ = shutdown_signal.notified() => break,
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    let mut stats_guard = stats.lock().await;
                    let now = Instant::now();
                    let elapsed = now.duration_since(last_update).as_secs_f64();

                    if elapsed > 0.0 {
                        let completed_delta = stats_guard.completed_jobs - last_completed;
                        stats_guard.throughput_per_second = completed_delta as f64 / elapsed;
                    }

                    last_completed = stats_guard.completed_jobs;
                    last_update = now;
                }
            }
        }
    })
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:532-559`

---

## Example 4: Timeouts with tokio::time::timeout

**Pattern**: Limit how long an operation can take

```rust
use tokio::time::{timeout, Duration};
use anyhow::Result;

async fn process_job(&self, job: ThumbnailJob) -> Result<()> {
    let start_time = Instant::now();

    debug!("Worker {} processing thumbnail for file {}", self.worker_id, job.file_id);

    // Generate thumbnail with timeout
    let generation_result = timeout(
        self.config.processing_timeout,
        self.generator.generate_from_file_path(&job.file_path, job.size),
    )
    .await;

    match generation_result {
        Ok(Ok(thumbnail)) => {
            // Save thumbnail to database
            self.repository.create_thumbnail(job.file_id, thumbnail).await?;

            let processing_time = start_time.elapsed();
            info!("Worker {} generated thumbnail in {:?}", self.worker_id, processing_time);
        }
        Ok(Err(e)) => {
            warn!("Worker {} failed to generate thumbnail: {}", self.worker_id, e);
            self.handle_failed_job(job).await;
            return Err(e);
        }
        Err(_) => {
            error!("Worker {} thumbnail generation timed out", self.worker_id);
            self.handle_failed_job(job).await;
            return Err(ThumbnailError::GenerationFailed {
                reason: "Thumbnail generation timed out".to_string(),
            })?;
        }
    }

    Ok(())
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:171-238`

**Pattern Explanation**:
1. Wrap async operation in `timeout(duration, future)`
2. Returns `Result<Result<T, E>, Elapsed>`
3. Outer `Result` is timeout error (Elapsed)
4. Inner `Result` is operation's own error
5. Handle both timeout and operation errors

**Use When**:
- Operations that might hang indefinitely
- User-facing operations that need responsiveness
- Resource-intensive tasks with time limits

---

## Example 5: Retry Logic with Exponential Backoff

**Pattern**: Retry failed operations with increasing delays

```rust
use tokio::time::Duration;

async fn handle_failed_job(&self, mut job: ThumbnailJob) {
    job.retry_count += 1;

    if job.retry_count < self.config.max_retries {
        // Retry the job after delay
        job.status = ThumbnailJobStatus::Pending;

        // Exponential backoff: delay increases with each retry
        let delay = self.config.retry_delay * job.retry_count;
        info!(
            "Worker {} retrying failed job (attempt {}/{})",
            self.worker_id, job.retry_count, self.config.max_retries
        );

        tokio::spawn({
            let job_queue = Arc::clone(&self.job_queue);
            async move {
                tokio::time::sleep(delay).await;
                let mut queue = job_queue.lock().await;
                queue.push(job);
            }
        });
    } else {
        // Move to failed jobs
        job.status = ThumbnailJobStatus::Failed;

        let mut stats = self.stats.lock().await;
        stats.failed_jobs += 1;

        warn!(
            "Worker {} job permanently failed after {} retries",
            self.worker_id, self.config.max_retries
        );
    }
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:240-273`

**Pattern Explanation**:
1. Track retry count on job
2. Multiply delay by retry count for exponential backoff
3. Spawn separate task to re-queue job after delay
4. Mark as permanently failed after max retries

**Use When**:
- Operations that may fail transiently (network, external services)
- Resource contention scenarios
- Building resilient systems

---

## Example 6: Message Passing with Channels

**Pattern**: Communicate between tasks using typed channels

```rust
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum ThumbnailMessage {
    QueueFiles {
        file_infos: Vec<File>,
        sizes: Vec<ThumbnailSize>,
    },
    QueueSingleFile {
        file_id: i32,
        file_path: PathBuf,
        size: ThumbnailSize,
    },
    GetStats {
        respond_to: oneshot::Sender<ProcessingStats>,
    },
    Shutdown,
}

pub struct ThumbnailProcessorHandler {
    pub sender: mpsc::UnboundedSender<ThumbnailMessage>,
}

impl ThumbnailProcessorHandler {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ThumbnailMessage>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    pub async fn queue_files(
        &self,
        file_infos: Vec<File>,
        sizes: Vec<ThumbnailSize>,
    ) -> Result<()> {
        self.sender.send(ThumbnailMessage::QueueFiles { file_infos, sizes })?;
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<ProcessingStats> {
        let (respond_to, response) = oneshot::channel();

        self.sender.send(ThumbnailMessage::GetStats { respond_to })?;

        Ok(response.await?)
    }
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:43-653`

**Pattern Explanation**:
1. Define message enum with all possible commands
2. Use `mpsc::unbounded_channel` for command sending
3. Use `oneshot::channel` for request-response pattern
4. Handler provides type-safe API over raw channel

**Use When**:
- Actor-like patterns with message-driven state
- Background services with external control
- Decoupling components in async systems

---

## Example 7: Spawning Long-Running Services

**Pattern**: Spawn application services at startup

```rust
use anyhow::Result;

pub async fn create_file_watcher(&mut self) -> Result<()> {
    let (fw_sender, fw_receiver) = tokio::sync::mpsc::unbounded_channel();

    // Store handler for sending messages
    self.set_file_watcher_handler(FileWatcherHandler { sender: fw_sender });

    // Create event handler with database operations
    let fw_file_operations = FileOperations::new(Arc::clone(&self.database_manager));
    let fw_event_handler = DatabaseFileWatcherEventHandler {
        db_operations: fw_file_operations,
    };

    // Spawn long-running watcher task
    tokio::spawn(async move {
        if let Err(e) = FileWatcher::new(fw_receiver)
            .run(Box::new(fw_event_handler))
            .await
        {
            error!("FileWatcher could not be created due to {e:#?}!")
        }
    });

    Ok(())
}
```

**From**: `app/src-tauri/src/config/app.rs:258-276`

**Pattern Explanation**:
1. Create channels for communication
2. Store sender in application state
3. Spawn receiver task with `tokio::spawn`
4. Long-running task processes messages until channel closes
5. Errors logged but don't crash main application

**Use When**:
- File system watchers
- Background job processors
- Event streams and monitoring

---

## Example 8: Async in Application State Initialization

**Pattern**: Initialize async components during app startup

```rust
pub async fn switch_library(&mut self, library: Library) -> Result<()> {
    info!("Switching to library: {:?}", library.library_config);

    // Update library
    self.library = library;

    // Get new database path
    let db_path = self.library.get_canon_database_path()?;
    info!("New database path: {:?}", db_path);

    // Update database connection
    self.update_database_connection(db_path).await?;

    // Recreate dependent components with new database connection
    self.reinitialize_components().await?;

    info!("Successfully switched library");
    Ok(())
}

async fn reinitialize_components(&mut self) -> Result<()> {
    info!("Reinitializing components with new database connection");

    // Recreate file operations with new database manager
    self.file_operations = FileOperations::new(Arc::clone(&self.database_manager));

    // Preload file type cache for better performance
    if let Err(e) = self.file_operations.preload_file_type_cache().await {
        tracing::warn!("Warning: Failed to preload file type cache: {:?}", e);
    }

    // Recreate directory scanner with new file operations
    let file_operations_for_scanner = FileOperations::new(Arc::clone(&self.database_manager));
    self.directory_scanner = DirectoryScanner::new(Arc::new(file_operations_for_scanner));

    // Recreate thumbnail engine
    self.create_new_thumbnail_engine(self.database_manager.clone()).await?;

    info!("Successfully reinitialized components");
    Ok(())
}
```

**From**: `app/src-tauri/src/config/app.rs:88-159`

**Pattern Explanation**:
1. Library switching requires async database connection update
2. Sequential initialization of dependent components
3. Each component initialized with proper error handling
4. Non-critical errors (cache preload) logged as warnings

---

## Channels Overview

### Channel Types

| Type | Capacity | Use Case |
|------|----------|----------|
| `mpsc::unbounded_channel` | Unlimited | Command channels, non-blocking senders |
| `mpsc::channel(n)` | Bounded | Backpressure control, memory limits |
| `oneshot::channel` | Single value | Request-response pattern |
| `broadcast::channel(n)` | Multiple receivers | Event broadcasting |
| `watch::channel` | Latest value | State monitoring |

### mpsc (Multi-Producer, Single-Consumer)

```rust
use tokio::sync::mpsc;

// Unbounded channel (unlimited buffer)
let (tx, mut rx) = mpsc::unbounded_channel::<String>();

// Bounded channel (backpressure)
let (tx, mut rx) = mpsc::channel::<String>(100);

// Sending
tx.send("message".to_string())?;

// Receiving
while let Some(msg) = rx.recv().await {
    println!("Received: {}", msg);
}
```

### oneshot (Single Value)

```rust
use tokio::sync::oneshot;

let (tx, rx) = oneshot::channel::<String>();

// Send exactly one value
let _ = tx.send("response".to_string());

// Receive the value
let value = rx.await?;
```

---

## Best Practices

### DO ✅

1. **Use anyhow::Result for internal async functions**
   ```rust
   async fn internal_operation() -> anyhow::Result<Data> {
       database_query().await.context("Failed to query database")?;
       Ok(data)
   }
   ```

2. **Spawn tasks with clear error handling**
   ```rust
   tokio::spawn(async move {
       if let Err(e) = background_task().await {
           error!("Background task failed: {}", e);
       }
   });
   ```

3. **Use Arc for sharing across tasks**
   ```rust
   let shared_state = Arc::new(Mutex::new(State::new()));
   let state_clone = Arc::clone(&shared_state);
   tokio::spawn(async move {
       let mut state = state_clone.lock().await;
       // Use state
   });
   ```

4. **Instrument async functions for observability**
   ```rust
   use tracing::{info, instrument};

   #[instrument(skip(db))]
   async fn process_data(id: i32, db: &DatabaseConnection) -> Result<()> {
       info!("Processing data for ID: {}", id);
       // Implementation
       Ok(())
   }
   ```

5. **Use tokio::select! for cancellation**
   ```rust
   tokio::select! {
       _ = shutdown_signal.notified() => {
           // Clean shutdown
       }
       result = work_task() => {
           // Handle work result
       }
   }
   ```

### DON'T ❌

1. **Don't use async for CPU-bound work**
   ```rust
   // ❌ BAD: Blocks async runtime
   async fn compute_heavy() -> i64 {
       (0..1_000_000_000).sum()
   }

   // ✅ GOOD: Use spawn_blocking
   async fn compute_heavy() -> i64 {
       tokio::task::spawn_blocking(|| {
           (0..1_000_000_000).sum()
       }).await.unwrap()
   }
   ```

2. **Don't hold locks across .await points**
   ```rust
   // ❌ BAD: Lock held across await
   let mut data = mutex.lock().await;
   expensive_async_operation().await;
   data.update();

   // ✅ GOOD: Release lock before await
   let value = {
       let data = mutex.lock().await;
       data.clone()
   };
   expensive_async_operation().await;
   ```

3. **Don't ignore JoinHandle errors**
   ```rust
   // ❌ BAD: Task panic goes unnoticed
   tokio::spawn(async { panic!("oops") });

   // ✅ GOOD: Handle task completion
   let handle = tokio::spawn(async { work().await });
   if let Err(e) = handle.await {
       error!("Task failed: {}", e);
   }
   ```

4. **Don't create unbounded task queues without backpressure**
   ```rust
   // ❌ BAD: Memory can grow unbounded
   let (tx, rx) = mpsc::unbounded_channel();

   // ✅ GOOD: Bounded channel with backpressure
   let (tx, rx) = mpsc::channel(1000);
   ```

5. **Don't forget Send + 'static bounds**
   ```rust
   // ❌ BAD: Won't compile (borrowed reference)
   let data = vec![1, 2, 3];
   tokio::spawn(async {
       println!("{:?}", data); // Error: borrowed value
   });

   // ✅ GOOD: Move ownership into task
   let data = vec![1, 2, 3];
   tokio::spawn(async move {
       println!("{:?}", data); // OK: owned
   });
   ```

---

## Quick Reference

### Common Patterns

| Pattern | Code |
|---------|------|
| Spawn task | `tokio::spawn(async move { ... })` |
| Race futures | `tokio::select! { ... }` |
| Join futures | `tokio::join!(fut1, fut2)` |
| Timeout | `tokio::time::timeout(duration, fut).await` |
| Sleep | `tokio::time::sleep(duration).await` |
| Spawn blocking | `tokio::task::spawn_blocking(\|\| { ... })` |
| Channel (unbounded) | `mpsc::unbounded_channel()` |
| Channel (bounded) | `mpsc::channel(100)` |
| Oneshot | `oneshot::channel()` |
| Notify | `Arc::new(Notify::new())` |

### Error Handling

```rust
// Internal operations: anyhow::Result
async fn service_operation() -> anyhow::Result<Data> {
    let result = database_query().await
        .context("Failed to query database")?;
    Ok(result)
}

// IPC boundary: Result<T, E> where E: Serialize
#[command]
async fn tauri_command() -> Result<Data, ApiError> {
    service_operation()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))
}
```

### Testing Async Code

```rust
#[tokio::test]
async fn test_async_operation() -> Result<(), AppError> {
    let result = async_operation().await?;
    assert_eq!(result, expected);
    Ok(())
}

#[tokio::test]
async fn test_with_timeout() -> Result<(), AppError> {
    match tokio::time::timeout(
        Duration::from_secs(5),
        long_operation()
    ).await {
        Ok(result) => assert!(result.is_ok()),
        Err(_) => panic!("Operation timed out"),
    }
    Ok(())
}
```

---

## Related Resources

- [Tauri Commands](./tauri-commands.md) - Command handler patterns with async
- [Error Handling](./error-handling.md) - Error patterns in async code
- [State Management](./state-management.md) - Arc/Mutex patterns for shared state
- [Testing Guide](./testing-guide.md) - Testing async functions

---

## Summary

Async patterns enable concurrent, non-blocking operations in Rust. Key takeaways:

1. **Use `async fn` and `.await`** for I/O-bound operations
2. **Spawn tasks with `tokio::spawn`** for background work
3. **Use channels for message passing** between tasks
4. **Apply `tokio::select!`** for cancellation and timeouts
5. **Share state with `Arc`** across task boundaries
6. **Always handle errors** in spawned tasks
7. **Use `anyhow::Result`** for internal operations
8. **Test async code with `#[tokio::test]`**

All examples in this guide are drawn from the Hestia codebase and represent production-ready patterns.
