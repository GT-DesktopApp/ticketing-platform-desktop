// Device + tenant identity for sync.
//
//  * device_id  — generated ONCE per install and persisted in app_meta. Stable
//                 across restarts; identifies this machine to the server.
//  * tenant_id  — which project-owner this install belongs to. From config for
//                 now (later: the logged-in account).
//
// Both are stamped onto every record and every queue row so the server can
// scope writes to a tenant and attribute them to a device.

use crate::config::AppConfig;
use crate::error::AppResult;
use sqlx::SqlitePool;
use uuid::Uuid;

const DEVICE_ID_KEY: &str = "device_id";

#[derive(Debug, Clone)]
pub struct Identity {
    pub device_id: String,
    pub tenant_id: String,
    pub device_token: String,
}

/// Load the persisted device_id, generating + storing one on first run.
/// Idempotent: subsequent calls return the same id.
pub async fn resolve(pool: &SqlitePool, config: &AppConfig) -> AppResult<Identity> {
    let existing: Option<String> =
        sqlx::query_scalar("SELECT value FROM app_meta WHERE key = ?")
            .bind(DEVICE_ID_KEY)
            .fetch_optional(pool)
            .await?;

    let device_id = match existing {
        Some(id) => id,
        None => {
            let id = Uuid::new_v4().to_string();
            // INSERT OR IGNORE guards the race where two callers init at once.
            sqlx::query("INSERT OR IGNORE INTO app_meta (key, value) VALUES (?, ?)")
                .bind(DEVICE_ID_KEY)
                .bind(&id)
                .execute(pool)
                .await?;
            // Re-read in case another writer won the race.
            sqlx::query_scalar("SELECT value FROM app_meta WHERE key = ?")
                .bind(DEVICE_ID_KEY)
                .fetch_one(pool)
                .await?
        }
    };

    Ok(Identity {
        device_id,
        tenant_id: config.tenant_id.clone(),
        device_token: config.device_token.clone(),
    })
}
