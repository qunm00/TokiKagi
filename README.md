# TokiKagi

> A TOTP desktop authenticator for 2FA codes, built with Tauri and SvelteKit.

Store and manage your 2FA secrets locally, generate time-based one-time passwords (TOTP), and copy codes with a click — no phone or cloud sync required.

## Features

- Add and manage TOTP secrets
- Live 6-digit code display with 30-second countdown
- Click to copy code to clipboard
- Local storage (no cloud, no server)
- Cross-platform (macOS, Windows, Linux)

## Getting started

```bash
# Install dependencies
pnpm install

# Development (Tauri + SvelteKit)
pnpm run tauri dev
```

The app opens in a native window. Frontend dev server runs on http://localhost:1420.

## Stack

- Desktop: Tauri (https://v2.tauri.app) v2
- Frontend: SvelteKit (https://svelte.dev) + Svelte (https://svelte.dev) 5
- Styling: Melt UI (https://melt-ui.com) (Melt headless components)
- Language: TypeScript (frontend), Rust (backend)
- Linting: Biome (https://biomejs.dev)
- Testing: Vitest (https://vitest.dev) (frontend), Cargo test (Rust)

## License

GPL-3.0-only. See LICENSE (./LICENSE).