use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{command, State};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set, TransactionTrait,
};

use crate::config::app::AppState;
use crate::errors::{DbError, DbErrorKind};

use entity::{file_has_tags, files, prelude::*, tags};

/// Tag information for frontend display
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

/// File-tag relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTagInfo {
    pub id: i32,
    pub file_id: i32,
    pub tag_id: i32,
    pub file_name: Option<String>,
    pub tag_name: Option<String>,
}

/// Create a new tag
#[command]
pub async fn create_tag(
    app_state: State<'_, Mutex<AppState>>,
    tag_name: String,
) -> Result<TagInfo, String> {
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
        Ok(None) => {
            // Tag doesn't exist, continue with creation
        }
        Err(e) => {
            return Err(format!("Database error checking existing tag: {}", e));
        }
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
        Err(e) => Err(format!("Failed to create tag: {}", e)),
    }
}

/// Get all tags
#[command]
pub async fn get_all_tags(app_state: State<'_, Mutex<AppState>>) -> Result<Vec<TagInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    match Tags::find().all(&*connection).await {
        Ok(tags) => Ok(tags.into_iter().map(|t| t.into()).collect()),
        Err(e) => Err(format!("Failed to get tags: {}", e)),
    }
}

/// Get tag by ID
#[command]
pub async fn get_tag_by_id(
    app_state: State<'_, Mutex<AppState>>,
    tag_id: i32,
) -> Result<Option<TagInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    match Tags::find_by_id(tag_id).one(&*connection).await {
        Ok(Some(tag)) => Ok(Some(tag.into())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get tag: {}", e)),
    }
}

/// Get tag by name
#[command]
pub async fn get_tag_by_name(
    app_state: State<'_, Mutex<AppState>>,
    tag_name: String,
) -> Result<Option<TagInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    match Tags::find()
        .filter(tags::Column::Name.eq(&tag_name))
        .one(&*connection)
        .await
    {
        Ok(Some(tag)) => Ok(Some(tag.into())),
        Ok(None) => Ok(None),
        Err(e) => Err(format!("Failed to get tag by name: {}", e)),
    }
}

/// Update tag name
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

    // Check if new name already exists (but not for the same tag)
    if let Ok(Some(_)) = Tags::find()
        .filter(tags::Column::Name.eq(&new_name))
        .filter(tags::Column::Id.ne(tag_id))
        .one(&*connection)
        .await
    {
        return Err("Tag name already exists".to_string());
    }

    // Update tag
    let mut active_model = existing_tag.into_active_model();
    active_model.name = Set(new_name);
    active_model.updated_at = Set(chrono::Utc::now().naive_utc());

    match active_model.update(&*connection).await {
        Ok(updated_tag) => Ok(updated_tag.into()),
        Err(e) => Err(format!("Failed to update tag: {}", e)),
    }
}

/// Delete a tag
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
                return Err(format!("Failed to delete file-tag relationships and rollback failed: {} (rollback error: {})", e, rollback_err));
            }
            return Err(format!("Failed to delete file-tag relationships: {}", e));
        }
    }

    // Delete the tag
    let delete_result = match Tags::delete_by_id(tag_id).exec(&transaction).await {
        Ok(result) => result,
        Err(e) => {
            if let Err(rollback_err) = transaction.rollback().await {
                return Err(format!(
                    "Failed to delete tag and rollback failed: {} (rollback error: {})",
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

/// Add tag to file
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

    // Create new file-tag relationship
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

/// Remove tag from file
#[command]
pub async fn remove_tag_from_file(
    app_state: State<'_, Mutex<AppState>>,
    file_id: i32,
    tag_id: i32,
) -> Result<bool, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    let delete_result = match file_has_tags::Entity::delete_many()
        .filter(file_has_tags::Column::FileId.eq(file_id))
        .filter(file_has_tags::Column::TagId.eq(tag_id))
        .exec(&*connection)
        .await
    {
        Ok(result) => result,
        Err(e) => return Err(format!("Failed to remove tag from file: {}", e)),
    };

    Ok(delete_result.rows_affected > 0)
}

/// Get all tags for a file
#[command]
pub async fn get_tags_for_file(
    app_state: State<'_, Mutex<AppState>>,
    file_id: i32,
) -> Result<Vec<TagInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Get file-tag relationships
    let file_tag_relationships = match file_has_tags::Entity::find()
        .filter(file_has_tags::Column::FileId.eq(file_id))
        .all(&*connection)
        .await
    {
        Ok(relationships) => relationships,
        Err(e) => return Err(format!("Failed to get file-tag relationships: {}", e)),
    };

    // Get tags for each relationship
    let mut tags = Vec::new();
    for relationship in file_tag_relationships {
        if let Ok(Some(tag)) = Tags::find_by_id(relationship.tag_id)
            .one(&*connection)
            .await
        {
            tags.push(tag.into());
        }
    }

    Ok(tags)
}

/// Get all files for a tag
#[command]
pub async fn get_files_for_tag(
    app_state: State<'_, Mutex<AppState>>,
    tag_id: i32,
) -> Result<Vec<super::file_operations::FileInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    // Get file-tag relationships
    let file_tag_relationships = match file_has_tags::Entity::find()
        .filter(file_has_tags::Column::TagId.eq(tag_id))
        .all(&*connection)
        .await
    {
        Ok(relationships) => relationships,
        Err(e) => return Err(format!("Failed to get file-tag relationships: {}", e)),
    };

    // Get files for each relationship
    let mut files = Vec::new();
    for relationship in file_tag_relationships {
        if let Ok(Some(file)) = Files::find_by_id(relationship.file_id)
            .one(&*connection)
            .await
        {
            files.push(file.into());
        }
    }

    Ok(files)
}

/// Get all file-tag relationships
#[command]
pub async fn get_all_file_tag_relationships(
    app_state: State<'_, Mutex<AppState>>,
) -> Result<Vec<FileTagInfo>, String> {
    let connection = {
        let state = app_state.lock().unwrap();
        state.database_manager.get_connection()
    };

    match file_has_tags::Entity::find().all(&*connection).await {
        Ok(relationships) => {
            let mut result = Vec::new();
            for relationship in relationships {
                result.push(FileTagInfo {
                    id: relationship.id,
                    file_id: relationship.file_id,
                    tag_id: relationship.tag_id,
                    file_name: None,
                    tag_name: None,
                });
            }
            Ok(result)
        }
        Err(e) => Err(format!("Failed to get file-tag relationships: {}", e)),
    }
}

/// Search tags by name pattern
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
