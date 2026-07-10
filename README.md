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

### Download

Download the latest release from [GitHub Releases](https://github.com/qunm00/TokiKagi/releases).

| Platform | File |
|----------|------|
| macOS (Intel) | `tokikagi_x64.dmg` |
| macOS (Apple Silicon) | `tokikagi_aarch64.dmg` |
| Linux | Build from source (see below) |

> **macOS note:** The app is not signed with an Apple Developer certificate. On first launch, right-click the app → Open, or go to System Settings → Privacy & Security → "Open anyway." This is a one-time step.

### Development

```bash
# Install dependencies
pnpm install

# Development (Tauri + SvelteKit)
pnpm run tauri dev
```

The app opens in a native window. Frontend dev server runs on http://localhost:1420.

### Build from source

```bash
pnpm install
pnpm tauri build
```

Requires Rust and system dependencies for your platform. See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Stack

- Desktop: Tauri (https://v2.tauri.app) v2
- Frontend: SvelteKit (https://svelte.dev) + Svelte (https://svelte.dev) 5
- Styling: Melt UI (https://melt-ui.com) (Melt headless components)
- Language: TypeScript (frontend), Rust (backend)
- Linting: Biome (https://biomejs.dev)
- Testing: Vitest (https://vitest.dev) (frontend), Cargo test (Rust)

## License

GPL-3.0-only. See LICENSE (./LICENSE).