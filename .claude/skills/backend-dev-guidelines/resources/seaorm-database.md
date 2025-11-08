# SeaORM Database Patterns for Hestia

A comprehensive guide to SeaORM patterns, entity relationships, and database operations in the Hestia project. This resource covers entity definitions, CRUD operations, transaction handling, and real-world patterns used throughout the codebase.

## Table of Contents
1. [SeaORM Overview](#seaorm-overview)
2. [Entity Definitions & Relationships](#entity-definitions--relationships)
3. [ActiveModel Pattern](#activemodel-pattern)
4. [CRUD Operations](#crud-operations)
5. [Transactions & Batch Operations](#transactions--batch-operations)
6. [Complex Queries](#complex-queries)
7. [Relationship Handling](#relationship-handling)
8. [Query Optimization](#query-optimization)
9. [Best Practices](#best-practices)
10. [Quick Reference](#quick-reference)

---

## SeaORM Overview

**SeaORM** (Sea Object-Relational Mapping) is a robust async Rust ORM that provides:
- Type-safe query building (compile-time validation)
- Full transaction support with ACID guarantees
- Efficient connection pooling via Arc<DatabaseConnection>
- Flexible relationship handling (1-to-many, many-to-many, self-referential)
- Cross-database compatibility (SQLite, PostgreSQL, MySQL)

### Core Concepts

**Model**: Represents a database row in strongly-typed Rust
```rust
pub struct files::Model {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub content_hash: String,
    pub created_at: DateTime<FixedOffset>,
    // ... more fields
}
```

**ActiveModel**: Wraps a Model to enable inserts/updates with optional fields
```rust
pub struct files::ActiveModel {
    pub id: NotSet,
    pub name: Set(String),
    pub path: Set(String),
    // ... fields wrapped with Set/NotSet/Unchanged
}
```

**Entity**: Represents the database table schema and relationships
```rust
pub struct files::Entity;
// Generated relation definitions:
// files::Relation::FileTypes
// files::Relation::FileHasTags
// files::Relation::Thumbnails
```

**ConnectionTrait**: Abstraction allowing functions to work with connections AND transactions
```rust
pub async fn query<C: ConnectionTrait>(&self, db: &C) -> Result<Data>
```

---

## Entity Definitions & Relationships

### Hestia Entity Model

Hestia's database uses 7 core entities with a hierarchical relationship model:

```
FileSystemIdentifier (platform-specific identity)
    ↓
    ├─→ Files (with file_type_id, content_hash)
    │    ├─→ FileHasTags (many-to-many junction)
    │    │    └─→ Tags
    │    └─→ Thumbnails (binary data storage)
    │
    └─→ Folders (hierarchical with parent_folder_id)
```

### Example: Files Entity Definition

```rust
// src-tauri/src-tauri/entity/src/files.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "files")]
pub struct Model {
    #[primary_key]
    pub id: i32,

    #[sea_orm(column_type = "Integer", not_null)]
    pub file_system_id: i32,

    #[sea_orm(column_type = "String")]
    pub name: String,

    #[sea_orm(column_type = "String", not_null)]
    pub path: String,

    #[sea_orm(column_type = "String")]
    pub content_hash: String,

    #[sea_orm(column_type = "Integer", not_null)]
    pub file_type_id: i32,

    #[sea_orm(column_type = "DateTime")]
    pub created_at: DateTime<FixedOffset>,

    #[sea_orm(column_type = "DateTime")]
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "super::file_types::Entity",
              foreign_key = "file_type_id")]
    FileTypes,

    #[sea_orm(has_many = "super::file_has_tags::Entity")]
    FileHasTags,

    #[sea_orm(has_many = "super::thumbnails::Entity")]
    Thumbnails,
}

impl Related<super::file_types::Entity> for Entity {
    fn to() -> RelationTwoTuples {
        Relation::FileTypes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

### Example: Self-Referential Relationship (Folders)

```rust
// src-tauri/src-tauri/entity/src/folders.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "folders")]
pub struct Model {
    #[primary_key]
    pub id: i32,

    pub file_system_id: i32,

    // Self-referential relationship for folder hierarchy
    pub parent_folder_id: Option<i32>,

    pub name: String,
    pub path: String,
    pub content_hash: String,

    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(belongs_to = "Entity",
              foreign_key = "parent_folder_id")]
    ParentFolder,

    #[sea_orm(has_many = "Entity")]
    ChildFolders,
}

// Allow loading parent folder recursively
impl Related<Entity> for Entity {
    fn to() -> RelationTwoTuples {
        Relation::ParentFolder.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    // Hook for maintaining data consistency
    async fn before_save<C>(
        mut self,
        _db: &C,
        insert: bool,
    ) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            self.created_at = Set(chrono::Local::now().naive_local());
        }
        self.updated_at = Set(chrono::Local::now().naive_local());
        Ok(self)
    }
}
```

### Example: Many-to-Many Junction Table

```rust
// src-tauri/src-tauri/entity/src/file_has_tags.rs
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "file_has_tags")]
pub struct Model {
    #[primary_key]
    pub id: i32,

    #[sea_orm(indexed)]
    pub file_id: i32,

    #[sea_orm(indexed)]
    pub tag_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::files::Entity",
        foreign_key = "file_id"
    )]
    Files,

    #[sea_orm(
        belongs_to = "super::tags::Entity",
        foreign_key = "tag_id"
    )]
    Tags,
}

// Enable loading related files through this junction
impl Related<super::files::Entity> for super::tags::Entity {
    fn to() -> RelationTwoTuples {
        super::file_has_tags::Relation::Files.def()
    }

    fn via() -> Option<RelationTwoTuples> {
        Some(super::file_has_tags::Relation::Tags.def().rev())
    }
}

impl Related<super::tags::Entity> for super::files::Entity {
    fn to() -> RelationTwoTuples {
        super::file_has_tags::Relation::Tags.def()
    }

    fn via() -> Option<RelationTwoTuples> {
        Some(super::file_has_tags::Relation::Files.def().rev())
    }
}
```

---

## ActiveModel Pattern

The **ActiveModel** pattern is central to SeaORM's update/insert logic. Use it when modifying entities.

### Pattern 1: Insert with NotSet Primary Key

```rust
// For auto-increment IDs, use ActiveValue::NotSet
let new_file = files::ActiveModel {
    id: sea_orm::ActiveValue::NotSet,  // Database generates ID
    name: Set("document.pdf".to_string()),
    path: Set("/path/to/document.pdf".to_string()),
    content_hash: Set("abc123def456".to_string()),
    file_type_id: Set(1),
    created_at: Set(chrono::Local::now().naive_local()),
    updated_at: Set(chrono::Local::now().naive_local()),
};

// Insert and get back the generated ID
let inserted = new_file.insert(&connection).await?;
println!("Generated ID: {}", inserted.id);  // Access id after insert
```

**Key Point**: `NotSet` tells the database to generate the ID; `Set` provides explicit values.

### Pattern 2: Update Selective Fields

```rust
// Convert Model to ActiveModel, then update only changed fields
let existing_file = Files::find_by_id(42)
    .one(&connection)
    .await?
    .ok_or("File not found")?;

let mut active_file = existing_file.into_active_model();

// Only update these fields
active_file.content_hash = Set("new_hash_xyz".to_string());
active_file.updated_at = Set(chrono::Local::now().naive_local());

// Other fields remain Unchanged (not sent to DB)
let updated = active_file.update(&connection).await?;
```

### Pattern 3: Save (Insert-or-Update)

```rust
// .save() handles both insert and update intelligently
let active_model = tags::ActiveModel {
    id: if existing_id.is_some() {
        Set(existing_id.unwrap())
    } else {
        sea_orm::ActiveValue::NotSet
    },
    name: Set(tag_name),
    created_at: Set(chrono::Local::now().naive_local()),
    updated_at: Set(chrono::Local::now().naive_local()),
};

// If id is set: performs UPDATE; if NotSet: performs INSERT
let result = active_model.save(&connection).await?;
```

### Pattern 4: Bulk Update with Condition

```rust
// Update multiple rows matching a condition
Files::update_many()
    .col_expr(files::Column::UpdatedAt,
               Expr::value(chrono::Local::now().naive_local()))
    .filter(files::Column::FileTypeId.eq(1))
    .exec(&connection)
    .await?;
```

---

## CRUD Operations

### Create: Insert with Idempotency Check

```rust
pub async fn create_tag_idempotent(
    db: &DatabaseConnection,
    tag_name: String,
) -> anyhow::Result<tags::Model> {
    // Check if tag already exists
    if let Some(existing) = Tags::find()
        .filter(tags::Column::Name.eq(&tag_name))
        .one(db)
        .await
        .context("Failed to check for existing tag")?
    {
        return Ok(existing);  // Return existing tag
    }

    // Create new tag
    let new_tag = tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(tag_name),
        created_at: Set(chrono::Local::now().naive_local()),
        updated_at: Set(chrono::Local::now().naive_local()),
    };

    new_tag
        .insert(db)
        .await
        .context("Failed to insert new tag")
}
```

### Read: Find with Filters

```rust
pub async fn find_file_by_path(
    db: &DatabaseConnection,
    path: &str,
) -> anyhow::Result<Option<files::Model>> {
    Files::find()
        .filter(files::Column::Path.eq(path))
        .one(db)
        .await
        .context("Failed to query database")
}

pub async fn find_files_by_type(
    db: &DatabaseConnection,
    file_type_id: i32,
) -> anyhow::Result<Vec<files::Model>> {
    Files::find()
        .filter(files::Column::FileTypeId.eq(file_type_id))
        .all(db)
        .await
        .context("Failed to fetch files by type")
}

pub async fn find_files_with_tag(
    db: &DatabaseConnection,
    tag_id: i32,
) -> anyhow::Result<Vec<files::Model>> {
    // Query through the junction table
    Tags::find_by_id(tag_id)
        .find_related(Files)  // Uses the Many-to-Many relation
        .all(db)
        .await
        .context("Failed to fetch files with tag")
}
```

### Update: Modify Existing Record

```rust
pub async fn update_file_hash(
    db: &DatabaseConnection,
    file_id: i32,
    new_hash: String,
) -> anyhow::Result<()> {
    let file = Files::find_by_id(file_id)
        .one(db)
        .await
        .context("Failed to fetch file")?
        .ok_or(anyhow!("File not found"))?;

    let mut active_file = file.into_active_model();
    active_file.content_hash = Set(new_hash);
    active_file.updated_at = Set(chrono::Local::now().naive_local());

    active_file
        .update(db)
        .await
        .context("Failed to update file")?;

    Ok(())
}
```

### Delete: Remove Records

```rust
pub async fn delete_file(
    db: &DatabaseConnection,
    file_id: i32,
) -> anyhow::Result<()> {
    Files::delete_by_id(file_id)
        .exec(db)
        .await
        .context("Failed to delete file")?;

    Ok(())
}

pub async fn delete_files_batch(
    db: &DatabaseConnection,
    file_ids: Vec<i32>,
) -> anyhow::Result<u64> {
    let result = Files::delete_many()
        .filter(files::Column::Id.is_in(file_ids))
        .exec(db)
        .await
        .context("Failed to batch delete files")?;

    Ok(result.rows_affected)
}
```

---

## Transactions & Batch Operations

### Transaction Pattern: Atomic Multi-Step Operations

```rust
pub async fn link_file_to_tags(
    db: &DatabaseConnection,
    file_id: i32,
    tag_ids: Vec<i32>,
) -> anyhow::Result<()> {
    // Begin atomic transaction
    let txn = db
        .begin()
        .await
        .context("Failed to start transaction")?;

    // Step 1: Clear existing tags for this file
    file_has_tags::Entity::delete_many()
        .filter(file_has_tags::Column::FileId.eq(file_id))
        .exec(&txn)
        .await
        .context("Failed to delete existing tag links")?;

    // Step 2: Insert new tag links
    for tag_id in tag_ids {
        let new_link = file_has_tags::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            file_id: Set(file_id),
            tag_id: Set(tag_id),
        };

        new_link
            .insert(&txn)
            .await
            .context("Failed to insert tag link")?;
    }

    // Step 3: Update file's updated_at timestamp
    let mut file = Files::find_by_id(file_id)
        .one(&txn)
        .await
        .context("Failed to fetch file")?
        .ok_or(anyhow!("File not found"))?
        .into_active_model();

    file.updated_at = Set(chrono::Local::now().naive_local());
    file.update(&txn)
        .await
        .context("Failed to update file timestamp")?;

    // Commit atomically - all succeed or all fail
    txn.commit()
        .await
        .context("Failed to commit transaction")
}
```

### Batch Upsert with Statistics

```rust
pub struct BatchReport {
    pub inserted: usize,
    pub updated: usize,
}

