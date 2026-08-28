use sea_orm::{DbErr, SqlErr};
use thiserror::Error;

pub type OrmResult<T> = Result<T, OrmError>;

#[derive(Debug, Error)]
pub enum OrmError {
    #[error("record not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("unique constraint violation: {0}")]
    UniqueViolation(String),

    #[error("foreign key violation: {0}")]
    ForeignKeyViolation(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("hook cancelled operation: {0}")]
    HookCancelled(String),
}

impl OrmError {
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

pub(crate) fn map_db_err(err: DbErr) -> OrmError {
    match err.sql_err() {
        Some(SqlErr::UniqueConstraintViolation(m)) => OrmError::UniqueViolation(m),
        Some(SqlErr::ForeignKeyConstraintViolation(m)) => OrmError::ForeignKeyViolation(m),
        Some(_) | None => OrmError::Database(err.to_string()),
    }
}

pub fn cancelled_by_hook(op: &str) -> OrmError {
    OrmError::HookCancelled(format! {"{op} cancelled by hook"})
}
