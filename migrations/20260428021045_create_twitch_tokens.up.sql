CREATE TABLE
    IF NOT EXISTS twitch_tokens (
        id INTEGER PRIMARY KEY CHECK (id = 1),
        access_token TEXT NOT NULL,
        refresh_token TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime ('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime ('now')),
        CONSTRAINT single_row CHECK (id = 1)
    );

INSERT
OR REPLACE INTO twitch_tokens (
    id,
    access_token,
    refresh_token,
    created_at,
    updated_at
)
VALUES
    (1, '', '', datetime ('now'), datetime ('now'));