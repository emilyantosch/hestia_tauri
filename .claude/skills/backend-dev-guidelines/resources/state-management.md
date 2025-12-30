# State Management in Tauri

## Overview

State management is critical for Tauri applications. Shared application state needs to be safely accessible across async tasks, Tauri commands, and background services. Rust provides powerful primitives for safe shared state through `Arc`, `Mutex`, and `RwLock`.

### Why State Management Matters

State in Tauri applications typically includes:
- Database connections shared across all operations
- Application configuration and settings
- Caches for performance optimization
- Handles to background services (file watchers, job processors)
- Runtime statistics and metrics

### Key Concepts

- **`Arc<T>`**: Atomic reference counting for thread-safe shared ownership
- **`Mutex<T>`**: Mutual exclusion for exclusive mutable access
- **`RwLock<T>`**: Read-write lock allowing multiple readers or one writer
- **`tauri::State`**: Tauri's managed state with `app.manage()`
- **Cloning `Arc`**: Creates new reference, not a deep clone

---

## Shared Ownership with Arc

### Arc Fundamentals

`Arc<T>` (Atomic Reference Counted) enables multiple owners of the same data across thread boundaries:

```rust
use std::sync::Arc;

// Create Arc-wrapped data
let database_manager = Arc::new(DatabaseManager::new().await?);

// Clone Arc to create new reference (cheap operation)
let db_clone = Arc::clone(&database_manager);

// Both variables point to the same data
// Data is dropped when last Arc reference is dropped
```

**Key Points**:
- `Arc::clone()` is cheap (increments reference count only)
- Thread-safe: Can be sent across threads (`Send + Sync`)
- Immutable by default (need `Mutex` or `RwLock` for mutation)
- Automatically deallocates when last reference drops

### Arc in Practice

```rust
use std::sync::Arc;
use sea_orm::DatabaseConnection;

#[derive(Debug)]
pub struct DatabaseManager {
    connection: Arc<DatabaseConnection>,
    settings: DatabaseSettings,
}

impl DatabaseManager {
    pub async fn new(settings: DatabaseSettings) -> Result<Self> {
        let connection = Self::create_connection(&settings).await?;

        Ok(Self {
            connection: Arc::new(connection),  // Wrap in Arc
            settings,
        })
    }

    /// Return cloned Arc (cheap operation)
    pub fn get_connection(&self) -> Arc<DatabaseConnection> {
        Arc::clone(&self.connection)
    }
}
```

**From**: `app/src-tauri/src/database/manager.rs:11-41`

**Pattern Explanation**:
1. Database connection wrapped in `Arc` during creation
2. `get_connection()` returns cloned Arc (not the connection itself)
3. Multiple components can share the same connection safely
4. Connection lives as long as any Arc reference exists

---

## Example 1: Arc for Shared Database Access

**Pattern**: Share database manager across multiple components

```rust
use std::sync::Arc;
use anyhow::Result;

pub struct AppState {
    pub library: Library,
    pub database_manager: Arc<DatabaseManager>,
    pub file_operations: FileOperations,
    pub directory_scanner: DirectoryScanner,
    pub file_watcher_handler: Option<FileWatcherHandler>,
    pub thumbnail_processor_handler: Option<ThumbnailProcessorHandler>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Create database manager once
        let database_manager = Arc::new(DatabaseManager::new_sqlite_default().await?);

        // Test database connection
        database_manager.test_connection().await?;

        // Share across multiple components
        let file_operations = FileOperations::new(Arc::clone(&database_manager));

        let file_operations_for_scanner = FileOperations::new(Arc::clone(&database_manager));
        let directory_scanner = DirectoryScanner::new(Arc::new(file_operations_for_scanner));

        let library = Library::last_or_new();

        Ok(Self {
            library,
            database_manager,  // Original Arc moved here
            file_operations,
            directory_scanner,
            file_watcher_handler: None,
            thumbnail_processor_handler: None,
        })
    }
}
```

**From**: `app/src-tauri/src/config/app.rs:19-55`

**Pattern Explanation**:
1. Single `DatabaseManager` created and wrapped in `Arc`
2. Arc cloned for each component that needs database access
3. All components share same underlying connection
4. Memory-efficient: only one `DatabaseManager` instance exists

**Use When**:
- Sharing expensive resources (database connections, caches)
- Multiple components need read access to same data
- Building service-oriented architectures

---

## Example 2: FileOperations with Arc and RwLock Cache

