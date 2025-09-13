use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QuerySelect,
    QueryTrait, Set, TransactionTrait, TryIntoModel,
};

use entity::{prelude::*, thumbnails};

use crate::database::DatabaseManager;
use crate::file_system::thumbnails::{Thumbnail, ThumbnailSize};

/// Statistics about thumbnails in the database
#[derive(Debug, Clone)]
pub struct ThumbnailStats {
    pub total_thumbnails: u64,
    pub thumbnails_by_size: HashMap<String, u64>,
    pub thumbnails_by_type: HashMap<String, u64>,
    pub total_storage_bytes: u64,
}

/// Repository for thumbnail database operations
#[derive(Debug)]
pub struct ThumbnailRepository {
    database_manager: Arc<DatabaseManager>,
}

impl ThumbnailRepository {
    pub fn new(database_manager: Arc<DatabaseManager>) -> Self {
        Self { database_manager }
    }

    /// Create a new thumbnail entry in the database
    pub async fn create_thumbnail(
        &self,
        file_id: i32,
        thumbnail: Thumbnail,
    ) -> Result<thumbnails::Model> {
        let db = self.database_manager.get_connection();

        let active_model = thumbnail.to_active_model(file_id);
        let result = active_model
            .save(db.as_ref())
            .await
            .context("Failed to save thumbnail to database")?;

        let model = result
            .try_into_model()
            .map_err(|_| anyhow::anyhow!("Failed to convert saved thumbnail to model"))?;

        Ok(model)
    }

    /// Get a thumbnail by file ID and size
    pub async fn get_by_file_and_size(
        &self,
        file_id: i32,
        size: ThumbnailSize,
    ) -> Result<Option<Thumbnail>> {
        let db = self.database_manager.get_connection();

        let model = Thumbnails::find()
            .filter(thumbnails::Column::FileId.eq(file_id))
            .filter(thumbnails::Column::Size.eq(size.to_string()))
            .one(db.as_ref())
            .await
            .context("Failed to query thumbnail by file ID and size")?;

        match model {
            Some(m) => Ok(Some(Thumbnail::from_model(m)?)),
            None => Ok(None),
        }
    }

    /// Get all thumbnails for a specific file
    pub async fn get_thumbnails_for_file(&self, file_id: i32) -> Result<Vec<Thumbnail>> {
        let db = self.database_manager.get_connection();

        let models = Thumbnails::find()
            .filter(thumbnails::Column::FileId.eq(file_id))
            .all(db.as_ref())
            .await
            .context("Failed to query thumbnails for file")?;

        let thumbnails: Result<Vec<Thumbnail>, _> =
            models.into_iter().map(Thumbnail::from_model).collect();

        thumbnails.context("Failed to convert thumbnail models")
    }

    /// Delete all thumbnails for a specific file
    pub async fn delete_thumbnails_for_file(&self, file_id: i32) -> Result<u64> {
        let db = self.database_manager.get_connection();

        let delete_result = Thumbnails::delete_many()
            .filter(thumbnails::Column::FileId.eq(file_id))
            .exec(db.as_ref())
            .await
            .context("Failed to delete thumbnails for file")?;

        Ok(delete_result.rows_affected)
    }

    /// Get a thumbnail by its ID
    pub async fn get_thumbnail_by_id(&self, id: i32) -> Result<Option<Thumbnail>> {
        let db = self.database_manager.get_connection();

        let model = Thumbnails::find_by_id(id)
            .one(db.as_ref())
            .await
            .context("Failed to query thumbnail by ID")?;

        match model {
            Some(m) => Ok(Some(Thumbnail::from_model(m)?)),
            None => Ok(None),
        }
    }

    /// Update or insert (upsert) a thumbnail
    pub async fn upsert_thumbnail(
        &self,
        file_id: i32,
        thumbnail: Thumbnail,
    ) -> Result<thumbnails::Model> {
        let db = self.database_manager.get_connection();

        // Check if thumbnail already exists
        let existing = Thumbnails::find()
            .filter(thumbnails::Column::FileId.eq(file_id))
            .filter(thumbnails::Column::Size.eq(thumbnail.size().to_string()))
            .one(db.as_ref())
            .await
            .context("Failed to check for existing thumbnail")?;

        if let Some(existing_model) = existing {
            // Update existing thumbnail
            let mut active_model: thumbnails::ActiveModel = existing_model.into();
            active_model.data = Set(thumbnail.data().to_vec());
            active_model.mime_type = Set(thumbnail.mime_type().to_string());
            active_model.file_size = Set(thumbnail.file_size() as i32);
            active_model.updated_at = Set(chrono::Local::now().naive_local());

            let updated = active_model
                .update(db.as_ref())
                .await
                .context("Failed to update existing thumbnail")?;

            Ok(updated)
        } else {
            // Create new thumbnail
            self.create_thumbnail(file_id, thumbnail).await
        }
    }

    // ===== BATCH PROCESSING METHODS =====

    /// Get file IDs that don't have thumbnails for the specified size
    pub async fn get_files_without_thumbnails(
        &self,
        size: ThumbnailSize,
        limit: Option<u64>,
    ) -> Result<Vec<i32>> {
        let db = self.database_manager.get_connection();

        // Build the query to find files without thumbnails of the specified size
        let mut query = entity::files::Entity::find()
            .filter(
                entity::files::Column::Id.not_in_subquery(
                    Thumbnails::find()
                        .select_only()
                        .column(thumbnails::Column::FileId)
                        .filter(thumbnails::Column::Size.eq(size.to_string()))
                        .into_query(),
                ),
            )
            .select_only()
            .column(entity::files::Column::Id);

        if let Some(limit_value) = limit {
            query = query.limit(limit_value);
        }

        let file_ids: Vec<i32> = query
            .into_tuple()
            .all(db.as_ref())
            .await
            .context("Failed to query files without thumbnails")?;

        Ok(file_ids)
    }

