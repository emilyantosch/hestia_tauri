# Tauri Command Handlers

**Purpose**: Define backend functions that the frontend can invoke via Tauri's IPC mechanism.

**When to Use**: Creating any functionality that the frontend needs to access from the Rust backend.

---

## Overview

Tauri commands are Rust functions marked with the `#[tauri::command]` attribute that can be invoked from the frontend TypeScript code. They serve as the entry point to your backend logic, handling IPC communication and delegating to the service/controller layer.

### Key Characteristics

- Marked with `#[tauri::command]` attribute
- Can inject application state via `State<'_, Mutex<AppState>>`
- Return `Result<T, E>` where both `T` and `E` implement `Serialize`
- Can be synchronous or asynchronous (`async fn`)
- Must be registered in `tauri::generate_handler![]`

### Architectural Role

```
Frontend (invoke)
    ↓
Tauri IPC Layer
    ↓
Command Handler (#[tauri::command]) ← YOU ARE HERE
    ↓
Controller (validates, delegates)
    ↓
Service (business logic)
    ↓
Repository (database)
```

**Command responsibilities**:
- ✅ Accept parameters from frontend
- ✅ Inject application state
- ✅ Delegate to controller/service
- ✅ Return serializable results
- ❌ NO business logic
- ❌ NO direct database access

---

## Basic Command Pattern

### Minimal Command

```rust
#[tauri::command]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}
```

**Frontend invocation**:
```typescript
import { invoke } from '@tauri-apps/api/core';

const greeting = await invoke<string>('greet', { name: 'Alice' });
```

### Command with Result

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
}

#[tauri::command]
pub fn validate_email(email: String) -> Result<bool, ValidationError> {
    if email.contains('@') {
        Ok(true)
    } else {
        Err(ValidationError {
            message: "Invalid email format".to_string(),
        })
    }
}
```

**Frontend invocation**:
```typescript
try {
    const isValid = await invoke<boolean>('validate_email', {
        email: 'user@example.com'
    });
} catch (error) {
    console.error('Validation failed:', error);
}
```

---

## State Injection Pattern

Application state is injected into commands using `State<'_, Mutex<AppState>>`. This provides access to shared resources like database connections, caches, and configuration.

### State Structure (from Hestia)

```rust
use std::sync::{Arc, Mutex};
use crate::database::DatabaseManager;
use crate::file_system::{DirectoryScanner, FileWatcherHandler};

#[derive(Debug)]
pub struct AppState {
    pub library: Library,
    pub database_manager: Arc<DatabaseManager>,
    pub file_operations: FileOperations,
    pub directory_scanner: DirectoryScanner,
    pub file_watcher_handler: Option<FileWatcherHandler>,
    pub thumbnail_processor_handler: Option<ThumbnailProcessorHandler>,
}
```

### Injecting State

```rust
use tauri::State;
use std::sync::Mutex;
use crate::config::app::AppState;

#[tauri::command]
pub async fn get_user_count(
    app_state: State<'_, Mutex<AppState>>
) -> Result<usize, String> {
    // Lock the state (keep scope minimal!)
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Use connection after lock is released
    let count = UserEntity::find()
        .count(&*connection)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(count)
}
```

### ⚠️ Lock Scope Best Practices

```rust
// ❌ NEVER: Hold lock across await points
#[tauri::command]
pub async fn bad_example(app_state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = app_state.lock().unwrap();
    // Lock held here...
    let result = some_async_operation(&state.database_manager).await; // DEADLOCK RISK!
    // ...and here
    Ok(())
}

// ✅ ALWAYS: Release lock before async operations
#[tauri::command]
pub async fn good_example(app_state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    }; // Lock released here

    let result = some_async_operation(&connection).await; // Safe!
    Ok(())
}
```

---

## Result Return Types and Error Handling

Commands that can fail must return `Result<T, E>` where `E` implements `Serialize` for frontend communication.

### Using anyhow::Result (Preferred for Internal Operations)

```rust
use anyhow::{Context, Result};

pub async fn internal_operation() -> Result<Data> {
    let data = fetch_data()
        .await
        .context("Failed to fetch data")?;

    process_data(&data)
        .context("Failed to process data")?;

    Ok(data)
}
```

