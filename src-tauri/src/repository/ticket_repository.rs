// Ticket data access. All ticket SQL lives here.
//
// Mutations (insert/update/mark_used/delete) are TRANSACTIONAL OUTBOX writes:
// the business row and its sync_queue row commit in one transaction, so a change
// is never persisted without also being queued for the server (and vice versa).

use crate::domain::Ticket;
use crate::error::{AppError, AppResult};
use crate::sync::identity::Identity;
use crate::sync::outbox::{self, OP_DELETE, OP_INSERT, OP_UPDATE};
use sqlx::SqlitePool;
use uuid::Uuid;

const COLUMNS: &str =
    "id, uuid, ticket_code, invoice_id, valid_date, status, used_at, created_at";

#[derive(Clone)]
pub struct TicketRepository {
    pool: SqlitePool,
}

impl TicketRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self) -> AppResult<Vec<Ticket>> {
        let rows = sqlx::query_as::<_, Ticket>(&format!(
            "SELECT {COLUMNS} FROM tickets ORDER BY created_at DESC"
        ))
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<Ticket>> {
        let row = sqlx::query_as::<_, Ticket>(&format!(
            "SELECT {COLUMNS} FROM tickets WHERE id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn find_by_code(&self, code: &str) -> AppResult<Option<Ticket>> {
        let row = sqlx::query_as::<_, Ticket>(&format!(
            "SELECT {COLUMNS} FROM tickets WHERE ticket_code = ?"
        ))
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// Insert a ticket and enqueue its sync row in ONE transaction.
    pub async fn insert(
        &self,
        identity: &Identity,
        code: &str,
        invoice_id: Option<i64>,
        valid_date: &str,
    ) -> AppResult<Ticket> {
        let uuid = Uuid::new_v4().to_string();
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query_as::<_, Ticket>(&format!(
            "INSERT INTO tickets
                (uuid, ticket_code, invoice_id, valid_date, status, tenant_id, device_id)
             VALUES (?, ?, ?, ?, 'active', ?, ?)
             RETURNING {COLUMNS}"
        ))
        .bind(&uuid)
        .bind(code)
        .bind(invoice_id)
        .bind(valid_date)
        .bind(&identity.tenant_id)
        .bind(&identity.device_id)
        .fetch_one(&mut *tx)
        .await?;

        outbox::enqueue(&mut tx, identity, "ticket", &uuid, OP_INSERT, &row).await?;
        tx.commit().await?;
        Ok(row)
    }

    /// Update a ticket and enqueue the change in ONE transaction.
    pub async fn update(
        &self,
        identity: &Identity,
        id: i64,
        status: Option<&str>,
        valid_date: Option<&str>,
    ) -> AppResult<Ticket> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query_as::<_, Ticket>(&format!(
            "UPDATE tickets
             SET status     = COALESCE(?, status),
                 valid_date = COALESCE(?, valid_date),
                 used_at    = CASE WHEN ? = 'used' THEN datetime('now') ELSE used_at END
             WHERE id = ?
             RETURNING {COLUMNS}"
        ))
        .bind(status)
        .bind(valid_date)
        .bind(status)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or(AppError::NotFound("ticket"))?;

        let uuid = row.uuid.clone().unwrap_or_default();
        outbox::enqueue(&mut tx, identity, "ticket", &uuid, OP_UPDATE, &row).await?;
        tx.commit().await?;
        Ok(row)
    }

    /// Mark a ticket used and enqueue the change in ONE transaction.
    pub async fn mark_used(&self, identity: &Identity, id: i64) -> AppResult<Ticket> {
        let mut tx = self.pool.begin().await?;

        let row = sqlx::query_as::<_, Ticket>(&format!(
            "UPDATE tickets SET status = 'used', used_at = datetime('now')
             WHERE id = ?
             RETURNING {COLUMNS}"
        ))
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;

        let uuid = row.uuid.clone().unwrap_or_default();
        outbox::enqueue(&mut tx, identity, "ticket", &uuid, OP_UPDATE, &row).await?;
        tx.commit().await?;
        Ok(row)
    }

    /// Delete a ticket and enqueue a delete in ONE transaction.
    pub async fn delete(&self, identity: &Identity, id: i64) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;

        // Capture the uuid before deleting so the server knows what to remove.
        let uuid: Option<String> =
            sqlx::query_scalar("SELECT uuid FROM tickets WHERE id = ?")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .flatten();

        let res = sqlx::query("DELETE FROM tickets WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("ticket"));
        }

        if let Some(uuid) = uuid {
            let payload = serde_json::json!({ "uuid": uuid });
            outbox::enqueue(&mut tx, identity, "ticket", &uuid, OP_DELETE, &payload).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Local date "YYYY-MM-DD" from the DB, for validity checks.
    pub async fn today(&self) -> AppResult<String> {
        let today: String = sqlx::query_scalar("SELECT date('now','localtime')")
            .fetch_one(&self.pool)
            .await?;
        Ok(today)
    }
}
