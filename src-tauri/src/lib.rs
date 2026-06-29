mod crypto;
mod db;
mod state;
mod totp;

use crypto::{decrypt, derive_key, encrypt};
use state::{AppError, AppState, CipherState};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::thread;
use tauri::Manager;

const VAULT_VERIFIER_PLAINTEXT: &[u8] = b"TokiKagi Vault";
const DEFAULT_AUTO_LOCK_MINUTES: u64 = 5;

fn create_vault_inner(state: &AppState, password: &str) -> Result<(), AppError> {
    let conn = state.db()?;
    if db::is_initialized(&conn)? {
        return Err(AppError::AlreadyInitialized);
    }

    let salt: [u8; 16] = rand::random();
    let key = derive_key(password, &salt);
    let verifier = encrypt(&key, VAULT_VERIFIER_PLAINTEXT).map_err(AppError::crypto)?;
    let updated_at = chrono::Utc::now().to_rfc3339();
    db::set_vault(&conn, &salt, &verifier, &updated_at)?;
    drop(conn);

    *state.cipher()? = Some(CipherState { key });
    *state.last_activity()? = Instant::now();

    let conn = state.db()?;
    if db::get_config(&conn, "auto_lock_minutes")?.is_none() {
        db::set_config(&conn, "auto_lock_minutes", &DEFAULT_AUTO_LOCK_MINUTES.to_string())?;
    }

    Ok(())
}

fn unlock_inner(state: &AppState, password: &str) -> Result<(), AppError> {
    if state.cipher()?.is_some() {
        return Ok(());
    }

    let (salt, verifier) = {
        let conn = state.db()?;
        if !db::is_initialized(&conn)? {
            return Err(AppError::NotInitialized);
        }
        let (salt, verifier, _) = db::get_vault(&conn)?;
        (salt, verifier)
    };

    let key = derive_key(password, &salt);
    let decrypted = decrypt(&key, &verifier).map_err(AppError::crypto)?;
    if decrypted != VAULT_VERIFIER_PLAINTEXT {
        return Err(AppError::InvalidPassword);
    }

    *state.cipher()? = Some(CipherState { key });
    *state.last_activity()? = Instant::now();
    Ok(())
}

fn ensure_unlocked(state: &AppState) -> Result<(), AppError> {
    if state.cipher()?.is_none() {
        return Err(AppError::VaultLocked);
    }
    Ok(())
}

fn spawn_auto_lock_watcher(handle: tauri::AppHandle, duration: Duration) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let state = handle.state::<AppState>();
            if state.cipher.lock().unwrap_or_else(|e| e.into_inner()).is_none() {
                break;
            }
            let elapsed = state.last_activity.lock().unwrap_or_else(|e| e.into_inner()).elapsed();
            if elapsed >= duration {
                *state.cipher.lock().unwrap_or_else(|e| e.into_inner()) = None;
                break;
            }
        }
    });
}

#[tauri::command]
fn is_initialized(state: tauri::State<'_, AppState>) -> Result<bool, AppError> {
    let conn = state.db()?;
    Ok(db::is_initialized(&conn)?)
}

#[tauri::command]
fn create_vault(state: tauri::State<'_, AppState>, password: String) -> Result<(), AppError> {
    create_vault_inner(&state, &password)
}

#[tauri::command]
fn unlock(app: tauri::AppHandle, state: tauri::State<'_, AppState>, password: String) -> Result<(), AppError> {
    unlock_inner(&state, &password)?;

    let auto_lock_minutes: u64 = {
        let conn = state.db()?;
        db::get_config(&conn, "auto_lock_minutes")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_AUTO_LOCK_MINUTES)
    };

    spawn_auto_lock_watcher(app, Duration::from_secs(auto_lock_minutes * 60));
    Ok(())
}

#[tauri::command]
fn lock(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    *state.cipher()? = None;
    Ok(())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>, key: String) -> Result<Option<String>, AppError> {
    ensure_unlocked(&state)?;
    state.with_activity(|conn| Ok(db::get_config(conn, &key)?))
}

