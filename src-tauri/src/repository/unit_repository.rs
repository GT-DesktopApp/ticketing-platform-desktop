// Unit data access. All unit SQL lives here.

use crate::domain::{Page, PageRequest, Unit};
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

const COLUMNS: &str = "id, unit_name, unit_code, is_active, created_at, updated_at";

#[derive(Clone)]
pub struct UnitRepository {
    pool: SqlitePool,
}

impl UnitRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, req: &PageRequest) -> AppResult<Page<Unit>> {
        let push_filters = |qb: &mut QueryBuilder<Sqlite>| {
            let mut first = true;
            if req.active_only {
                qb.push(" WHERE is_active = 1");
                first = false;
            }
            if let Some(search) = &req.search {
                qb.push(if first { " WHERE (" } else { " AND (" });
                qb.push("unit_name LIKE ");
                qb.push_bind(format!("%{search}%"));
                qb.push(" OR unit_code LIKE ");
                qb.push_bind(format!("%{search}%"));
                qb.push(")");
            }
        };

        let mut count_qb: QueryBuilder<Sqlite> = QueryBuilder::new("SELECT COUNT(*) FROM units");
        push_filters(&mut count_qb);
        let total: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;

        let mut data_qb: QueryBuilder<Sqlite> =
            QueryBuilder::new(format!("SELECT {COLUMNS} FROM units"));
        push_filters(&mut data_qb);
        data_qb.push(" ORDER BY id ASC LIMIT ");
        data_qb.push_bind(req.limit());
        data_qb.push(" OFFSET ");
        data_qb.push_bind(req.offset());

        let rows = data_qb.build_query_as::<Unit>().fetch_all(&self.pool).await?;
        Ok(Page::new(rows, req, total))
    }

    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<Unit>> {
        let row = sqlx::query_as::<_, Unit>(&format!(
            "SELECT {COLUMNS} FROM units WHERE id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn exists_code(&self, code: &str, exclude_id: Option<i64>) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM units WHERE unit_code = ? AND id != ?")
                .bind(code)
                .bind(exclude_id.unwrap_or(-1))
                .fetch_one(&self.pool)
                .await?;
        Ok(count > 0)
    }

    pub async fn insert(&self, unit_name: &str, unit_code: &str) -> AppResult<Unit> {
        let row = sqlx::query_as::<_, Unit>(&format!(
            "INSERT INTO units (unit_name, unit_code) VALUES (?, ?) RETURNING {COLUMNS}"
        ))
        .bind(unit_name)
        .bind(unit_code)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update(
        &self,
        id: i64,
        unit_name: Option<&str>,
        unit_code: Option<&str>,
    ) -> AppResult<Unit> {
        let mut qb: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE units SET updated_at = datetime('now')");
        if let Some(n) = unit_name {
            qb.push(", unit_name = ");
            qb.push_bind(n.to_string());
        }
        if let Some(c) = unit_code {
            qb.push(", unit_code = ");
            qb.push_bind(c.to_string());
        }
        qb.push(" WHERE id = ");
        qb.push_bind(id);
        qb.push(&format!(" RETURNING {COLUMNS}"));

        let row = qb
            .build_query_as::<Unit>()
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("unit"))?;
        Ok(row)
    }

    pub async fn set_active(&self, id: i64, is_active: bool) -> AppResult<Unit> {
        let row = sqlx::query_as::<_, Unit>(&format!(
            "UPDATE units SET is_active = ?, updated_at = datetime('now')
             WHERE id = ? RETURNING {COLUMNS}"
        ))
        .bind(is_active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound("unit"))?;
        Ok(row)
    }
}
