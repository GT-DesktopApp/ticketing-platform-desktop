// Unit entity. Backs the Sales Unit / Purchase Unit / Stock Unit dropdowns in
// the Item form, and the Units admin list.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Unit {
    pub id: i64,
    pub unit_name: String,
    pub unit_code: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
