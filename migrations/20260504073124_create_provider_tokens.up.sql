DROP TABLE IF EXISTS twitch_tokens;

CREATE TABLE
    IF NOT EXISTS provider_tokens (
        account_variant TEXT NOT NULL,
        provider TEXT NOT NULL,
        provider_id TEXT NOT NULL,
        expires_at TEXT,
        raw_data TEXT NOT NULL,
        PRIMARY KEY (account_variant, provider)
    );