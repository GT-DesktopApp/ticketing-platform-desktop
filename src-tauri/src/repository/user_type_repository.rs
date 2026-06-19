// User type data access. All user_type SQL lives here.

use crate::domain::{Page, PageRequest, UserType};
use crate::error::{AppError, AppResult};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

const COLUMNS: &str = "id, name, is_active, created_at, updated_at";

#[derive(Clone)]
pub struct UserTypeRepository {
    pool: SqlitePool,
}

impl UserTypeRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(&self, req: &PageRequest) -> AppResult<Page<UserType>> {
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
                qb.push(")");
            }
        };

        let mut count_qb: QueryBuilder<Sqlite> =
            QueryBuilder::new("SELECT COUNT(*) FROM user_types");
        push_filters(&mut count_qb);
        let total: i64 = count_qb.build_query_scalar().fetch_one(&self.pool).await?;

        let mut data_qb: QueryBuilder<Sqlite> =
            QueryBuilder::new(format!("SELECT {COLUMNS} FROM user_types"));
        push_filters(&mut data_qb);
        data_qb.push(" ORDER BY id ASC LIMIT ");
        data_qb.push_bind(req.limit());
        data_qb.push(" OFFSET ");
        data_qb.push_bind(req.offset());

        let rows = data_qb
            .build_query_as::<UserType>()
            .fetch_all(&self.pool)
            .await?;
        Ok(Page::new(rows, req, total))
    }

    pub async fn exists_name(&self, name: &str, exclude_id: Option<i64>) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM user_types WHERE name = ? AND id != ?")
                .bind(name)
                .bind(exclude_id.unwrap_or(-1))
                .fetch_one(&self.pool)
                .await?;
        Ok(count > 0)
    }

    pub async fn insert(&self, name: &str) -> AppResult<UserType> {
        let row = sqlx::query_as::<_, UserType>(&format!(
            "INSERT INTO user_types (name) VALUES (?) RETURNING {COLUMNS}"
        ))
        .bind(name)
        .fetch_one(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn update(&self, id: i64, name: &str) -> AppResult<UserType> {
        let row = sqlx::query_as::<_, UserType>(&format!(
            "UPDATE user_types SET name = ?, updated_at = datetime('now')
             WHERE id = ? RETURNING {COLUMNS}"
        ))
        .bind(name)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound("user type"))?;
        Ok(row)
    }

    pub async fn set_active(&self, id: i64, is_active: bool) -> AppResult<UserType> {
        let row = sqlx::query_as::<_, UserType>(&format!(
            "UPDATE user_types SET is_active = ?, updated_at = datetime('now')
             WHERE id = ? RETURNING {COLUMNS}"
        ))
        .bind(is_active)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound("user type"))?;
        Ok(row)
    }
}
