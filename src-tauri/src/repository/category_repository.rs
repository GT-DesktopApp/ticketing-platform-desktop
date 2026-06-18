// Category data access. All category SQL lives here.
// Dynamic search/active filters are built with QueryBuilder so they stay
// parameterised (no string interpolation of user input).

use crate::domain::{Category, Page, PageRequest};
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

const COLUMNS: &str = "id, name, hsn_code, is_active, created_at, updated_at";

#[derive(Clone)]
pub struct CategoryRepository {
    pool: SqlitePool,
}

impl CategoryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Paginated list with optional name/HSN search and active-only filter.
    pub async fn list(&self, req: &PageRequest) -> AppResult<Page<Category>> {
        // Shared WHERE clause, applied identically to the data + count queries.
        let push_filters = |qb: &mut QueryBuilder<Sqlite>| {
            let mut first = true;
            if req.active_only {
                qb.push(" WHERE is_active = 1");
                first = false;
            }
            if let Some(search) = &req.search {
                qb.push(if first { " WHERE (" } else { " AND (" });
                qb.push("name LIKE ");
                qb.push_bind(format!("%{search}%"));
                qb.push(" OR hsn_code LIKE ");
                qb.push_bind(format!("%{search}%"));
                qb.push(")");
            }
        };

        // total
        let mut count_qb: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM categories");
        push_filters(&mut count_qb);
        let total: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;

        // page
        let mut data_qb: QueryBuilder<Sqlite> =
            QueryBuilder::new(format!("SELECT {COLUMNS} FROM categories"));
        push_filters(&mut data_qb);
        data_qb.push(" ORDER BY id ASC LIMIT ");
        data_qb.push_bind(req.limit());
        data_qb.push(" OFFSET ");
        data_qb.push_bind(req.offset());

        let rows = data_qb
            .build_query_as::<Category>()
            .fetch_all(&self.pool)
            .await?;

        Ok(Page::new(rows, req, total))
    }

    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<Category>> {
        let row = sqlx::query_as::<_, Category>(&format!(
            "SELECT {COLUMNS} FROM categories WHERE id = ?"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn exists_name(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM categories WHERE name = ? AND id != ?",
        )
        .bind(name)
        .bind(exclude_id.unwrap_or(-1))
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    pub async fn insert(&self, name: &str, hsn_code: Option<&str>) -> AppResult<Category> {
        let row = sqlx::query_as::<_, Category>(&format!(
            "INSERT INTO categories (name, hsn_code) VALUES (?, ?) RETURNING {COLUMNS}"
        ))
        .bind(name)
        .bind(hsn_code)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        hsn_code: Option<Option<&str>>, // Some(None) clears HSN, None leaves unchanged
    ) -> AppResult<Category> {
        // Build a partial UPDATE so only provided fields change; always bump updated_at.
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new("UPDATE categories SET updated_at = datetime('now')");
        if let Some(name) = name {
            qb.push(", name = ");
            qb.push_bind(name.to_string());
        }
        if let Some(hsn) = hsn_code {
            qb.push(", hsn_code = ");
            qb.push_bind(hsn.map(|s| s.to_string()));
        }
        qb.push(" WHERE id = ");
        qb.push_bind(id);
        qb.push(&format!(" RETURNING {COLUMNS}"));

        let row = qb
            .build_query_as::<Category>()
            .fetch_optional(&self.pool)
            .await?
            .ok_or(AppError::NotFound("category"))?;
        Ok(row)
    }

    pub async fn set_active(&self, id: i64, is_active: bool) -> AppResult<Category> {
        let row = sqlx::query_as::<_, Category>(&format!(
            "UPDATE categories SET is_active = ?, updated_at = datetime('now')
             WHERE id = ? RETURNING {COLUMNS}"
        ))
        .bind(is_active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound("category"))?;
        Ok(row)
    }
}
