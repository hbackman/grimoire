# Grimoire WORKLOG

## 2026-03-24 — Addon Download Implementation

### Problem
CurseForge addon downloads don't work with a simple `GET` to the download URL:
- The download page shows a 5-second JavaScript countdown
- After the countdown, the browser initiates an actual file download (redirect to CDN)
- The old code passed the countdown page URL to `reqwest::get()` which just returns HTML, not a zip file

### Approach Chosen: Tauri WebView Download Interception

**Tauri v2 has first-class download interception via `WebviewWindowBuilder::on_download`.**

This was the cleanest approach because:
1. The hidden WebView (already used for scraping) can navigate to the download page
2. The 5-second countdown runs naturally in the WebView's JavaScript engine
3. Tauri's `DownloadEvent::Requested` fires when the actual file download starts
4. We can redirect the download destination to a temp directory we control
5. `DownloadEvent::Finished` tells us when it's done
6. We then extract the zip into the AddOns folder

### Implementation

**`src-tauri/src/lib.rs`** — New `download_addon_via_webview` command:
- Creates a dedicated hidden `addon-downloader` WebView (destroys any existing one first)
- Navigates to the CurseForge download page URL (e.g. `curseforge.com/wow/addons/{slug}/download/{fileId}`)
- `on_download` handler intercepts the download, redirects to `{tmp}/grimoire-downloads/`
- Polls every 500ms for up to 60 seconds
- macOS workaround: `DownloadEvent::Finished` path is always `None` on macOS due to WKWebView API limitations; we track the expected destination from `Requested` event and fall back to it
- Extracts the zip, cleans up the temp file

**`src/lib/curseforge.js`** — `installAddon()` now calls `download_addon_via_webview` instead of `install_addon_zip` with a raw URL

**`src/components/InstalledView.vue`** — `updateAddon()` uses same mechanism

**`src/App.vue`** — `installAddon()` uses `cfInstallAddon()` from curseforge lib

### Why Not Other Approaches

1. **Direct `reqwest` download** — CurseForge blocks direct downloads; the countdown is a Cloudflare-enforced JS challenge. Cookies from WebView session can't easily be shared to Rust.

2. **Configure WebView download directory** — Tauri doesn't expose a "default download directory" setting; `on_download` is the right API.

3. **JS/Service Worker interception** — Would require injecting a service worker into an external page (CurseForge), which is not possible due to cross-origin restrictions.

4. **CurseForge official API** — Requires an API key application process (for "studios/game developers"). Not suitable for a community tool.

### API Notes

- `tauri::webview::DownloadEvent` is in `tauri::webview` module, no extra feature flags needed (wry is default in tauri 2.x)
- `DownloadEvent` is `#[non_exhaustive]` — requires `_ => {}` wildcard arm
- `DownloadEvent::Requested { url, destination }` — `url` is `tauri::Url`, `destination` is `&mut PathBuf`
- `DownloadEvent::Finished { url, path, success }` — `path` is `Option<PathBuf>` (None on macOS)
- The `on_download` closure must be `Fn(WebviewWindow<R>, DownloadEvent) -> bool + Send + Sync + 'static`

### Caveats

- The 5-second countdown means install takes ~6-8 seconds minimum
- If CurseForge changes their download page JS significantly, the countdown may not fire → 60s timeout
- The `getAddonDownloadUrl` scraping step still needs to work correctly to get the file page URL
- File name extraction from URL path segments assumes standard CF URL structure
