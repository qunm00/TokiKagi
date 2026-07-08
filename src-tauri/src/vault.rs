use crate::crypto::{decrypt, derive_key, encrypt};
use crate::db;
use crate::state::{AppError, AppState, CipherState};
use secrecy::SecretBox;
use std::time::{Duration, Instant};
use std::thread;
use tauri::{Emitter, Manager};

pub const VAULT_VERIFIER_PLAINTEXT: &[u8] = b"TokiKagi Vault";
pub const DEFAULT_AUTO_LOCK_MINUTES: u64 = 5;

pub fn create_vault_inner(state: &AppState, password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::PasswordTooShort);
    }

    let conn = state.db()?;
    if db::is_initialized(&conn)? {
        return Err(AppError::AlreadyInitialized);
    }

    let salt: [u8; 16] = rand::random();
    let key = derive_key(password, &salt);
    let verifier = encrypt(&key, VAULT_VERIFIER_PLAINTEXT)?;
    let updated_at = chrono::Utc::now().to_rfc3339();
    db::set_vault(&conn, &salt, &verifier, &updated_at)?;
    drop(conn);

    *state.cipher()? = Some(CipherState { key: SecretBox::new(Box::new(key)) });
    *state.last_activity()? = Instant::now();

    let conn = state.db()?;
    if db::get_config(&conn, "auto_lock_minutes")?.is_none() {
        db::set_config(&conn, "auto_lock_minutes", &DEFAULT_AUTO_LOCK_MINUTES.to_string())?;
    }

    Ok(())
}

pub fn unlock_inner(state: &AppState, password: &str) -> Result<(), AppError> {
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
    let decrypted = decrypt(&key, &verifier).map_err(|_| AppError::InvalidPassword)?;
    if decrypted != VAULT_VERIFIER_PLAINTEXT {
        return Err(AppError::InvalidPassword);
    }

    *state.cipher()? = Some(CipherState { key: SecretBox::new(Box::new(key)) });
    *state.last_activity()? = Instant::now();
    Ok(())
}

pub fn spawn_auto_lock_watcher(handle: tauri::AppHandle, duration: Duration) {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let state = handle.state::<AppState>();
            if state.cipher.lock().unwrap_or_else(|e| e.into_inner()).is_none() {
                break;
            }
            let elapsed = state.last_activity.lock().unwrap_or_else(|e| e.into_inner()).elapsed();
            if elapsed >= duration {
                handle.emit("vault-locked", ()).ok();
                *state.cipher.lock().unwrap_or_else(|e| e.into_inner()) = None;
                break;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn make_state() -> AppState {
        let conn = db::open_in_memory().unwrap();
        AppState {
            db: Mutex::new(conn),
            cipher: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
        }
    }

    fn make_unlocked() -> (AppState, String) {
        let state = make_state();
        let password = "test_password".to_string();
        create_vault_inner(&state, &password).unwrap();
        (state, password)
    }

    #[test]
    fn test_create_vault_creates_vault() {
        let state = make_state();
        create_vault_inner(&state, "my_password").unwrap();
        assert!(state.cipher().unwrap().is_some());
        let conn = state.db().unwrap();
        assert!(db::is_initialized(&conn).unwrap());
    }

    #[test]
    fn test_unlock_with_correct_password() {
        let (state, password) = make_unlocked();
        *state.cipher().unwrap() = None;
        assert!(state.cipher().unwrap().is_none());
        unlock_inner(&state, &password).unwrap();
        assert!(state.cipher().unwrap().is_some());
    }

    #[test]
    fn test_unlock_with_wrong_password_fails() {
        let (state, _) = make_unlocked();
        *state.cipher().unwrap() = None;
        let result = unlock_inner(&state, "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_clears_cipher() {
        let (state, _) = make_unlocked();
        assert!(state.cipher().unwrap().is_some());
        *state.cipher().unwrap() = None;
        assert!(state.cipher().unwrap().is_none());
    }

    #[test]
    fn test_create_vault_sets_default_auto_lock_config() {
        let state = make_state();
        create_vault_inner(&state, "password").unwrap();
        let conn = state.db().unwrap();
        assert_eq!(
            db::get_config(&conn, "auto_lock_minutes").unwrap(),
            Some("5".to_string())
        );
    }

    #[test]
    fn test_create_vault_rejects_when_already_initialized() {
        let state = make_state();
        create_vault_inner(&state, "password1").unwrap();
        let result = create_vault_inner(&state, "password2");
        assert!(matches!(result, Err(AppError::AlreadyInitialized)));
    }
}
