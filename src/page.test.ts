import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import Page from "./routes/+page.svelte";

describe("+page.svelte", () => {
  it("renders the welcome heading", () => {
    render(Page);
    expect(
      screen.getByRole("heading", { name: /welcome to tauri \+ svelte/i }),
    ).toBeInTheDocument();
  });

  it("renders the greet input and submit button", () => {
    render(Page);
    expect(screen.getByPlaceholderText(/enter a name/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /greet/i })).toBeInTheDocument();
  });
});
