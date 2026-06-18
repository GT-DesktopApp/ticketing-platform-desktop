// Central error type. Every layer returns `AppError`; commands convert it to a
// String at the very edge so the frontend gets a clean message.

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("{0} not found")]
    NotFound(&'static str),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("{0}")]
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

/// Commands return Result<T, CommandError> so Tauri serialises a clean message.
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(e: AppError) -> Self {
        CommandError {
            message: e.to_string(),
        }
    }
}

// Allow `?` on AppError inside commands that return Result<T, String>.
impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
