mod accounts;
mod crypto;
mod db;
mod state;
mod totp;
mod vault;
mod update;

use accounts::Account;
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
    state.with_unlocked(|conn, _| Ok(db::get_config(conn, &key)?))
}

#[tauri::command]
fn set_config(state: tauri::State<'_, AppState>, key: String, value: String) -> Result<(), AppError> {
    state.with_unlocked(|conn, _| Ok(db::set_config(conn, &key, &value)?))
}

#[tauri::command]
fn add_account(state: tauri::State<'_, AppState>, label: String, secret: String) -> Result<Account, AppError> {
    state.with_unlocked(|conn, key| Ok(accounts::add_account(conn, key, &label, &secret)?))
}

#[tauri::command]
fn get_accounts(state: tauri::State<'_, AppState>) -> Result<Vec<Account>, AppError> {
    state.with_unlocked(|conn, key| Ok(accounts::get_accounts(conn, key)?))
}

#[tauri::command]
fn delete_account(state: tauri::State<'_, AppState>, id: String) -> Result<(), AppError> {
    state.with_unlocked(|conn, _| Ok(accounts::delete_account(conn, &id)?))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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

            #[cfg(not(debug_assertions))]
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    match update::check_and_install(&handle).await {
                        Ok(true) => handle.restart(),
                        Ok(false) => {}
                        Err(e) => eprintln!("update: {e}"),
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            totp::generate_totp,
            is_initialized,
            create_vault,
            unlock,
            lock,
            get_config,
            set_config,
            add_account,
            get_accounts,
            delete_account
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
