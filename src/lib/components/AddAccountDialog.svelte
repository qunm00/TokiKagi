<script lang="ts">
import Alert from "$lib/components/Alert.svelte";
import { Dialog } from "melt/builders";
import { accountsStore } from "$lib/stores/accounts.svelte";

let {
  open,
  onOpenChange,
  onAccountAdded,
}: {
  open: boolean;
  onOpenChange: (v: boolean) => void;
  onAccountAdded?: () => void;
} = $props();

const dialog = new Dialog({
  open: () => open,
  onOpenChange(v) {
    onOpenChange(v);
  },
  closeOnEscape: true,
  closeOnOutsideClick: true,
});

$effect(() => {
  dialog.open = open;
});

let label = $state("");
let secret = $state("");
let error = $state<string | null>(null);
let submitting = $state(false);

async function handleSubmit(e: Event) {
  e.preventDefault();
  const trimmedLabel = label.trim();
  const trimmedSecret = secret.trim();
  if (!trimmedLabel || !trimmedSecret) return;
  submitting = true;
  error = null;
  try {
    await accountsStore.addAccount(trimmedLabel, trimmedSecret);
    label = "";
    secret = "";
    onAccountAdded?.();
    dialog.open = false;
  } catch {
    error = accountsStore.error;
  } finally {
    submitting = false;
  }
}

function handleCancel() {
  dialog.open = false;
}
</script>

<dialog {...dialog.content} class="dialog">
  <form class="form" onsubmit={handleSubmit}>
    <h2 class="title">Add Account</h2>

    {#if error}
      <Alert onDismiss={() => (error = null)}>{error}</Alert>
    {/if}

    <div class="field">
      <label for="label-input" class="label">Label</label>
      <input
        id="label-input"
        type="text"
        bind:value={label}
        placeholder="e.g. GitHub"
        required
        disabled={submitting}
        class="input"
      />
    </div>

    <div class="field">
      <label for="secret-input" class="label">Secret Key</label>
      <input
        id="secret-input"
        type="text"
        bind:value={secret}
        placeholder="Base32 encoded secret"
        required
        disabled={submitting}
        class="input"
        autocomplete="off"
      />
    </div>

    <div class="actions">
      <button
        type="button"
        class="btn-cancel"
        onclick={handleCancel}
        disabled={submitting}
      >
        Cancel
      </button>
      <button
        type="submit"
        class="btn-submit"
        disabled={submitting || !label.trim() || !secret.trim()}
      >
        {submitting ? "Adding…" : "Add Account"}
      </button>
    </div>
  </form>
</dialog>

<style>
.dialog {
  padding: 0;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  background: var(--color-surface);
  width: 100%;
  max-width: 400px;
  margin: auto;
}

.dialog::backdrop {
  background: rgba(0, 0, 0, 0.3);
}

.form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
  padding: var(--spacing-6);
}

.title {
  font-size: var(--font-size-xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-text);
}

.field {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-1);
}

.label {
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  color: var(--color-text);
}

.input {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-3);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-base);
  background: var(--color-bg);
  color: var(--color-text);
  font-size: var(--font-size-base);
  font-family: var(--font-family);
  transition: border-color var(--transition-fast);
}

.input:focus {
  border-color: var(--color-primary);
  outline: none;
  box-shadow: 0 0 0 2px var(--color-focus-ring);
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-2);
  margin-top: var(--spacing-2);
}

.btn-cancel {
  padding: var(--spacing-2) var(--spacing-4);
  background: var(--color-bg);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-base);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: border-color var(--transition-fast);
}

.btn-cancel:hover:not(:disabled) {
  border-color: var(--color-border-hover);
}

.btn-submit {
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

.btn-submit:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.btn-submit:active:not(:disabled) {
  background: var(--color-primary-active);
}

.btn-submit:disabled,
.btn-cancel:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
