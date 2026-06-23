// Drain worker: the background task that pushes the outbox to the server.
//
// Correctness contract (matches the target design):
//  * Selects pending rows whose next_attempt_at <= now, oldest id first (FIFO,
//    so an invoice precedes its tickets).
//  * Marks a row in_flight, POSTs it with the record UUID + device auth.
//  * Marks done ONLY on a 2xx from the server (never before).
//  * On failure: attempts+1, compute next_attempt_at = now + backoff(attempts)
//    with jitter (stored in the ROW — we do NOT sleep the whole worker, so one
//    stuck row can't block the queue). After max_attempts -> 'failed'.
//  * Resets in_flight -> pending on startup (crash recovery).
//  * Catches every error so the loop never dies; runs off the UI thread.

use crate::config::AppConfig;
use crate::sync::client::{SyncClient, SyncOutcome};
use crate::sync::identity::Identity;
use rand::Rng;
use sqlx::SqlitePool;
use std::time::Duration;

/// One claimable outbox row.
#[derive(Debug, sqlx::FromRow)]
struct QueueRow {
    id: i64,
    idempotency_key: String,
    entity_type: String,
    entity_id: String,
    operation: String,
    payload: String,
    tenant_id: String,
    device_id: String,
    attempts: i64,
}

/// Spawn the drain loop. Returns immediately; runs for the app's lifetime.
pub fn spawn(pool: SqlitePool, identity: Identity, config: AppConfig) {
    tauri::async_runtime::spawn(async move {
        // Crash recovery: any row left in_flight by a previous run is retryable.
        if let Err(e) = reset_in_flight(&pool).await {
            tracing::error!("sync: reset in_flight failed: {e}");
        }

        let client = SyncClient::new(&config.cloud_api_url, &identity.device_token);
        let idle = Duration::from_secs(config.sync_interval_secs);

        loop {
            // Drain everything currently due, one row at a time, until none left.
            loop {
                match drain_one(&pool, &client, &config).await {
                    Ok(true) => continue,        // handled a row; look for the next
                    Ok(false) => break,          // nothing due right now
                    Err(e) => {
                        // DB-level error — log and stop this burst; try again next tick.
                        tracing::error!("sync: drain error: {e}");
                        break;
                    }
                }
            }
            // Wake on an interval; the queue is also drained explicitly on app
            // events (see trigger()).
            tokio::time::sleep(idle).await;
        }
    });
}

/// Reset crashed in-flight rows so they are retried.
pub async fn reset_in_flight(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE sync_queue SET status = 'pending'
         WHERE status = 'in_flight'",
    )
    .execute(pool)
    .await?;
    Ok(res.rows_affected())
}

/// Claim and process at most one due row. Ok(true) = handled one, Ok(false) = none due.
async fn drain_one(
    pool: &SqlitePool,
    client: &SyncClient,
    config: &AppConfig,
) -> Result<bool, sqlx::Error> {
    // Claim the oldest due row atomically (tx prevents double-claim).
    let mut tx = pool.begin().await?;

    let row = sqlx::query_as::<_, QueueRow>(
        "SELECT id, idempotency_key, entity_type, entity_id, operation, payload,
                tenant_id, device_id, attempts
         FROM sync_queue
         WHERE status = 'pending' AND next_attempt_at <= datetime('now')
         ORDER BY id ASC
         LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(row) = row else {
        tx.commit().await?;
        return Ok(false);
    };

    sqlx::query(
        "UPDATE sync_queue SET status = 'in_flight', last_attempt_at = datetime('now')
         WHERE id = ?",
    )
    .bind(row.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    // Network call OUTSIDE any DB transaction (it can be slow / fail).
    let outcome = client.send(&row.into_request()).await;

    match outcome {
        // Server confirmed (2xx, including idempotent "already have it").
        SyncOutcome::Confirmed => {
            sqlx::query("UPDATE sync_queue SET status = 'done', last_error = NULL WHERE id = ?")
                .bind(row.id)
                .execute(pool)
                .await?;
        }
        // Any failure: schedule a future retry via next_attempt_at (no worker sleep).
        SyncOutcome::Failed(err) => {
            let attempts = row.attempts + 1;
            if attempts >= config.sync_max_attempts {
                sqlx::query(
                    "UPDATE sync_queue SET status = 'failed', attempts = ?, last_error = ?
                     WHERE id = ?",
                )
                .bind(attempts)
                .bind(&err)
                .bind(row.id)
                .execute(pool)
                .await?;
                tracing::warn!("sync: row {} moved to failed after {attempts} attempts: {err}", row.id);
            } else {
                let delay = backoff_secs(attempts);
                sqlx::query(
                    "UPDATE sync_queue
                     SET status = 'pending', attempts = ?, last_error = ?,
                         next_attempt_at = datetime('now', ?)
                     WHERE id = ?",
                )
                .bind(attempts)
                .bind(&err)
                .bind(format!("+{delay} seconds"))
                .bind(row.id)
                .execute(pool)
                .await?;
                tracing::debug!(
                    "sync: row {} attempt {attempts} failed, retry in {delay}s: {err}",
                    row.id
                );
            }
        }
    }

    Ok(true)
}

/// Exponential backoff with full jitter: base 2^attempts capped at 300s, then a
/// random amount in [0, base] is chosen. Jitter spreads retries across devices
/// so a recovering server isn't hit by a synchronized thundering herd.
fn backoff_secs(attempts: i64) -> i64 {
    let exp = attempts.clamp(1, 8) as u32;
    let base = (2_i64.saturating_pow(exp)).min(300);
    let jitter = rand::thread_rng().gen_range(0..=base.max(1));
    // At least 1s so next_attempt_at always moves forward.
    (base / 2 + jitter / 2).max(1)
}

impl QueueRow {
    fn into_request(&self) -> crate::sync::client::SyncRequest {
        crate::sync::client::SyncRequest {
            idempotency_key: self.idempotency_key.clone(),
            entity_type: self.entity_type.clone(),
            entity_uuid: self.entity_id.clone(),
            operation: self.operation.clone(),
            payload: self.payload.clone(),
            tenant_id: self.tenant_id.clone(),
            device_id: self.device_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_increases_and_is_capped() {
        // Monotonic-ish growth and never exceeds the 300s cap region.
        let a1 = backoff_secs(1);
        let a8 = backoff_secs(8);
        assert!(a1 >= 1);
        assert!(a8 <= 300);
        assert!(a8 >= a1, "later attempts should back off at least as long");
    }

    #[test]
    fn backoff_never_zero() {
        for attempts in 1..=12 {
            assert!(backoff_secs(attempts) >= 1, "delay must always advance the clock");
        }
    }
}
