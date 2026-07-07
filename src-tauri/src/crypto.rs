use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::Argon2;

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("ciphertext too short")]
    CiphertextTooShort,
    #[error("decryption failed: authentication tag mismatch")]
    DecryptionFailed,
}

/// Derives a 256-bit key from a password + salt using Argon2id.
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .expect("Argon2id key derivation should not fail with valid params");
    key
}

/// Encrypts plaintext using AES-256-GCM with a random nonce.
///
/// Returns `[nonce (12 bytes) || ciphertext + tag]` as a single vec.
pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(format!("{e}")))?;

    let mut out = nonce.to_vec();
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypts data produced by [`encrypt`].
///
/// Expects `data` to be `[nonce (12 bytes) || ciphertext + tag]`.
pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if data.len() < 12 {
        return Err(CryptoError::CiphertextTooShort);
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_is_deterministic() {
        let password = "my_master_password";
        let salt = b"0123456789abcdef";
        let k1 = derive_key(password, salt);
        let k2 = derive_key(password, salt);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_derive_key_different_salts_produce_different_keys() {
        let password = "my_master_password";
        let k1 = derive_key(password, b"0123456789abcdef");
        let k2 = derive_key(password, b"fedcba9876543210");
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_derive_key_different_passwords_produce_different_keys() {
        let salt = b"0123456789abcdef";
        let k1 = derive_key("password_a", salt);
        let k2 = derive_key("password_b", salt);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = b"Hello, TokiKagi!";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_binary_data() {
        let key = [0x42u8; 32];
        let plaintext = b"\x00\x01\x02\xff\xfe";

        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_produces_unique_ciphertexts() {
        let key = [0x42u8; 32];
        let plaintext = b"same data";

        let c1 = encrypt(&key, plaintext).unwrap();
        let c2 = encrypt(&key, plaintext).unwrap();

        // Nonce should differ, so ciphertexts must differ
        assert_ne!(c1, c2, "encrypt should use a random nonce each time");
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key_correct = [0x42u8; 32];
        let key_wrong = [0x24u8; 32];
        let plaintext = b"secret";

        let encrypted = encrypt(&key_correct, plaintext).unwrap();
        let result = decrypt(&key_wrong, &encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails() {
        let key = [0x42u8; 32];
        let plaintext = b"secret";

        let mut encrypted = encrypt(&key, plaintext).unwrap();
        // Flip a bit in the ciphertext portion
        if let Some(byte) = encrypted.get_mut(14) {
            *byte ^= 0xff;
        }

        let result = decrypt(&key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short_fails() {
        let key = [0x42u8; 32];
        let result = decrypt(&key, b"tooshort");
        assert!(result.is_err());
    }
}
