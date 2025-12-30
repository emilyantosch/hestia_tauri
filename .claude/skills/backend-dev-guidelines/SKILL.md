---
name: backend-dev-guidelines
description: Comprehensive Rust backend development guide for Tauri applications. Use when creating Tauri commands, database operations, error handling, state management, or async patterns. Covers layered architecture (commands → services → operations → database), Result-based error handling, SeaORM patterns, tokio async/await, testing with cargo test, and type-driven design principles.
---

# Backend Development Guidelines

## Purpose

Establish consistency and best practices for Rust backend development in Tauri applications using type-driven design, robust error handling, and async patterns with tokio.

## When to Use This Skill

Automatically activates when working on:
- Creating or modifying Tauri commands (backend-to-frontend functions)
- Implementing service layer business logic
- Database operations with SeaORM (Entity, ActiveModel, transactions)
- Error handling with thiserror and anyhow
- State management (Arc, Mutex, RwLock, Tauri State)
- Async operations with tokio (spawn, join!, timeout)
- Backend testing with cargo test and tokio::test, unit tests in file, integration test in tests/
    - Prefer property-based testing
- Type-driven design (newtypes, enums, Result/Option)

---

## Quick Start

### New Backend Feature Checklist

- [ ] **Command Handler**: Define with `#[command]`, accept State injection
- [ ] **Service Layer**: Implement business logic with clear error handling
- [ ] **Operations**: SeaORM database operations (Entity, ActiveModel)
- [ ] **Error Types**: Define domain errors with thiserror
- [ ] **Tests**: Write tests first with `#[tokio::test]`
- [ ] **State**: Update AppState if needed for shared resources

### New Tauri Command Checklist

- [ ] Define function signature with proper types
- [ ] Add `#[command]` attribute
- [ ] Inject `State<'_, Mutex<AppState>>` if needed
- [ ] Return `Result<T, E>` with serializable error type
- [ ] Register in `tauri::generate_handler![]`
- [ ] Write comprehensive tests

---

## Architecture Overview

### Layered Architecture

```
Frontend (TypeScript/React)
    ↓
Tauri IPC (invoke)
    ↓
Command Handler (#[command])
    ↓
Controller Layer (hands off command to service layer)
    ↓
Service Layer (business logic)
    ↓
Repository (SeaORM)
    ↓
Database (SQLite via SeaORM)
```

**Key Principle**: Each layer has ONE responsibility.
- **Commands**: Handle Tauri IPC, validate input, delegate to services
- **Services**: Implement business logic, coordinate operations
- **Operations**: Execute database queries, manage transactions
- **Database**: Store and retrieve data

---

## Directory Structure

```
src-tauri/src/
├── commands/            # Tauri command handlers (#[command])
├── controllers/         # Controller handlers send IPC commands to service business logic
├── services/            # Business logic layer
├── database/            # SeaORM operations and manager
│   ├── repository.rs    # Database operation structs
│   └── manager.rs       # DatabaseManager, connection pool
├── entities/            # SeaORM entity definitions (generated)
├── errors/              # Error type definitions (thiserror)
│   ├── db.rs            # Database errors
│   ├── app.rs           # Application errors
│   └── file_system.rs   # File system errors
├── config/              # Configuration and state
│   ├── app.rs           # AppState struct
│   └── database.rs      # Database configuration
├── data/                # Data structures and types
├── tests/               # Integration and unit tests
├── lib.rs               # Library root
└── main.rs              # Application entry point
```

**Naming Conventions:**
- Files: `snake_case` - `tag_management.rs`, `watched_folder_management.rs`
- Structs: `PascalCase` - `AppState`, `DatabaseManager`, `FileOperations`
- Functions: `snake_case` - `create_tag`, `get_all_tags`, `scan_directory`
- Modules: `snake_case` - `mod commands;`, `mod database;`

---

## Core Principles (8 Key Rules)

### 1. Commands Only Delegate, Controllers delegate, Services Implement Logic

```rust
// ❌ NEVER: Business logic in command handlers
#[command]
pub async fn create_tag(state: State<'_, Mutex<AppState>>, name: String) -> Result<Tag, AppError> {
    // 200 lines of validation, database operations, etc.
}

// ✅ ALWAYS: Delegate to service layer
#[command]
pub async fn create_tag(state: State<'_, Mutex<AppState>>, name: String) -> Result<Tag, AppError> {
    tag_service::create_tag(&state, name).await
}
```

### 2. Use anyhow::Result<T> for All Fallible Operations, Result<T, E> for IPC endpoints

```rust
// ❌ NEVER: Panicking or unwrapping in production code
let user = database.get_user(id).unwrap();

// ✅ ALWAYS: Return Result and use ? operator
pub async fn get_user(id: i32) -> Result<User, DbError> {
    let user = Entity::find_by_id(id)
        .one(&*connection)
        .await?
        .ok_or(DbError::NotFound)?;
    Ok(user)
}
```

### 3. Leverage the Type System (Make Illegal States Unrepresentable)

```rust
// ❌ NEVER: Primitive obsession
pub fn create_email(email: String) -> Result<(), Error> { ... }

// ✅ ALWAYS: Use newtypes for domain concepts
pub struct Email(String);
impl Email {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        // Validation logic
        Ok(Email(email))
    }
}
```

### 4. All new types should live in app/data

```rust
// ❌ NEVER: Primitive obsession
pub fn create_email(email: String) -> Result<(), Error> { ... }

// ✅ ALWAYS: Use newtypes for domain concepts
pub struct Email(String);
impl Email {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        // Validation logic
        Ok(Email(email))
    }
}
```
- Data is split in Internal (for backend use) and commands (for IPC endpoints)
- For database representation, entites live in app/entity

