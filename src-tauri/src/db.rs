use include_dir::{include_dir, Dir};
use rusqlite::Connection;
use rusqlite_migration::Migrations;

/// Directory containing migration subdirectories (`01-init/up.sql`, …).
static MIGRATION_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("failed to open database: {0}")]
    Open(String),
    #[error("database migration failed: {0}")]
    Migration(String),
    #[error("failed to read vault: {0}")]
    VaultRead(String),
    #[error("invalid hex: {0}")]
    Decode(String),
    #[error("failed to insert vault: {0}")]
    VaultWrite(String),
    #[error("failed to read config: {0}")]
    ConfigRead(String),
    #[error("failed to write config: {0}")]
    ConfigWrite(String),
    #[error("failed to check initialization: {0}")]
    InitCheck(String),
}

/// Loads migrations from the embedded directory.
fn migrations() -> Migrations<'static> {
    Migrations::from_directory(&MIGRATION_DIR).expect("invalid migration directory")
}

/// Opens (or creates) the database at `path` and runs pending migrations.
pub fn open(path: &str) -> Result<Connection, DbError> {
    let mut conn = Connection::open(path).map_err(|e| DbError::Open(format!("{e}")))?;
    migrations()
        .to_latest(&mut conn)
        .map_err(|e| DbError::Migration(format!("{e}")))?;
    Ok(conn)
}

/// Opens an in-memory database (for testing).
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection, DbError> {
    let mut conn = Connection::open_in_memory()
        .map_err(|e| DbError::Open(format!("{e}")))?;
    migrations()
        .to_latest(&mut conn)
        .map_err(|e| DbError::Migration(format!("{e}")))?;
    Ok(conn)
}

/// Returns the vault row: (salt, verifier, updated_at).
pub fn get_vault(conn: &Connection) -> Result<(Vec<u8>, Vec<u8>, String), DbError> {
    let (salt_hex, verifier_hex, updated_at): (String, String, String) = conn
        .query_row("SELECT salt, verifier, updated_at FROM vault WHERE id = 1", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| DbError::VaultRead(format!("{e}")))?;
    let salt = hex::decode(&salt_hex).map_err(|e| DbError::Decode(format!("{e}")))?;
    let verifier = hex::decode(&verifier_hex).map_err(|e| DbError::Decode(format!("{e}")))?;
    Ok((salt, verifier, updated_at))
}

/// Inserts (or replaces) the single vault row.
pub fn set_vault(conn: &Connection, salt: &[u8], verifier: &[u8], updated_at: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO vault (id, salt, verifier, updated_at) VALUES (1, ?1, ?2, ?3)",
        rusqlite::params![hex::encode(salt), hex::encode(verifier), updated_at],
    )
    .map_err(|e| DbError::VaultWrite(format!("{e}")))?;
    Ok(())
}

/// Reads a config value.
pub fn get_config(conn: &Connection, key: &str) -> Result<Option<String>, DbError> {
    let result: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM config WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get(0),
    );
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DbError::ConfigRead(format!("{e}"))),
    }
}

