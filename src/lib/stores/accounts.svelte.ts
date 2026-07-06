import type { Account } from "$lib/types";
import * as actions from "$lib/actions/accounts";
import { parseVaultError } from "$lib/types";

function createAccountsStore() {
  let accounts = $state<Account[]>([]);
  let error = $state<string | null>(null);
  let loading = $state<boolean>(false);

  async function loadAccounts(): Promise<void> {
    loading = true;
    error = null;
    try {
      accounts = await actions.getAccounts();
    } catch (err) {
      error = parseVaultError(err).message;
    } finally {
      loading = false;
    }
  }

  async function addAccount(label: string, secret: string): Promise<void> {
    loading = true;
    error = null;
    try {
      const account = await actions.addAccount(label, secret);
      accounts = [...accounts, account];
    } catch (err) {
      error = parseVaultError(err).message;
      throw err;
    } finally {
      loading = false;
    }
  }

  async function deleteAccount(id: string): Promise<void> {
    loading = true;
    error = null;
    try {
      await actions.deleteAccount(id);
      accounts = accounts.filter((a) => a.id !== id);
    } catch (err) {
      error = parseVaultError(err).message;
      throw err;
    } finally {
      loading = false;
    }
  }

  function clearError(): void {
    error = null;
  }

  return {
    get accounts() {
      return accounts;
    },
    get error() {
      return error;
    },
    get loading() {
      return loading;
    },
    loadAccounts,
    addAccount,
    deleteAccount,
    clearError,
  };
}

export const accountsStore = createAccountsStore();
