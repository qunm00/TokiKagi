use serde::{Deserialize, Serialize};

use crate::crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub label: String,
    pub secret: String,
    pub algorithm: String,
    pub digits: i32,
    pub period: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum AccountsError {
    #[error("failed to insert account: {0}")]
    InsertFailed(String),
    #[error("failed to prepare accounts query: {0}")]
    QueryPrepareFailed(String),
    #[error("failed to query accounts: {0}")]
    QueryFailed(String),
    #[error("failed to read account row: {0}")]
    ReadFailed(String),
    #[error("invalid secret hex: {0}")]
    InvalidHex(String),
    #[error("decrypted secret is not UTF-8: {0}")]
    SecretNotUtf8(String),
    #[error("failed to delete account: {0}")]
    DeleteFailed(String),
    #[error("{0}")]
    Crypto(#[from] crypto::CryptoError),
}

pub fn add_account(
    conn: &rusqlite::Connection,
    key: &[u8; 32],
    label: &str,
    secret: &str,
) -> Result<Account, AccountsError> {
    let id = uuid::Uuid::new_v4().to_string();
    let encrypted = crypto::encrypt(key, secret.as_bytes())?;
    let encrypted_hex = hex::encode(encrypted);
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO accounts (id, label, secret, algorithm, digits, period, created_at, updated_at) VALUES (?1, ?2, ?3, 'SHA1', 6, 30, ?4, ?5)",
        rusqlite::params![id, label, encrypted_hex, now, now],
    )
    .map_err(|e| AccountsError::InsertFailed(format!("{e}")))?;

    Ok(Account {
        id,
        label: label.to_string(),
        secret: secret.to_string(),
        algorithm: "SHA1".to_string(),
        digits: 6,
        period: 30,
        created_at: now.clone(),
        updated_at: now,
    })
}

pub fn get_accounts(conn: &rusqlite::Connection, key: &[u8; 32]) -> Result<Vec<Account>, AccountsError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, label, secret, algorithm, digits, period, created_at, updated_at \
             FROM accounts ORDER BY created_at ASC",
        )
        .map_err(|e| AccountsError::QueryPrepareFailed(format!("{e}")))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(|e| AccountsError::QueryFailed(format!("{e}")))?;

    let mut accounts = Vec::new();
    for row in rows {
        let (id, label, encrypted_hex, algorithm, digits, period, created_at, updated_at) =
            row.map_err(|e| AccountsError::ReadFailed(format!("{e}")))?;
        let encrypted =
            hex::decode(&encrypted_hex).map_err(|e| AccountsError::InvalidHex(format!("{e}")))?;
        let decrypted = crypto::decrypt(key, &encrypted)?;
        let secret =
            String::from_utf8(decrypted).map_err(|e| AccountsError::SecretNotUtf8(format!("{e}")))?;
        accounts.push(Account {
            id,
            label,
            secret,
            algorithm,
            digits,
            period,
            created_at,
            updated_at,
        });
    }

    Ok(accounts)
}

pub fn delete_account(conn: &rusqlite::Connection, id: &str) -> Result<(), AccountsError> {
    conn.execute("DELETE FROM accounts WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| AccountsError::DeleteFailed(format!("{e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0x42u8; 32]
    }

    fn open_conn() -> rusqlite::Connection {
        crate::db::open_in_memory().expect("in-memory db should open")
    }

    #[test]
    fn test_migration_creates_accounts_table() {
        let conn = open_conn();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_add_account_roundtrip() {
        let conn = open_conn();
        let key = test_key();
        let account = add_account(&conn, &key, "Test Label", "KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ")
            .unwrap();
        assert_eq!(account.label, "Test Label");
        assert_eq!(account.secret, "KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ");
        assert_eq!(account.algorithm, "SHA1");
        assert_eq!(account.digits, 6);
        assert_eq!(account.period, 30);
    }

    #[test]
    fn test_get_accounts_returns_inserted() {
        let conn = open_conn();
        let key = test_key();

        add_account(&conn, &key, "Account A", "SECRETA").unwrap();
        add_account(&conn, &key, "Account B", "SECRETB").unwrap();

        let accounts = get_accounts(&conn, &key).unwrap();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].label, "Account A");
        assert_eq!(accounts[0].secret, "SECRETA");
        assert_eq!(accounts[1].label, "Account B");
        assert_eq!(accounts[1].secret, "SECRETB");
    }

    #[test]
    fn test_get_accounts_encrypted_at_rest() {
        let conn = open_conn();
        let key = test_key();

        add_account(&conn, &key, "Secret", "MYPLAINTEXT").unwrap();

        // The database should contain hex-encoded ciphertext, not plaintext
        let stored: String = conn
            .query_row("SELECT secret FROM accounts LIMIT 1", [], |row| row.get(0))
            .unwrap();
        assert_ne!(stored, "MYPLAINTEXT", "secret should be encrypted at rest");
        // And the plaintext should still decrypt correctly
        let accounts = get_accounts(&conn, &key).unwrap();
        assert_eq!(accounts[0].secret, "MYPLAINTEXT");
    }

    #[test]
    fn test_delete_account_removes_row() {
        let conn = open_conn();
        let key = test_key();

        add_account(&conn, &key, "To Delete", "DELETEME").unwrap();
        assert_eq!(get_accounts(&conn, &key).unwrap().len(), 1);

        let id = {
            let accounts = get_accounts(&conn, &key).unwrap();
            accounts[0].id.clone()
        };
        delete_account(&conn, &id).unwrap();

        let remaining = get_accounts(&conn, &key).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_delete_account_nonexistent_does_not_error() {
        let conn = open_conn();
        let key = test_key();

        add_account(&conn, &key, "Keep me", "KEEP").unwrap();
        delete_account(&conn, "nonexistent-uuid").unwrap();

        let accounts = get_accounts(&conn, &key).unwrap();
        assert_eq!(accounts.len(), 1);
    }

    #[test]
    fn test_multiple_accounts_roundtrip() {
        let conn = open_conn();
        let key = test_key();
        let labels = ["Alpha", "Beta", "Gamma"];
        let secrets = ["AAAA", "BBBB", "CCCC"];

        for (l, s) in labels.iter().zip(secrets.iter()) {
            add_account(&conn, &key, l, s).unwrap();
        }

        let accounts = get_accounts(&conn, &key).unwrap();
        assert_eq!(accounts.len(), 3);
        for (i, label) in labels.iter().enumerate() {
            assert_eq!(accounts[i].label, *label);
            assert_eq!(accounts[i].secret, secrets[i]);
        }
    }

    #[test]
    fn test_different_keys_cannot_decrypt() {
        let conn = open_conn();
        let key_a = [0x42u8; 32];
        let key_b = [0x24u8; 32];

        add_account(&conn, &key_a, "Mine", "SECRET").unwrap();

        let result = get_accounts(&conn, &key_b);
        assert!(result.is_err(), "wrong key should fail to decrypt");
    }

    #[test]
    fn test_add_account_generates_uuid() {
        let conn = open_conn();
        let key = test_key();

        let a1 = add_account(&conn, &key, "First", "ONE").unwrap();
        let a2 = add_account(&conn, &key, "Second", "TWO").unwrap();

        assert_ne!(a1.id, a2.id, "each account should have a unique UUID");
        assert!(uuid::Uuid::parse_str(&a1.id).is_ok());
        assert!(uuid::Uuid::parse_str(&a2.id).is_ok());
    }
}
