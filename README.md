# TokiKagi

> A TOTP desktop authenticator for 2FA codes, built with Tauri and SvelteKit.

Store and manage your 2FA secrets locally, generate time-based one-time passwords (TOTP), and copy codes with a click — no phone or cloud sync required.

## Features

- Add and manage TOTP secrets
- Live 6-digit code display with 30-second countdown
- Click to copy code to clipboard
- Local storage (no cloud, no server)
- macOS (Apple Silicon)

## Getting started

### Install

Install on macOS with one command:

```sh
curl -fsSL https://raw.githubusercontent.com/qunm00/TokiKagi/main/scripts/install.sh | sh
```

To install a given version, pass the tag:

```sh
curl -fsSL https://raw.githubusercontent.com/qunm00/TokiKagi/main/scripts/install.sh | sh -s -- v0.1.1
```

The script downloads the app with curl, checks the SHA-256, and adds a local signature, so Apple Silicon runs it. The app goes to `~/Applications` by default. To use another folder, set `INSTALL_DIR`:

```sh
curl -fsSL https://raw.githubusercontent.com/qunm00/TokiKagi/main/scripts/install.sh | INSTALL_DIR="/Applications" sh
```

The app is not signed by Apple and it is not notarized. The local signature in the script is enough for macOS to run it.

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

## Updates

TokiKagi checks for updates on launch. It reads `latest.json` from the newest GitHub release, verifies a signature over the download, installs the app in place, and restarts.

That signature comes from the key of the person who publishes the release. It does not need an Apple certificate. Generate the key pair one time:

```sh
cargo tauri signer generate -w ~/.tauri/tokikagi.key
```

## Release

A tag push builds the app and attaches these files to the release:

- `tokikagi.app.zip` and `tokikagi.app.zip.sha256`, for a manual install.
- `tokikagi.app.tar.gz` and `tokikagi.app.tar.gz.sig`, for the updater.
- `latest.json`, the manifest that the app reads to find an update.

The workflow needs two secrets on the `release` environment: `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Add a required reviewer to that environment to hold every release until it is approved.

Push a tag to start the workflow:

```sh
git tag -a v0.1.1 -m "v0.1.1"
git push origin v0.1.1
```

## Stack

- Desktop: Tauri (https://v2.tauri.app) v2
- Frontend: SvelteKit (https://svelte.dev) + Svelte (https://svelte.dev) 5
- Styling: Melt UI (https://melt-ui.com) (Melt headless components)
- Language: TypeScript (frontend), Rust (backend)
- Linting: Biome (https://biomejs.dev)
- Testing: Vitest (https://vitest.dev) (frontend), Cargo test (Rust)

## License

GPL-3.0-only. See LICENSE (./LICENSE).