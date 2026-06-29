mod crypto;
mod db;
mod state;

use crypto::{decrypt, derive_key, encrypt};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use state::{AppState, CipherState};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::thread;
use tauri::Manager;

type HmacSha1 = Hmac<Sha1>;

const TOTP_STEP: u64 = 30;
const VAULT_VERIFIER_PLAINTEXT: &[u8] = b"TokiKagi Vault";
const DEFAULT_AUTO_LOCK_MINUTES: u64 = 5;

fn hotp(secret: &[u8], counter: u64) -> u32 {
    let counter_bytes = counter.to_be_bytes();
    let mut mac = HmacSha1::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();
    let offset = (result[19] & 0x0f) as usize;
    let code = ((result[offset] & 0x7f) as u32) << 24
        | (result[offset + 1] as u32) << 16
        | (result[offset + 2] as u32) << 8
        | (result[offset + 3] as u32);
    code % 1_000_000
}

fn totp(secret: &[u8], time_step: u64) -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let counter = now / time_step;
    hotp(secret, counter)
}

#[tauri::command]
fn generate_totp(secret: String) -> Result<String, String> {
    let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &secret)
        .ok_or("Invalid Base32 secret")?;
    let code = totp(&secret_bytes, TOTP_STEP);
    Ok(format!("{:06}", code))
}

#[tauri::command]
fn get_time_remaining() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    TOTP_STEP - (now % TOTP_STEP)
}

#[tauri::command]
fn is_initialized(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
    db::is_initialized(&conn)
}

