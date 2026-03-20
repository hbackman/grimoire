# Grimoire — Work Log

## Session 1 (MVP — branch: `feature/mvp-addon-manager`)

### Goals
Build a functional WoW Classic addon manager desktop app from scratch using Tauri v2 + Vue 3.

### Completed

#### Backend (Rust — `src-tauri/src/lib.rs`)
- **`.toc` file parsing** (`parse_toc`) — reads `## Key: Value` metadata lines, normalises keys to lowercase, handles the `Interface-Classic` / `Interface-Classic-Era` precedence logic so Classic-specific interface versions win over the generic `Interface:` key.
- **`validate_wow_path`** — accepts a WoW installation root, `Interface/` subdirectory, or direct `AddOns/` directory. Detects `_classic_`, `_classic_era_`, `_classic_ptr_`, `_retail_`, `_ptr_` variants.
- **`scan_addons`** — walks a directory, parses each addon's `.toc`, returns sorted `InstalledAddon` structs.
- **`install_addon_zip`** — downloads a zip from a URL (async, reqwest) and extracts it into the AddOns directory, skipping `__MACOSX` junk.
- **`remove_addon`** — removes a folder with path traversal sanitization.
- **`create_test_addon_dir`** — creates a fake WoW Classic AddOns directory with 3 sample addons for development.
- **Scraper commands** — `scrape_in_webview`, `handle_scrape_result`, `handle_scrape_error` — route CurseForge page scraping through a hidden Tauri webview.

#### Frontend (Vue 3 + Tailwind)
- **`App.vue`** — tab routing (Browse / Installed), WoW path check banner, infinite scroll on search.
- **`InstalledView.vue`** — lists installed addons with check-updates, update-all, remove per addon.
- **`Addon.vue`** — card component for both search results and installed addons.
- **`AddonSkeleton.vue`** — loading skeleton.
- **`Search.vue`** — debounced search input.
- **`Settings.vue`** — WoW path picker and game version selector.
- **`Chips.vue`** — tab navigation chips.
- **`ScrollToTopButton.vue`** — scroll utility.
- **`lib/curseforge.js`** — CurseForge API integration: browse, search, version check, download URL resolution. Uses a scraper-based fallback for download links not exposed in the public API.

#### Configuration
- Tauri plugins: `tauri-plugin-store` (settings persistence), `tauri-plugin-dialog` (file pickers), `tauri-plugin-opener`.
- Dark mode via Tailwind's `dark:` classes.
- Vite config with `@/` path alias.

---

## Session 2 (Tests + Post-MVP — branches: `feature/mvp-addon-manager`, `feature/post-mvp`)

### Task 1: Unit Tests (`feature/mvp-addon-manager`)

Created `src-tauri/addon-core/` — a standalone Rust workspace crate with **no system library dependencies** (no GTK, no WebKit). This allows `cargo test -p addon-core` to run in any environment, including CI sandboxes without a GUI stack.

The crate contains the pure business logic extracted from `lib.rs` plus:

#### Test coverage (48 tests, all passing)

| Module | Tests |
|--------|-------|
| `parse_toc` | Basic fields (Title, Version, Interface, Notes, Author), Interface-Classic priority, Interface-Classic-Era priority, Classic wins over generic, generic sets canonical if no Classic, missing fields, empty input, extra whitespace, non-metadata lines ignored, keys lowercased, Dependencies field |
| `validate_wow_path` | Direct AddOns dir, case-insensitive AddOns, Interface/ subdir, `_classic_` root, `_classic_era_` root, `_retail_` root, nonexistent path, wrong dir |
| `scan_addons` | Basic (2 addons, sorted), all metadata fields, skips no-toc folders, empty dir, nonexistent path, folder-name fallback title |
| `create_test_addon_dir` | Creates TestAddOns folder, all 3 addon dirs present, .toc parseable, scannable by scan_addons |
| `remove_addon` | Rejects `../../../etc`, rejects forward slash, rejects backslash, rejects `..`, removes real folder, ok when already gone |
| `disable_addon` | Renames to `-disabled`, rejects traversal |
| `enable_addon` | Renames back from `-disabled` |
| `scan_addons_with_disabled` | Marks `disabled: true` for `-disabled` folders, `disabled: false` for normal |
| `AddonProfiles` | Create/save/load, delete, snapshot from addons |
| `AddonNotes` | set/get/save/load, remove |
| `export_addon_list` | Manifest v1, correct fields |
| `backup_addon` | Creates zip file, rejects traversal |

