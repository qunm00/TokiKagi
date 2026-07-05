<script lang="ts">
import { onMount } from "svelte";
import SetPasswordScreen from "$lib/components/SetPasswordScreen.svelte";
import UnlockScreen from "$lib/components/UnlockScreen.svelte";
import { vault } from "$lib/stores/vault.svelte";

let checking = $state(true);

onMount(() => {
  vault.checkInitialized().finally(() => {
    checking = false;
  });
});

function handleCreated() {
  // vault.isLocked becomes false, vault.isInitialized becomes true
}

function handleUnlock() {
  // vault.isLocked becomes false, re-renders automatically
}
</script>

{#if checking}
  <div class="loading-screen">
    <p class="loading-text">Loading…</p>
  </div>
{:else if !vault.isInitialized}
  <SetPasswordScreen onCreated={handleCreated} />
{:else if vault.isLocked}
  <UnlockScreen onUnlock={handleUnlock} />
{:else}
  <div class="screen">
    <div class="placeholder">
      <h1 class="placeholder-title">TokiKagi</h1>
      <p class="placeholder-subtitle">Vault is unlocked</p>
      <button class="btn-lock" onclick={() => vault.lockVault()}>
        Lock Vault
      </button>
    </div>
  </div>
{/if}

<style>
.loading-screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.loading-text {
  font-size: var(--font-size-lg);
  color: var(--color-text-secondary);
}

.screen {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: var(--spacing-4);
}

.placeholder {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-4);
}

.placeholder-title {
  font-size: var(--font-size-3xl);
  font-weight: var(--font-weight-bold);
  color: var(--color-text);
}

.placeholder-subtitle {
  font-size: var(--font-size-lg);
  color: var(--color-text-secondary);
}

.btn-lock {
  padding: var(--spacing-2) var(--spacing-6);
  background: var(--color-danger);
  color: white;
  border: none;
  border-radius: var(--radius-base);
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-lock:hover {
  background: var(--color-danger-hover);
}
</style>
