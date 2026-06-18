// Application configuration, resolved once at startup from environment
// variables with safe defaults. Centralising this keeps env access out of the
// rest of the codebase.

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Central cloud Ticket API base URL (used by the future sync layer).
    pub cloud_api_url: String,
    /// Log filter, e.g. "info" or "debug".
    pub log_level: String,
    /// SQLite filename stored inside the OS app-data directory.
    pub db_filename: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        AppConfig {
            cloud_api_url: env_or("CLOUD_API_URL", "http://localhost:8080"),
            log_level: env_or("APP_LOG_LEVEL", "info"),
            db_filename: env_or("APP_DB_FILENAME", "ticketing.db"),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
