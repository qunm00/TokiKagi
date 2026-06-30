// # Keychain Security Notes
//
// Uses security-framework v3 with SecAccessControl requiring user presence
// (Touch ID or passcode). Reading triggers the OS biometric prompt.
//
// ## Accepted risks
//
// P1 — IPC bridge exposure:
//   An XSS'd webview could invoke unlock_from_keychain. Practically
//   impossible in a desktop TOTP app with no network or user-rendered
//   content. Accepted.

use crate::state::AppError;
use secrecy::SecretString;
use security_framework::access_control::{ProtectionMode, SecAccessControl};
use security_framework::passwords::{
    delete_generic_password, generic_password, set_generic_password_options,
    AccessControlOptions, PasswordOptions,
};

pub fn has_keychain_entry() -> bool {
    let options = PasswordOptions::new_generic_password("tokikagi", "master_password");
    generic_password(options).is_ok()
}

pub fn save_to_keychain(password: &str) -> Result<(), AppError> {
    let access_control = SecAccessControl::create_with_protection(
        Some(ProtectionMode::AccessibleWhenUnlockedThisDeviceOnly),
        AccessControlOptions::USER_PRESENCE.bits(),
    )
    .map_err(|e| AppError::Keychain(e.to_string()))?;

    let mut options = PasswordOptions::new_generic_password("tokikagi", "master_password");
    options.set_access_control(access_control);

    set_generic_password_options(password.as_bytes(), options)
        .map_err(|e| AppError::Keychain(e.to_string()))?;

    Ok(())
}

pub fn get_password_from_keychain() -> Result<SecretString, AppError> {
    let options = PasswordOptions::new_generic_password("tokikagi", "master_password");
    let data = generic_password(options)
        .map_err(|e| AppError::Keychain(e.to_string()))?;
    let password = String::from_utf8(data)
        .map_err(|_| AppError::Keychain("invalid UTF-8 in keychain".into()))?;
    Ok(SecretString::from(password))
}

pub fn remove_from_keychain() -> Result<(), AppError> {
    delete_generic_password("tokikagi", "master_password")
        .map_err(|e| AppError::Keychain(e.to_string()))
}
