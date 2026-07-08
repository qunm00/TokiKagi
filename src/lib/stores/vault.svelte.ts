import { listen } from "@tauri-apps/api/event";
import * as actions from "$lib/actions/vault";
import { parseVaultError } from "$lib/types";

function createVaultStore() {
  let isInitialized = $state<boolean>(false);
  let isLocked = $state<boolean>(true);
  let error = $state<string | null>(null);
  let loading = $state<boolean>(false);

  async function checkInitialized(): Promise<void> {
    try {
      isInitialized = await actions.isInitialized();
    } catch (err) {
      error = parseVaultError(err).message;
    }
  }

  async function createVault(password: string): Promise<void> {
    loading = true;
    error = null;
    try {
      await actions.createVault(password);
      isInitialized = true;
      isLocked = false;
    } catch (err) {
      error = parseVaultError(err).message;
      throw err;
    } finally {
      loading = false;
    }
  }

  async function unlockVault(password: string): Promise<void> {
    loading = true;
    error = null;
    try {
      await actions.unlock(password);
      isLocked = false;
    } catch (err) {
      error = parseVaultError(err).message;
      throw err;
    } finally {
      loading = false;
    }
  }

  async function lockVault(): Promise<void> {
    loading = true;
    error = null;
    try {
      await actions.lock();
      isLocked = true;
    } catch (err) {
      error = parseVaultError(err).message;
    } finally {
      loading = false;
    }
  }

  function clearError(): void {
    error = null;
  }

  listen("vault-locked", () => {
    isLocked = true;
  });

  return {
    get isInitialized() {
      return isInitialized;
    },
    get isLocked() {
      return isLocked;
    },
    get error() {
      return error;
    },
    get loading() {
      return loading;
    },
    checkInitialized,
    createVault,
    unlockVault,
    lockVault,
    clearError,
  };
}

export const vault = createVaultStore();
