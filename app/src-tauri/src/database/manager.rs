use std::sync::Arc;

use sea_orm::DatabaseConnection;

pub struct DatabaseManager {
    connection: Arc<DatabaseConnection>,
}
