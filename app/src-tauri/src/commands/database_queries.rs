use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{command, State};

use sea_orm::{
    ColumnTrait, Condition, EntityTrait, JoinType, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, RelationTrait,
};

use super::file_operations::FileInfo;
use super::tag_management::TagInfo;
use crate::database::DatabaseManager;

use entity::{file_has_tags, file_types, files, prelude::*, tags};

/// Search filters for file queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchFilters {
    pub name_pattern: Option<String>,
    pub file_type: Option<String>,
    pub path_pattern: Option<String>,
    pub tags: Option<Vec<String>>,
    pub require_all_tags: Option<bool>, // true = AND, false = OR
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub updated_after: Option<String>,
    pub updated_before: Option<String>,
}

/// Search results with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// File with associated tags and type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWithDetails {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub content_hash: String,
    pub identity_hash: String,
    pub file_type_name: String,
    pub file_system_id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Vec<TagInfo>,
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_files: u64,
    pub total_tags: u64,
    pub total_file_types: u64,
    pub total_file_tag_relationships: u64,
    pub files_by_type: Vec<FileTypeCount>,
    pub most_used_tags: Vec<TagUsageCount>,
}

/// File type count for statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeCount {
    pub file_type: String,
    pub count: u64,
}

/// Tag usage count for statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagUsageCount {
    pub tag_name: String,
    pub file_count: u64,
}

