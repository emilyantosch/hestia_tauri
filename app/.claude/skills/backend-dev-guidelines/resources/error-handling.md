# Error Handling in Rust Backend

This guide covers error handling patterns for the Hestia Tauri backend, with a focus on the distinction between **internal operations** (using `anyhow::Result`) and **IPC endpoints** (using `Result<T, E>`).

## Error Philosophy

Errors serve two distinct purposes:

1. **Control Flow** (For Machines)
   - `Result<T, E>` communicates success/failure to calling code
   - Guides what happens next in the program
   - Used for IPC boundaries where serialization matters

2. **Reporting** (For Humans)
   - `Debug` and `Display` implementations aid troubleshooting
   - Logged at higher levels with context
   - Frontend receives serialized error information

### The Critical Distinction: `anyhow::Result` vs `Result<T, E>`

This is a **fundamental pattern in Hestia**:

- **`anyhow::Result<T>`**: Use for all internal operations (services, repositories, helpers)
  - Preferred for its convenience in adding context
  - Allows flexible error propagation
  - Internal code doesn't need serialization

- **`Result<T, E>`**: Use ONLY for Tauri `#[command]` endpoints
  - `E` must implement `Serialize` + `Deserialize`
  - Frontend expects serialized error information
  - Marks the boundary between backend and frontend

```rust
// ✅ CORRECT: Internal service uses anyhow::Result
use anyhow::{Context, Result};

pub async fn internal_fetch_user(id: u32) -> Result<User> {
    let user = database.find_user(id)
        .await
        .context("Failed to find user")?;
    Ok(user)
}

// ✅ CORRECT: Tauri command uses Result<T, E>
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("User not found")]
    NotFound,
}

#[command]
pub async fn get_user(id: u32) -> Result<User, ApiError> {
    // Implementation...
}
```

## Defining Error Enums with `thiserror`

The `thiserror` crate provides a derive macro that automatically implements `Error`, `Debug`, and `Display`.

### Basic Enum with Unit Variants

Use unit variants for simple error conditions:

```rust
use thiserror::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum DbError {
    #[error("Database connection could not be established!")]
    ConnectionError,

    #[error("Database configuration is invalid!")]
    ConfigurationError,

    #[error("Database query failed!")]
    QueryError,

    #[error("Database insert could not be completed!")]
    InsertError,

    #[error("Database update could not be completed!")]
    UpdateError,
}
```

### Tuple Variants with Data

Carry additional context in error variants:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Permission denied for path: {0}")]
    PermissionDenied(std::path::PathBuf),

    #[error("Invalid file size: expected {expected}, got {actual}")]
    InvalidSize { expected: u64, actual: u64 },

    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),
}
```

### Struct Variants

For complex error states:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid field '{field}': {reason}")]
    InvalidField { field: String, reason: String },

    #[error("Missing required field: {0}")]
    MissingField(String),
}
```

## Error Conversions with `#[from]`

The `#[from]` attribute automatically implements `From<T>` for automatic error conversion:

```rust
use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error")]
    Database(#[from] DbError),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

// Usage: automatic conversion with ?
fn process_file() -> Result<String, AppError> {
    let data = std::fs::read_to_string("data.json")?;  // io::Error -> AppError
    let parsed: MyStruct = serde_json::from_str(&data)?;  // serde_json::Error -> AppError
    Ok(String::new())
}
```

## Adding Error Context with `anyhow`

Use `anyhow::Context` to wrap errors with additional information:

```rust
use anyhow::{Context, Result};
use sqlx::PgPool;

pub async fn fetch_user(pool: &PgPool, id: u32) -> Result<User> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .context("Failed to fetch user from database")?;

    Ok(user)
}

pub async fn complex_operation() -> Result<String> {
    let pool = create_pool()
        .context("Failed to create database connection pool")?;

    let user = fetch_user(&pool, 42)
        .context("Failed during user lookup phase")?;

    Ok(user.name)
}
```

Error chains automatically propagate:
```
Error: Failed during user lookup phase

Caused by:
    0: Failed to fetch user from database
    1: relation "users" does not exist
```

## Example 1: Simple DbError Enum (From Hestia)

