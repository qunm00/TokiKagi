# M3 — Search & Keyboard Copy: Delivery Plan

> **Goal:** User can filter accounts by typing, auto-copy when narrowed to
> one result, and use Ctrl+C on a focused card.
>
> Architecture reference: [`docs/plan.mdx`](./plan.mdx)

## Strategy

M3 is split into **2 PRs**, each:
- Compiles and tests pass independently
- Is conceptually coherent (one concern per PR)
- Is small enough to review in one sitting (~2–3 files each)
- Builds on the previous PR without leaving dead code

---

## PR 1 — Search bar + filtering

**Files affected:**
```
src/lib/components/SearchBar.svelte   (new)
src/routes/+page.svelte               (edit)
```

**What it delivers:**

- `SearchBar.svelte`:
  - Text input with magnifying glass icon
  - Clear button (X) visible when `query` is non-empty
  - `query: string` prop, bound via `bind:query` from parent
- `+page.svelte`:
  - `query` state variable (`$state`)
  - `filtered` derived list: `$derived(accounts.filter(a => a.label.toLowerCase().includes(query.toLowerCase())))`
  - `<SearchBar bind:query />` rendered in the toolbar area
  - `AccountGrid` iterates over `filtered` instead of `accounts`
  - `EmptyState` switches to `"no-results"` variant when `query` is non-empty and `filtered` is empty

**Why standalone:** SearchBar is a self-contained presentational component. Filtering logic lives in `+page.svelte` (orchestrator). No store changes needed. Can be tested independently.

**Dependencies:** None beyond existing M2 components.

---

## PR 2 — Auto-copy on single result

**Files affected:**
```
src/routes/+page.svelte               (edit)
src/lib/components/AccountCard.svelte (edit)
```

**What it delivers:**

- When `filtered.length === 1`, auto-copy that account's TOTP code to clipboard via `navigator.clipboard.writeText()`
- 'Copied!' flash with label: AccountCard badge shows `"Copied {label}"` instead of generic `"Copied!"`
- Flash auto-dismisses after 3 seconds (matching the existing copy feedback timeout)
- Snapshot test for auto-copy behavior (Vitest + jsdom, mock `navigator.clipboard`)

**Why standalone:** Pure UX enhancement on top of the search infrastructure from PR 1. Zero new components — only wiring and feedback polish.

**Dependencies:** PR 1 (needs `filtered` list to exist).

---

## Dependency graph

```
PR 1 (SearchBar + filter) ──→ PR 2 (auto-copy + feedback)
```

PR 1 is the infrastructure — component + filtering logic.
PR 2 enhances the UX once the filter data is available.

---

## What's already in place (from M2)

| Feature | Status |
|---|---|
| `AccountCard` focusable (`tabindex="0"`) | ✅ Done |
| `Ctrl+C` handler on focused card | ✅ Done |
| Focus ring styles | ✅ Done |
| `EmptyState` "empty" / "no-results" variants | ✅ Done |
