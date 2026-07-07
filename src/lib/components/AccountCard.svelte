<script lang="ts">
import type { Account } from "$lib/types";

let {
  account,
  totpCode,
  totpRemaining,
  onDelete,
  onCopy,
  onCopyError,
  copiedLabel,
}: {
  account: Account;
  totpCode: string;
  totpRemaining: number;
  onDelete?: (id: string) => void;
  onCopy?: (code: string) => void;
  onCopyError?: (err: unknown) => void;
  copiedLabel?: string | null;
} = $props();

let copied = $state(false);
let showDelete = $state(false);

let total = $derived(account.period || 30);
let percent = $derived(total > 0 ? (totpRemaining / total) * 100 : 0);
let circleColor = $derived(
  percent > 50
    ? "var(--color-success)"
    : percent > 20
      ? "var(--color-warning)"
      : "var(--color-danger)",
);
const circumference = 100;

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

function handleDelete(e: MouseEvent) {
  e.stopPropagation();
  onDelete?.(account.id);
}
</script>

<div
  class="card"
  tabindex="0"
  role="button"
  aria-label="{account.label}, TOTP code"
  onclick={handleCopy}
  onkeydown={handleKeydown}
  onmouseenter={() => (showDelete = true)}
  onmouseleave={() => (showDelete = false)}
  onfocus={() => (showDelete = true)}
  onblur={() => (showDelete = false)}
>
  <div class="card-header">
    <span class="label">{account.label}</span>
    <div class="header-actions">
      <svg viewBox="0 0 36 36" class="countdown-circle">
        <circle
          cx="18" cy="18" r="15.915"
          fill="none"
          stroke="var(--color-bg-tertiary)"
          stroke-width="4"
        />
        <circle
          cx="18" cy="18" r="15.915"
          fill="none"
          stroke={circleColor}
          stroke-width="4"
          stroke-dasharray={circumference}
          stroke-dashoffset={circumference - (circumference * percent) / 100}
          stroke-linecap="round"
          transform="rotate(-90 18 18)"
        />
      </svg>
      {#if showDelete && onDelete}
        <button
          class="delete-btn"
          onclick={handleDelete}
          aria-label="Delete {account.label}"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
          </svg>
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
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  cursor: pointer;
  transition: border-color var(--transition-fast), box-shadow var(--transition-fast);
  outline: none;
}

.card:hover {
  border-color: var(--color-border-hover);
  box-shadow: var(--shadow-base);
}

.card:focus-visible {
  border-color: var(--color-primary);
  box-shadow: 0 0 0 2px var(--color-focus-ring);
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

.delete-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--color-text-muted);
  cursor: pointer;
  flex-shrink: 0;
  transition: background var(--transition-fast), color var(--transition-fast);
}

.delete-btn:hover {
  background: rgba(220, 38, 38, 0.1);
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

.countdown-circle {
  width: 24px;
  height: 24px;
  display: block;
}

.countdown-circle circle {
  transition: stroke-dashoffset 1s linear, stroke 1s linear;
}
</style>
