use std::error::Error;

use crate::errors::FileError;

#[derive(Debug, thiserror::Error)]
pub struct DbError {
    pub kind: DbErrorKind,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

#[derive(Debug)]
pub enum DbErrorKind {
    ConnectionError,
    ConfigurationError,
    TransactionError,
    QueryError,
    InsertError,
    UpdateError,
    DeleteError,
    RollbackError,
    IntegrityConstraintError,
    ReferentialConstraintError,
    MigrationError,
    SeaOrmError,
}

impl DbError {
    pub fn new(kind: DbErrorKind, message: String) -> Self {
        Self {
            kind,
            message,
            source: None,
        }
    }

    pub fn with_source<E>(kind: DbErrorKind, message: String, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            kind,
            message,
            source: Some(Box::new(source)),
        }
    }
}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl From<sea_orm::DbErr> for DbError {
    fn from(value: sea_orm::DbErr) -> Self {
        DbError::with_source(
            DbErrorKind::SeaOrmError,
            format!("SeaORM encountered an error due to {value:#?}!"),
            value,
        )
    }
}

impl From<FileError> for DbError {
    fn from(value: FileError) -> Self {
        DbError::with_source(
            DbErrorKind::QueryError,
            "A file system error occured!".to_string(),
            value,
        )
    }
}
