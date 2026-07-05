<script lang="ts">
import { onMount } from "svelte";
import { hasKeychainEntry } from "$lib/actions/vault";
import { vault } from "$lib/stores/vault.svelte";

let {
  onUnlock = () => {},
}: {
  onUnlock?: () => void;
} = $props();

let password = $state("");
let showPassword = $state(false);
let hasKeychain = $state(false);
let showKeychainPrompt = $state(false);

onMount(async () => {
  try {
    hasKeychain = await hasKeychainEntry();
  } catch {
    hasKeychain = false;
  }
});

async function handleSubmit(e: Event) {
  e.preventDefault();
  if (!password) return;
  try {
    await vault.unlockVault(password);
    if (!hasKeychain) {
      showKeychainPrompt = true;
    } else {
      password = "";
      onUnlock();
    }
  } catch {
    // error is set in store
  }
}

async function handleKeychainSave() {
  try {
    await vault.unlockAndSaveToKeychain(password);
    password = "";
    showKeychainPrompt = false;
    onUnlock();
  } catch {
    // error is set in store
  }
}

function handleKeychainSkip() {
  password = "";
  showKeychainPrompt = false;
  onUnlock();
}

async function handleBiometric() {
  try {
    await vault.unlockFromKeychain();
    onUnlock();
  } catch {
    // error is set in store
  }
}
</script>

{#if showKeychainPrompt}
  <div class="screen">
    <div class="card">
      <h2 class="title">Save to Keychain?</h2>
      <p class="description">
        Would you like to save your master password to the system keychain?
        You'll be able to unlock with Touch ID next time.
      </p>

      {#if vault.error}
        <div class="error" role="alert">{vault.error}</div>
      {/if}

      <div class="actions">
        <button
          type="button"
          class="btn-secondary"
          onclick={handleKeychainSkip}
          disabled={vault.loading}
        >
          Skip
        </button>
        <button
          type="button"
          class="btn-primary"
          onclick={handleKeychainSave}
          disabled={vault.loading}
        >
          {vault.loading ? "Saving…" : "Save to Keychain"}
        </button>
      </div>
    </div>
  </div>
{:else}
  <form class="screen" onsubmit={handleSubmit}>
    <div class="card">
      <h1 class="title">Unlock Vault</h1>

      {#if vault.error}
        <div class="error" role="alert">{vault.error}</div>
      {/if}

      <div class="field">
        <label for="password" class="label">Master Password</label>
        <div class="input-wrapper">
          <input
            id="password"
            type={showPassword ? "text" : "password"}
            bind:value={password}
            placeholder="Enter master password"
            autocomplete="current-password"
            disabled={vault.loading}
          />
          <button
            type="button"
            class="toggle-visibility"
            onclick={() => (showPassword = !showPassword)}
            aria-label={showPassword ? "Hide password" : "Show password"}
          >
            {showPassword ? "Hide" : "Show"}
          </button>
        </div>
      </div>

      <button type="submit" class="btn-primary" disabled={vault.loading || !password}>
        {vault.loading ? "Unlocking…" : "Unlock"}
      </button>

      {#if hasKeychain}
        <div class="divider">
          <span class="divider-text">or</span>
        </div>
        <button
          type="button"
          class="btn-bio"
          onclick={handleBiometric}
          disabled={vault.loading}
        >
          Unlock with Touch ID
        </button>
      {/if}
    </div>
  </form>
{/if}

<style>
.screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-4);
}

.card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  padding: var(--spacing-8);
  width: 100%;
  max-width: 400px;
  display: flex;
  flex-direction: column;
  gap: var(--spacing-4);
}

.title {
  font-size: var(--font-size-2xl);
  font-weight: var(--font-weight-bold);
  text-align: center;
  color: var(--color-text);
}

.error {
  background: rgba(220, 38, 38, 0.1);
  color: var(--color-danger);
  border: 1px solid var(--color-danger);
  border-radius: var(--radius-base);
  padding: var(--spacing-2) var(--spacing-3);
  font-size: var(--font-size-sm);
  text-align: center;
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

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.input-wrapper input {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-3);
  padding-right: 3.5rem;
  border: 1px solid var(--color-border);
  border-radius: var(--radius-base);
  background: var(--color-bg);
  color: var(--color-text);
  font-size: var(--font-size-base);
  transition: border-color var(--transition-fast);
}

.input-wrapper input:focus {
  border-color: var(--color-primary);
  outline: none;
  box-shadow: 0 0 0 2px var(--color-focus-ring);
}

.toggle-visibility {
  position: absolute;
  right: var(--spacing-2);
  background: none;
  border: none;
  color: var(--color-text-secondary);
  font-size: var(--font-size-xs);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  padding: var(--spacing-1);
  border-radius: var(--radius-sm);
  transition: color var(--transition-fast);
}

.toggle-visibility:hover {
  color: var(--color-text);
}

.btn-primary {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-4);
  background: var(--color-primary);
  color: var(--color-primary-text);
  border: none;
  border-radius: var(--radius-base);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

.btn-primary:active:not(:disabled) {
  background: var(--color-primary-active);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.divider {
  display: flex;
  align-items: center;
  gap: var(--spacing-2);
}

.divider::before,
.divider::after {
  content: "";
  flex: 1;
  height: 1px;
  background: var(--color-border);
}

.divider-text {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  text-transform: uppercase;
}

.description {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  text-align: center;
  line-height: var(--line-height);
}

.actions {
  display: flex;
  gap: var(--spacing-3);
}

.actions button {
  flex: 1;
}

.btn-secondary {
  padding: var(--spacing-2) var(--spacing-4);
  background: var(--color-bg-secondary);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-base);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--color-bg-tertiary);
  border-color: var(--color-border-hover);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-bio {
  width: 100%;
  padding: var(--spacing-2) var(--spacing-4);
  background: var(--color-bg-secondary);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-base);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--transition-fast), border-color var(--transition-fast);
}

.btn-bio:hover:not(:disabled) {
  background: var(--color-bg-tertiary);
  border-color: var(--color-border-hover);
}

.btn-bio:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