/// Search files with filters and pagination
#[command]
pub async fn search_files(
    db_manager: State<'_, Arc<DatabaseManager>>,
    filters: FileSearchFilters,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<SearchResults<FileInfo>, String> {
    let connection = db_manager.get_connection();
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(50).min(200); // Cap at 200 items per page
    let offset = (page - 1) * per_page;

    let mut query = Files::find();

    // Apply name filter
    if let Some(name_pattern) = &filters.name_pattern {
        let pattern = format!("%{}%", name_pattern);
        query = query.filter(files::Column::Name.like(&pattern));
    }

    // Apply path filter
    if let Some(path_pattern) = &filters.path_pattern {
        let pattern = format!("%{}%", path_pattern);
        query = query.filter(files::Column::Path.like(&pattern));
    }

    // Apply file type filter
    if let Some(file_type) = &filters.file_type {
        query = query
            .join(JoinType::InnerJoin, files::Relation::FileTypes.def())
            .filter(file_types::Column::Name.eq(file_type));
    }

    // Apply date filters
    if let Some(created_after) = &filters.created_after {
        if let Ok(date) = chrono::NaiveDateTime::parse_from_str(created_after, "%Y-%m-%d %H:%M:%S")
        {
            query = query.filter(files::Column::CreatedAt.gt(date));
        }
    }

    if let Some(created_before) = &filters.created_before {
        if let Ok(date) = chrono::NaiveDateTime::parse_from_str(created_before, "%Y-%m-%d %H:%M:%S")
        {
            query = query.filter(files::Column::CreatedAt.lt(date));
        }
    }

    if let Some(updated_after) = &filters.updated_after {
        if let Ok(date) = chrono::NaiveDateTime::parse_from_str(updated_after, "%Y-%m-%d %H:%M:%S")
        {
            query = query.filter(files::Column::UpdatedAt.gt(date));
        }
    }

    if let Some(updated_before) = &filters.updated_before {
        if let Ok(date) = chrono::NaiveDateTime::parse_from_str(updated_before, "%Y-%m-%d %H:%M:%S")
        {
            query = query.filter(files::Column::UpdatedAt.lt(date));
        }
    }

    // Apply tag filters
    if let Some(tag_names) = &filters.tags {
        if !tag_names.is_empty() {
            let require_all = filters.require_all_tags.unwrap_or(false);

            if require_all {
                // For AND logic, we need files that have ALL specified tags
                for tag_name in tag_names {
                    query = query
                        .join(JoinType::InnerJoin, files::Relation::FileHasTags.def())
                        .join(JoinType::InnerJoin, file_has_tags::Relation::Tags.def())
                        .filter(tags::Column::Name.eq(tag_name));
                }
            } else {
                // For OR logic, we need files that have ANY of the specified tags
                query = query
                    .join(JoinType::InnerJoin, files::Relation::FileHasTags.def())
                    .join(JoinType::InnerJoin, file_has_tags::Relation::Tags.def())
                    .filter(tags::Column::Name.is_in(tag_names.clone()));
            }
        }
    }

    // Get total count
    let total_count = match query.clone().count(&*connection).await {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count search results: {}", e)),
    };

    // Get paginated results
    let files = match query.offset(offset).limit(per_page).all(&*connection).await {
        Ok(files) => files,
        Err(e) => return Err(format!("Failed to execute search query: {}", e)),
    };

    let total_pages = (total_count + per_page - 1) / per_page;

    Ok(SearchResults {
        items: files.into_iter().map(|f| f.into()).collect(),
        total_count,
        page,
        per_page,
        total_pages,
    })
}

/// Get files with full details (including tags and type)
#[command]
pub async fn get_files_with_details(
    db_manager: State<'_, Arc<DatabaseManager>>,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<SearchResults<FileWithDetails>, String> {
    let connection = db_manager.get_connection();
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    // Get files with pagination
    let files = match Files::find()
        .offset(offset)
        .limit(per_page)
        .all(&*connection)
        .await
    {
        Ok(files) => files,
        Err(e) => return Err(format!("Failed to get files: {}", e)),
    };

    let total_count = match Files::find().count(&*connection).await {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count files: {}", e)),
    };

    // Get details for each file
    let mut detailed_files = Vec::new();
    for file in files {
        // Get file type
        let file_type_name = match file_types::Entity::find_by_id(file.file_type_id)
            .one(&*connection)
            .await
        {
            Ok(Some(file_type)) => file_type.name,
            Ok(None) => "unknown".to_string(),
            Err(_) => "unknown".to_string(),
        };

        // Get tags for this file
        let tags = match file_has_tags::Entity::find()
            .filter(file_has_tags::Column::FileId.eq(file.id))
            .all(&*connection)
            .await
        {
            Ok(relationships) => {
                let mut file_tags = Vec::new();
                for relationship in relationships {
                    if let Ok(Some(tag)) = Tags::find_by_id(relationship.tag_id)
                        .one(&*connection)
                        .await
                    {
                        file_tags.push(tag.into());
                    }
                }
                file_tags
            }
            Err(_) => Vec::new(),
        };

        detailed_files.push(FileWithDetails {
            id: file.id,
            name: file.name,
            path: file.path,
            content_hash: file.content_hash,
            identity_hash: file.identity_hash,
            file_type_name,
            file_system_id: file.file_system_id,
            created_at: file.created_at.to_string(),
            updated_at: file.updated_at.to_string(),
            tags,
        });
    }

    let total_pages = (total_count + per_page - 1) / per_page;

    Ok(SearchResults {
        items: detailed_files,
        total_count,
        page,
        per_page,
        total_pages,
    })
}

/// Get database statistics
#[command]
pub async fn get_database_stats(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<DatabaseStats, String> {
    let connection = db_manager.get_connection();

    // Get total counts
    let total_files = match Files::find().count(&*connection).await {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count files: {}", e)),
    };

    let total_tags = match Tags::find().count(&*connection).await {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count tags: {}", e)),
    };

    let total_file_types = match file_types::Entity::find().count(&*connection).await {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count file types: {}", e)),
    };

    let total_file_tag_relationships = match file_has_tags::Entity::find().count(&*connection).await
    {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count file-tag relationships: {}", e)),
    };

    // Get files by type
    let file_types_data = match file_types::Entity::find().all(&*connection).await {
        Ok(types) => types,
        Err(e) => return Err(format!("Failed to get file types: {}", e)),
    };

    let mut files_by_type = Vec::new();
    for file_type in file_types_data {
        let count = match Files::find()
            .filter(files::Column::FileTypeId.eq(file_type.id))
            .count(&*connection)
            .await
        {
            Ok(count) => count,
            Err(_) => 0,
        };
        files_by_type.push(FileTypeCount {
            file_type: file_type.name,
            count,
        });
    }

    // Get most used tags
    let tags_data = match Tags::find().all(&*connection).await {
        Ok(tags) => tags,
        Err(e) => return Err(format!("Failed to get tags: {}", e)),
    };

    let mut most_used_tags = Vec::new();
    for tag in tags_data {
        let count = match file_has_tags::Entity::find()
            .filter(file_has_tags::Column::TagId.eq(tag.id))
            .count(&*connection)
            .await
        {
            Ok(count) => count,
            Err(_) => 0,
        };
        most_used_tags.push(TagUsageCount {
            tag_name: tag.name,
            file_count: count,
        });
    }

    // Sort by usage
    most_used_tags.sort_by(|a, b| b.file_count.cmp(&a.file_count));
    files_by_type.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(DatabaseStats {
        total_files,
        total_tags,
        total_file_types,
        total_file_tag_relationships,
        files_by_type,
        most_used_tags,
    })
}

/// Search files by multiple tag names with AND/OR logic
#[command]
pub async fn search_files_by_tags(
    db_manager: State<'_, Arc<DatabaseManager>>,
    tag_names: Vec<String>,
    require_all_tags: Option<bool>,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<SearchResults<FileInfo>, String> {
    let filters = FileSearchFilters {
        name_pattern: None,
        file_type: None,
        path_pattern: None,
        tags: Some(tag_names),
        require_all_tags,
        created_after: None,
        created_before: None,
        updated_after: None,
        updated_before: None,
    };

    search_files(db_manager, filters, page, per_page).await
}

/// Find duplicate files based on content hash
#[command]
pub async fn find_duplicate_files(
    db_manager: State<'_, Arc<DatabaseManager>>,
) -> Result<Vec<Vec<FileInfo>>, String> {
    let connection = db_manager.get_connection();

    // Get all files
    let all_files = match Files::find().all(&*connection).await {
        Ok(files) => files,
        Err(e) => return Err(format!("Failed to get files: {}", e)),
    };

    // Group by content hash
    let mut hash_groups: std::collections::HashMap<String, Vec<files::Model>> =
        std::collections::HashMap::new();
    for file in all_files {
        hash_groups
            .entry(file.content_hash.clone())
            .or_insert_with(Vec::new)
            .push(file);
    }

    // Filter groups with more than one file (duplicates)
    let mut duplicates = Vec::new();
    for (_, files) in hash_groups {
        if files.len() > 1 {
            duplicates.push(files.into_iter().map(|f| f.into()).collect());
        }
    }

    Ok(duplicates)
}

/// Get files without any tags
#[command]
pub async fn get_untagged_files(
    db_manager: State<'_, Arc<DatabaseManager>>,
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<SearchResults<FileInfo>, String> {
    let connection = db_manager.get_connection();
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(50).min(200);
    let offset = (page - 1) * per_page;

    // Get files that don't have any tags
    let files = match Files::find()
        .filter(
            files::Column::Id.not_in_subquery(
                file_has_tags::Entity::find()
                    .select_only()
                    .column(file_has_tags::Column::FileId)
                    .into_query(),
            ),
        )
        .offset(offset)
        .limit(per_page)
        .all(&*connection)
        .await
    {
        Ok(files) => files,
        Err(e) => return Err(format!("Failed to get untagged files: {}", e)),
    };

    let total_count = match Files::find()
        .filter(
            files::Column::Id.not_in_subquery(
                file_has_tags::Entity::find()
                    .select_only()
                    .column(file_has_tags::Column::FileId)
                    .into_query(),
            ),
        )
        .count(&*connection)
        .await
    {
        Ok(count) => count,
        Err(e) => return Err(format!("Failed to count untagged files: {}", e)),
    };

    let total_pages = (total_count + per_page - 1) / per_page;

    Ok(SearchResults {
        items: files.into_iter().map(|f| f.into()).collect(),
        total_count,
        page,
        per_page,
        total_pages,
    })
}

/// Get recently added files
#[command]
pub async fn get_recent_files(
    db_manager: State<'_, Arc<DatabaseManager>>,
    limit: Option<u64>,
) -> Result<Vec<FileInfo>, String> {
    let connection = db_manager.get_connection();
    let limit = limit.unwrap_or(50).min(200);

    match Files::find()
        .order_by_desc(files::Column::CreatedAt)
        .limit(limit)
        .all(&*connection)
        .await
    {
        Ok(files) => Ok(files.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to get recent files: {}", e)),
    }
}

/// Get recently updated files
#[command]
pub async fn get_recently_updated_files(
    db_manager: State<'_, Arc<DatabaseManager>>,
    limit: Option<u64>,
) -> Result<Vec<FileInfo>, String> {
    let connection = db_manager.get_connection();
    let limit = limit.unwrap_or(50).min(200);

    match Files::find()
        .order_by_desc(files::Column::UpdatedAt)
        .limit(limit)
        .all(&*connection)
        .await
    {
        Ok(files) => Ok(files.into_iter().map(|f| f.into()).collect()),
        Err(e) => Err(format!("Failed to get recently updated files: {}", e)),
    }
}
