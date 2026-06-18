// Ticket data access. All ticket SQL lives here.

use crate::domain::Ticket;
use crate::error::{AppError, AppResult};
use sqlx::SqlitePool;

const COLUMNS: &str =
    "id, ticket_code, invoice_id, valid_date, status, used_at, created_at";

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

    pub async fn insert(
        &self,
        code: &str,
        invoice_id: Option<i64>,
        valid_date: &str,
    ) -> AppResult<Ticket> {
        let row = sqlx::query_as::<_, Ticket>(&format!(
            "INSERT INTO tickets (ticket_code, invoice_id, valid_date, status)
             VALUES (?, ?, ?, 'active')
             RETURNING {COLUMNS}"
        ))
        .bind(code)
        .bind(invoice_id)
        .bind(valid_date)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update(
        &self,
        id: i64,
        status: Option<&str>,
        valid_date: Option<&str>,
    ) -> AppResult<Ticket> {
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
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound("ticket"))?;
        Ok(row)
    }

    /// Mark a ticket used (sets used_at to now). Returns the updated row.
    pub async fn mark_used(&self, id: i64) -> AppResult<Ticket> {
        let row = sqlx::query_as::<_, Ticket>(&format!(
            "UPDATE tickets SET status = 'used', used_at = datetime('now')
             WHERE id = ?
             RETURNING {COLUMNS}"
        ))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn delete(&self, id: i64) -> AppResult<()> {
        let res = sqlx::query("DELETE FROM tickets WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(AppError::NotFound("ticket"));
        }
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
