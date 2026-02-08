# Messagify

A desktop messaging client that connects to Discord and focuses on a clean, fast, Discord‑style UI. The app is built with a React/Vite frontend and a Tauri (Rust) backend.

## Features

- Real‑time messaging via the Discord Gateway
- Guild and DM navigation
- Local token storage for quick reconnects
- Voice gateway support 

## Tech stack

- **Frontend:** React + TypeScript, Vite, TailwindCSS, React Router, Zustand
- **Backend:** Tauri 2, Tokio, reqwest, tokio‑tungstenite, serde_json, tracing

## Getting started (development)

### Prereqs

- Node.js (for the Vite frontend)
- Rust toolchain (stable)
- Tauri system dependencies: https://tauri.app/start/prerequisites/

### Install

```bash
npm install
```

### Run

```bash
npm run tauri dev
```

This starts the Vite dev server on `http://localhost:1420` (configured in `src-tauri/tauri.conf.json`) and launches the Tauri window.

Frontend‑only dev server:

```bash
npm run dev
```

## Build

```bash
npm run build
npm run tauri build
```

`npm run build` produces the frontend bundle in `dist/`. `npm run tauri build` produces the desktop app bundle for your platform.

## Project layout

- `src/` — React UI and routes
- `src/components/Discord/` — core Discord UI widgets (guilds, channels, chat, members)
- `src/hooks/useDiscordEvents.ts` — websocket event wiring
- `src/stores/messageStore.ts` — Zustand message state
- `src-tauri/src/lib.rs` — Tauri entry point and command wiring
- `src-tauri/src/messangers/discord/` — Discord REST + gateway + voice + UDP
- `src-tauri/src/messangers/token_storage.rs` — token persistence

## Token storage

The app stores a Discord token in `tokens.json` at the project root (outside `src-tauri` to avoid rebuilds). On startup, the token is loaded and validated; if it’s invalid, it’s cleared. This file is git‑ignored, but treat it like a secret.

## Voice (WIP)

Voice work lives in `src-tauri/src/messangers/discord/voice_gateway.rs` and `udp_socket.rs`. The gateway handshake is implemented, with UDP discovery in place, but audio transport is still in progress.

## Troubleshooting

- **Port 1420 already in use**: stop the process using that port or update `devUrl` in `src-tauri/tauri.conf.json`.
- **Tauri build errors**: verify system dependencies for your OS (link above) and that `rustup` is installed.
- **Token issues**: delete `tokens.json` and re‑authenticate.

## License

See `LICENSE`.