pub async fn batch_upsert_files(
    db: &DatabaseConnection,
    files_data: Vec<FileImportData>,
) -> anyhow::Result<BatchReport> {
    if files_data.is_empty() {
        return Ok(BatchReport {
            inserted: 0,
            updated: 0,
        });
    }

    let txn = db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let mut report = BatchReport {
        inserted: 0,
        updated: 0,
    };

    for file_data in files_data {
        // Check if file exists by path (unique constraint)
        let existing = Files::find()
            .filter(files::Column::Path.eq(&file_data.path))
            .one(&txn)
            .await
            .context("Failed to check existing file")?;

        match existing {
            Some(file_model) => {
                // UPDATE path
                let mut active = file_model.into_active_model();
                active.content_hash = Set(file_data.content_hash);
                active.updated_at = Set(chrono::Local::now().naive_local());

                active
                    .update(&txn)
                    .await
                    .context("Failed to update file")?;

                report.updated += 1;
            }
            None => {
                // INSERT path
                let new_file = files::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    name: Set(file_data.name),
                    path: Set(file_data.path),
                    content_hash: Set(file_data.content_hash),
                    file_type_id: Set(file_data.file_type_id),
                    created_at: Set(chrono::Local::now().naive_local()),
                    updated_at: Set(chrono::Local::now().naive_local()),
                };

                new_file
                    .insert(&txn)
                    .await
                    .context("Failed to insert file")?;

                report.inserted += 1;
            }
        }
    }

    txn.commit()
        .await
        .context("Failed to commit batch upsert")?;

    Ok(report)
}
```

### Batch Delete with Cascade Awareness

```rust
pub async fn delete_folder_and_contents(
    db: &DatabaseConnection,
    folder_id: i32,
) -> anyhow::Result<()> {
    let txn = db
        .begin()
        .await
        .context("Failed to start transaction")?;

    // Step 1: Find all files in this folder
    let files = Files::find()
        .filter(files::Column::FolderId.eq(folder_id))
        .all(&txn)
        .await
        .context("Failed to fetch folder files")?;

    // Step 2: Delete tag relationships (must happen before file delete)
    for file in &files {
        file_has_tags::Entity::delete_many()
            .filter(file_has_tags::Column::FileId.eq(file.id))
            .exec(&txn)
            .await
            .context("Failed to delete file tags")?;
    }

    // Step 3: Delete files (cascade deletes thumbnails automatically)
    Files::delete_many()
        .filter(files::Column::FolderId.eq(folder_id))
        .exec(&txn)
        .await
        .context("Failed to delete files")?;

    // Step 4: Delete subfolders recursively
    Folders::delete_many()
        .filter(folders::Column::ParentFolderId.eq(folder_id))
        .exec(&txn)
        .await
        .context("Failed to delete subfolders")?;

    // Step 5: Delete the folder itself
    Folders::delete_by_id(folder_id)
        .exec(&txn)
        .await
        .context("Failed to delete folder")?;

    txn.commit()
        .await
        .context("Failed to commit deletion transaction")
}
```

---

## Complex Queries

### Multi-Table Join with Filtering

```rust
pub struct FileSearchFilters {
    pub name_pattern: Option<String>,
    pub file_types: Vec<String>,
    pub tags: Vec<String>,
    pub require_all_tags: bool,
}

