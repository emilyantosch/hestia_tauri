use std::sync::Arc;
use std::time::Duration;

use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};

use crate::config::database::{DatabaseSettings, DatabaseType};
use crate::errors::{DbError, DbErrorKind};

#[derive(Debug)]
pub struct DatabaseManager {
    connection: Arc<DatabaseConnection>,
    settings: DatabaseSettings,
}

impl DatabaseManager {
    /// Create a new DatabaseManager with the provided settings
    pub async fn new(settings: DatabaseSettings) -> Result<Self, DbError> {
        if !settings.is_configured() {
            return Err(DbError::new(
                DbErrorKind::ConnectionError,
                "Database settings are not properly configured".to_string(),
            ));
        }

        let connection = Self::create_connection(&settings).await?;

        Ok(Self {
            connection: Arc::new(connection),
            settings,
        })
    }

    /// Create a new DatabaseManager with default SQLite settings
    pub async fn new_sqlite_default() -> Result<Self, DbError> {
        let sqlite_config = crate::config::database::SqliteConfig::default();
        let settings = DatabaseSettings::new_sqlite(sqlite_config);
        Self::new(settings).await
    }

    /// Get a reference to the database connection
    pub fn get_connection(&self) -> Arc<DatabaseConnection> {
        Arc::clone(&self.connection)
    }

    /// Get a reference to the database settings
    pub fn get_settings(&self) -> &DatabaseSettings {
        &self.settings
    }

    /// Test the database connection
    pub async fn test_connection(&self) -> Result<(), DbError> {
        match sea_orm::query::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "SELECT 1".to_string(),
        ) {
            statement => {
                self.connection.execute(statement).await.map_err(|e| {
                    DbError::with_source(
                        DbErrorKind::ConnectionError,
                        "Failed to test database connection".to_string(),
                        e,
                    )
                })?;
            }
        }
        Ok(())
    }

    /// Create a database connection based on the provided settings
    async fn create_connection(settings: &DatabaseSettings) -> Result<DatabaseConnection, DbError> {
        let connection_string = settings.get_connection_string().map_err(|e| {
            DbError::new(
                DbErrorKind::ConfigurationError,
                format!("Failed to build connection string: {}", e),
            )
        })?;

        let mut options = ConnectOptions::new(connection_string);

        match settings.db_type {
            DatabaseType::Sqlite => {
                if let Some(sqlite_config) = &settings.sqlite_config {
                    options
                        .connect_timeout(Duration::from_millis(
                            sqlite_config.connection_timeout_ms as u64,
                        ))
                        .sqlx_logging(false); // Disable sqlx logging for production

                    // SQLite-specific settings are handled via the connection string
                }
            }
            DatabaseType::Postgres => {
                if let Some(postgres_config) = &settings.postgres_config {
                    options
                        .max_connections(postgres_config.max_connections)
                        .min_connections(postgres_config.min_connections)
                        .connect_timeout(Duration::from_millis(
                            postgres_config.connection_timeout_ms as u64,
                        ))
                        .acquire_timeout(Duration::from_millis(
                            postgres_config.acquire_timeout_ms as u64,
                        ))
                        .sqlx_logging(false); // Disable sqlx logging for production

                    if let Some(idle_timeout) = postgres_config.idle_timeout_ms {
                        options.idle_timeout(Duration::from_millis(idle_timeout as u64));
                    }
                }
            }
            DatabaseType::None => {
                return Err(DbError::new(
                    DbErrorKind::ConfigurationError,
                    "No database type configured".to_string(),
                ));
            }
        }

        Database::connect(options).await.map_err(|e| {
            DbError::with_source(
                DbErrorKind::ConnectionError,
                format!("Failed to connect to database: {}", e),
                e,
            )
        })
    }
}
