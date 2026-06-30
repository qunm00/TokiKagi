use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use secrecy::SecretBox;

/// In-memory cipher key held while the vault is unlocked.
/// Zeroed on drop (locks vault, drops CipherState, or app exit).
pub struct CipherState {
    #[allow(dead_code)]
    pub key: SecretBox<[u8; 32]>,
}

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub cipher: Mutex<Option<CipherState>>,
    pub last_activity: Mutex<Instant>,
}

impl AppState {
    pub fn db(&self) -> Result<MutexGuard<'_, rusqlite::Connection>, AppError> {
        self.db.lock().map_err(|e| AppError::Lock(format!("lock error: {}", e)))
    }

    pub fn cipher(&self) -> Result<MutexGuard<'_, Option<CipherState>>, AppError> {
        self.cipher.lock().map_err(|e| AppError::Lock(format!("lock error: {}", e)))
    }

    pub fn last_activity(&self) -> Result<MutexGuard<'_, Instant>, AppError> {
        self.last_activity.lock().map_err(|e| AppError::Lock(format!("lock error: {}", e)))
    }

    pub fn with_activity<T>(
        &self,
        f: impl FnOnce(&rusqlite::Connection) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let conn = self.db()?;
        let result = f(&conn)?;
        *self.last_activity()? = Instant::now();
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Lock(String),
    Db(String),
    Crypto(String),
    VaultLocked,
    NotInitialized,
    AlreadyInitialized,
    InvalidPassword,
    Keychain(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Lock(msg) => write!(f, "lock error: {msg}"),
            AppError::Db(msg) => write!(f, "{msg}"),
            AppError::Crypto(msg) => write!(f, "{msg}"),
            AppError::VaultLocked => write!(f, "vault is locked"),
            AppError::NotInitialized => write!(f, "not initialized"),
            AppError::AlreadyInitialized => write!(f, "already initialized"),
            AppError::InvalidPassword => write!(f, "invalid password"),
            AppError::Keychain(msg) => write!(f, "{msg}"),
        }
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Db(msg)
    }
}

impl AppError {
    pub fn crypto(msg: impl Into<String>) -> Self {
        AppError::Crypto(msg.into())
    }
}
