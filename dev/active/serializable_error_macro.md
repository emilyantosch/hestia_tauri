# Plan: SerializableError Proc-Macro

Create a derive macro that automatically implements `serde::Serialize` for thiserror enums, eliminating the manual boilerplate pattern currently used in `AppError`, `StateError`, and `LibraryError`.

## Target Output Format

```json
{"kind":"variantName","message":"The error message from Display"}
```

Matches the existing pattern in `crates/app/src/lib.rs:138-144` and `crates/config/src/lib.rs:78-91`.

## Usage Example

**Before (manual):**
```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("The file watcher could not be found!")]
    WatcherNotFound,
    #[error("An internal error has occurred: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
enum AppErrorKind { ... }  // 10+ lines of boilerplate

impl Serialize for AppError { ... }  // 15+ lines of boilerplate
```

**After (with macro):**
```rust
#[derive(Debug, Error, SerializableError)]
pub enum AppError {
    #[error("The file watcher could not be found!")]
    WatcherNotFound,
    #[error("An internal error has occurred: {0}")]
    Internal(#[from] anyhow::Error),
}
```

## Implementation Steps

### Step 1: Create `errors-derive` crate

Create new proc-macro crate:
```
crates/errors-derive/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Files to create:**
- `crates/errors-derive/Cargo.toml`
- `crates/errors-derive/src/lib.rs`

### Step 2: Update workspace configuration

**File:** `Cargo.toml` (root)

Add to `workspace.members`:
```toml
"crates/errors-derive",
```

Add to `workspace.dependencies`:
```toml
errors-derive = { path = "crates/errors-derive" }
syn = { version = "2", features = ["full", "parsing", "derive"] }
quote = "1"
proc-macro2 = "1"
convert_case = "0.6"
```

### Step 3: Implement the derive macro

**File:** `crates/errors-derive/src/lib.rs`

The macro will:
1. Parse enum variants using `syn`
2. Convert variant names to camelCase (default) or use `#[error_code("CUSTOM")]`
3. Generate a `Serialize` impl using `SerializeStruct`

Key logic:
- `ImageDecode` → `"imageDecode"` (camelCase default)
- `#[error_code("FILE_NOT_FOUND")]` → `"FILE_NOT_FOUND"` (custom override)
- Uses `self.to_string()` for the message (leverages thiserror's Display)

### Step 4: Update `errors` crate

**File:** `crates/errors/Cargo.toml`

Add dependency:
```toml
errors-derive = { workspace = true }
```

**File:** `crates/errors/src/lib.rs`

Re-export the macro:
```rust
pub use errors_derive::SerializableError;
```

### Step 5: Migrate existing error types

**Files to update:**
1. `crates/errors/src/thumbnail.rs` - Add `SerializableError` derive
2. `crates/errors/src/config.rs` - Add `SerializableError` derive
3. `crates/config/src/lib.rs` - Replace manual impl (~40 lines removed)
4. `crates/app/src/lib.rs` - Replace manual impl (~20 lines removed)
5. `crates/app/src/state.rs` - Replace manual impl (~20 lines removed)

### Step 6: Write tests

**File:** `crates/errors-derive/tests/integration.rs`

Test cases:
- Unit variants serialize correctly
- Tuple variants serialize correctly
- Named field variants serialize correctly
- `#[error_code]` attribute works
- camelCase conversion works

## Crate Dependencies

```
errors-derive/Cargo.toml:
  syn = "2" (features: full, parsing, derive)
  quote = "1"
  proc-macro2 = "1"
  convert_case = "0.6"
```

## Verification

1. Run `cargo build -p errors-derive` to verify macro compiles
2. Run `cargo test -p errors-derive` to verify tests pass
3. Run `cargo build` to verify full workspace compiles
4. Test serialization manually:
   ```rust
   let err = ThumbnailServiceError::ThumbnailNotFound;
   let json = serde_json::to_string(&err).unwrap();
   assert!(json.contains("thumbnailNotFound"));
   ```

## Critical Files

| File | Action |
|------|--------|
| `Cargo.toml` | Add workspace members and dependencies |
| `crates/errors-derive/Cargo.toml` | Create (new) |
| `crates/errors-derive/src/lib.rs` | Create (new) - ~80 lines |
| `crates/errors/Cargo.toml` | Add errors-derive dependency |
| `crates/errors/src/lib.rs` | Add re-export |
| `crates/errors/src/thumbnail.rs` | Add derive |
| `crates/config/src/lib.rs` | Replace manual impl |
| `crates/app/src/lib.rs` | Replace manual impl |
| `crates/app/src/state.rs` | Replace manual impl |