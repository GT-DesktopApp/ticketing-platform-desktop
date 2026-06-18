// Database access layer: resolves the on-disk location, opens a pooled
// connection with WAL enabled, and runs migrations. No business logic here.

use crate::config::AppConfig;
use crate::error::AppResult;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::str::FromStr;
use tauri::{AppHandle, Manager};

/// Open (creating if absent) the local SQLite DB in the OS app-data directory,
/// enable WAL, and run migrations. Returns a ready connection pool.
pub async fn init(app: &AppHandle, config: &AppConfig) -> AppResult<SqlitePool> {
    let path = resolve_db_path(app, &config.db_filename)?;
    tracing::info!(db = %path.display(), "opening local database");

    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .map_err(|e| crate::error::AppError::Config(format!("bad sqlite url: {e}")))?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // WAL: concurrent reads don't block writes.
    sqlx::query("PRAGMA journal_mode=WAL;").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys=ON;").execute(&pool).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    Ok(pool)
}

/// app-data-dir/<filename>, ensuring the directory exists.
fn resolve_db_path(app: &AppHandle, filename: &str) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| crate::error::AppError::Config(format!("cannot resolve app data dir: {e}")))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| crate::error::AppError::Config(format!("cannot create app data dir: {e}")))?;
    Ok(dir.join(filename))
}
