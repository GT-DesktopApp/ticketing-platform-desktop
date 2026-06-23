// Application configuration, resolved once at startup from environment
// variables with safe defaults. Centralising this keeps env access out of the
// rest of the codebase.

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Central cloud server base URL (the sync target).
    pub cloud_api_url: String,
    /// Log filter, e.g. "info" or "debug".
    pub log_level: String,
    /// SQLite filename stored inside the OS app-data directory.
    pub db_filename: String,
    /// Tenant (project-owner) this install belongs to. Until real auth/login
    /// exists, this comes from config; later it comes from the logged-in account.
    pub tenant_id: String,
    /// Per-device bearer token the server uses to authenticate this device.
    /// Stubbed from config for now; a real token is issued at device registration.
    pub device_token: String,
    /// How often the drain worker wakes to look for pending rows (seconds).
    pub sync_interval_secs: u64,
    /// Move a queue row to `failed` after this many failed attempts.
    pub sync_max_attempts: i64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        AppConfig {
            cloud_api_url: env_or("CLOUD_API_URL", "http://localhost:8080"),
            log_level: env_or("APP_LOG_LEVEL", "info"),
            db_filename: env_or("APP_DB_FILENAME", "ticketing.db"),
            tenant_id: env_or("TENANT_ID", "tenant-default"),
            device_token: env_or("DEVICE_TOKEN", "dev-token"),
            sync_interval_secs: env_or("SYNC_INTERVAL_SECS", "120")
                .parse()
                .unwrap_or(120),
            sync_max_attempts: env_or("SYNC_MAX_ATTEMPTS", "12").parse().unwrap_or(12),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
