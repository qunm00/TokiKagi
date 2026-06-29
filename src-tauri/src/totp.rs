use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha1 = Hmac<Sha1>;

const TOTP_STEP: u64 = 30;

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
pub fn generate_totp(secret: String) -> Result<String, String> {
    let secret_bytes = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &secret)
        .ok_or("Invalid Base32 secret")?;
    let code = totp(&secret_bytes, TOTP_STEP);
    Ok(format!("{:06}", code))
}

#[tauri::command]
pub fn get_time_remaining() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    TOTP_STEP - (now % TOTP_STEP)
}

#[cfg(test)]
mod tests {
    use super::*;

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