pub async fn search_files(
    db: &DatabaseConnection,
    filters: FileSearchFilters,
    page: u64,
    per_page: u64,
) -> anyhow::Result<SearchResults> {
    let per_page = per_page.min(200);  // Cap at 200 for safety
    let offset = (page - 1) * per_page;

    // Build query step by step
    let mut query = Files::find();

    // Filter by name pattern
    if let Some(pattern) = &filters.name_pattern {
        let like_pattern = format!("%{}%", pattern);
        query = query.filter(files::Column::Name.like(&like_pattern));
    }

    // Filter by file type
    if !filters.file_types.is_empty() {
        query = query
            .join(JoinType::InnerJoin, files::Relation::FileTypes.def())
            .filter(file_types::Column::Name.is_in(&filters.file_types));
    }

    // Complex tag filtering with AND/OR logic
    if !filters.tags.is_empty() {
        if filters.require_all_tags {
            // Files must have ALL tags (multiple inner joins)
            for tag_name in &filters.tags {
                query = query
                    .join(JoinType::InnerJoin, files::Relation::FileHasTags.def())
                    .join(JoinType::InnerJoin, file_has_tags::Relation::Tags.def())
                    .filter(tags::Column::Name.eq(tag_name));
            }
        } else {
            // Files must have ANY tag (single join with IN)
            query = query
                .join(JoinType::InnerJoin, files::Relation::FileHasTags.def())
                .join(JoinType::InnerJoin, file_has_tags::Relation::Tags.def())
                .filter(tags::Column::Name.is_in(&filters.tags));
        }
    }

    // Clone query for count (before adding pagination)
    let total_count = query
        .clone()
        .count(db)
        .await
        .context("Failed to count results")?;

    // Add pagination
    let files = query
        .offset(offset)
        .limit(per_page)
        .all(db)
        .await
        .context("Failed to fetch search results")?;

    let total_pages = (total_count + per_page - 1) / per_page;

    Ok(SearchResults {
        items: files,
        total_count,
        page,
        per_page,
        total_pages,
    })
}
```

### Subquery Pattern: Exclude Records

```rust
pub async fn get_untagged_files(
    db: &DatabaseConnection,
    page: u64,
    per_page: u64,
) -> anyhow::Result<Vec<files::Model>> {
    let per_page = per_page.min(200);
    let offset = (page - 1) * per_page;

    // Subquery: find file IDs that have tags
    let tagged_file_ids = file_has_tags::Entity::find()
        .select_only()
        .column(file_has_tags::Column::FileId)
        .into_query();

    // Main query: find files NOT in that subquery
    Files::find()
        .filter(
            files::Column::Id.not_in_subquery(tagged_file_ids)
        )
        .offset(offset)
        .limit(per_page)
        .all(db)
        .await
        .context("Failed to fetch untagged files")
}
```

### Hierarchical Query (Self-Referential)

```rust
pub async fn get_folder_tree(
    db: &DatabaseConnection,
    parent_id: Option<i32>,
) -> anyhow::Result<Vec<FolderWithChildren>> {
    // Get folders at this level
    let folders = Folders::find()
        .filter(folders::Column::ParentFolderId.eq(parent_id))
        .all(db)
        .await
        .context("Failed to fetch folders")?;

    let mut results = Vec::new();

    for folder_model in folders {
        // Recursively load children
        let children = get_folder_tree(db, Some(folder_model.id)).await?;

        results.push(FolderWithChildren {
            folder: folder_model,
            children,
        });
    }

    Ok(results)
}
```

---

## Relationship Handling

### Load Related Data (Eager Loading)

```rust
pub async fn get_file_with_tags(
    db: &DatabaseConnection,
    file_id: i32,
) -> anyhow::Result<(files::Model, Vec<tags::Model>)> {
    // Load file
    let file = Files::find_by_id(file_id)
        .one(db)
        .await
        .context("Failed to find file")?
        .ok_or(anyhow!("File not found"))?;

    // Load related tags through the junction table
    let tags = file
        .find_related(Tags)  // Uses the Many-to-Many relation
        .all(db)
        .await
        .context("Failed to load file tags")?;

    Ok((file, tags))
}
```

### Load Related Data (Lazy Loading)

```rust
pub async fn get_file_type_for_file(
    db: &DatabaseConnection,
    file: &files::Model,
) -> anyhow::Result<file_types::Model> {
    // Load the foreign key relationship lazily
    file
        .find_related(FileTypes)
        .one(db)
        .await
        .context("Failed to load file type")?
        .ok_or(anyhow!("File type not found"))
}
```

### Create With Related Data

```rust
pub async fn create_file_with_tags(
    db: &DatabaseConnection,
    file_data: FileImportData,
    tag_ids: Vec<i32>,
) -> anyhow::Result<files::Model> {
    let txn = db
        .begin()
        .await
        .context("Failed to start transaction")?;

    // Insert file
    let new_file = files::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        name: Set(file_data.name),
        path: Set(file_data.path),
        content_hash: Set(file_data.content_hash),
        file_type_id: Set(file_data.file_type_id),
        created_at: Set(chrono::Local::now().naive_local()),
        updated_at: Set(chrono::Local::now().naive_local()),
    };

    let file = new_file
        .insert(&txn)
        .await
        .context("Failed to insert file")?;

    // Insert tag links
    for tag_id in tag_ids {
        let link = file_has_tags::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            file_id: Set(file.id),
            tag_id: Set(tag_id),
        };

        link
            .insert(&txn)
            .await
            .context("Failed to insert tag link")?;
    }

    txn.commit()
        .await
        .context("Failed to commit transaction")?;

    Ok(file)
}
```

---

## Query Optimization

### Pagination for Large Results

```rust
pub async fn paginated_files(
    db: &DatabaseConnection,
    page: u64,
    per_page: u64,
) -> anyhow::Result<PaginatedResults<files::Model>> {
    // Safety: cap at 200 per page
    let per_page = per_page.min(200);
    let offset = (page - 1) * per_page;

    let total_count = Files::find()
        .count(db)
        .await
        .context("Failed to count files")?;

    let files = Files::find()
        .offset(offset)
        .limit(per_page)
        .all(db)
        .await
        .context("Failed to fetch paginated files")?;

    let total_pages = (total_count + per_page - 1) / per_page;

    Ok(PaginatedResults {
        items: files,
        current_page: page,
        per_page,
        total_count,
        total_pages,
    })
}
```

### Selective Column Loading

```rust
pub async fn get_file_names(
    db: &DatabaseConnection,
    folder_id: i32,
) -> anyhow::Result<Vec<(i32, String)>> {
    Files::find()
        .select_only()
        .column(files::Column::Id)
        .column(files::Column::Name)
        .filter(files::Column::FolderId.eq(folder_id))
        .into_tuple::<(i32, String)>()
        .all(db)
        .await
        .context("Failed to fetch file names")
}
```

### Caching with RwLock Pattern

```rust
pub struct FileTypeCache {
    cache: Arc<RwLock<HashMap<String, i32>>>,
    db: Arc<DatabaseConnection>,
}