This enum handles all database-related errors and is used in Tauri commands:

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum DbError {
    #[error("Database connection could not be established!")]
    ConnectionError,

    #[error("Database configuration is invalid!")]
    ConfigurationError,

    #[error("Database transaction issue occurred!")]
    TransactionError,

    #[error("Database query failed!")]
    QueryError,

    #[error("Database insert could not be completed!")]
    InsertError,

    #[error("Database update could not be completed!")]
    UpdateError,

    #[error("Database delete could not be completed!")]
    DeleteError,

    #[error("Database integrity constraint has been violated!")]
    IntegrityConstraintError,

    #[error("Database foreign key constraint has been violated!")]
    ReferentialConstraintError,
}

// Usage in a Tauri command
use tauri::command;
use sea_orm::EntityTrait;

#[command]
pub async fn create_tag(tag_name: String) -> Result<TagInfo, DbError> {
    // Query with error mapping
    match Tags::find()
        .filter(tags::Column::Name.eq(&tag_name))
        .one(&connection)
        .await
    {
        Ok(Some(existing)) => Ok(existing.into()),
        Ok(None) => {
            // Insert new tag
            let new_tag = tags::ActiveModel {
                id: sea_orm::ActiveValue::NotSet,
                name: sea_orm::Set(tag_name),
                // ... other fields
            };

            match new_tag.insert(&connection).await {
                Ok(tag) => Ok(tag.into()),
                Err(_) => Err(DbError::InsertError),
            }
        }
        Err(_) => Err(DbError::QueryError),
    }
}
```

## Example 2: AppError with Custom Serialization (From Hestia)

This example shows how to implement custom `Serialize` for complex error handling:

```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("The file watcher could not be found!")]
    WatcherNotFound,

    #[error("An internal error has occurred: {0}")]
    Internal(#[from] anyhow::Error),
}

// Custom serialization for frontend consumption
#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum AppErrorKind {
    WatcherNotFound(String),
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_message = self.to_string();
        let error_kind = match self {
            Self::WatcherNotFound => AppErrorKind::WatcherNotFound(error_message),
            Self::Internal(_) => AppErrorKind::Internal(error_message),
        };
        error_kind.serialize(serializer)
    }
}

// Usage in command
#[tauri::command]
pub async fn start_watcher() -> Result<(), AppError> {
    initialize_watcher()
        .context("Failed to initialize watcher")
        .map_err(|e| AppError::Internal(e))?;
    Ok(())
}
```

## Example 3: Error Conversion Chain

Shows how to layer multiple error types:

```rust
use thiserror::Error;
use anyhow::Context;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid tag name: {0}")]
    InvalidTagName(String),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("Database error: {0}")]
    Database(#[from] DbError),
}

// Service layer uses anyhow::Result
pub async fn create_tag_service(name: String) -> anyhow::Result<Tag> {
    // Validate
    if name.is_empty() {
        return Err(ValidationError::InvalidTagName("empty".to_string()))
            .context("Tag validation failed")?;
    }

    // Database operation
    let tag = sqlx::query_as("INSERT INTO tags ...")
        .fetch_one(&pool)
        .await
        .context("Failed to insert tag")?;

    Ok(tag)
}

// Command layer converts to serializable error
#[tauri::command]
pub async fn create_tag(name: String) -> Result<Tag, DbError> {
    create_tag_service(name)
        .await
        .map_err(|e| {
            // Log full context
            eprintln!("Service error: {:?}", e);
            // Return serializable error to frontend
            DbError::InsertError
        })
}
```

## Example 4: Wrapping External Errors

Handle errors from external crates:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path is not valid UTF-8: {0:?}")]
    InvalidPath(std::path::PathBuf),

    #[error("Failed to watch directory: {0}")]
    WatcherError(String),
}

pub async fn watch_directory(path: &std::path::Path) -> anyhow::Result<()> {
    // Automatic conversion from std::io::Error
    let metadata = std::fs::metadata(path)?;

    if !metadata.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory"));
    }

    Ok(())
}

// Or with custom error wrapping
pub fn validate_path(path: &str) -> Result<std::path::PathBuf, FileSystemError> {
    let path_buf = std::path::PathBuf::from(path);

    if !path_buf.as_os_str().is_ascii() {
        return Err(FileSystemError::InvalidPath(path_buf));
    }

    Ok(path_buf)
}
```

## Example 5: Error Handling in Database Operations

Pattern for SeaORM error conversion:

