-- Complete initialization schema (single file, contains final form of all tables)

-- Global configuration table
CREATE TABLE IF NOT EXISTS settings (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 2FA secret table
CREATE TABLE IF NOT EXISTS two_factor_secrets (
    username   TEXT PRIMARY KEY NOT NULL,
    secret     TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- 2FA player session persistence table
CREATE TABLE IF NOT EXISTS two_factor_sessions (
    username   TEXT PRIMARY KEY NOT NULL,
    expires_at INTEGER NOT NULL  -- Unix timestamp (seconds)
);
CREATE INDEX IF NOT EXISTS idx_2fa_sessions_expires ON two_factor_sessions(expires_at);

-- Whitelist table
CREATE TABLE IF NOT EXISTS whitelist (
    username TEXT PRIMARY KEY NOT NULL,
    added_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Blacklist table
CREATE TABLE IF NOT EXISTS blacklist (
    username TEXT PRIMARY KEY NOT NULL,
    added_at DATETIME NOT NULL DEFAULT (datetime('now')),
    reason   TEXT
);

-- Backend server table
CREATE TABLE IF NOT EXISTS backend_servers (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT NOT NULL,
    remote_address      TEXT NOT NULL,
    remote_port         INTEGER NOT NULL DEFAULT 25565,
    max_player          INTEGER NOT NULL DEFAULT -1,
    motd_json           TEXT,
    limbo_message       TEXT,
    log_dir             TEXT,
    show_log_level      INTEGER NOT NULL DEFAULT 0,
    save_log_level      INTEGER NOT NULL DEFAULT 0,
    is_default          INTEGER NOT NULL DEFAULT 0,
    enabled             INTEGER NOT NULL DEFAULT 1,
    maintenance         INTEGER NOT NULL DEFAULT 0,
    maintenance_message TEXT,
    ip_forward          INTEGER NOT NULL DEFAULT 0,
    ping_passthrough    INTEGER NOT NULL DEFAULT 0,
    motd_passthrough    INTEGER NOT NULL DEFAULT 0,
    language            TEXT NOT NULL DEFAULT 'en',
    created_at          DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at          DATETIME NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_backend_servers_enabled    ON backend_servers(enabled);
CREATE INDEX IF NOT EXISTS idx_backend_servers_is_default ON backend_servers(is_default);

-- Domain route table
CREATE TABLE IF NOT EXISTS domain_routes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern     TEXT NOT NULL,
    target_addr TEXT NOT NULL,
    target_port INTEGER NOT NULL,
    priority    INTEGER NOT NULL DEFAULT 0,
    backend_id  INTEGER REFERENCES backend_servers(id),
    created_at  DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at  DATETIME NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_domain_routes_priority ON domain_routes(priority DESC, id ASC);

-- User-level route table
CREATE TABLE IF NOT EXISTS user_routes (
    username    TEXT PRIMARY KEY NOT NULL,
    target_addr TEXT NOT NULL,
    target_port INTEGER NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at  DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Player session log table
CREATE TABLE IF NOT EXISTS player_sessions (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    username         TEXT NOT NULL,
    uuid             TEXT NOT NULL,
    backend_addr     TEXT NOT NULL,
    backend_port     INTEGER NOT NULL,
    login_at         DATETIME NOT NULL DEFAULT (datetime('now')),
    logout_at        DATETIME,
    upload_bytes     INTEGER NOT NULL DEFAULT 0,
    download_bytes   INTEGER NOT NULL DEFAULT 0,
    protocol_version INTEGER DEFAULT 0,
    kick_reason      TEXT
);
CREATE INDEX IF NOT EXISTS idx_sessions_username ON player_sessions(username);
CREATE INDEX IF NOT EXISTS idx_sessions_login_at ON player_sessions(login_at);

-- Moderation audit log table
CREATE TABLE IF NOT EXISTS moderation_logs (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    action     TEXT NOT NULL CHECK(action IN ('kick', 'ban', 'pardon')),
    target     TEXT NOT NULL,
    operator   TEXT NOT NULL,
    reason     TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_modlogs_target     ON moderation_logs(target);
CREATE INDEX IF NOT EXISTS idx_modlogs_created_at ON moderation_logs(created_at);

-- Admin account table
CREATE TABLE IF NOT EXISTS admin_accounts (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    username         TEXT NOT NULL UNIQUE,
    password_hash    TEXT NOT NULL,
    totp_secret      TEXT,
    totp_bound       INTEGER NOT NULL DEFAULT 0,
    preferred_locale TEXT NOT NULL DEFAULT 'zh-CN',
    created_at       DATETIME NOT NULL DEFAULT (datetime('now')),
    last_login_at    DATETIME
);
CREATE INDEX IF NOT EXISTS idx_admin_username ON admin_accounts(username);
