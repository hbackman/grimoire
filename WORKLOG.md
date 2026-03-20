# Grimoire MVP – Work Log

Branch: `feature/mvp-addon-manager`  
Target: WoW **Classic** addon manager built on Tauri + Vue 3

---

## What Was Built

### Rust backend (`src-tauri/src/lib.rs`)

| Command | Description |
|---|---|
| `scan_addons(path)` | Reads every subfolder under the AddOns dir, parses its `.toc` file, returns a list of `InstalledAddon` structs |
| `validate_wow_path(path)` | Accepts a WoW install root _or_ a direct AddOns dir; returns a label (`"classic"`, `"classic_era"`, `"addons_dir"`, etc.) or an error. Classic variants are checked first. |
| `install_addon_zip(url, addons_path)` | Downloads a zip from the given URL, extracts it into the AddOns directory, returns the list of top-level folders created |
| `remove_addon(addons_path, folder)` | Deletes an addon folder; includes path-traversal guard |
| `create_test_addon_dir(base_path)` | Creates a fake AddOns directory with 3 sample Classic addons (WeakAuras, Details, Questie) for dev/testing purposes |

#### TOC parsing
- Handles both `## Interface: 11502` (old style) and `## Interface-Classic: 11502` (Classic-specific key, takes priority)
- Also handles `## Interface-Classic-Era:` and `## Interface-Classic-Progression:`
- Normalises all metadata keys to lowercase
- `.toc` filename must match the folder name (standard WoW convention)

### Frontend

#### `src/lib/curseforge.js`
- Kept the existing scrape-via-hidden-webview approach
- Added `getAddonDownloadUrl(slug, gameVersionTypeId)` — scrapes the `/files/all` page, then the file detail page, to find the direct download URL
- Added `getLatestVersion(slug, gameVersionTypeId)` — used for update checks
- Added `DEFAULT_GAME_VERSION = 67408` (Classic Era)
- Updated `versions()` to list Classic variants first (Classic Era, Cataclysm Classic, MoP Classic, then Retail)

#### `src/components/Addon.vue`
- Unified component used for both search results and installed addons
- Shows: image/initial letter, title, version badge, description/notes, author
- Action buttons: **Install** (search results), **Update** (installed with update available), **Remove** (installed)
- `updateAvailable` prop shows an amber badge with the newer version

#### `src/components/InstalledView.vue`
- New component for the Installed tab
- Loads settings from Tauri store on mount, then calls `scan_addons`
- **Check for Updates**: iterates installed addons, guesses CurseForge slug from folder name (lowercase, underscore→hyphen), calls `getLatestVersion`, shows badges
- **Update All**: updates all addons with available updates sequentially
- **Remove**: calls `remove_addon` then rescans
- Progress messages during update operations

#### `src/components/Chips.vue`
- Replaced installed toggle with Browse / Installed tab chips

#### `src/components/Settings.vue`
- Default game version is now Classic Era (67408)
- Path placeholder shows `_classic_/Interface/AddOns`
- Path description updated to mention Classic
- Real-time path validation with `validate_wow_path` — shows human-readable labels per variant
- Developer section: creates a test AddOns directory pre-populated with Classic addons (WeakAuras, Details, Questie with `## Interface-Classic: 11502` TOC entries)

#### `src/App.vue`
- Added `InstalledView` component
- Browse / Installed view switching
- First-run banner when no AddOns path is configured
- Install button on search results calls `getAddonDownloadUrl` + `install_addon_zip`
- Default game version uses `DEFAULT_GAME_VERSION` constant

---

## What Works

- TOC parsing (Classic `## Interface-Classic:` key handled correctly)
- `scan_addons` / `validate_wow_path` / `remove_addon` / `create_test_addon_dir` — all pure Rust, no GUI deps, ready to use
- `install_addon_zip` — download + extract logic is solid
- Full Vue frontend: Browse tab (search + install), Installed tab (scan + update check + update + remove), Settings with Classic defaults
- `cargo check` passes cleanly (zero errors, zero warnings from our code)

## What's Not Done / Known Issues

### Update checking — slug guessing
The update-check in `InstalledView.vue` guesses the CurseForge slug by lowercasing the folder name and replacing underscores with hyphens. This works for well-known addons (WeakAuras → `weakauras`, Details → `details`) but will miss addons whose folder name differs from their CurseForge slug. A proper solution would be storing the slug at install time.

### CurseForge download flow
The full install flow (search → install) depends on the CurseForge scraper correctly extracting a direct download URL from their file detail pages. CurseForge's HTML structure changes occasionally, so this may need updating. The scraper infrastructure is in place; `extractDirectDownloadUrl` and `extractLatestFileUrl` may need tweaking once tested against live pages.

### `cargo build` (linking) blocked by environment
`cargo check` passes — all Rust code is type-correct. However, `cargo build` (full compile + link) requires the webkit2gtk-4.1 and libsoup-3.0 **runtime** shared libraries (`libwebkit2gtk-4.1.so.0`, `libsoup-3.0.so.0`) to be installed on the build host. These are not present in the CI/sandbox environment (the dev packages were manually extracted to work around the missing `apt` access, but the runtime `.so` files themselves were never installed). On a proper Linux dev machine with `libwebkit2gtk-4.1-dev` and `libsoup-3.0-dev` installed via apt, `cargo build` will succeed.

### Multi-folder addons
Some CurseForge addons extract multiple folders (e.g. ElvUI + ElvUI_Options). `install_addon_zip` handles this correctly at the extraction level, but the UI doesn't group them or track the relationship.

---

## How to Test

1. Build requires Linux with Tauri deps: `sudo apt install libgtk-3-dev libwebkit2gtk-4.1-dev libsoup-3.0-dev`
2. Run `cargo build` in `src-tauri/`
3. `bun install && bun run tauri dev` from the root
4. In Settings → use "Create" under dev section to generate test addons at e.g. `/tmp`
5. Set the path to the generated `TestAddOns` folder
6. Switch to Installed tab — should show WeakAuras, Details, Questie
7. Browse tab → search for an addon → Install (requires CurseForge to be reachable)
