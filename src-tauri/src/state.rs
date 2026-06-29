use std::time::Instant;

/// In-memory cipher key held while the vault is unlocked.
pub struct CipherState {
    #[allow(dead_code)]
    pub key: [u8; 32],
}

/// Shared application state managed by Tauri.
pub struct AppState {
    pub db: std::sync::Mutex<rusqlite::Connection>,
    pub cipher: std::sync::Mutex<Option<CipherState>>,
    pub last_activity: std::sync::Mutex<Instant>,
}