### Using Result<T, E> for IPC Endpoints

```rust
use serde::{Serialize, Deserialize};
use thiserror::Error;

// Please note that this is is not where you should put Errors and AppError is not a valid (make them more specific!)
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

#[tauri::command]
pub async fn create_user(
    app_state: State<'_, Mutex<AppState>>,
    name: String,
    email: String,
) -> Result<UserInfo, AppError> {
    if email.is_empty() {
        return Err(AppError::Validation("Email is required".to_string()));
    }

    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Delegate to service
    user_service::create_user(&connection, name, email)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}
```

### Mixed Error Handling (from Hestia)

```rust
#[command]
pub async fn update_tag(
    app_state: State<'_, Mutex<AppState>>,
    tag_id: i32,
    new_name: String,
) -> Result<TagInfo, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Check if tag exists
    let existing_tag = match Tags::find_by_id(tag_id).one(&*connection).await {
        Ok(Some(tag)) => tag,
        Ok(None) => return Err("Tag not found".to_string()),
        Err(e) => return Err(format!("Database error: {}", e)),
    };

    // Update tag
    let mut active_model = existing_tag.into_active_model();
    active_model.name = Set(new_name);
    active_model.updated_at = Set(chrono::Utc::now().naive_utc());

    match active_model.update(&*connection).await {
        Ok(updated_tag) => Ok(updated_tag.into()),
        Err(e) => Err(format!("Failed to update tag: {}", e)),
    }
}
```

---

## Async Command Patterns

Most database and file system operations require async commands.

### Basic Async Command

```rust
use tauri::{command, State};
use std::sync::Mutex;

#[command]
pub async fn fetch_data(
    app_state: State<'_, Mutex<AppState>>
) -> Result<Vec<Item>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    Entity::find()
        .all(&*connection)
        .await
        .map(|items| items.into_iter().map(|i| i.into()).collect())
        .map_err(|e| format!("Query failed: {}", e))
}
```

### Async Command with Transaction

```rust
#[command]
pub async fn delete_tag_cascade(
    app_state: State<'_, Mutex<AppState>>,
    tag_id: i32,
) -> Result<bool, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Start transaction
    let transaction = connection.begin().await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // Delete relationships first
    file_has_tags::Entity::delete_many()
        .filter(file_has_tags::Column::TagId.eq(tag_id))
        .exec(&transaction)
        .await
        .map_err(|e| format!("Failed to delete relationships: {}", e))?;

    // Delete the tag
    let delete_result = Tags::delete_by_id(tag_id)
        .exec(&transaction)
        .await
        .map_err(|e| format!("Failed to delete tag: {}", e))?;

    // Commit transaction
    transaction.commit().await
        .map_err(|e| format!("Failed to commit: {}", e))?;

    Ok(delete_result.rows_affected > 0)
}
```

### Background Task Spawning

```rust
use tokio;

#[command]
pub async fn start_scan(
    app_state: State<'_, Mutex<AppState>>,
    path: String,
) -> Result<(), String> {
    // Get scanner from state
    let scanner = {
        let state = app_state.lock().unwrap();
        Arc::clone(&state.directory_scanner)
    };

    // Spawn background task
    tokio::spawn(async move {
        if let Err(e) = scanner.scan_directory(&path).await {
            tracing::error!("Scan failed: {:?}", e);
        }
    });

    // Return immediately
    Ok(())
}
```

---

## Command Registration

Commands must be registered with Tauri's builder to be accessible from the frontend.

### Registration in main.rs

```rust
use tauri::Builder;

fn main() {
    Builder::default()
        .manage(std::sync::Mutex::new(AppState::new().await?))
        .invoke_handler(tauri::generate_handler![
            // Tag commands
            create_tag,
            get_all_tags,
            get_tag_by_id,
            update_tag,
            delete_tag,
            search_tags_by_name,
            // File commands
            add_tag_to_file,
            remove_tag_from_file,
            get_tags_for_file,
            get_files_for_tag,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Organizing Command Modules

```rust
// src/commands/mod.rs
pub mod tag_management;
pub mod file_operations;
pub mod watched_folder_management;

