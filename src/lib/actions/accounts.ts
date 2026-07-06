import { invoke } from "@tauri-apps/api/core";
import type { Account } from "$lib/types";

export async function getAccounts(): Promise<Account[]> {
  return invoke<Account[]>("get_accounts");
}

export async function addAccount(label: string, secret: string): Promise<Account> {
  return invoke<Account>("add_account", { label, secret });
}

export async function deleteAccount(id: string): Promise<void> {
  await invoke("delete_account", { id });
}