Also added a `Cargo.toml` workspace root at the repo root so `cargo test -p addon-core` works from anywhere in the project.

---

### Task 2: Post-MVP Features (`feature/post-mvp`)

#### Backend new commands (`src-tauri/src/lib.rs`)

| Command | Description |
|---------|-------------|
| `scan_addons_with_disabled` | Scans addons including `-disabled` folders; returns `disabled: true` flag |
| `disable_addon` | Renames `AddonFolder` → `AddonFolder-disabled` (reversible, WoW ignores it) |
| `enable_addon` | Renames back from `-disabled` |
| `load_profiles` | Loads named addon profiles from `Interface/grimoire-profiles.json` |
| `save_profile` | Saves a named profile (list of folder names) |
| `delete_profile` | Deletes a profile |
| `load_notes` | Loads user notes from `Interface/grimoire-notes.json` |
| `save_note` | Saves/clears a note for a specific addon |
| `export_addon_list` | Returns installed addons as a JSON manifest (v1 format) |
| `backup_addon` | Zips an addon folder to a backup directory before updating |

#### Frontend new features

- **Disable/Enable toggle** (`InstalledView.vue`) — each addon card gets a disable/enable button. Disabled addons are shown at reduced opacity.
- **Dependency info** (`InstalledView.vue`) — if a `.toc` file has `## Dependencies:`, the required addons are shown below the card.
- **Inline user notes** (`InstalledView.vue`) — click `+ add note` to type a personal note for any addon; saved per-addon in `grimoire-notes.json`.
- **Export addon list** (`InstalledView.vue`) — "Export List" button opens a modal with the JSON manifest; includes a "Copy to Clipboard" button.
- **Profiles tab** (`ProfilesView.vue` + `Chips.vue`) — new purple "Profiles" chip; create profiles by snapshotting current addons, list/delete saved profiles.
- **`App.vue`** — wired up the new `ProfilesView` component.

#### Distribution plan
`DISTRIBUTION.md` added to repo root covering:
- `tauri build` and what it produces per platform
- GitHub Releases as primary distribution channel
- macOS notarization walkthrough
- Windows code signing walkthrough
- Auto-update via `tauri-plugin-updater` with manifest format
- Full GitHub Actions CI/CD pipeline (matrix: macOS universal, Windows x64, Linux x64) triggered on version tags
- Release checklist

---

## Known limitations / future work

- **CurseForge slug mapping** — update checks rely on guessing the CurseForge slug from the folder name. A proper `## X-Curse-Project-ID:` → slug mapping would make this reliable.
- **Bulk disable/enable** — UI doesn't yet support selecting multiple addons for batch operations. The backend supports it with multiple individual calls.
- **Changelog display** — `DISTRIBUTION.md` mentions this as a future feature. The CurseForge API can return changelog HTML; it would need a webview to display richly.
- **Dark/light theme toggle** — Tailwind dark mode is wired to the OS preference (`prefers-color-scheme`). A manual toggle button in Settings would be a small addition.
- **Import addon list** — `export_addon_list` is implemented but `import` (re-install all addons from a manifest) is not yet implemented.
- **Last updated date** — `InstalledAddon.last_updated` is populated from the `.toc` file's mtime in `addon-core`, but not yet surfaced in the UI.
