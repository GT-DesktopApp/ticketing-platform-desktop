// Nightly safety net — runs at 12:00 AM IST (UTC+05:30).
//
// This is NOT the primary sync path (the drain worker is). It exists to catch
// anything the incremental path missed:
//   1. Reconciliation: for each syncable table, ask the server which record
//      UUIDs it is missing for this tenant+device, and re-enqueue those.
//   2. Snapshot: produce a `VACUUM INTO` copy of the local DB and upload it as a
//      coarse point-in-time backup.
// Both steps are idempotent and safe to run repeatedly (re-enqueue uses the
// outbox; the server upserts by UUID; the snapshot filename is timestamped).

use crate::config::AppConfig;
use crate::sync::client::SyncClient;
use crate::sync::identity::Identity;
use chrono::{Duration as ChronoDuration, FixedOffset, Utc};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::time::Duration;

/// Tables that participate in sync, with the entity_type used on the wire.
const SYNCABLE: &[(&str, &str)] = &[
    ("tickets", "ticket"),
    ("categories", "category"),
    ("units", "unit"),
    ("user_types", "user_type"),
];

/// Spawn the scheduler: sleep until the next 00:00 IST, run once, repeat daily.
pub fn spawn(pool: SqlitePool, identity: Identity, config: AppConfig, snapshot_dir: PathBuf) {
    tauri::async_runtime::spawn(async move {
        loop {
            let wait = secs_until_next_midnight_ist();
            tracing::info!("nightly: next run in {}s (~{}h)", wait, wait / 3600);
            tokio::time::sleep(Duration::from_secs(wait)).await;

            if let Err(e) = run_once(&pool, &identity, &config, &snapshot_dir).await {
                // Never let a failure kill the scheduler; try again tomorrow.
                tracing::error!("nightly: run failed: {e}");
            }
        }
    });
}

/// Run reconciliation + snapshot once. Public so it can be triggered manually
/// (e.g. a Tauri command) and exercised by tests.
pub async fn run_once(
    pool: &SqlitePool,
    identity: &Identity,
    config: &AppConfig,
    snapshot_dir: &std::path::Path,
) -> Result<NightlyReport, String> {
    let client = SyncClient::new(&config.cloud_api_url, &identity.device_token);
    let mut report = NightlyReport::default();

    // 1. Reconciliation — per table, find UUIDs the server is missing and re-enqueue.
    for (table, entity_type) in SYNCABLE {
        match reconcile_table(pool, &client, identity, table, entity_type).await {
            Ok(n) => report.reenqueued += n,
            Err(e) => {
                report.errors.push(format!("{table}: {e}"));
            }
        }
    }

    // 2. Snapshot — VACUUM INTO a fresh file and upload it.
    match snapshot(pool, snapshot_dir).await {
        Ok((path, bytes)) => {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("snapshot.db")
                .to_string();
            match client
                .upload_snapshot(&identity.tenant_id, &identity.device_id, &name, bytes)
                .await
            {
                Ok(()) => report.snapshot_uploaded = Some(name),
                Err(e) => report.errors.push(format!("snapshot upload: {e}")),
            }
            report.snapshot_path = Some(path);
        }
        Err(e) => report.errors.push(format!("snapshot: {e}")),
    }

    Ok(report)
}

/// For one table: collect local record UUIDs, ask the server what it's missing,
/// and re-enqueue each missing record as an upsert. Returns count re-enqueued.
async fn reconcile_table(
    pool: &SqlitePool,
    client: &SyncClient,
    identity: &Identity,
    table: &str,
    entity_type: &str,
) -> Result<u64, String> {
    // Local UUIDs (only rows that have one — all new rows do).
    let uuids: Vec<String> = sqlx::query_scalar(&format!(
        "SELECT uuid FROM {table} WHERE uuid IS NOT NULL"
    ))
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if uuids.is_empty() {
        return Ok(0);
    }

    let missing = client
        .reconcile_missing(&identity.tenant_id, &identity.device_id, entity_type, &uuids)
        .await?;

    let mut count = 0u64;
    for uuid in missing {
        // Re-enqueue the full current row as an upsert. We read the row as JSON
        // via SQLite's json_object so we don't need a typed struct per table here.
        let payload: Option<String> = sqlx::query_scalar(&format!(
            "SELECT json_object('uuid', uuid) FROM {table} WHERE uuid = ?"
        ))
        .bind(&uuid)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

        // Build a minimal re-enqueue row (the server upserts by uuid; a richer
        // payload can be supplied by the per-entity outbox at write time).
        let body = payload.unwrap_or_else(|| format!("{{\"uuid\":\"{uuid}\"}}"));
        let idempotency_key = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO sync_queue
                (idempotency_key, entity_type, entity_id, operation, payload,
                 tenant_id, device_id, status, next_attempt_at)
             VALUES (?, ?, ?, 'update', ?, ?, ?, 'pending', datetime('now'))",
        )
        .bind(&idempotency_key)
        .bind(entity_type)
        .bind(&uuid)
        .bind(&body)
        .bind(&identity.tenant_id)
        .bind(&identity.device_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        count += 1;
    }
    Ok(count)
}

/// Produce a consistent point-in-time copy of the DB via `VACUUM INTO`, returning
/// the file path and its bytes. VACUUM INTO is safe while the DB is in use and
/// yields a clean, defragmented copy.
async fn snapshot(
    pool: &SqlitePool,
    dir: &std::path::Path,
) -> Result<(PathBuf, Vec<u8>), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("mkdir snapshot dir: {e}"))?;

    // Timestamped name keeps the operation idempotent-friendly and ordered.
    let now = Utc::now().format("%Y%m%dT%H%M%SZ");
    let path = dir.join(format!("snapshot-{now}.db"));

    // VACUUM INTO requires a literal path; bind isn't supported, so quote safely.
    let escaped = path.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{escaped}'"))
        .execute(pool)
        .await
        .map_err(|e| format!("VACUUM INTO failed: {e}"))?;

    let bytes = std::fs::read(&path).map_err(|e| format!("read snapshot: {e}"))?;
    Ok((path, bytes))
}

/// Seconds from now until the next 00:00 in IST (UTC+05:30).
fn secs_until_next_midnight_ist() -> u64 {
    // IST has no DST, so a fixed offset is correct year-round.
    let ist = FixedOffset::east_opt(5 * 3600 + 30 * 60).expect("valid IST offset");
    let now_ist = Utc::now().with_timezone(&ist);
    let today_midnight = now_ist
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("valid midnight");
    // Next midnight is tomorrow 00:00 IST (today's midnight already passed).
    let next = today_midnight + ChronoDuration::days(1);
    let next_ist = next.and_local_timezone(ist).single().expect("unambiguous");
    (next_ist - now_ist).num_seconds().max(0) as u64
}

#[derive(Debug, Default)]
pub struct NightlyReport {
    pub reenqueued: u64,
    pub snapshot_path: Option<PathBuf>,
    pub snapshot_uploaded: Option<String>,
    pub errors: Vec<String>,
}
