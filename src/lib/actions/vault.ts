import { invoke } from "@tauri-apps/api/core";
import { parseVaultError } from "$lib/types";

export async function isInitialized(): Promise<boolean> {
  return invoke<boolean>("is_initialized");
}

export async function createVault(password: string): Promise<void> {
  try {
    await invoke("create_vault", { password });
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function unlock(password: string): Promise<void> {
  try {
    await invoke("unlock", { password });
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function lock(): Promise<void> {
  try {
    await invoke("lock");
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function getConfig(key: string): Promise<string | null> {
  try {
    const result = await invoke<string | null>("get_config", { key });
    return result;
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function setConfig(key: string, value: string): Promise<void> {
  try {
    await invoke("set_config", { key, value });
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function hasKeychainEntry(): Promise<boolean> {
  return invoke<boolean>("has_keychain_entry");
}

export async function saveToKeychain(password: string): Promise<void> {
  try {
    await invoke("save_to_keychain", { password });
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function removeFromKeychain(): Promise<void> {
  try {
    await invoke("remove_from_keychain");
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function unlockFromKeychain(): Promise<void> {
  try {
    await invoke("unlock_from_keychain");
  } catch (err) {
    throw parseVaultError(err);
  }
}

export async function generateTotp(secret: string): Promise<string> {
  return invoke<string>("generate_totp", { secret });
}

export async function getTimeRemaining(): Promise<number> {
  return invoke<number>("get_time_remaining");
}