```rust
use sea_orm::{DbErr, ActiveModelTrait, EntityTrait};
use thiserror::Error;
use anyhow::Context;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("Record not found")]
    NotFound,

    #[error("Database error")]
    Database,

    #[error("Constraint violation")]
    ConstraintViolation,
}

impl From<DbErr> for RepositoryError {
    fn from(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => RepositoryError::NotFound,
            DbErr::Custom(msg) if msg.contains("UNIQUE constraint failed") => {
                RepositoryError::ConstraintViolation
            }
            _ => RepositoryError::Database,
        }
    }
}

// Service layer with anyhow::Result
pub async fn insert_tag(
    db: &sea_orm::DatabaseConnection,
    name: String,
) -> anyhow::Result<tags::Model> {
    let new_tag = tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: sea_orm::Set(name),
        created_at: sea_orm::Set(chrono::Utc::now().naive_utc()),
        updated_at: sea_orm::Set(chrono::Utc::now().naive_utc()),
    };

    new_tag
        .insert(db)
        .await
        .context("Failed to insert tag into database")?;

    Ok(new_tag)
}

// Repository pattern with Result<T, E>
pub async fn repository_insert_tag(
    db: &sea_orm::DatabaseConnection,
    name: String,
) -> Result<tags::Model, RepositoryError> {
    let new_tag = tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: sea_orm::Set(name),
        created_at: sea_orm::Set(chrono::Utc::now().naive_utc()),
        updated_at: sea_orm::Set(chrono::Utc::now().naive_utc()),
    };

    new_tag.insert(db).await.map_err(Into::into)
}
```

## Example 6: Mixed Error Types in Command Flow

End-to-end error handling from repository to command:

```rust
use tauri::command;
use std::sync::Mutex;
use anyhow::Context;

// 1. Repository layer - Result<T, RepositoryError>
pub async fn repository_find_tag(
    db: &sea_orm::DatabaseConnection,
    id: i32,
) -> Result<tags::Model, RepositoryError> {
    Tags::find_by_id(id)
        .one(db)
        .await
        .map_err(RepositoryError::from)?
        .ok_or(RepositoryError::NotFound)
}

// 2. Service layer - anyhow::Result<T>
pub async fn service_get_tag(
    db: &sea_orm::DatabaseConnection,
    id: i32,
) -> anyhow::Result<Tag> {
    let model = repository_find_tag(db, id)
        .await
        .context("Repository failed to find tag")?;

    Ok(Tag::from(model))
}

// 3. Command layer - Result<T, CommandError>
#[derive(Debug, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum CommandError {
    #[error("Tag not found")]
    NotFound,
}

#[command]
pub async fn get_tag(
    state: tauri::State<'_, Mutex<AppState>>,
    tag_id: i32,
) -> Result<Tag, CommandError> {
    let db = {
        let app_state = state.lock().unwrap();
        app_state.database_manager.get_connection()
    };

    service_get_tag(&db, tag_id)
        .await
        .map_err(|e| {
            eprintln!("Failed to get tag: {:?}", e);
            CommandError::NotFound
        })
}
```

## Error Propagation with `?`

The `?` operator works with any type that implements `From<ErrorType>`:

```rust
use anyhow::Result;

pub async fn multi_step_operation() -> Result<String> {
    // All these errors are automatically converted to anyhow::Error
    let data = load_file("data.json")?;  // std::io::Error -> anyhow::Error
    let parsed: MyStruct = serde_json::from_str(&data)?;  // serde_json::Error -> anyhow::Error
    let processed = process_data(parsed)?;  // ProcessError -> anyhow::Error (automatic)

    Ok(format!("Processed: {}", processed))
}

// ProcessError automatically converts to anyhow::Error
#[derive(Debug, thiserror::Error)]
pub enum ProcessError {
    #[error("Invalid data")]
    InvalidData,
}

// anyhow::Error automatically implements From for all Error types
// so no manual impl is needed!

fn process_data(data: MyStruct) -> Result<String, ProcessError> {
    if data.is_invalid() {
        return Err(ProcessError::InvalidData);
    }
    Ok(String::new())
}
```

## Serialization for Tauri/Frontend

When an error must travel to the frontend, it needs to serialize properly:

