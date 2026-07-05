import { render, screen } from "@testing-library/svelte";
import { describe, expect, it, vi } from "vitest";
import * as vaultActions from "$lib/actions/vault";
import Page from "./routes/+page.svelte";

vi.mock("$lib/actions/vault", () => ({
  isInitialized: vi.fn(),
  hasKeychainEntry: vi.fn().mockResolvedValue(false),
  createVault: vi.fn().mockResolvedValue(undefined),
  unlock: vi.fn().mockResolvedValue(undefined),
  lock: vi.fn().mockResolvedValue(undefined),
  unlockFromKeychain: vi.fn().mockResolvedValue(undefined),
  saveToKeychain: vi.fn().mockResolvedValue(undefined),
  removeFromKeychain: vi.fn().mockResolvedValue(undefined),
  getConfig: vi.fn().mockResolvedValue(null),
  setConfig: vi.fn().mockResolvedValue(undefined),
  generateTotp: vi.fn().mockResolvedValue(""),
  getTimeRemaining: vi.fn().mockResolvedValue(30),
}));

describe("+page.svelte", () => {
  it("renders the create vault screen when not initialized", async () => {
    vi.mocked(vaultActions.isInitialized).mockResolvedValue(false);

    render(Page);
    const heading = await screen.findByRole("heading", {
      name: /create master password/i,
    });
    expect(heading).toBeInTheDocument();
  });

  it("renders the unlock screen when initialized", async () => {
    vi.mocked(vaultActions.isInitialized).mockResolvedValue(true);

    render(Page);
    const heading = await screen.findByRole("heading", {
      name: /unlock vault/i,
    });
    expect(heading).toBeInTheDocument();
  });
});
