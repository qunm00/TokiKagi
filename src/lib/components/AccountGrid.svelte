<script lang="ts">
import type { Snippet } from "svelte";
import { fade, fly } from "svelte/transition";
import type { Account } from "$lib/types";

let {
  accounts,
  children,
}: {
  accounts: Account[];
  children: Snippet<[Account]>;
} = $props();
</script>

<div class="grid">
  {#each accounts as account (account.id)}
    <div class="item" in:fly={{ y: 12, duration: 200 }} out:fade={{ duration: 120 }}>
      {@render children(account)}
    </div>
  {/each}
</div>

<style>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-4);
  padding: var(--spacing-4);
}

.item {
  min-width: 0;
}
</style>
