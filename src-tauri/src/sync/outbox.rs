// Transactional outbox helper.
//
// `enqueue` inserts a sync_queue row INSIDE the caller's transaction, so the
// business write and its queue entry commit (or roll back) atomically. The
// queue row carries the RECORD uuid as entity_id — the server upserts by that,
// making re-sends idempotent at the record level.

use crate::sync::identity::Identity;
use serde::Serialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

/// Operation kind recorded in the outbox.
pub const OP_INSERT: &str = "insert";
pub const OP_UPDATE: &str = "update";
pub const OP_DELETE: &str = "delete";

/// Enqueue a change within an open transaction.
///
/// * `entity_type` — "ticket" | "category" | "unit" | "user_type"
/// * `entity_uuid` — the record's stable cross-device UUID
/// * `operation`   — OP_INSERT | OP_UPDATE | OP_DELETE
/// * `payload`     — full intended state, serialised to JSON for the server
pub async fn enqueue<T: Serialize>(
    tx: &mut Transaction<'_, Sqlite>,
    identity: &Identity,
    entity_type: &str,
    entity_uuid: &str,
    operation: &str,
    payload: &T,
) -> Result<(), sqlx::Error> {
    let body = serde_json::to_string(payload).map_err(|e| {
        sqlx::Error::Protocol(format!("sync payload serialize failed: {e}"))
    })?;
    // Unique per queued change; stable across retries of THIS row.
    let idempotency_key = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO sync_queue
            (idempotency_key, entity_type, entity_id, operation, payload,
             tenant_id, device_id, status, next_attempt_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', datetime('now'))",
    )
    .bind(&idempotency_key)
    .bind(entity_type)
    .bind(entity_uuid)
    .bind(operation)
    .bind(&body)
    .bind(&identity.tenant_id)
    .bind(&identity.device_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}