/// Writes a config value.
pub fn set_config(conn: &Connection, key: &str, value: &str) -> Result<(), DbError> {
    conn.execute(
        "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| DbError::ConfigWrite(format!("{e}")))?;
    Ok(())
}

/// Returns `true` if vault have been created (has a row).
pub fn is_initialized(conn: &Connection) -> Result<bool, DbError> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM vault", [], |row| row.get(0))
        .map_err(|e| DbError::InitCheck(format!("{e}")))?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_validate() {
        assert!(migrations().validate().is_ok());
    }

    #[test]
    fn test_migrate_creates_vault_table() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM vault", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_migrate_creates_config_table() {
        let conn = open_in_memory().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM config", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_migrate_is_idempotent() {
        // Each in-memory DB starts fresh; this test verifies that opening
        // (which applies migrations) works repeatedly without error.
        let _ = open_in_memory().unwrap();
        let _ = open_in_memory().unwrap();
    }

    #[test]
    fn test_is_initialized_returns_false_empty_vault() {
        let conn = open_in_memory().unwrap();
        assert!(!is_initialized(&conn).unwrap());
    }

    #[test]
    fn test_is_initialized_returns_true_after_insert() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO vault (id, salt, verifier, updated_at) VALUES (1, 'salt', 'verifier', '2024-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        assert!(is_initialized(&conn).unwrap());
    }

    #[test]
    fn test_vault_check_constraint_enforces_id_equals_one() {
        let conn = open_in_memory().unwrap();
        let err = conn.execute(
            "INSERT INTO vault (id, salt, verifier, updated_at) VALUES (2, 'salt', 'verifier', '2024-01-01T00:00:00Z')",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_vault_primary_key_enforces_single_row() {
        let conn = open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO vault (id, salt, verifier, updated_at) VALUES (1, 'salt', 'verifier', '2024-01-01T00:00:00Z')",
            [],
        )
        .unwrap();
        let err = conn.execute(
            "INSERT INTO vault (id, salt, verifier, updated_at) VALUES (1, 'salt2', 'verifier2', '2024-01-01T00:00:00Z')",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_insert_and_get_vault_roundtrip() {
        let conn = open_in_memory().unwrap();
        let salt = b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10";
        let verifier = b"\xbe\xef\xca\xfe";
        let updated_at = "2025-06-28T12:00:00+00:00";

        set_vault(&conn, salt, verifier, updated_at).unwrap();
        let (got_salt, got_verifier, got_updated) = get_vault(&conn).unwrap();

        assert_eq!(got_salt, salt);
        assert_eq!(got_verifier, verifier);
        assert_eq!(got_updated, updated_at);
    }

    #[test]
    fn test_set_vault_replaces_existing_row() {
        let conn = open_in_memory().unwrap();
        let salt1 = b"aaaaaaaaaaaaaaaa";
        let salt2 = b"bbbbbbbbbbbbbbbb";

        set_vault(&conn, salt1, b"v1", "2024-01-01T00:00:00Z").unwrap();
        set_vault(&conn, salt2, b"v2", "2025-01-01T00:00:00Z").unwrap();

        let (got_salt, got_verifier, _) = get_vault(&conn).unwrap();
        assert_eq!(got_salt, salt2);
        assert_eq!(got_verifier, b"v2");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM vault", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_vault_errors_on_empty() {
        let conn = open_in_memory().unwrap();
        let result = get_vault(&conn);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_config_returns_none_for_missing_key() {
        let conn = open_in_memory().unwrap();
        assert_eq!(get_config(&conn, "nonexistent").unwrap(), None);
    }

    #[test]
    fn test_set_config_and_get_config_roundtrip() {
        let conn = open_in_memory().unwrap();
        set_config(&conn, "theme", "dark").unwrap();
        assert_eq!(
            get_config(&conn, "theme").unwrap(),
            Some("dark".to_string())
        );
    }

    #[test]
    fn test_set_config_upserts_existing_key() {
        let conn = open_in_memory().unwrap();
        set_config(&conn, "auto_lock_minutes", "5").unwrap();
        set_config(&conn, "auto_lock_minutes", "10").unwrap();
        assert_eq!(
            get_config(&conn, "auto_lock_minutes").unwrap(),
            Some("10".to_string())
        );
    }

    #[test]
    fn test_config_multiple_keys_are_independent() {
        let conn = open_in_memory().unwrap();
        set_config(&conn, "key_a", "value_a").unwrap();
        set_config(&conn, "key_b", "value_b").unwrap();
        assert_eq!(
            get_config(&conn, "key_a").unwrap(),
            Some("value_a".to_string())
        );
        assert_eq!(
            get_config(&conn, "key_b").unwrap(),
            Some("value_b".to_string())
        );
    }
}
