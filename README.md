# Grimoire

A lightweight World of Warcraft addon manager. Browse, install, update, and remove your addons without ads, paywalls, or bloat.

Grimoire is a free, open-source desktop app that makes managing your addons simple and fast.

## Features

- **Search & browse** thousands of addons
- **One-click install** — addons are downloaded and extracted to your AddOns folder automatically
- **Update checking** — see which addons have new versions available and update them individually or all at once
- **Multi-version support** — works with Classic Era, Cataclysm Classic, MoP Classic, and Retail (game versions are fetched dynamically so new versions appear automatically)
- **Clean uninstall** — removes all folders belonging to an addon, not just the main one

## Getting Started

1. Download the latest release for your platform
2. Open Grimoire and go to **Settings**
3. Point it to your WoW `Interface/AddOns` folder (e.g. `/Applications/World of Warcraft/_classic_era_/Interface/AddOns`)
4. Select your game version
5. Search for addons and hit **Install**

## Development

Grimoire is built with [Tauri v2](https://v2.tauri.app), [Vue 3](https://vuejs.org), and [Tailwind CSS](https://tailwindcss.com). The backend is Rust.

### Prerequisites

- [Rust](https://rustup.rs)
- [Bun](https://bun.sh) (or Node.js)
- Platform-specific Tauri dependencies — see the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/)

### Run locally

```sh
bun install
bun run tauri dev
```

### Build for production

```sh
bun run tauri build
```

## License

MIT
