<script lang="ts">
import type { Snippet } from "svelte";

let {
  variant = "error",
  onDismiss,
  children,
}: {
  variant?: "error" | "warning";
  onDismiss?: () => void;
  children?: Snippet;
} = $props();
</script>

<div class="alert" class:warning={variant === "warning"} role="alert">
  <span class="message">{@render children()}</span>
  {#if onDismiss}
    <button class="dismiss" onclick={onDismiss} aria-label="Dismiss error">
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
    </button>
  {/if}
</div>

<style>
.alert {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-2);
  background: rgba(220, 38, 38, 0.1);
  color: var(--color-danger);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-base);
  padding: var(--spacing-2) var(--spacing-3);
  font-size: var(--font-size-sm);
  text-align: center;
}

.alert.warning {
  background: rgba(245, 158, 11, 0.1);
  color: var(--color-warning);
  border-color: var(--color-warning);
}

.message {
  flex: 1;
}

.dismiss {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: currentColor;
  cursor: pointer;
  flex-shrink: 0;
  opacity: 0.7;
}

.dismiss:hover {
  opacity: 1;
}
</style>
