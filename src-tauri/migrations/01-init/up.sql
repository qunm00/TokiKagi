CREATE TABLE vault (
    id         INTEGER PRIMARY KEY CHECK (id = 1),
    salt       TEXT NOT NULL,
    verifier   TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE config (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