**Pattern**: Combine Arc for sharing with RwLock for cached data

```rust
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct FileOperations {
    database_manager: Arc<DatabaseManager>,
    file_type_cache: Arc<RwLock<HashMap<String, i32>>>,
    thumbnail_repository: ThumbnailOperations,
}

impl FileOperations {
    pub fn new(database_manager: Arc<DatabaseManager>) -> Self {
        let thumbnail_repository = ThumbnailOperations::new(Arc::clone(&database_manager));

        Self {
            database_manager,
            file_type_cache: Arc::new(RwLock::new(HashMap::new())),
            thumbnail_repository,
        }
    }

    /// Preload file type cache for better performance
    pub async fn preload_file_type_cache(&self) -> Result<()> {
        let connection = self.database_manager.get_connection();

        let file_types = FileTypes::find()
            .all(&*connection)
            .await?;

        // Acquire write lock to populate cache
        let mut cache = self.file_type_cache.write().await;

        for file_type in file_types {
            cache.insert(file_type.extension.clone(), file_type.id);
        }

        info!("Preloaded {} file types into cache", cache.len());
        Ok(())
    }

    /// Get file type ID from cache (fast path) or database (slow path)
    async fn get_or_create_file_type_id(&self, extension: &str) -> Result<i32> {
        // Try read lock first (allows concurrent reads)
        {
            let cache = self.file_type_cache.read().await;
            if let Some(&id) = cache.get(extension) {
                return Ok(id);
            }
        }

        // Cache miss, query database
        let connection = self.database_manager.get_connection();
        let file_type = self.find_or_create_file_type(extension, &connection).await?;

        // Update cache with write lock
        let mut cache = self.file_type_cache.write().await;
        cache.insert(extension.to_string(), file_type.id);

        Ok(file_type.id)
    }
}
```

**From**: `app/src-tauri/src/database/operations.rs:50-67`

**Pattern Explanation**:
1. `Arc<DatabaseManager>` shared across operations
2. `Arc<RwLock<HashMap>>` for cached lookup data
3. Read locks allow concurrent cache hits
4. Write lock for cache updates
5. Two-phase check: read lock → cache miss → write lock

**Use When**:
- Implementing caches for frequently accessed data
- Read-heavy workloads with occasional writes
- Performance-critical lookup operations

---

## Interior Mutability: Mutex vs RwLock

### Mutex<T> - Exclusive Access

`Mutex<T>` provides exclusive access to wrapped data:

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

let shared_state = Arc::new(Mutex::new(State::new()));

// Acquire lock for exclusive access
let mut state = shared_state.lock().await;
state.modify();
// Lock automatically released when `state` drops
```

**Characteristics**:
- Only one task can hold the lock at a time
- Blocks all other access (reads and writes)
- Simple, straightforward locking semantics
- Use `tokio::sync::Mutex` for async code

### RwLock<T> - Multiple Readers or One Writer

`RwLock<T>` allows concurrent reads but exclusive writes:

```rust
use tokio::sync::RwLock;
use std::sync::Arc;

let cache = Arc::new(RwLock::new(HashMap::new()));

// Multiple readers can hold read locks simultaneously
let data1 = cache.read().await;
let data2 = cache.read().await;  // OK: concurrent reads

// Only one writer allowed
let mut data = cache.write().await;
data.insert(key, value);
```

**Characteristics**:
- Multiple concurrent readers OR one writer
- Better performance for read-heavy workloads
- More complex than Mutex
- Use `tokio::sync::RwLock` for async code

### Choosing Between Mutex and RwLock

| Use Mutex When | Use RwLock When |
|----------------|-----------------|
| Simple exclusive access needed | Read-heavy workloads |
| Short critical sections | Expensive operations under read lock |
| Writes are as common as reads | Many concurrent readers expected |
| Simplicity preferred | Performance critical |

---

## Example 3: Tauri State Management

**Pattern**: Manage application state with Tauri's `State` injection

```rust
use std::sync::Mutex;
use tauri::{Manager, State};

// Application setup
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            info!("Initializing Tauri application with unified AppState");

            // Initialize app state asynchronously
            let rt = tokio::runtime::Runtime::new()
                .expect("Failed to create Tokio runtime");

            let app_state = rt.block_on(async {
                match AppState::new().await {
                    Ok(state) => state,
                    Err(e) => {
                        error!("Failed to initialize AppState: {:?}", e);
                        panic!("Cannot continue without proper app state initialization");
                    }
                }
            });

            info!("AppState initialized successfully");

            // Manage the state (accessible in all commands)
            app.manage(Mutex::new(app_state));

            info!("Unified AppState managed as application state");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            create_tag,
            get_all_tags,
            // ... more commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**From**: `app/src-tauri/src/lib.rs:44-80`

