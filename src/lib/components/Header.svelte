<script lang="ts">
let {
  accountCount = 0,
  onLock,
}: {
  accountCount?: number;
  onLock?: () => void;
} = $props();
</script>

<header class="header">
  <h1 class="title">TokiKagi</h1>
  <div class="right">
    <span class="count">{accountCount} {accountCount === 1 ? "account" : "accounts"}</span>
    {#if onLock}
      <button class="btn-lock" onclick={onLock} aria-label="Lock vault">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="18"
          height="18"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
          <path d="M7 11V7a5 5 0 0 1 10 0v4" />
        </svg>
      </button>
    {/if}
  </div>
</header>

<style>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-3) var(--spacing-4);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.title {
  font-size: var(--font-size-lg);
  font-weight: var(--font-weight-bold);
  color: var(--color-text);
}

.right {
  display: flex;
  align-items: center;
  gap: var(--spacing-3);
}

.count {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-variant-numeric: tabular-nums;
}

.btn-lock {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-base);
  background: var(--color-bg);
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: border-color var(--transition-fast), color var(--transition-fast);
}

.btn-lock:hover {
  border-color: var(--color-border-hover);
  color: var(--color-text);
}
</style>
