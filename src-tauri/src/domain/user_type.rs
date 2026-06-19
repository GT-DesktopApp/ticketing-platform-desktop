// User type entity (e.g. "admin", "user"). Drives the Add User form's
// "User Type" dropdown and the "Manage User Types" admin list.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserType {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
