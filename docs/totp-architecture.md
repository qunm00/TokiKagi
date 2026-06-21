# TOTP Architecture

## Crypto stack

Layer        | Standard          | Implementation   | Location
-------------|-------------------|------------------|--------------------------
TOTP         | RFC 6238          | lib.rs           | `src-tauri/src/lib.rs:22`
HOTP         | RFC 4226          | lib.rs           | `src-tauri/src/lib.rs:9`
HMAC-SHA1    | RFC 2104          | `hmac` + `sha1`  | crates (RustCrypto)
SHA-1        | RFC 3174          | `sha1` crate     | crates (RustCrypto)
Base32       | RFC 4648          | `base32` crate   | crates

## Dependency versions

```toml
hmac = "0.12"
sha1 = "0.10"
base32 = "0.5"
```

All pure Rust, no native dependencies.

---

## Commands

### `generate_totp`

Property       | Value
---------------|-----------------------------------------------------------
File           | `src-tauri/src/lib.rs:31`
Signature      | `fn generate_totp(secret: String) -> Result<String, String>`
Frontend call  | `invoke('generate_totp', { secret: "KRSXG5CT..." })`
Input          | `secret` — Base32-encoded key (RFC 4648, no padding)
Output (ok)    | `"382947"` — 6-digit code, zero-padded
Output (err)   | `"Invalid Base32 secret"` — if decoding fails
Algorithm      | TOTP with SHA-1, 6 digits, 30s step, skew=1

### `get_time_remaining`

Property       | Value
---------------|-----------------------------------------------------------
File           | `src-tauri/src/lib.rs:39`
Signature      | `fn get_time_remaining() -> u64`
Frontend call  | `invoke('get_time_remaining')`
Output         | Seconds 1..=30 until the current code expires
Note           | Returns 30 at window start, counts down to 1

---

## Data flow

### `generate_totp`

```
 invoke('generate_totp', { secret: "KRSXG5CT..." })
     │
     ▼
 base32::decode("KRSXG5CT...")                 lib.rs:33
     │
     ▼ [raw secret bytes]
 totp(secret_bytes, TOTP_STEP)                 lib.rs:35
     │
     ├─ now = unix_timestamp()                 lib.rs:23
     ├─ counter = now / 30                     lib.rs:27
     └─ hotp(secret_bytes, counter)            lib.rs:28
          │
          ├─ HMAC-SHA1(secret, counter)        lib.rs:11-13
          ├─ dynamic truncation → 31-bit int   lib.rs:14-18
          └─ % 1_000_000 → 6-digit code       lib.rs:19
     │
     ▼
 Ok("382947")
```

### `get_time_remaining`

```
 invoke('get_time_remaining')
     │
     ▼
 now = unix_timestamp()                        lib.rs:41-44
     │
     ▼
 remaining = TOTP_STEP - (now % TOTP_STEP)     lib.rs:45
     │
     ▼
 23
```

---

## Constants

Name         | Value | Defined in              | Used in
-------------|-------|-------------------------|-------------------------------
TOTP_STEP    | 30    | `src-tauri/src/lib.rs:7`| `totp()`, `get_time_remaining()`

---

## Tests

File: `src-tauri/src/lib.rs:57` (module `tests`)

Test                        | What it verifies                                           | Known values
----------------------------|------------------------------------------------------------|---------------------------
`test_hotp_rfc_4226`       | HOTP matches RFC 4226 §5.4 test vectors                   | 10 values (counter 0..9)
`test_totp_6_digits`       | TOTP output is always < 1,000,000                         | N/A
`test_invalid_base32`      | Invalid Base32 returns `Err("Invalid Base32 secret")`      | `"!!!invalid!!!"`
`test_time_remaining_bounds` | Remaining time is always 1..=TOTP_STEP                    | N/A

Run: `pnpm cargo:test`

---

## Key design notes

- All crypto is pure Rust. No OpenSSL, no system libraries.
- `unwrap_or_default()` on `duration_since(UNIX_EPOCH)` — returns Duration::ZERO if clock is before 1970 instead of panicking.
- Tauri commands are registered in `generate_handler![]` at `lib.rs:52`. Missing registration = frontend gets "Command not found".
- Base32 decoding uses `Rfc4648 { padding: false }` — secrets from QR codes (Google Auth, etc.) omit padding.
