CREATE TABLE accounts (
    id         TEXT PRIMARY KEY,
    label      TEXT NOT NULL,
    secret     TEXT NOT NULL,
    algorithm  TEXT NOT NULL DEFAULT 'SHA1',
    digits     INTEGER NOT NULL DEFAULT 6,
    period     INTEGER NOT NULL DEFAULT 30,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
