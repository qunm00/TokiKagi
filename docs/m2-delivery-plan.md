# M2 — Accounts: Delivery Plan

> **Goal:** User can add accounts, see live TOTP codes with countdown timers, copy codes by clicking, and delete accounts.
>
> Architecture reference: [`docs/plan.mdx`](./plan.mdx)

## Strategy

M2 is split into **4 PRs**, each:
- Compiles and tests pass independently
- Maintains or improves line coverage
- Is conceptually coherent (one concern per PR)
- Is small enough to review in one sitting (~3–5 files each)
- Builds on previous PRs without leaving dead code

---

## PR 1 — Rust: Accounts DB + commands

**Files affected:** `src-tauri/src/db.rs`, `src-tauri/src/accounts.rs` (new), `src-tauri/src/lib.rs`

**What it delivers:**

- DB migration: add `accounts` table with columns `id`, `label`, `secret`, `algorithm`, `digits`, `period`, `created_at`, `updated_at`
- `accounts.rs` module with three public functions:
  - `add_account(conn, label, secret)` — generate UUID, encrypt secret with AES-256-GCM, insert row, return `Account`
  - `get_accounts(conn)` — decrypt all secrets, return `Vec<Account>`
  - `delete_account(conn, id)` — remove row
- Three new Tauri commands wired into `run()`:
  - `add_account(label, secret)` — encrypt + insert
  - `get_accounts()` — decrypt all
  - `delete_account(id)` — remove
- `last_activity` bumped on each command
- Unit tests for account CRUD round-trip

**Why standalone:** Pure backend, no frontend dependency. The `accounts` table schema and encrypted CRUD are the foundation for all M2 frontend work.

---

## PR 2 — Frontend: Accounts store + actions

**Files affected:**
```
src/lib/types.ts                          (update)
src/lib/stores/accounts.svelte.ts         (new)
src/lib/stores/totp.svelte.ts             (new)
src/lib/actions/accounts.ts               (new)
```

**What it delivers:**

- `types.ts` — `Account` interface with `id`, `label`, `secret`, `algorithm`, `digits`, `period`, `created_at`, `updated_at`
- `accounts.svelte.ts` — reactive store:
  - `accounts: Account[]` — list of decrypted accounts
  - `loadAccounts()` — calls `get_accounts()`, sets accounts
  - `addAccount(label, secret)` — calls `add_account()`, appends to list
  - `deleteAccount(id)` — calls `delete_account()`, removes from list
  - `error` / `loading` runes
- `totp.svelte.ts` — reactive TOTP timer store:
  - `remaining: number` — seconds left in current window
  - `refreshCounter: number` — bump to trigger TOTP regeneration
  - 1-second `setInterval` that decrements `remaining` and bumps `refreshCounter` on window boundary
- `accounts.ts` — typed `invoke()` wrappers: `addAccount`, `getAccounts`, `deleteAccount`

**Why standalone:** Frontend infrastructure for accounts. No UI components yet — stores and actions are the foundation the screens will use.

---

## PR 3 — Frontend: Account display components

**Files affected:**
```
src/lib/components/AccountCard.svelte     (new)
src/lib/components/AccountGrid.svelte     (new)
src/lib/components/CountdownBar.svelte    (new)
src/lib/components/EmptyState.svelte      (new)
```

**What it delivers:**

- `AccountCard.svelte` — single account card:
  - Label, 6-digit TOTP code (monospace, ~2rem)
  - CountdownBar component
  - Click-to-copy (writes to clipboard, brief "Copied" feedback)
  - Delete button visible on hover
  - Focusable (`tabindex="0"`)
- `AccountGrid.svelte` — responsive CSS grid wrapper:
  - `grid-template-columns: repeat(auto-fill, minmax(280px, 1fr))`
  - Accepts `accounts: Account[]` prop
- `CountdownBar.svelte` — Melt `Progress` component:
  - `remaining: number`, `total: number` props
  - Visual bar shrinks as time elapses
  - Color shifts: green → yellow → red as remaining drops
- `EmptyState.svelte` — contextual empty state:
  - "No accounts yet" variant (with prompt to add one)
  - "No results" variant (search returned nothing)
  - `variant: 'empty' | 'no-results'` prop

**Why standalone:** Pure presentational components. No store imports here — props in, markup out. Can be reviewed for layout, accessibility, and styling in isolation.

---

## PR 4 — Frontend: Header + Add dialog + wire up `+page.svelte`

**Files affected:**
```
src/lib/components/Header.svelte           (new)
src/lib/components/AddAccountDialog.svelte (new)
src/routes/+page.svelte                    (rewrite)
src/page.test.ts                           (update)
```

**What it delivers:**

- `Header.svelte` — top bar with:
  - App title "TokiKagi"
  - Account count badge (e.g., "3 accounts")
  - Lock button (calls `vault.lockVault()`)
- `AddAccountDialog.svelte` — Melt `Dialog` builder:
  - Label input (text, required)
  - Secret key input (text, required, placeholder for Base32)
  - Submit / Cancel buttons
  - Validation: both fields required, secret trimmed
  - Calls `accounts.addAccount()` on submit, closes on success
- `+page.svelte` — full rewrite to orchestrate accounts:
  - Import and use all M2 components
  - On mount (after unlock): `accounts.loadAccounts()`
  - 1-second `setInterval` driving `totp` store
  - Render: `Header` → `AccountGrid` (or `EmptyState`) → `AddAccountDialog`
  - TOTP code refresh: when `totp.refreshCounter` changes, regenerate codes via `generate_totp()` for each account
  - Lock button gracefully returns to `UnlockScreen`
- `page.test.ts` — update tests for the new unlocked state

**Why standalone:** The integration layer. This PR wires all M2 components together with the vault flow and TOTP timer logic. Review focuses on orchestration, data flow, and timing correctness.

---

## Dependency graph

```
PR 1 (Rust accounts) ──→ PR 2 (frontend store + actions)
                                │
                                ├──→ PR 3 (display components)
                                │
                                └──→ PR 4 (dialog + header + wire-up)
                                          │
                                     (depends on PR 3 for components)
```

PR 1 is Rust-only — DB schema and encrypted CRUD.
PR 2 is frontend infrastructure — stores and invoke wrappers.
PR 3 is pure presentational — can be built against the `Account` interface without the real store.
PR 4 depends on PR 2 (for accounts store) and PR 3 (for components) — the integration layer.

---

## Cross-cutting: TOTP timing strategy

The 30-second TOTP window is managed entirely on the frontend:

| Timing | Mechanism | Purpose |
|---|---|---|
| Every 1s | `setInterval` in `totp.svelte.ts` | Decrement `remaining`, detect window boundary |
| Every 30s (window boundary) | `refreshCounter` increment | Signal components to regenerate codes |
| On `refreshCounter` change | `$effect` in `+page.svelte` | Call `generate_totp(secret)` for each account |

Only `generate_totp` and `get_time_remaining` require IPC to Rust. The 1s tick is pure JS. This minimizes IPC calls (1 per account per 30s instead of 1 per account per second).
