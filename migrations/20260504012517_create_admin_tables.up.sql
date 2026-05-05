CREATE TABLE
    admin_sessions (
        id TEXT PRIMARY KEY,
        username TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        expires_at DATETIME NOT NULL,
        FOREIGN KEY (provider, provider_id) REFERENCES admin_whitelist (provider, provider_id) ON DELETE CASCADE
    );

CREATE TABLE
    admin_whitelist (
        provider TEXT NOT NULL,
        provider_id TEXT NOT NULL,
        username TEXT,
        role TEXT NOT NULL,
        added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        added_by TEXT,
        PRIMARY KEY (provider, provider_id)
    );

CREATE INDEX idx_admin_sessions_expires_at ON admin_sessions (expires_at);

CREATE INDEX idx_admin_sessions_twitch_id ON admin_sessions (twitch_id);