    /// Batch create thumbnails within a transaction
    pub async fn batch_create_thumbnails(
        &self,
        thumbnails_data: Vec<(i32, Thumbnail)>,
    ) -> Result<u64> {
        let db = self.database_manager.get_connection();

        let txn = db
            .begin()
            .await
            .context("Failed to start transaction for batch thumbnail creation")?;

        let mut created_count = 0u64;

        for (file_id, thumbnail) in thumbnails_data {
            let active_model = thumbnail.to_active_model(file_id);

            active_model
                .save(&txn)
                .await
                .context("Failed to save thumbnail in batch operation")?;

            created_count += 1;
        }

        txn.commit()
            .await
            .context("Failed to commit batch thumbnail creation transaction")?;

        Ok(created_count)
    }

    /// Get comprehensive thumbnail statistics
    pub async fn get_thumbnail_stats(&self) -> Result<ThumbnailStats> {
        let db = self.database_manager.get_connection();

        // Get all thumbnails for processing statistics
        let all_thumbnails = Thumbnails::find()
            .all(db.as_ref())
            .await
            .context("Failed to fetch thumbnails for statistics")?;

        let total_thumbnails = all_thumbnails.len() as u64;
        let mut thumbnails_by_size = HashMap::new();
        let mut thumbnails_by_type = HashMap::new();
        let mut total_storage_bytes = 0u64;

        for thumbnail_model in all_thumbnails {
            // Count by size
            *thumbnails_by_size
                .entry(thumbnail_model.size.clone())
                .or_insert(0) += 1;

            // Count by MIME type
            *thumbnails_by_type
                .entry(thumbnail_model.mime_type.clone())
                .or_insert(0) += 1;

            // Sum storage bytes
            total_storage_bytes += thumbnail_model.file_size as u64;
        }

        Ok(ThumbnailStats {
            total_thumbnails,
            thumbnails_by_size,
            thumbnails_by_type,
            total_storage_bytes,
        })
    }

    /// Batch upsert thumbnails with conflict resolution
    pub async fn batch_upsert_thumbnails(
        &self,
        thumbnails_data: Vec<(i32, Thumbnail)>,
    ) -> Result<(u64, u64)> {
        let db = self.database_manager.get_connection();

        let txn = db
            .begin()
            .await
            .context("Failed to start transaction for batch thumbnail upsert")?;

        let mut created_count = 0u64;
        let mut updated_count = 0u64;

        for (file_id, thumbnail) in thumbnails_data {
            // Check if thumbnail exists
            let existing = Thumbnails::find()
                .filter(thumbnails::Column::FileId.eq(file_id))
                .filter(thumbnails::Column::Size.eq(thumbnail.size().to_string()))
                .one(&txn)
                .await
                .context("Failed to check for existing thumbnail in batch upsert")?;

            if let Some(existing_model) = existing {
                // Update existing
                let mut active_model: thumbnails::ActiveModel = existing_model.into();
                active_model.data = Set(thumbnail.data().to_vec());
                active_model.mime_type = Set(thumbnail.mime_type().to_string());
                active_model.file_size = Set(thumbnail.file_size() as i32);
                active_model.updated_at = Set(chrono::Local::now().naive_local());

                active_model
                    .update(&txn)
                    .await
                    .context("Failed to update thumbnail in batch operation")?;

                updated_count += 1;
            } else {
                // Create new
                let active_model = thumbnail.to_active_model(file_id);
                active_model
                    .save(&txn)
                    .await
                    .context("Failed to create thumbnail in batch operation")?;

                created_count += 1;
            }
        }

        txn.commit()
            .await
            .context("Failed to commit batch thumbnail upsert transaction")?;

        Ok((created_count, updated_count))
    }

    /// Delete orphaned thumbnails (thumbnails for files that no longer exist)
    pub async fn delete_orphaned_thumbnails(&self) -> Result<u64> {
        let db = self.database_manager.get_connection();

        let delete_result = Thumbnails::delete_many()
            .filter(
                thumbnails::Column::FileId.not_in_subquery(
                    entity::files::Entity::find()
                        .select_only()
                        .column(entity::files::Column::Id)
                        .into_query(),
                ),
            )
            .exec(db.as_ref())
            .await
            .context("Failed to delete orphaned thumbnails")?;

        Ok(delete_result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseManager;
    use chrono::Local;
    use entity::thumbnails;

    // Helper function to create a test thumbnail
    fn create_test_thumbnail() -> Thumbnail {
        Thumbnail::with_image_data(ThumbnailSize::Small, vec![1, 2, 3, 4, 5])
    }

    #[tokio::test]
    async fn test_thumbnail_repository_creation() {
        let db_manager = Arc::new(DatabaseManager::new_sqlite_default().await.unwrap());
        let repository = ThumbnailRepository::new(db_manager);

        // Just test that the repository can be created
        assert!(matches!(
            repository.database_manager.get_settings().db_type,
            crate::config::database::DatabaseType::Sqlite
        ));
    }

    // NOTE: More comprehensive tests would require database setup/teardown
    // These would be better placed in integration tests
}

