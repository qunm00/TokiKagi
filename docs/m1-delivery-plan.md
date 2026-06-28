# M1 — Vault: Delivery Plan

> **Goal:** User can open the app, create a master password, unlock with password or fingerprint, and the app auto-locks after inactivity.
>
> Architecture reference: [`docs/plan.mdx`](./plan.mdx)

## Strategy

M1 is split into **6 PRs**, each:
- Compiles and tests pass independently
- Maintains or improves line coverage
- Is conceptually coherent (one concern per PR)
- Is small enough to review in one sitting (~3–5 files each)
- Builds on previous PRs without leaving dead code

---

## PR 0 — Tooling: Coverage + CI

**Files affected:** `.github/workflows/ci.yml` (new), `Cargo.toml`, `vitest.config.ts`

**What it delivers:**

- Rust coverage via `cargo tarpaulin` or `cargo-llvm-cov`
- Frontend coverage via `vitest --coverage`
- GitHub Actions CI that runs tests + coverage on every PR
- Coverage thresholds (optional, can be soft-fail initially)

**Why standalone:** Zero feature work. Sets up infrastructure so every subsequent PR gets coverage checks automatically. Without it, coverage is an intention that never gets verified.

---

## PR 1 — Rust: DB + Crypto foundation

**Files affected:** `Cargo.toml`, `src/crypto.rs` (new), `src/db.rs` (new), `src/lib.rs`

**What it delivers:**

- Add deps: `rusqlite`, `argon2`, `aes-gcm`, `rand`, `chrono`, `uuid` (not `keyring` yet)
- `crypto.rs` — `derive_key()` (Argon2id), `encrypt()`/`decrypt()` (AES-256-GCM)
- `db.rs` — SQLite init, create `vault` + `config` tables
- `AppState` with `db: Mutex<Connection>`
- Command: `is_initialized() -> bool`
- Tests for crypto round-trip and DB init

**Why standalone:** Pure backend, no frontend dependency. Crypto + schema are the foundation everything else builds on.

---

## PR 2 — Rust: Vault commands + auto-lock

**Files affected:** `src/lib.rs`, `src/state.rs` (new, optional)

**What it delivers:**

- `AppState` grows `cipher: Mutex<Option<CipherState>>` + `last_activity: Mutex<Instant>`
- Commands: `create_vault`, `unlock`, `lock`
- Auto-lock timer thread (spawned on unlock, checks `auto_lock_minutes`)
- Commands: `get_config`, `set_config`
- Wire new commands into `run()`

**Why standalone:** The core vault UX loop — create pw → unlock → lock → auto-lock.

---

## PR 3 — Rust: OS Keychain integration

**Files affected:** `Cargo.toml`, `src/keychain.rs` (new), `src/lib.rs`

**What it delivers:**

- Add `keyring` dep
- Commands: `has_keychain_entry`, `save_to_keychain`, `remove_from_keychain`, `unlock_from_keychain`
- Wired into `run()`

**Why standalone:** Additive feature — the app works without keychain. Keeps the keychain API surface and edge cases focused in one review.

---

## PR 4 — Frontend: Design system + layout + vault store/actions

**Files affected:**
```
src/app.css                          (new)
src/routes/+layout.svelte           (new)
src/lib/types.ts                     (new)
src/lib/stores/vault.svelte.ts       (new)
src/lib/actions/vault.ts             (new)
```

**What it delivers:**

- `app.css` — design tokens (CSS custom properties for colors, spacing, typography)
- `+layout.svelte` — app shell (imports `app.css`, renders `<slot/>`)
- `types.ts` — `Account` interface, etc.
- `vault.svelte.ts` — reactive store with `isLocked`, `isInitialized`, `error` runes
- `vault.ts` — typed `invoke()` wrappers for all vault commands

**Why standalone:** Pure frontend infrastructure. No UI components yet — this is the foundation the screens will use.

---

## PR 5 — Frontend: Vault screens + wire up `+page.svelte`

**Files affected:**
```
src/lib/components/SetPasswordScreen.svelte   (new)
src/lib/components/UnlockScreen.svelte        (new)
src/lib/components/KeychainPrompt.svelte      (new)
src/routes/+page.svelte                       (rewrite)
```

**What it delivers:**

- `SetPasswordScreen.svelte` — first-launch master password creation
- `UnlockScreen.svelte` — password entry + biometric button (conditionally visible)
- `KeychainPrompt.svelte` — "Save to keychain?" toggle after unlock
- `+page.svelte` — orchestrator: `is_initialized?` → SetPasswordScreen / UnlockScreen → placeholder after unlock (lock button, "app unlocked" state)

**Why standalone:** The actual user-facing screens. This PR is where the app becomes functional. Review focuses on UX, state management, and screen flow orchestration.

---

## Dependency graph

```
PR 0 (coverage + CI) ──→ all PRs (enforced by CI)

PR 1 (DB + crypto) ───→ PR 2 (vault commands) ──→ PR 3 (keychain)
                                                         │
PR 4 (frontend foundation) ──────────────────────────────┤
                                                         │
                                                         └──→ PR 5 (screens + wire-up)
```

PR 0 is infrastructure — it runs in CI for every PR.
PRs 1–3 are Rust-only and can be reviewed sequentially.
PR 4 (frontend foundation) is independent of PRs 1–3 and can be done in parallel.
PR 5 depends on both PR 3 (for keychain invoke wrappers) and PR 4 (for the store/actions).

---

## Cross-cutting: Compiler warnings as CI errors

CI uses `cargo clippy -- -D clippy::all`, which denies clippy-specific lints
but allows compiler warnings (e.g. `dead_code`) to pass.

After PR 2 wires `crypto.rs` functions into vault commands, the `dead_code`
warnings resolve naturally. At that point CI should be switched to
`-D warnings` to block all compiler warnings from entering the codebase.
