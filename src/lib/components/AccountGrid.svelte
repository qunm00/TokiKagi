<script lang="ts">
import type { Snippet } from "svelte";
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
    {@render children(account)}
  {/each}
</div>

<style>
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-4);
  padding: var(--spacing-4);
}
</style>