// Re-export all commands for easy registration
pub use tag_management::*;
pub use file_operations::*;
pub use watched_folder_management::*;
```

```rust
// main.rs
mod commands;

use commands::*;

Builder::default()
    .invoke_handler(tauri::generate_handler![
        // From tag_management
        create_tag,
        get_all_tags,
        // From file_operations
        scan_file,
        get_file_info,
        // From watched_folder_management
        add_watched_folder,
        remove_watched_folder,
    ])
    // ...
```

---

## Complete Examples from Hestia

### Example 1: Simple CRUD Command

**Create Tag** (from `commands/tag_management.rs:47-86`):

```rust
use tauri::{command, State};
use std::sync::Mutex;
use serde::{Serialize, Deserialize};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use crate::config::app::AppState;
use crate::errors::DbError;
use entity::{prelude::*, tags};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub id: i32,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<tags::Model> for TagInfo {
    fn from(tag: tags::Model) -> Self {
        Self {
            id: tag.id,
            name: tag.name,
            created_at: tag.created_at.to_string(),
            updated_at: tag.updated_at.to_string(),
        }
    }
}

#[command]
pub async fn create_tag(
    app_state: State<'_, Mutex<AppState>>,
    tag_name: String,
) -> Result<TagInfo, DbError> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Check if tag already exists
    match Tags::find()
        .filter(tags::Column::Name.eq(&tag_name))
        .one(&*connection)
        .await
    {
        Ok(Some(existing_tag)) => {
            return Ok(existing_tag.into());
        }
        Ok(None) => { /* Continue */ }
        Err(_) => return Err(DbError::QueryError),
    }

    // Create new tag
    let new_tag = tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(tag_name),
        created_at: Set(chrono::Utc::now().naive_utc()),
        updated_at: Set(chrono::Utc::now().naive_utc()),
    };

    match new_tag.insert(&*connection).await {
        Ok(tag) => Ok(tag.into()),
        Err(_) => Err(DbError::InsertError),
    }
}
```

### Example 2: Query with Filter

**Get All Tags** (from `commands/tag_management.rs:89-100`):

```rust
#[command]
pub async fn get_all_tags(
    app_state: State<'_, Mutex<AppState>>
) -> Result<Vec<TagInfo>, DbError> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    match Tags::find().all(&*connection).await {
        Ok(tags) => Ok(tags.into_iter().map(|t| t.into()).collect()),
        Err(_) => Err(DbError::QueryError),
    }
}
```

### Example 3: Search with Pattern Matching

**Search Tags** (from `commands/tag_management.rs:421-441`):

```rust
#[command]
pub async fn search_tags_by_name(
    app_state: State<'_, Mutex<AppState>>,
    search_pattern: String,
) -> Result<Vec<TagInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    let pattern = format!("%{}%", search_pattern);

    match Tags::find()
        .filter(tags::Column::Name.like(&pattern))
        .all(&*connection)
        .await
    {
        Ok(tags) => Ok(tags.into_iter().map(|t| t.into()).collect()),
        Err(e) => Err(format!("Failed to search tags: {}", e)),
    }
}
```

### Example 4: Relationship Management

**Add Tag to File** (from `commands/tag_management.rs:237-295`):

```rust
#[command]
pub async fn add_tag_to_file(
    app_state: State<'_, Mutex<AppState>>,
    file_id: i32,
    tag_id: i32,
) -> Result<FileTagInfo, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Check if file exists
    if Files::find_by_id(file_id)
        .one(&*connection)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return Err("File not found".to_string());
    }

    // Check if tag exists
    if Tags::find_by_id(tag_id)
        .one(&*connection)
        .await
        .unwrap_or(None)
        .is_none()
    {
        return Err("Tag not found".to_string());
    }

    // Check if relationship already exists
    if let Ok(Some(_)) = file_has_tags::Entity::find()
        .filter(file_has_tags::Column::FileId.eq(file_id))
        .filter(file_has_tags::Column::TagId.eq(tag_id))
        .one(&*connection)
        .await
    {
        return Err("Tag already added to file".to_string());
    }

    // Create new relationship
    let new_relationship = file_has_tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        file_id: Set(file_id),
        tag_id: Set(tag_id),
    };

    match new_relationship.insert(&*connection).await {
        Ok(relationship) => Ok(FileTagInfo {
            id: relationship.id,
            file_id: relationship.file_id,
            tag_id: relationship.tag_id,
            file_name: None,
            tag_name: None,
        }),
        Err(e) => Err(format!("Failed to add tag to file: {}", e)),
    }
}
```

### Example 5: Transaction with Cascade Delete

**Delete Tag with Cascade** (from `commands/tag_management.rs:183-234`):

```rust
#[command]
pub async fn delete_tag(
    app_state: State<'_, Mutex<AppState>>,
    tag_id: i32,
) -> Result<bool, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Start transaction
    let transaction = match connection.begin().await {
        Ok(txn) => txn,
        Err(e) => return Err(format!("Failed to start transaction: {}", e)),
    };

    // Delete all file-tag relationships first
    match file_has_tags::Entity::delete_many()
        .filter(file_has_tags::Column::TagId.eq(tag_id))
        .exec(&transaction)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            if let Err(rollback_err) = transaction.rollback().await {
                return Err(format!(
                    "Delete failed and rollback failed: {} (rollback: {})",
                    e, rollback_err
                ));
            }
            return Err(format!("Failed to delete relationships: {}", e));
        }
    }

    // Delete the tag
    let delete_result = match Tags::delete_by_id(tag_id).exec(&transaction).await {
        Ok(result) => result,
        Err(e) => {
            if let Err(rollback_err) = transaction.rollback().await {
                return Err(format!(
                    "Delete failed and rollback failed: {} (rollback: {})",
                    e, rollback_err
                ));
            }
            return Err(format!("Failed to delete tag: {}", e));
        }
    };

    // Commit transaction
    if let Err(e) = transaction.commit().await {
        return Err(format!("Failed to commit transaction: {}", e));
    }

    Ok(delete_result.rows_affected > 0)
}
```

---

## Best Practices

### ✅ DO

1. **Keep commands minimal** - Delegate to controllers/services
2. **Release locks quickly** - Extract data before async operations
3. **Use Result<T, E>** - Always handle errors explicitly
4. **Implement Serialize** - All return types and errors must serialize
5. **Document command parameters** - Use doc comments
6. **Prefer anyhow::Result internally** - Use Result<T, E> for IPC boundaries
7. **Test commands** - Write integration tests for each command

### ❌ DON'T

1. **Hold locks across await points** - Risk of deadlocks
2. **Put business logic in commands** - Keep them thin
3. **Use unwrap/expect** - Return Result instead
4. **Ignore errors** - Always propagate or handle
5. **Forget to register commands** - Add to generate_handler![]
6. **Use non-serializable errors** - Implement Serialize/Deserialize
7. **Clone Arc contents** - Clone the Arc pointer, not the data

---

## Testing Commands

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    async fn setup_test_state() -> Mutex<AppState> {
        let state = AppState::new().await.unwrap();
        Mutex::new(state)
    }

    #[tokio::test]
    async fn test_create_tag() {
        let app_state = setup_test_state().await;
        let state = tauri::State::from(&app_state);

        let result = create_tag(state, "test-tag".to_string()).await;

        assert!(result.is_ok());
        let tag = result.unwrap();
        assert_eq!(tag.name, "test-tag");
    }

    #[tokio::test]
    async fn test_get_all_tags() {
        let app_state = setup_test_state().await;
        let state = tauri::State::from(&app_state);

        // Create some test tags
        create_tag(state.clone(), "tag1".to_string()).await.unwrap();
        create_tag(state.clone(), "tag2".to_string()).await.unwrap();

        let result = get_all_tags(state).await;

        assert!(result.is_ok());
        let tags = result.unwrap();
        assert!(tags.len() >= 2);
    }
}
```

---

## Related Resources

- [error-handling.md](error-handling.md) - Defining error types for commands
- [state-management.md](state-management.md) - AppState design patterns
- [seaorm-database.md](seaorm-database.md) - Database operations in commands
- [async-patterns.md](async-patterns.md) - Async/await patterns
- [testing-guide.md](testing-guide.md) - Testing command handlers

---

**Key Takeaway**: Commands are the IPC boundary between frontend and backend. Keep them simple, delegate to controllers/services, and always handle errors gracefully.
