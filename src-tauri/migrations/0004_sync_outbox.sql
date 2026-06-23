-- Durable sync outbox + per-record cross-device identity.
--
-- DESIGN
--  * Every syncable record gets a stable `uuid` (generated locally). The local
--    integer `id` stays for joins/UI; the `uuid` is what the SERVER upserts by,
--    so re-sending a stalled item never duplicates (idempotent by record).
--  * Records carry `tenant_id` + `device_id` so the server can scope writes to a
--    tenant and know which device produced them.
--  * `sync_queue` is the transactional outbox: every business write enqueues a
--    row in the SAME transaction. The queue — not any timestamp diff — is the
--    source of truth for what is unsent.

-- ---------------------------------------------------------------------------
-- Per-record identity columns on existing syncable tables.
-- Added nullable (SQLite can't add a NOT NULL column without a constant default
-- to an existing table); the app backfills + always writes them going forward.
-- A UNIQUE index on uuid enforces idempotent identity once populated.
-- ---------------------------------------------------------------------------
ALTER TABLE tickets    ADD COLUMN uuid       TEXT;
ALTER TABLE tickets    ADD COLUMN tenant_id  TEXT;
ALTER TABLE tickets    ADD COLUMN device_id  TEXT;

ALTER TABLE categories ADD COLUMN uuid       TEXT;
ALTER TABLE categories ADD COLUMN tenant_id  TEXT;
ALTER TABLE categories ADD COLUMN device_id  TEXT;

ALTER TABLE units      ADD COLUMN uuid       TEXT;
ALTER TABLE units      ADD COLUMN tenant_id  TEXT;
ALTER TABLE units      ADD COLUMN device_id  TEXT;

ALTER TABLE user_types ADD COLUMN uuid       TEXT;
ALTER TABLE user_types ADD COLUMN tenant_id  TEXT;
ALTER TABLE user_types ADD COLUMN device_id  TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS ux_tickets_uuid    ON tickets    (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS ux_categories_uuid ON categories (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS ux_units_uuid      ON units      (uuid);
CREATE UNIQUE INDEX IF NOT EXISTS ux_user_types_uuid ON user_types (uuid);

-- ---------------------------------------------------------------------------
-- Local key/value store (device_id persisted once per install, etc.).
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS app_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- ---------------------------------------------------------------------------
-- Transactional outbox.
-- ---------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS sync_queue (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,  -- FIFO sequence (creation order)
    idempotency_key TEXT NOT NULL UNIQUE,               -- per-attempt-group key; stable across retries of THIS change
    entity_type     TEXT NOT NULL,                      -- 'ticket' | 'category' | 'unit' | 'user_type'
    entity_id       TEXT NOT NULL,                      -- the RECORD uuid (server upserts by this)
    operation       TEXT NOT NULL,                      -- 'insert' | 'update' | 'delete'
    payload         TEXT NOT NULL,                      -- JSON body sent to the server
    tenant_id       TEXT NOT NULL,
    device_id       TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',    -- pending | in_flight | done | failed
    attempts        INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TEXT NOT NULL DEFAULT (datetime('now')), -- earliest time this row may be sent (backoff gate)
    last_error      TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    last_attempt_at TEXT
);

-- Drain query is: pending AND next_attempt_at <= now, oldest id first.
CREATE INDEX IF NOT EXISTS idx_sync_queue_drain
    ON sync_queue (status, next_attempt_at, id);