### 5. Prefer Arc for Shared Ownership

```rust
// ❌ NEVER: Clone expensive data everywhere
let db_copy = database_manager.clone(); // Expensive!

// ✅ ALWAYS: Use Arc for cheap, thread-safe sharing
pub struct AppState {
    pub database: Arc<DatabaseManager>,
    pub cache: Arc<RwLock<HashMap<String, Value>>>,
}
```

### 6. Use thiserror for Domain Errors

```rust
// ✅ Define domain-specific error types
#[derive(Debug, Error, Serialize, Deserialize)]
pub enum TagError {
    #[error("Tag not found: {id}")]
    NotFound { id: i32 },

    #[error("Tag already exists: {name}")]
    AlreadyExists { name: String },

    #[error("Database error: {0}")]
    Database(#[from] DbError),
}
```

### 7. Instrument with Tracing

```rust
// ✅ Use tracing for structured logging
use tracing::{info, error, instrument};

#[instrument(skip(state))]
pub async fn scan_directory(state: &AppState, path: PathBuf) -> Result<Vec<File>, ScanError> {
    info!(path = ?path, "Starting directory scan");
    // Implementation
}
```

### 8. Write Tests First (TDD with std::test and tokio::test)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_tag() -> Result<()> {
        let db = setup_test_db().await?;
        let tag = create_tag(&db, "test".to_string()).await?;
        assert_eq!(tag.name, "test");
        Ok(())
    }
}
```

- Prefer `std::test` where async is not necessary

---

## Common Imports

```rust
// Tauri
use tauri::{command, State};

// Database (SeaORM)
use sea_orm::{
    entity::prelude::*,
    ActiveModelTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QuerySelect, Set,
};

// Error Handling
use thiserror::Error;
use anyhow::{Context, Result};

// Async Runtime
use tokio;

// Serialization (for Tauri IPC)
use serde::{Serialize, Deserialize};

// State Management
use std::sync::{Arc, Mutex, RwLock};

// Tracing
use tracing::{info, debug, warn, error, instrument};

// Standard Library
use std::path::PathBuf;
use std::collections::HashMap;
```

---

## Quick Reference

### Common Result Types

| Pattern | Use Case |
|---------|----------|
| `anyhow::Result<T>` | Fallible operations (preferred if applicable)|
| `Result<T, E>` | Fallible operations |
| `Option<T>` | Optional values |
| `Result<(), E>` | Operations with no return value |
| `anyhow::Result<T>` | Quick error handling in applications |


---

## Anti-Patterns to Avoid

❌ Business logic in command handlers (delegate to services)
❌ Using `.unwrap()` or `.expect()` in production code
❌ Missing error handling (always return `Result<T, E>`)
❌ Primitive obsession (use newtypes for domain concepts)
❌ Large lock scopes (keep Mutex/RwLock guards minimal)
❌ `println!` for logging (use `tracing` macros instead)
❌ Cloning `Arc` contents (clone the Arc, not the data)

---

## Navigation Guide

| Need to... | Read this |
|------------|-----------|
| Create Tauri commands | [resources/tauri-commands.md](resources/tauri-commands.md) |
| Handle errors properly | [resources/error-handling.md](resources/error-handling.md) |
| Work with SeaORM database | [resources/seaorm-database.md](resources/seaorm-database.md) |
| Use async/await patterns | [resources/async-patterns.md](resources/async-patterns.md) |
| Manage application state | [resources/state-management.md](resources/state-management.md) |
| Write tests | [resources/testing-guide.md](resources/testing-guide.md) |
| Add logging/tracing | [resources/tracing-logging.md](resources/tracing-logging.md) |
| Use type-driven design | [resources/type-driven-design.md](resources/type-driven-design.md) |
| Understand ownership | [resources/ownership-patterns.md](resources/ownership-patterns.md) |
| See complete examples | [resources/complete-examples.md](resources/complete-examples.md) |

---

## Resource Files

### [resources/tauri-commands.md](resources/tauri-commands.md)
Command handlers with `#[command]`, State injection, Result returns, registration

### [resources/error-handling.md](resources/error-handling.md)
thiserror error enums, anyhow Context, `?` operator, Tauri serialization

### [resources/seaorm-database.md](resources/seaorm-database.md)
Entity and ActiveModel patterns, CRUD operations, transactions, query builders

### [resources/async-patterns.md](resources/async-patterns.md)
tokio async/await, spawn, join!/select!, timeout, async traits

### [resources/state-management.md](resources/state-management.md)
Arc/Mutex/RwLock patterns, Tauri State, AppState structure, lock scopes

### [resources/testing-guide.md](resources/testing-guide.md)
Cargo test framework, `#[tokio::test]`, assertions, mocking, test organization

### [resources/tracing-logging.md](resources/tracing-logging.md)
Tracing macros, `#[instrument]`, structured logging, subscriber setup

### [resources/type-driven-design.md](resources/type-driven-design.md)
Newtype pattern, ADTs with enums, Option/Result transforms, From/Into traits

### [resources/ownership-patterns.md](resources/ownership-patterns.md)
Borrowing rules, lifetimes, Arc for shared ownership, Clone strategies

### [resources/complete-examples.md](resources/complete-examples.md)
End-to-end examples from Hestia, full CRUD flows, testing examples

---

## Related Skills

- **error-tracking** - Error tracking and monitoring patterns
- **skill-developer** - Meta-skill for creating and managing skills
- **frontend-dev-guidelines** - Frontend development patterns for Tauri apps

---

**Skill Status**: Phase 1.1 COMPLETE ✅
**Line Count**: < 350 ✅
**Progressive Disclosure**: 10 resource files (4 created, 6 remaining) 🚧
**Transformation**: TypeScript → Rust COMPLETE ✅
