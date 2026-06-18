-- Catalog entities backing the Item module's dynamic dropdowns.
-- Both use soft delete: "delete" sets is_active = 0; an explicit toggle flips it.

CREATE TABLE IF NOT EXISTS categories (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    hsn_code   TEXT,
    is_active  INTEGER NOT NULL DEFAULT 1,        -- 1 = active, 0 = inactive
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
-- Category name is the user-facing identifier; keep it unique.
CREATE UNIQUE INDEX IF NOT EXISTS ux_categories_name ON categories (name);

CREATE TABLE IF NOT EXISTS units (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    unit_name  TEXT NOT NULL,
    unit_code  TEXT NOT NULL,
    is_active  INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
-- Unit code is the short identifier shown in the units table; keep it unique.
CREATE UNIQUE INDEX IF NOT EXISTS ux_units_code ON units (unit_code);