impl FileTypeCache {
    pub async fn get_or_create(&self, name: &str) -> anyhow::Result<i32> {
        // Check cache first (read lock, allows concurrency)
        {
            let cache = self.cache.read().await;
            if let Some(&id) = cache.get(name) {
                return Ok(id);
            }
        } // Lock released here

        // Not in cache; query database
        let file_type = FileTypes::find()
            .filter(file_types::Column::Name.eq(name))
            .one(&*self.db)
            .await
            .context("Failed to query file type")?;

        let type_id = match file_type {
            Some(ft) => ft.id,
            None => {
                // Create new type
                let new_type = file_types::ActiveModel {
                    id: sea_orm::ActiveValue::NotSet,
                    name: Set(name.to_string()),
                    created_at: Set(chrono::Local::now().naive_local()),
                    updated_at: Set(chrono::Local::now().naive_local()),
                };

                let inserted = new_type
                    .insert(&*self.db)
                    .await
                    .context("Failed to insert file type")?;

                inserted.id
            }
        };

        // Update cache (write lock)
        {
            let mut cache = self.cache.write().await;
            cache.insert(name.to_string(), type_id);
        }

        Ok(type_id)
    }

    pub async fn preload(&self) -> anyhow::Result<()> {
        let types = FileTypes::find()
            .all(&*self.db)
            .await
            .context("Failed to fetch file types")?;

        let mut cache = self.cache.write().await;
        for ft in types {
            cache.insert(ft.name, ft.id);
        }

        Ok(())
    }
}
```

---

## Best Practices

### DO: Use Anyhow for Internal Operations

```rust
// ✅ GOOD: Services use anyhow::Result
pub async fn process_file(db: &DatabaseConnection, file_id: i32) -> anyhow::Result<FileMetadata> {
    let file = Files::find_by_id(file_id)
        .one(db)
        .await
        .context("Failed to load file")?
        .ok_or(anyhow!("File not found"))?;

    // Process...
    Ok(metadata)
}
```

### DON'T: Expose SeaORM Errors

```rust
// ❌ BAD: Exposes SeaORM DbErr to frontend
#[command]
pub async fn get_file(file_id: i32) -> Result<File, DbErr> {
    // This leaks database internals!
}

