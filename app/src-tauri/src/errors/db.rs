pub struct DbError {
    kind: DbErrorKind,
    table: String,
}

enum DbErrorKind {
    RollbackError(String),
    IntegrityConstraintError(String),
    ReferentialConstraintError(String),
}
