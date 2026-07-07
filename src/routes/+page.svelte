<script lang="ts">
import { onMount } from "svelte";
import * as vaultActions from "$lib/actions/vault";
import AccountCard from "$lib/components/AccountCard.svelte";
import AccountGrid from "$lib/components/AccountGrid.svelte";
import AddAccountDialog from "$lib/components/AddAccountDialog.svelte";
import EmptyState from "$lib/components/EmptyState.svelte";
import Header from "$lib/components/Header.svelte";
import SetPasswordScreen from "$lib/components/SetPasswordScreen.svelte";
import UnlockScreen from "$lib/components/UnlockScreen.svelte";
import { accountsStore } from "$lib/stores/accounts.svelte";
import { totp } from "$lib/stores/totp.svelte";
import { vault } from "$lib/stores/vault.svelte";

let checking = $state(true);
let addDialogOpen = $state(false);
let totpCodes = $state<Record<string, string>>({});
let unlocked = $state(false);

onMount(() => {
  vault.checkInitialized().finally(() => {
    checking = false;
  });
  return () => {
    totp.stop();
  };
});

$effect(() => {
  if (checking || vault.isLocked || !vault.isInitialized) {
    totp.stop();
    totpCodes = {};
    unlocked = false;
    return;
  }

  if (!unlocked) {
    unlocked = true;
    totp.start();
    accountsStore.loadAccounts();
  }

  void totp.refreshCounter;
  const count = accountsStore.accounts.length;

  if (count > 0) {
    refreshAllCodes();
  }
});

async function refreshAllCodes() {
  const codes: Record<string, string> = {};
  for (const account of accountsStore.accounts) {
    try {
      codes[account.id] = await vaultActions.generateTotp(account.secret);
    } catch (err) {
      console.error("Failed to generate TOTP for account", account.id, err);
      codes[account.id] = "";
    }
  }
  totpCodes = codes;
}

async function handleDelete(id: string) {
  await accountsStore.deleteAccount(id);
}

function handleAccountAdded() {
  // accountsStore.accounts already updated by the store;
  // the effect will regenerate codes
}

function handleLock() {
  vault.lockVault();
}

function handleCopyError(err: unknown) {
  console.error("Clipboard copy failed:", err);
}
</script>

{#if checking}
  <div class="loading-screen">
    <p class="loading-text">Loading…</p>
  </div>
{:else if !vault.isInitialized}
  <SetPasswordScreen onCreated={() => {}} />
{:else if vault.isLocked}
  <UnlockScreen onUnlock={() => {}} />
{:else}
  <div class="app-layout">
    <Header accountCount={accountsStore.accounts.length} onLock={handleLock} />

    <main class="main">
      {#if accountsStore.accounts.length > 0}
        <div class="toolbar">
          <button class="btn-add" onclick={() => (addDialogOpen = true)}>
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
              <path d="M12 5v14" />
              <path d="M5 12h14" />
            </svg>
            Add Account
          </button>
        </div>

        <AccountGrid accounts={accountsStore.accounts}>
          {#snippet children(account)}
            <AccountCard
              {account}
              totpCode={totpCodes[account.id] ?? ""}
              totpRemaining={totp.remaining}
              onDelete={handleDelete}
              onCopyError={handleCopyError}
            />
          {/snippet}
        </AccountGrid>
      {:else}
        <EmptyState variant="empty" onAddAccount={() => (addDialogOpen = true)} />
      {/if}
    </main>

    <AddAccountDialog
      open={addDialogOpen}
      onOpenChange={(v) => (addDialogOpen = v)}
      onAccountAdded={handleAccountAdded}
    />
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

.app-layout {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.main {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: var(--spacing-3) var(--spacing-4);
  border-bottom: 1px solid var(--color-border);
}

.btn-add {
  display: inline-flex;
  align-items: center;
  gap: var(--spacing-1);
  padding: var(--spacing-1) var(--spacing-3);
  background: var(--color-primary);
  color: var(--color-primary-text);
  border: none;
  border-radius: var(--radius-base);
  font-size: var(--font-size-sm);
  font-weight: var(--font-weight-medium);
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-add:hover {
  background: var(--color-primary-hover);
}

.btn-add:active {
  background: var(--color-primary-active);
}
</style>