```rust
use serde::{Serialize, Deserialize};
use thiserror::Error;

// ✅ Simple approach: implement Serialize
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SimpleError {
    #[error("Not found")]
    NotFound,
}

// ✅ Complex approach: custom serialization
#[derive(Debug, Error)]
pub enum ComplexError {
    #[error("Validation failed: {0}")]
    ValidationFailed(Vec<String>),
}

impl Serialize for ComplexError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::ValidationFailed(errors) => {
                let message = format!("Validation failed: {}", errors.join(", "));
                serializer.serialize_str(&message)
            }
        }
    }
}

// ✅ Tagged approach: for better frontend handling
#[derive(Serialize)]
#[serde(tag = "type", content = "data")]
enum FrontendError {
    #[serde(rename = "not_found")]
    NotFound { id: i32 },

    #[serde(rename = "validation_error")]
    ValidationError { fields: Vec<String> },
}

impl Serialize for ComplexError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let frontend_error = match self {
            Self::ValidationFailed(errors) => FrontendError::ValidationError {
                fields: errors.clone(),
            },
        };
        frontend_error.serialize(serializer)
    }
}
```

## Best Practices Summary

### ✅ DO

1. **Use `anyhow::Result<T>` for internal operations**
   ```rust
   pub async fn service_operation() -> anyhow::Result<Data> {
       // ...
   }
   ```

2. **Use `Result<T, E>` ONLY for Tauri commands**
   ```rust
   #[command]
   pub async fn handle_request() -> Result<Response, ApiError> {
       // ...
   }
   ```

3. **Add context to errors as they propagate**
   ```rust
   operation().context("Additional context")?;
   ```

4. **Implement `#[from]` for automatic conversions**
   ```rust
   #[derive(Error)]
   pub enum MyError {
       #[error("IO: {0}")]
       Io(#[from] std::io::Error),
   }
   ```

5. **Serialize frontend errors carefully**
   ```rust
   #[derive(Serialize, Deserialize)]
   pub enum ApiError {
       #[error("Not found")]
       NotFound,
   }
   ```

### ❌ DON'T

1. **Don't use `Result<T, E>` for internal operations**
   ```rust
   // ❌ WRONG
   fn internal_helper() -> Result<Data, MyError> { }
   // ✅ RIGHT
   fn internal_helper() -> anyhow::Result<Data> { }
   ```

2. **Don't forget `Serialize` on command error types**
   ```rust
   // ❌ WRONG - won't serialize
   #[derive(Error)]
   pub enum CommandError {
       #[error("Failed")]
       Failed,
   }

   // ✅ RIGHT
   #[derive(Error, Serialize, Deserialize)]
   pub enum CommandError {
       #[error("Failed")]
       Failed,
   }
   ```

3. **Don't swallow errors silently**
   ```rust
   // ❌ WRONG
   match operation() {
       Ok(v) => process(v),
       Err(_) => return Err(GenericError),
   }

   // ✅ RIGHT
   operation()
       .context("Operation failed")?;
   ```

4. **Don't create error types outside `app/data`**
   ```rust
   // ❌ WRONG - in src-tauri/src/errors/custom.rs
   pub enum CustomError { }

   // ✅ RIGHT - in src-tauri/src/data/internal/ or src-tauri/src/data/commands/
   pub enum CustomError { }
   ```

5. **Don't ignore error chains**
   ```rust
   // ❌ WRONG
   let result = operation().unwrap_or_default();

   // ✅ RIGHT
   let result = operation()
       .context("Failed to complete operation")?;
   ```

## Related Resources

- [SKILL.md](../SKILL.md) - Main backend guidelines
- [tauri-commands.md](./tauri-commands.md) - Tauri command patterns
- [seaorm-database.md](./seaorm-database.md) - Database operations
- [testing-guide.md](./testing-guide.md) - Testing error scenarios

## Quick Reference

| Scenario | Type | Example |
|----------|------|---------|
| Internal service | `anyhow::Result<T>` | `pub async fn fetch_user() -> Result<User>` |
| Tauri command | `Result<T, E>` | `#[command] pub async fn get_user() -> Result<User, ApiError>` |
| Converting error | `#[from]` | `#[error("...")]Custom(#[from] StdError)` |
| Adding context | `.context()` | `operation().context("Failed to...")?` |
| Serializing | `Serialize` | `#[derive(Serialize)] pub enum ApiError` |
| Database error | `RepositoryError` | `impl From<DbErr> for RepositoryError` |

