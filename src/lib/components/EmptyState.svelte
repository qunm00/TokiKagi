<script lang="ts">
let {
  variant = "empty",
  onAddAccount,
}: {
  variant?: "empty" | "no-results";
  onAddAccount?: () => void;
} = $props();
</script>

<div class="empty-state" role="status">
  {#if variant === "empty"}
    <svg
      class="icon"
      xmlns="http://www.w3.org/2000/svg"
      width="48"
      height="48"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M12 2L2 7l10 5 10-5-10-5z" />
      <path d="M2 17l10 5 10-5" />
      <path d="M2 12l10 5 10-5" />
    </svg>
    <h2 class="title">No accounts yet</h2>
    <p class="description">
      Add your first account to start generating TOTP codes.
    </p>
    {#if onAddAccount}
      <button class="btn-primary" onclick={onAddAccount}>
        Add Account
      </button>
    {/if}
  {:else if variant === "no-results"}
    <svg
      class="icon"
      xmlns="http://www.w3.org/2000/svg"
      width="48"
      height="48"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
    <h2 class="title">No results</h2>
    <p class="description">
      No accounts match your search. Try a different term.
    </p>
    {#if onAddAccount}
      <button class="btn-primary" onclick={onAddAccount}>
        Add Account
      </button>
    {/if}
  {/if}
</div>

<style>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--spacing-3);
  padding: var(--spacing-12) var(--spacing-4);
  text-align: center;
}

.icon {
  color: var(--color-text-muted);
  margin-bottom: var(--spacing-2);
}

.title {
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text);
}

.description {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  max-width: 280px;
  line-height: var(--line-height);
}

.btn-primary {
  margin-top: var(--spacing-2);
  padding: var(--spacing-2) var(--spacing-4);
  background: var(--color-primary);
  color: var(--color-primary-text);
  border: none;
  border-radius: var(--radius-base);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-primary:hover {
  background: var(--color-primary-hover);
}

.btn-primary:active {
  background: var(--color-primary-active);
}
</style>
