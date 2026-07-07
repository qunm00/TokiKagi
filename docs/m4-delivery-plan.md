# M4 — Polish: Delivery Plan

> **Goal:** The app feels polished with smooth transitions, proper dark mode,
> and graceful error handling for all edge cases.
>
> Architecture reference: [`docs/plan.mdx`](./plan.mdx)

## Strategy

M4 is split into **4 PRs**, each:
- Is conceptually coherent (one concern per PR)
- Is small enough to review in one sitting (~1–3 files each)
- Builds on previous PRs without conflicts

---

## PR 1 — Window sizing + minimums

**Files affected:**
```
src-tauri/tauri.conf.json   (edit)
```

**What it delivers:**

- Set `minWidth: 480` and `minHeight: 400` in the Tauri window config
- Prevents the window from being shrunk to a broken state

**Why standalone:** Single config change, zero risk, immediate impact.

---

## PR 2 — Screen transitions

**Files affected:**
```
src/routes/+page.svelte                   (edit)
src/lib/components/UnlockScreen.svelte    (edit)
src/lib/components/SetPasswordScreen.svelte (edit)
```

**What it delivers:**

- Svelte `transition:fly` on the locked/unlocked screen switches in `+page.svelte`
- Svelte `transition:fade` or `transition:fly` on `UnlockScreen` and `SetPasswordScreen` themselves

**Why standalone:** Pure presentational — no logic changes.

---

## PR 3 — Empty search action + Add/remove card transitions

**Files affected:**
```
src/lib/components/EmptyState.svelte   (edit)
src/routes/+page.svelte                (edit)
```

**What it delivers:**

- `EmptyState` "no-results" variant gets an "Add Account" button (same as the "empty" variant) so users can add an account even while searching
- `{#key filtered}` block in `+page.svelte` to animate account cards in/out of the grid as the filter changes
- Brief `transition:fly` on account cards when they appear/disappear from the grid

**Why standalone:** UX improvements for the search flow introduced in M3.

---

## PR 4 — Error state enhancements

**Files affected:**
```
src/lib/components/UnlockScreen.svelte       (edit)
src/lib/components/SetPasswordScreen.svelte  (edit)
src/lib/components/AddAccountDialog.svelte   (edit)
```

**What it delivers:**

- Dismiss (X) button on error banners in all three components
- Visual distinction between validation errors and server errors
- Ensure error copy is user-friendly (not raw Rust error strings)

**Why standalone:** Error handling polish, no visual/structural changes elsewhere.

---

## Dependency graph

```
PR 1 (window sizing)
PR 2 (screen transitions)
PR 3 (empty search + card animations) ── depends on M3
PR 4 (error state polish)
```

All four PRs are independent of each other — they can be done in any order.

---

## Out of scope (already done or deferred)

| Area | Status |
|---|---|
| Dark mode (OS-level `prefers-color-scheme`) | ✅ Already fully implemented in `app.css` |
| Loading states (button text + disable) | ✅ Already in all 3 forms |
| Manual dark mode toggle | ❌ Deferred — OS-level coverage is sufficient for now |
| Skeleton loading / spinner animations | ❌ Deferred — low impact vs effort |
| Inline field errors | ❌ Deferred — banner-level errors are adequate for this app |
| `prefers-reduced-motion` / `prefers-contrast` | ❌ Deferred — niche accessibility concerns |
| Loading states for search filtering | ❌ Filtering is instant/local, no loading needed |
