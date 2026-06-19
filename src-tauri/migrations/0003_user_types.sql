-- User types (admin / user / custom roles) managed from the Admin Dashboard's
-- "Manage User Types" screen. Soft delete: "delete" sets is_active = 0.
-- Backs the "User Type" dropdown in the Add User form.

CREATE TABLE IF NOT EXISTS user_types (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    is_active  INTEGER NOT NULL DEFAULT 1,        -- 1 = active, 0 = inactive
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX IF NOT EXISTS ux_user_types_name ON user_types (name);