// ✅ GOOD: Map to domain error
#[command]
pub async fn get_file(file_id: i32) -> Result<File, ApiError> {
    service::get_file(file_id)
        .await
        .map_err(|e| ApiError::NotFound)
}
```

### DO: Use Transactions for Multi-Step Operations

```rust
// ✅ GOOD: Atomic operation
pub async fn reorganize_files(db: &DatabaseConnection) -> anyhow::Result<()> {
    let txn = db.begin().await?;

    // Multiple operations...

    txn.commit().await?;
    Ok(())
}
```

### DON'T: Forget to Commit Transactions

```rust
// ❌ BAD: Transaction dropped without commit (implicit rollback)
pub async fn update_file(db: &DatabaseConnection) -> anyhow::Result<()> {
    let txn = db.begin().await?;
    // Operations...
    // Missing: txn.commit().await?;
    Ok(())  // Transaction silently rolled back!
}
```

### DO: Use ConnectionTrait for Function Flexibility

```rust
// ✅ GOOD: Works with both connection and transaction
pub async fn insert_file<C: ConnectionTrait>(
    db: &C,
    file_data: FileData,
) -> anyhow::Result<files::Model> {
    let model = files::ActiveModel {
        // ...
    };
    model.insert(db).await.context("Failed to insert")
}