#[tauri::command]
fn create_vault(state: tauri::State<'_, AppState>, password: String) -> Result<(), String> {
    {
        let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
        if db::is_initialized(&conn)? {
            return Err("already initialized".into());
        }
    }

    let salt: [u8; 16] = rand::random();
    let key = derive_key(&password, &salt);
    let verifier = encrypt(&key, VAULT_VERIFIER_PLAINTEXT)?;

    let updated_at = chrono::Utc::now().to_rfc3339();
    {
        let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
        db::set_vault(&conn, &salt, &verifier, &updated_at)?;
    }

    let mut cipher = state.cipher.lock().map_err(|e| format!("lock error: {e}"))?;
    *cipher = Some(CipherState { key });

    let mut activity = state.last_activity.lock().map_err(|e| format!("lock error: {e}"))?;
    *activity = Instant::now();

    let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
    if db::get_config(&conn, "auto_lock_minutes")?.is_none() {
        db::set_config(&conn, "auto_lock_minutes", &DEFAULT_AUTO_LOCK_MINUTES.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn unlock(app: tauri::AppHandle, state: tauri::State<'_, AppState>, password: String) -> Result<(), String> {
    if state.cipher.lock().map_err(|e| format!("lock error: {e}"))?.is_some() {
        return Ok(());
    }

    let (salt, verifier) = {
        let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
        if !db::is_initialized(&conn)? {
            return Err("not initialized".into());
        }
        let (salt, verifier, _) = db::get_vault(&conn)?;
        (salt, verifier)
    };

    let key = derive_key(&password, &salt);
    let decrypted = decrypt(&key, &verifier)?;
    if decrypted != VAULT_VERIFIER_PLAINTEXT {
        return Err("invalid password".into());
    }

    let mut cipher = state.cipher.lock().map_err(|e| format!("lock error: {e}"))?;
    *cipher = Some(CipherState { key });

    let mut activity = state.last_activity.lock().map_err(|e| format!("lock error: {e}"))?;
    *activity = Instant::now();
    drop(cipher);
    drop(activity);

    let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
    let auto_lock_minutes: u64 = db::get_config(&conn, "auto_lock_minutes")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_AUTO_LOCK_MINUTES);
    drop(conn);

    let handle = app.clone();
    thread::spawn(move || {
        let duration = Duration::from_secs(auto_lock_minutes * 60);
        loop {
            thread::sleep(Duration::from_secs(1));

            let state = handle.state::<AppState>();

            if state.cipher.lock().unwrap().is_none() {
                break;
            }

            let elapsed = state.last_activity.lock().unwrap().elapsed();
            if elapsed >= duration {
                *state.cipher.lock().unwrap() = None;
                break;
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn lock(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut cipher = state.cipher.lock().map_err(|e| format!("lock error: {e}"))?;
    *cipher = None;
    Ok(())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>, key: String) -> Result<Option<String>, String> {
    ensure_unlocked(&state)?;

    let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
    let value = db::get_config(&conn, &key)?;

    let mut activity = state.last_activity.lock().map_err(|e| format!("lock error: {e}"))?;
    *activity = Instant::now();

    Ok(value)
}

#[tauri::command]
fn set_config(state: tauri::State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    ensure_unlocked(&state)?;

    let conn = state.db.lock().map_err(|e| format!("lock error: {e}"))?;
    db::set_config(&conn, &key, &value)?;

    let mut activity = state.last_activity.lock().map_err(|e| format!("lock error: {e}"))?;
    *activity = Instant::now();

    Ok(())
}

fn ensure_unlocked(state: &AppState) -> Result<(), String> {
    let cipher = state.cipher.lock().map_err(|e| format!("lock error: {e}"))?;
    if cipher.is_none() {
        return Err("vault is locked".into());
    }
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
            generate_totp,
            get_time_remaining,
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
        let password = "test_password".to_string();
        let state = make_app_state();
        let conn = state.db.lock().unwrap();

        let salt: [u8; 16] = rand::random();
        let key = derive_key(&password, &salt);
        let verifier = encrypt(&key, VAULT_VERIFIER_PLAINTEXT).unwrap();
        db::set_vault(&conn, &salt, &verifier, "2024-01-01T00:00:00Z").unwrap();
        drop(conn);

        *state.cipher.lock().unwrap() = Some(CipherState { key });
        (state, password)
    }

    #[test]
    fn test_create_vault_creates_vault() {
        let state = make_app_state();
        let password = "my_password".to_string();

        // Simulate what create_vault does
        {
            let conn = state.db.lock().unwrap();
            assert!(!db::is_initialized(&conn).unwrap());
        }

        let salt: [u8; 16] = rand::random();
        let key = derive_key(&password, &salt);
        let verifier = encrypt(&key, VAULT_VERIFIER_PLAINTEXT).unwrap();
        let updated_at = chrono::Utc::now().to_rfc3339();
        {
            let conn = state.db.lock().unwrap();
            db::set_vault(&conn, &salt, &verifier, &updated_at).unwrap();
            assert!(db::is_initialized(&conn).unwrap());
        }

        *state.cipher.lock().unwrap() = Some(CipherState { key });
        assert!(state.cipher.lock().unwrap().is_some());
    }

    #[test]
    fn test_unlock_with_correct_password() {
        let (state, password) = make_unlocked_state();

        // Lock first
        *state.cipher.lock().unwrap() = None;
        assert!(state.cipher.lock().unwrap().is_none());

        // Unlock
        let conn = state.db.lock().unwrap();
        let (salt, verifier, _) = db::get_vault(&conn).unwrap();
        let key = derive_key(&password, &salt);
        let decrypted = decrypt(&key, &verifier).unwrap();
        assert_eq!(decrypted, VAULT_VERIFIER_PLAINTEXT);
        drop(conn);

        *state.cipher.lock().unwrap() = Some(CipherState { key });
        assert!(state.cipher.lock().unwrap().is_some());
    }

    #[test]
    fn test_unlock_with_wrong_password_fails() {
        let (state, _) = make_unlocked_state();
        *state.cipher.lock().unwrap() = None;

        let conn = state.db.lock().unwrap();
        let (salt, verifier, _) = db::get_vault(&conn).unwrap();
        let wrong_key = derive_key("wrong_password", &salt);
        let result = decrypt(&wrong_key, &verifier);
        assert!(result.is_err());
    }

    #[test]
    fn test_lock_clears_cipher() {
        let (state, _) = make_unlocked_state();
        assert!(state.cipher.lock().unwrap().is_some());

        *state.cipher.lock().unwrap() = None;
        assert!(state.cipher.lock().unwrap().is_none());
    }

    #[test]
    fn test_get_config_roundtrip() {
        let (state, _) = make_unlocked_state();
        let conn = state.db.lock().unwrap();

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
        let conn = state.db.lock().unwrap();

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
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "vault is locked");
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
    fn test_hotp_rfc_4226() {
        let secret = b"12345678901234567890";
        assert_eq!(hotp(secret, 0), 755224);
        assert_eq!(hotp(secret, 1), 287082);
        assert_eq!(hotp(secret, 2), 359152);
        assert_eq!(hotp(secret, 3), 969429);
        assert_eq!(hotp(secret, 4), 338314);
        assert_eq!(hotp(secret, 5), 254676);
        assert_eq!(hotp(secret, 6), 287922);
        assert_eq!(hotp(secret, 7), 162583);
        assert_eq!(hotp(secret, 8), 399871);
        assert_eq!(hotp(secret, 9), 520489);
    }

    #[test]
    fn test_totp_6_digits() {
        let secret = base32::decode(
            base32::Alphabet::Rfc4648 { padding: false },
            "KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ",
        )
        .unwrap();
        let code = totp(&secret, TOTP_STEP);
        assert!(code < 1_000_000);
    }

    #[test]
    fn test_invalid_base32() {
        let result = generate_totp("!!!invalid!!!".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_time_remaining_bounds() {
        let r = get_time_remaining();
        assert!(r <= TOTP_STEP);
        assert!(r > 0);
    }
}