**Pattern Explanation**:
1. Initialize `AppState` asynchronously in setup
2. Wrap in `Mutex` for mutation across commands
3. `app.manage()` registers state globally
4. All Tauri commands can inject this state

### Accessing State in Commands

```rust
use tauri::{command, State};
use std::sync::Mutex;

#[command]
pub async fn create_tag(
    app_state: State<'_, Mutex<AppState>>,
    tag_name: String,
) -> Result<TagInfo, DbError> {
    // Lock the state to access it
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Perform database operation with connection
    let new_tag = tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(tag_name),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    let result = new_tag.insert(&*connection).await?;

    Ok(result.into())
}
```

**From**: `app/src-tauri/src/commands/tag_management.rs:47-80`

**Pattern Explanation**:
1. `State<'_, Mutex<AppState>>` injected by Tauri
2. Lock acquired to access state
3. Extract what's needed (database connection)
4. Release lock immediately (scope ends)
5. Perform work with extracted data

**Use When**:
- Building Tauri applications with shared state
- State needs mutation from multiple commands
- Simple locking semantics sufficient

---

## Example 4: Worker Pool with Shared State

**Pattern**: Multiple workers sharing job queue and statistics

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ThumbnailProcessor {
    message_receiver: mpsc::UnboundedReceiver<ThumbnailMessage>,
    job_queue: Arc<Mutex<Vec<ThumbnailJob>>>,
    repository: Arc<ThumbnailOperations>,
    generator: Arc<ThumbnailGenerator>,
    stats: Arc<Mutex<ProcessingStats>>,
    config: ProcessorConfig,
    shutdown_signal: Arc<tokio::sync::Notify>,
}

impl ThumbnailProcessor {
    pub async fn run(mut self) -> Result<()> {
        info!("Starting ThumbnailProcessor with {} workers", self.config.worker_count);

        // Spawn worker tasks, sharing queue and stats
        let mut worker_handles = Vec::new();
        for worker_id in 0..self.config.worker_count {
            let worker = ThumbnailWorker::new(
                worker_id,
                Arc::clone(&self.job_queue),       // Shared job queue
                Arc::clone(&self.repository),       // Shared repository
                Arc::clone(&self.generator),        // Shared generator
                Arc::clone(&self.stats),            // Shared stats
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
            match message {
                ThumbnailMessage::QueueFiles { file_infos, sizes } => {
                    self.queue_files_for_processing(file_infos, sizes).await?;
                }
                ThumbnailMessage::GetStats { respond_to } => {
                    let stats = self.get_current_stats().await;
                    let _ = respond_to.send(stats);
                }
                ThumbnailMessage::Shutdown => {
                    info!("Shutdown signal received, stopping processor");
                    break;
                }
            }
        }

        // Signal shutdown to all workers
        self.shutdown_signal.notify_waiters();

        // Wait for all workers to complete
        for handle in worker_handles {
            if let Err(e) = handle.await {
                error!("Worker task failed: {}", e);
            }
        }

        Ok(())
    }
}
```

**From**: `app/src-tauri/src/file_system/thumbnail_processor.rs:292-400`

**Pattern Explanation**:
1. Job queue wrapped in `Arc<Mutex<Vec<Job>>>`
2. Stats wrapped in `Arc<Mutex<ProcessingStats>>`
3. Arc cloned for each worker (N references)
4. Workers acquire lock to pop jobs
5. Coordinator acquires lock to push jobs

**Use When**:
- Implementing worker pools or thread pools
- Multiple tasks need to modify shared data structures
- Coordinating work across concurrent tasks

---

## Example 5: Reinitializing Components with New State

**Pattern**: Update shared state when configuration changes

```rust
use std::sync::Arc;
use anyhow::Result;

impl AppState {
    /// Switch to a new library and update all dependent components
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