// Can be called from transaction:
insert_file(&txn, data).await?;

// Or directly from connection:
insert_file(&connection, data).await?;
```

### DO: Cache Lookup Tables

```rust
// ✅ GOOD: Reduces database hits
let file_type_cache = FileTypeCache::new(connection.clone());
file_type_cache.preload().await?;

let type_id = file_type_cache.get_or_create("image/png").await?;
```

### DO: Use `.clone()` Query Before Count

```rust
// ✅ GOOD: Avoids rewriting complex queries
let query = Files::find()
    .filter(/*...complex conditions...*/)
    .filter(/*...more conditions...*/);

let total = query.clone().count(db).await?;
let page = query
    .offset(0)
    .limit(50)
    .all(db)
    .await?;
```

### DO: Cap Pagination Results

```rust
// ✅ GOOD: Prevents DoS through large result sets
let per_page = per_page.unwrap_or(50).min(200);  // Cap at 200
```

### DO: Validate Foreign Keys

```rust
// ✅ GOOD: Check relationships before creating
pub async fn add_tag_to_file(
    db: &DatabaseConnection,
    file_id: i32,
    tag_id: i32,
) -> anyhow::Result<()> {
    // Verify file exists
    Files::find_by_id(file_id)
        .one(db)
        .await
        .context("Failed to load file")?
        .ok_or(anyhow!("File not found"))?;

    // Verify tag exists
    Tags::find_by_id(tag_id)
        .one(db)
        .await
        .context("Failed to load tag")?
        .ok_or(anyhow!("Tag not found"))?;

    // Create link
    let link = file_has_tags::ActiveModel {
        id: sea_orm::ActiveValue::NotSet,
        file_id: Set(file_id),
        tag_id: Set(tag_id),
    };

    link.insert(db).await.context("Failed to create link")?;
    Ok(())
}
```

---

## Quick Reference

### Common Operations Cheat Sheet

```rust
// Find operations
Files::find_by_id(id).one(db).await?          // Single row by PK
Files::find().filter(...).one(db).await?      // Single row with filter
Files::find().filter(...).all(db).await?      // Multiple rows
Files::find().count(db).await?                // Count matching rows

