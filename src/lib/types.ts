export interface Account {
  id: string;
  label: string;
  secret: string;
  algorithm: string;
  digits: number;
  period: number;
  created_at: string;
  updated_at: string;
}

export type VaultErrorCode =
  | "Lock"
  | "Db"
  | "Crypto"
  | "VaultLocked"
  | "NotInitialized"
  | "AlreadyInitialized"
  | "InvalidPassword";

export class VaultError extends Error {
  constructor(
    public readonly code: VaultErrorCode,
    message: string,
  ) {
    super(message);
    this.name = "VaultError";
  }
}

export function parseVaultError(err: unknown): VaultError {
  if (err instanceof VaultError) return err;
  const message = err instanceof Error ? err.message : String(err);
  if (message.includes("not initialized")) return new VaultError("NotInitialized", message);
  if (message.includes("already initialized")) return new VaultError("AlreadyInitialized", message);
  if (message.includes("invalid password")) return new VaultError("InvalidPassword", message);
  if (message.includes("vault is locked")) return new VaultError("VaultLocked", message);
  if (message.includes("lock error")) return new VaultError("Lock", message);
  if (message.includes("crypto")) return new VaultError("Crypto", message);
  return new VaultError("Db", message);
}