    /// Update database connection to point to the new library's database
    async fn update_database_connection(&mut self, db_path: CanonPath) -> Result<()> {
        let connection_string = format!("sqlite:///{}", db_path.as_str()?);
        info!("Updating database connection to: {}", connection_string);

        // Create new database settings
        let sqlite_config = SqliteConfig {
            con_string: connection_string,
            create_if_missing: true,
            connection_timeout_ms: 30000,
            journal_mode: sea_orm::sqlx::sqlite::SqliteJournalMode::Wal,
            synchronous: sea_orm::sqlx::sqlite::SqliteSynchronous::Normal,
        };

        let settings = DatabaseSettings {
            db_type: DatabaseType::Sqlite,
            sqlite_config: Some(sqlite_config),
            postgres_config: None,
        };

        // Create new database manager
        self.database_manager = Arc::new(DatabaseManager::new(settings).await?);

        // Test the new connection
        self.database_manager.test_connection().await?;

        Ok(())
    }

    /// Reinitialize all components that depend on the database
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
}
```

**From**: `app/src-tauri/src/config/app.rs:88-159`

**Pattern Explanation**:
1. Library switch requires new database connection
2. Create new `Arc<DatabaseManager>` with new settings
3. Replace old Arc in state (old one dropped when no references remain)
4. Recreate all dependent components with new Arc
5. Components automatically use new database

**Use When**:
- Application configuration changes at runtime
- Switching between different data sources
- Hot-reloading or dynamic reconfiguration

---

## Example 6: Lock Scope Management

**Pattern**: Keep lock scopes as small as possible to avoid contention

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

// ❌ BAD: Lock held too long
pub async fn bad_pattern(state: Arc<Mutex<AppState>>) -> Result<()> {
    let mut state = state.lock().await;  // Lock acquired

    let data = state.get_data();
    expensive_async_operation(data).await;  // Still holding lock!

    state.update_result(result);
    // Lock released here
    Ok(())
}

// ✅ GOOD: Minimize lock scope
pub async fn good_pattern(state: Arc<Mutex<AppState>>) -> Result<()> {
    // Extract what you need
    let data = {
        let state = state.lock().await;
        state.get_data().clone()
    }; // Lock released immediately

    // Perform expensive work without holding lock
    let result = expensive_async_operation(data).await;

    // Acquire lock again briefly to update state
    {
        let mut state = state.lock().await;
        state.update_result(result);
    } // Lock released

    Ok(())
}
```

**Pattern Explanation**:
1. Lock scope delimited by `{ }` blocks
2. Extract necessary data, release lock
3. Perform expensive operations without lock
4. Re-acquire lock briefly to update state
5. Minimizes contention between tasks

**Use When**:
- Always! Keep lock scopes as small as possible
- Performing async operations that don't need lock
- Maximizing concurrency in multi-threaded code

---

## Lock Safety and Deadlock Prevention

### Common Deadlock Patterns

```rust
// ❌ DANGER: Potential deadlock
async fn deadlock_danger() {
    let state1 = state1.lock().await;
    let state2 = state2.lock().await;  // If another task locks in reverse order...
    // Deadlock!
}

// ✅ SAFE: Consistent lock ordering
async fn safe_locking() {
    // Always acquire locks in the same order
    let state1 = state1.lock().await;
    let state2 = state2.lock().await;
}

// ✅ SAFER: Single lock for related data
struct CombinedState {
    data1: Data1,
    data2: Data2,
}
// Single lock, no ordering issues
```

### Best Practices for Lock Safety

1. **Keep lock scopes minimal**
   ```rust
   // Extract data, release lock, then process
   let data = { state.lock().await.get_data().clone() };
   process(data).await;
   ```

2. **Don't hold locks across .await points**
   ```rust
   // ❌ BAD
   let mut state = state.lock().await;
   async_operation().await;  // Holding lock!

   // ✅ GOOD
   let data = { state.lock().await.get_data() };
   async_operation().await;
   ```

3. **Use consistent lock ordering**
   ```rust
   // Always lock in same order: db → cache → stats
   let db = state.db.lock().await;
   let cache = state.cache.lock().await;
   let stats = state.stats.lock().await;
   ```

4. **Prefer RwLock for read-heavy workloads**
   ```rust
   // Many concurrent readers
   let data = cache.read().await;

   // Single writer when needed
   let mut data = cache.write().await;
   ```

5. **Consider message passing over shared state**
   ```rust
   // Sometimes channels are simpler than locks
   sender.send(UpdateMessage { new_value }).await?;
   ```

---

## Best Practices

### DO ✅

1. **Use Arc for shared ownership across threads**
   ```rust
   let db_manager = Arc::new(DatabaseManager::new().await?);
   let clone1 = Arc::clone(&db_manager);
   let clone2 = Arc::clone(&db_manager);
   ```

2. **Prefer RwLock for read-heavy caches**
   ```rust
   let cache = Arc::new(RwLock::new(HashMap::new()));
   // Multiple concurrent readers
   let value1 = cache.read().await.get(&key);
   let value2 = cache.read().await.get(&key);
   ```

3. **Keep lock scopes minimal**
   ```rust
   let data = {
       let state = state.lock().await;
       state.data.clone()
   };
   process(data).await;
   ```

4. **Use tokio::sync primitives for async code**
   ```rust
   use tokio::sync::{Mutex, RwLock, Notify};
   // NOT std::sync::Mutex in async contexts
   ```

5. **Extract from Tauri State immediately**
   ```rust
   let connection = {
       let state = app_state.lock().unwrap();
       state.database_manager.get_connection()
   };
   // Use connection here
   ```

### DON'T ❌

1. **Don't hold locks across .await points**
   ```rust
   // ❌ BAD
   let mut state = state.lock().await;
   expensive_operation().await;  // Lock held too long

   // ✅ GOOD
   let data = { state.lock().await.clone() };
   expensive_operation().await;
   ```

2. **Don't use std::sync::Mutex in async code**
   ```rust
   // ❌ BAD: Blocks thread, not async-aware
   use std::sync::Mutex;

   // ✅ GOOD: Async-aware
   use tokio::sync::Mutex;
   ```

3. **Don't create circular Arc references**
   ```rust
   // ❌ BAD: Memory leak
   struct Node {
       next: Option<Arc<Mutex<Node>>>,
       prev: Option<Arc<Mutex<Node>>>,  // Circular!
   }

   // ✅ GOOD: Use Weak for back-references
   use std::sync::Weak;
   ```

4. **Don't acquire multiple locks without consistent ordering**
   ```rust
   // ❌ BAD: Potential deadlock
   let lock1 = state1.lock().await;
   let lock2 = state2.lock().await;  // Reverse order elsewhere = deadlock

   // ✅ GOOD: Always same order, or combine into single lock
   ```

5. **Don't clone data unnecessarily**
   ```rust
   // ❌ BAD: Expensive clone
   let huge_data = state.lock().await.huge_vec.clone();

   // ✅ GOOD: Use Arc if read-only access
   let huge_data = state.lock().await.huge_vec_arc.clone();  // Just clones Arc
   ```

---

## Quick Reference

### Arc Patterns

| Pattern | Code |
|---------|------|
| Create Arc | `Arc::new(data)` |
| Clone Arc | `Arc::clone(&arc)` |
| Get reference count | `Arc::strong_count(&arc)` |
| Downgrade to Weak | `Arc::downgrade(&arc)` |

### Lock Patterns

| Pattern | Code |
|---------|------|
| Acquire Mutex lock | `let data = mutex.lock().await;` |
| Acquire read lock | `let data = rwlock.read().await;` |
| Acquire write lock | `let mut data = rwlock.write().await;` |
| Scoped lock | `{ let data = mutex.lock().await; }` |

### Tauri State Patterns

| Pattern | Code |
|---------|------|
| Manage state | `app.manage(Mutex::new(state))` |
| Inject state | `state: State<'_, Mutex<AppState>>` |
| Access state | `let state = app_state.lock().unwrap();` |

---

## Related Resources

- [Async Patterns](./async-patterns.md) - Async task patterns with Arc
- [Error Handling](./error-handling.md) - Error handling in concurrent contexts
- [Tauri Commands](./tauri-commands.md) - State injection in commands
- [Testing Guide](./testing-guide.md) - Testing state-dependent code

---

## Summary

State management in Rust requires careful consideration of ownership, mutability, and thread safety. Key takeaways:

1. **Use `Arc` for shared ownership** across async tasks and threads
2. **Choose `Mutex` for exclusive access**, `RwLock` for read-heavy workloads
3. **Keep lock scopes minimal** to reduce contention
4. **Use `tokio::sync` primitives** in async code
5. **Extract data from locks immediately**, don't hold across `.await`
6. **Tauri State with `app.manage()`** provides command-level injection
7. **Consistent lock ordering** prevents deadlocks
8. **Clone `Arc` is cheap**, cloning data might not be

All examples in this guide are drawn from the Hestia codebase and represent production-ready patterns for building robust Tauri applications.