// Insert operations
model.insert(db).await?                        // Single insert
model.save(db).await?                          // Insert-or-update
delete_many().filter(...).exec(db).await?     // Bulk delete

// Transaction operations
let txn = db.begin().await?;                   // Start transaction
// ... use txn like normal connection ...
txn.commit().await?;                          // Commit atomically

// Relationship loading
model.find_related(Entity).all(db).await?     // Eager load related

// Filtering
.filter(Col::Field.eq(value))                 // Equals
.filter(Col::Field.like("pattern"))           // Like pattern
.filter(Col::Field.is_in(vec))                // IN list
.filter(Col::Field.gt(value))                 // Greater than
.filter(Col::Field.between(a, b))             // Between

// Pagination
.offset((page - 1) * per_page)
.limit(per_page)
.all(db).await?

// Joining tables
.join(JoinType::InnerJoin, Entity::Relation::Name.def())
.join(JoinType::LeftJoin, Entity::Relation::Name.def())
```

### Result Type Convention

```rust
// ✅ Service layer
pub async fn service() -> anyhow::Result<Data>

// ✅ Tauri command
#[command]
pub async fn command() -> Result<Data, ApiError>

// Conversion
service().await.map_err(|_| ApiError::InternalError)
```

### Error Handling Pattern

```rust
// Map database errors to domain errors
match operation.await {
    Ok(data) => Ok(data),
    Err(e) => {
        tracing::error!("Operation failed: {}", e);
        Err(DbError::OperationFailed)
    }
}
```

---

## References

- **Entity Module**: `app/entity/src/`
- **Database Operations**: `app/src-tauri/src/database/`
- **Commands**: `app/src-tauri/src/commands/`
- **Error Types**: `app/src-tauri/src/errors/db.rs`
- **SeaORM Docs**: https://www.sea-ql.org/SeaORM/

---

**Last Updated**: 2025-11-04
**Phase**: 1.4 (Resource Creation)
**Status**: Complete
