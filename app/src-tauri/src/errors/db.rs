use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection could not be established!")]
    ConnectionError,
    #[error("Database configuration is invalid!")]
    ConfigurationError,
    #[error("Database transaction issue occured!")]
    TransactionError,
    #[error("Database query failed!")]
    QueryError,
    #[error("Database insert could not be completed!")]
    InsertError,
    #[error("Database update could not be completed!")]
    UpdateError,
    #[error("Database delete could not be completed!")]
    DeleteError,
    #[error("Database rollback failed!")]
    RollbackError,
    #[error("Database data integrity has been violated!")]
    IntegrityConstraintError,
    #[error("Database foreign key constraint has been violated!")]
    ReferentialConstraintError,
    #[error("Database migration could not be completed!")]
    MigrationError,
    #[error("SeaORM process has failed!")]
    SeaOrmError,
}
