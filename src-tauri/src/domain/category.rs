// Category entity. Backs the Item form's "Category" dropdown and the
// Categories admin list (with the active/inactive toggle).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub hsn_code: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
