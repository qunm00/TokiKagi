mod crypto;
mod db;
mod keychain;
mod state;
mod totp;
mod vault;

use secrecy::ExposeSecret;
use state::{AppError, AppState};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::Manager;

#[tauri::command]
fn is_initialized(state: tauri::State<'_, AppState>) -> Result<bool, AppError> {
    let conn = state.db()?;
    Ok(db::is_initialized(&conn)?)
}

#[tauri::command]
fn create_vault(state: tauri::State<'_, AppState>, password: String) -> Result<(), AppError> {
    let password = secrecy::SecretString::from(password);
    vault::create_vault_inner(&state, password.expose_secret())
}

#[tauri::command]
fn unlock(app: tauri::AppHandle, state: tauri::State<'_, AppState>, password: String) -> Result<(), AppError> {
    let password = secrecy::SecretString::from(password);
    vault::unlock_inner(&state, password.expose_secret())?;

    let auto_lock_minutes: u64 = {
        let conn = state.db()?;
        db::get_config(&conn, "auto_lock_minutes")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(vault::DEFAULT_AUTO_LOCK_MINUTES)
    };

    vault::spawn_auto_lock_watcher(app, Duration::from_secs(auto_lock_minutes * 60));
    Ok(())
}

#[tauri::command]
fn lock(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    *state.cipher()? = None;
    Ok(())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>, key: String) -> Result<Option<String>, AppError> {
    vault::ensure_unlocked(&state)?;
    state.with_activity(|conn| Ok(db::get_config(conn, &key)?))
}

#[tauri::command]
fn set_config(state: tauri::State<'_, AppState>, key: String, value: String) -> Result<(), AppError> {
    vault::ensure_unlocked(&state)?;
    state.with_activity(|conn| Ok(db::set_config(conn, &key, &value)?))
}

#[tauri::command]
fn has_keychain_entry() -> bool {
    keychain::has_keychain_entry()
}

#[tauri::command]
fn save_to_keychain(password: String) -> Result<(), AppError> {
    let password = secrecy::SecretString::from(password);
    keychain::save_to_keychain(password.expose_secret())
}

#[tauri::command]
fn unlock_and_save_to_keychain(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    password: String,
) -> Result<(), AppError> {
    let password = secrecy::SecretString::from(password);
    vault::unlock_inner(&state, password.expose_secret())?;
    keychain::save_to_keychain(password.expose_secret())?;

    let auto_lock_minutes: u64 = {
        let conn = state.db()?;
        db::get_config(&conn, "auto_lock_minutes")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(vault::DEFAULT_AUTO_LOCK_MINUTES)
    };

    vault::spawn_auto_lock_watcher(app, Duration::from_secs(auto_lock_minutes * 60));
    Ok(())
}

#[tauri::command]
fn remove_from_keychain() -> Result<(), AppError> {
    keychain::remove_from_keychain()
}

#[tauri::command]
fn unlock_from_keychain(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let password = keychain::get_password_from_keychain()?;
    vault::unlock_inner(&state, password.expose_secret())?;

    let auto_lock_minutes: u64 = {
        let conn = state.db()?;
        db::get_config(&conn, "auto_lock_minutes")?
            .and_then(|v| v.parse().ok())
            .unwrap_or(vault::DEFAULT_AUTO_LOCK_MINUTES)
    };

    vault::spawn_auto_lock_watcher(app, Duration::from_secs(auto_lock_minutes * 60));
    Ok(())
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
            set_config,
            has_keychain_entry,
            save_to_keychain,
            remove_from_keychain,
            unlock_from_keychain,
            unlock_and_save_to_keychain
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
        vault::create_vault_inner(&state, &password).unwrap();
        (state, password)
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
    fn test_verifier_plaintext_constant() {
        assert!(!vault::VAULT_VERIFIER_PLAINTEXT.is_empty());
    }

    #[test]
    fn test_default_auto_lock_minutes() {
        assert_eq!(vault::DEFAULT_AUTO_LOCK_MINUTES, 5);
    }
}