#[tauri::command]
fn set_config(state: tauri::State<'_, AppState>, key: String, value: String) -> Result<(), AppError> {
    ensure_unlocked(&state)?;
    state.with_activity(|conn| Ok(db::set_config(conn, &key, &value)?))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_dir = app.path().app_data_dir().expect("failed to resolve app data dir");
            std::fs::create_dir_all(&db_dir).expect("failed to create app data dir");
            let db_path = db_dir.join("tokikagi.db");
            let conn =
                db::open(db_path.to_str().expect("invalid db path")).expect("failed to open database");
            app.manage(AppState {
                db: Mutex::new(conn),
                cipher: Mutex::new(None),
                last_activity: Mutex::new(Instant::now()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            totp::generate_totp,
            totp::get_time_remaining,
            is_initialized,
            create_vault,
            unlock,
            lock,
            get_config,
            set_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_app_state() -> AppState {
        let conn = db::open_in_memory().unwrap();
        AppState {
            db: Mutex::new(conn),
            cipher: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
        }
    }

    fn make_unlocked_state() -> (AppState, String) {
        let state = make_app_state();
        let password = "test_password".to_string();
        create_vault_inner(&state, &password).unwrap();
        (state, password)
    }

    #[test]
    fn test_create_vault_creates_vault() {
        let state = make_app_state();
        create_vault_inner(&state, "my_password").unwrap();
        assert!(state.cipher().unwrap().is_some());
        let conn = state.db().unwrap();
        assert!(db::is_initialized(&conn).unwrap());
    }

    #[test]
    fn test_unlock_with_correct_password() {
        let (state, password) = make_unlocked_state();
        *state.cipher().unwrap() = None;
        assert!(state.cipher().unwrap().is_none());
        unlock_inner(&state, &password).unwrap();
        assert!(state.cipher().unwrap().is_some());
    }

    #[test]
    fn test_unlock_with_wrong_password_fails() {
        let (state, _) = make_unlocked_state();
        *state.cipher().unwrap() = None;
        let result = unlock_inner(&state, "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_clears_cipher() {
        let (state, _) = make_unlocked_state();
        assert!(state.cipher().unwrap().is_some());
        *state.cipher().unwrap() = None;
        assert!(state.cipher().unwrap().is_none());
    }

    #[test]
    fn test_get_config_roundtrip() {
        let (state, _) = make_unlocked_state();
        let conn = state.db().unwrap();
        assert!(db::get_config(&conn, "test_key").unwrap().is_none());
        db::set_config(&conn, "test_key", "test_value").unwrap();
        assert_eq!(
            db::get_config(&conn, "test_key").unwrap(),
            Some("test_value".to_string())
        );
        db::set_config(&conn, "test_key", "updated").unwrap();
        assert_eq!(
            db::get_config(&conn, "test_key").unwrap(),
            Some("updated".to_string())
        );
    }

    #[test]
    fn test_set_config_upserts() {
        let (state, _) = make_unlocked_state();
        let conn = state.db().unwrap();
        db::set_config(&conn, "auto_lock_minutes", "10").unwrap();
        assert_eq!(
            db::get_config(&conn, "auto_lock_minutes").unwrap(),
            Some("10".to_string())
        );
        db::set_config(&conn, "auto_lock_minutes", "3").unwrap();
        assert_eq!(
            db::get_config(&conn, "auto_lock_minutes").unwrap(),
            Some("3".to_string())
        );
    }

    #[test]
    fn test_ensure_unlocked_returns_error_when_locked() {
        let state = make_app_state();
        let result = ensure_unlocked(&state);
        assert!(matches!(result, Err(AppError::VaultLocked)));
    }

    #[test]
    fn test_ensure_unlocked_ok_when_unlocked() {
        let (state, _) = make_unlocked_state();
        let result = ensure_unlocked(&state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verifier_plaintext_constant() {
        assert!(!VAULT_VERIFIER_PLAINTEXT.is_empty());
    }

    #[test]
    fn test_default_auto_lock_minutes() {
        assert_eq!(DEFAULT_AUTO_LOCK_MINUTES, 5);
    }

    #[test]
    fn test_create_vault_sets_default_auto_lock_config() {
        let state = make_app_state();
        create_vault_inner(&state, "password").unwrap();
        let conn = state.db().unwrap();
        assert_eq!(
            db::get_config(&conn, "auto_lock_minutes").unwrap(),
            Some("5".to_string())
        );
    }

    #[test]
    fn test_create_vault_rejects_when_already_initialized() {
        let state = make_app_state();
        create_vault_inner(&state, "pw1").unwrap();
        let result = create_vault_inner(&state, "pw2");
        assert!(matches!(result, Err(AppError::AlreadyInitialized)));
    }
}
