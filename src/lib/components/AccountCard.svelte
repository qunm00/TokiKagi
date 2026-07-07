<script lang="ts">
import type { Account } from "$lib/types";

let {
  account,
  totpCode,
  totpRemaining,
  editing,
  selected,
  onCopy,
  onCopyError,
  copiedLabel,
  onToggleSelect,
}: {
  account: Account;
  totpCode: string;
  totpRemaining: number;
  editing?: boolean;
  selected?: boolean;
  onCopy?: (code: string) => void;
  onCopyError?: (err: unknown) => void;
  copiedLabel?: string | null;
  onToggleSelect?: (id: string) => void;
} = $props();

let copied = $state(false);

let total = $derived(account.period || 30);
let percent = $derived(total > 0 ? (totpRemaining / total) * 100 : 0);
let barColor = $derived(
  percent > 50
    ? "var(--color-success)"
    : percent > 20
      ? "var(--color-warning)"
      : "var(--color-danger)",
);

function formatCode(code: string): string {
  const digits = account.digits || 6;
  const clean = code.replace(/\s/g, "").slice(0, digits);
  return `${clean.slice(0, 3)} ${clean.slice(3)}`;
}

async function handleCopy() {
  try {
    await navigator.clipboard.writeText(totpCode);
    copied = true;
    onCopy?.(totpCode);
    setTimeout(() => (copied = false), 3000);
  } catch (err) {
    onCopyError?.(err);
  }
}

function handleKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === "c") {
    e.preventDefault();
    handleCopy();
  }
}
</script>

<div
  class="card"
  class:editing
  tabindex="0"
  role="button"
  aria-label="{account.label}, TOTP code"
  onclick={handleCopy}
  onkeydown={handleKeydown}
>
  <div class="top-bar" style="width: {percent}%; background: {barColor};"></div>
  <div class="card-header">
    <span class="label">{account.label}</span>
    <div class="header-actions">
      {#if editing}
        <button
          class="select-btn"
          class:selected
          onclick={(e) => { e.stopPropagation(); onToggleSelect?.(account.id); }}
          aria-label="{selected ? 'Deselect' : 'Select'} {account.label}"
        >
          {#if selected}
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/></svg>
          {/if}
        </button>
      {/if}
    </div>
  </div>

  <div class="code-wrapper">
    <span class="code">{formatCode(totpCode)}</span>
    {#if copied || copiedLabel === account.label}
      <span class="copied-badge">Copied {account.label}!</span>
    {/if}
  </div>

</div>

<style>
.card {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
  padding: var(--spacing-4);
  padding-top: calc(var(--spacing-4) + 4px);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  outline: none;
  position: relative;
  overflow: hidden;
}

.card:hover {
  border-color: var(--color-border-hover);
  box-shadow: var(--shadow-base);
}

.card:focus-visible {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-focus-ring);
}

.card.editing {
  border-color: var(--color-primary);
}

.top-bar {
  position: absolute;
  top: 0;
  left: 0;
  height: 4px;
  transition: width 1s linear;
  border-radius: var(--radius-lg) var(--radius-lg) 0 0;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-2);
}

.label {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
  flex-shrink: 0;
}

.select-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.select-btn:hover {
  background: rgba(220, 38, 38, 0.1);
  color: var(--color-danger);
}

.select-btn.selected {
  color: var(--color-danger);
}

.code-wrapper {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
}

.code {
  font-family: var(--font-family-mono);
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  letter-spacing: 0.1em;
  color: var(--color-text);
  user-select: all;
}

.copied-badge {
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  color: var(--color-success);
  padding: var(--spacing-1) var(--spacing-2);
  border-radius: var(--radius-sm);
  animation: fadeIn 150ms ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-2px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
