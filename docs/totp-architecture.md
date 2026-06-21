# TOTP Architecture

## Crypto stack

```
TOTP (RFC 6238)
  └─ HOTP (RFC 4226)             — src-tauri/src/lib.rs
       └─ HMAC-SHA1 (RFC 2104)   — hmac + sha1 crates (RustCrypto)
            └─ SHA-1 (RFC 3174)   — sha1 crate (RustCrypto)

Base32 decoding (RFC 4648)        — base32 crate
```

## Tauri commands

### `generate_totp`

| Aspect | Detail |
|--------|--------|
| Route | `invoke('generate_totp', { secret })` |
| Input | `secret: String` — Base32-encoded secret key (no padding) |
| Output | `Result<String, String>` — 6-digit code or error |
| Error | `"Invalid Base32 secret"` if decoding fails |
| Protocol | 30-second time step (RFC 6238 default) |

### `get_time_remaining`

| Aspect | Detail |
|--------|--------|
| Route | `invoke('get_time_remaining')` |
| Input | None |
| Output | `u64` — seconds 1..=30 until code rotation |
| Behavior | Counts down 30 → 1, resets to 30 |

## Data flow: `generate_totp`

```
Frontend                            Rust
────────                              ────
invoke('generate_totp', {
  secret: "KRSXG5CT..."          →   base32::decode()
                                          ↓
                                  [raw secret bytes]
                                          ↓
                                  totp(secret, 30)
                                          ↓
                                  now / 30 = counter
                                          ↓
                                  hotp(secret, counter)
                                          ↓
                                  HMAC-SHA1 → 20 bytes
                                  truncate → 31-bit int
                                  % 1,000,000 → 6-digit code
                                          ↓
                              ←  Ok("382947")
```

## Data flow: `get_time_remaining`

```
Frontend                            Rust
────────                              ────
invoke('get_time_remaining')   →   now = unix_timestamp()
                                          ↓
                                  30 - (now % 30) = seconds left
                                          ↓
                              ←  23
```

## Key parameters

| Parameter | Value | Reason |
|-----------|-------|--------|
| `TOTP_STEP` | 30 seconds | RFC 6238 default, matches Google/Authy/Microsoft Authenticator |
| Digits | 6 | Standard across all authenticator apps |
| Algorithm | SHA-1 | RFC 6238 default, used by all major authenticators |
| Skew | 1 step | Allows ±30s clock drift (not configurable yet) |

## Tests

| Test | What it verifies |
|------|-----------------|
| `test_hotp_rfc_4226` | HOTP matches 10 known test vectors from RFC 4226 §5.4 |
| `test_totp_6_digits` | TOTP output is always < 1,000,000 (6 digits) |
| `test_invalid_base32` | Invalid Base32 input returns `Err` |
| `test_time_remaining_bounds` | Remaining time is always 1..=30 |

## Dependencies

```toml
hmac = "0.12"      # HMAC construction (RustCrypto)
sha1 = "0.10"      # SHA-1 hashing (RustCrypto)
base32 = "0.5"     # Base32 encoding/decoding (RFC 4648)
```

All are pure Rust, no native dependencies.
