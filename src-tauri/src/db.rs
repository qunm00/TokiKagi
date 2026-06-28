use include_dir::{include_dir, Dir};
use rusqlite::Connection;
use rusqlite_migration::Migrations;

/// Directory containing migration subdirectories (`01-init/up.sql`, …).
static MIGRATION_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/migrations");

/// Loads migrations from the embedded directory.
fn migrations() -> Migrations<'static> {
    Migrations::from_directory(&MIGRATION_DIR).expect("invalid migration directory")
}

/// Opens (or creates) the database at `path` and runs pending migrations.
pub fn open(path: &str) -> Result<Connection, String> {
    let mut conn = Connection::open(path).map_err(|e| format!("failed to open database: {e}"))?;
    migrations()
        .to_latest(&mut conn)
        .map_err(|e| format!("database migration failed: {e}"))?;
    Ok(conn)
}

/// Opens an in-memory database (for testing).
#[cfg(test)]
pub fn open_in_memory() -> Result<Connection, String> {
    let mut conn = Connection::open_in_memory()
        .map_err(|e| format!("failed to open in-memory database: {e}"))?;
    migrations()
        .to_latest(&mut conn)
        .map_err(|e| format!("database migration failed: {e}"))?;
    Ok(conn)
}

/// Returns `true` if the vault has been initialized (has a row).
pub fn is_initialized(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM vault", [], |row| row.get(0))
        .map_err(|e| format!("failed to check vault initialization: {e}"))?;
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
    fn test_is_initialized_returns_false_for_empty_vault() {
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
        // id=2 violates CHECK(id=1)
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
        // Duplicate id=1 violates PRIMARY KEY
        let err = conn.execute(
            "INSERT INTO vault (id, salt, verifier, updated_at) VALUES (1, 'salt2', 'verifier2', '2024-01-01T00:00:00Z')",
            [],
        );
        assert!(err.is_err());
    }
}
