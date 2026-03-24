use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri::webview::DownloadEvent;

// ── Scraper commands ─────────────────────────────────────────────────────────

#[tauri::command]
async fn scrape_in_webview(app: tauri::AppHandle, url: String) -> Result<(), String> {
    println!("scrape_in_webview called with URL: {}", url);

    let label = "scraper";

    let win = if let Some(w) = app.get_webview_window(label) {
        w
    } else {
        WebviewWindowBuilder::new(&app, label, WebviewUrl::App("blank.html".into()))
            .visible(false)
            .title("Scraper")
            .build()
            .map_err(|e| e.to_string())?
    };

    win.eval(&format!("window.location.replace({:?});", url))
        .map_err(|e| e.to_string())?;

    let ah = app.clone();
    let win_label = win.label().to_string();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_millis(2000)).await;

        if let Some(w) = ah.get_webview_window(&win_label) {
            let _ = w.eval(r#"
                window.__TAURI__.core.invoke("handle_scrape_result", {
                    html: document.documentElement.outerHTML,
                });
            "#);
        } else {
            println!("Webview window not found during polling");
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    });

    Ok(())
}

#[tauri::command]
async fn handle_scrape_result(app: tauri::AppHandle, html: String) -> Result<(), String> {
    let _ = app.emit("scraper:result", serde_json::json!({ "html": html }));
    Ok(())
}

#[tauri::command]
async fn handle_scrape_error(app: tauri::AppHandle, error: String) -> Result<(), String> {
    println!("handle_scrape_error called with error: {}", error);
    let _ = app.emit("scraper:error", error);
    Ok(())
}

// ── Addon types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledAddon {
    /// Folder name (e.g. "WeakAuras")
    pub folder: String,
    /// Title from .toc
    pub title: String,
    /// Version from .toc
    pub version: String,
    /// Notes/description from .toc
    pub notes: String,
    /// Author from .toc
    pub author: String,
    /// Interface version from .toc (e.g. "110002")
    pub interface: String,
    /// If an update is available, the newer version string
    pub update_available: Option<String>,
}

// ── TOC parsing ───────────────────────────────────────────────────────────────

fn parse_toc(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("##") {
            // "## Key: Value"
            // Keys can contain hyphens, e.g. "## Interface-Classic: 11502"
            let rest = line[2..].trim();
            if let Some(colon_pos) = rest.find(':') {
                let raw_key = rest[..colon_pos].trim();
                let value = rest[colon_pos + 1..].trim().to_string();

                // Normalise: store both the raw key and a simplified one.
                // e.g. "Interface-Classic" → also stored under "interface-classic"
                let key = raw_key.to_lowercase();
                map.insert(key.clone(), value.clone());

                // For the primary "interface" field, prefer variant-specific keys
                // over the generic one so Classic-specific data wins.
                // Priority: Interface-Classic > Interface-Classic-Era > Interface
                if key == "interface-classic"
                    || key == "interface-classic-era"
                    || key == "interface-classic-progression"
                {
                    map.insert("interface_canonical".to_string(), value);
                } else if key == "interface" && !map.contains_key("interface_canonical") {
                    map.insert("interface_canonical".to_string(), value);
                }
            }
        }
    }
    map
}

fn read_addon_from_folder(folder_path: &Path) -> Option<InstalledAddon> {
    let folder_name = folder_path.file_name()?.to_string_lossy().to_string();

    // Look for <FolderName>.toc
    let toc_path = folder_path.join(format!("{}.toc", folder_name));
    let toc_content = fs::read_to_string(&toc_path).ok()?;

    let meta = parse_toc(&toc_content);

    // Resolve the canonical interface string (Classic-specific key wins)
    let interface_str = meta.get("interface_canonical")
        .cloned()
        .unwrap_or_default();

    Some(InstalledAddon {
        folder:    folder_name.clone(),
        title:     meta.get("title").cloned().unwrap_or_else(|| folder_name),
        version:   meta.get("version").cloned().unwrap_or_default(),
        notes:     meta.get("notes").cloned().unwrap_or_default(),
        author:    meta.get("author").cloned().unwrap_or_default(),
        interface: interface_str,
        update_available: None,
    })
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Scan the AddOns directory and return parsed addon metadata.
#[tauri::command]
fn scan_addons(path: String) -> Result<Vec<InstalledAddon>, String> {
    let dir = Path::new(&path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }

    let mut addons = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(addon) = read_addon_from_folder(&path) {
                addons.push(addon);
            }
        }
    }

    // Sort alphabetically by title
    addons.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));

    Ok(addons)
}

/// Validate that a given path looks like an AddOns directory.
///
/// Returns a short descriptor string on success:
///   "addons_dir"  – the path is already an AddOns folder
///   "classic"     – WoW root containing _classic_
///   "classic_era" – WoW root containing _classic_era_
///   "retail"      – WoW root containing _retail_  (not our primary target)
///
/// On failure returns an Err describing the problem.
#[tauri::command]
fn validate_wow_path(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }

    // Accept a direct AddOns directory (ends with "AddOns", case-insensitive)
    let name_lc = p.file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if name_lc == "addons" {
        return Ok("addons_dir".to_string());
    }

    // Maybe the user pointed at Interface/ — go one level up and check
    if name_lc == "interface" {
        let addons_candidate = p.join("AddOns");
        if addons_candidate.exists() {
            return Ok("addons_dir".to_string());
        }
    }

    // Check for WoW sub-directories.  Classic variants come first so we return
    // the best match when multiple variants are present in the same install.
    let variants: &[(&str, &str)] = &[
        ("_classic_",      "classic"),
        ("_classic_era_",  "classic_era"),
        ("_classic_ptr_",  "classic_ptr"),
        ("_retail_",       "retail"),
        ("_ptr_",          "ptr"),
    ];

    for (folder, label) in variants {
        let candidate = p.join(folder).join("Interface").join("AddOns");
        if candidate.exists() {
            return Ok(label.to_string());
        }
    }

    Err("Could not find a WoW AddOns folder at this path. \
         Please point to your WoW installation root or directly to \
         the Interface/AddOns directory.".to_string())
}

/// Download a zip from `url` and extract it into `addons_path`.
/// Returns the list of top-level folders extracted.
#[tauri::command]
async fn install_addon_zip(url: String, addons_path: String) -> Result<Vec<String>, String> {
    println!("Downloading addon zip from: {}", url);

    // Download
    let bytes = reqwest::get(&url)
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    extract_zip_bytes(&bytes, &addons_path)
}

fn extract_zip_bytes(bytes: &[u8], addons_path: &str) -> Result<Vec<String>, String> {
    use std::io::Cursor;

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    let dest = Path::new(addons_path);
    let mut extracted_folders: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();

        // Skip __MACOSX and hidden junk
        if name.contains("__MACOSX") || name.starts_with('.') {
            continue;
        }

        let out_path = dest.join(&name);

        // Track top-level folder
        if let Some(first_component) = PathBuf::from(&name).components().next() {
            if let std::path::Component::Normal(s) = first_component {
                extracted_folders.insert(s.to_string_lossy().to_string());
            }
        }

        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            // Ensure parent exists
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out_file = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut out_file).map_err(|e| e.to_string())?;
        }
    }

    Ok(extracted_folders.into_iter().collect())
}

/// Download an addon via the Tauri WebView's native download mechanism.
///
/// This navigates a hidden WebView to the CurseForge download page (which starts
/// a 5-second countdown before triggering a file download). Tauri's `on_download`
/// handler intercepts the download request, redirects it to a temp file, waits
/// for completion, then extracts the zip into the AddOns directory.
///
/// Returns the list of top-level folders extracted.
#[tauri::command]
async fn download_addon_via_webview(
    app: tauri::AppHandle,
    download_page_url: String,
    addons_path: String,
) -> Result<Vec<String>, String> {
    println!("[downloader] navigating to: {}", download_page_url);

    // Shared state: the result of the download (set by on_download handler)
    // Contains either the path to the downloaded file, or an error string.
    // Also stores the expected destination path from the Requested event.
    let downloaded_path: Arc<Mutex<Option<Result<PathBuf, String>>>> = Arc::new(Mutex::new(None));
    let downloaded_path_clone = downloaded_path.clone();

    // Track the expected destination path (set in Requested handler)
    let expected_dest: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    let expected_dest_for_handler = expected_dest.clone();

    // Create (or reuse) a dedicated hidden downloader WebView.
    // We destroy any existing one first to ensure a clean state.
    let label = "addon-downloader";
    if let Some(existing) = app.get_webview_window(label) {
        let _ = existing.destroy();
        // Small delay to let it fully close
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    // Temp directory for the download
    let temp_dir = std::env::temp_dir().join("grimoire-downloads");
    fs::create_dir_all(&temp_dir).map_err(|e| e.to_string())?;
    let temp_dir_clone = temp_dir.clone();

    let _win = WebviewWindowBuilder::new(
        &app,
        label,
        WebviewUrl::External(download_page_url.parse().map_err(|e: url::ParseError| e.to_string())?),
    )
    .visible(false)
    .title("Addon Downloader")
    .on_download(move |_webview, event| {
        match event {
            DownloadEvent::Requested { url, destination } => {
                println!("[downloader] download requested: {}", url);
                // Redirect to our temp directory
                let file_name = url
                    .path_segments()
                    .and_then(|s| s.last())
                    .unwrap_or("addon.zip")
                    .to_string();
                let dest = temp_dir_clone.join(&file_name);
                *destination = dest.clone();
                // Store the expected destination so we can find it even on macOS
                // where the Finished event doesn't include the path.
                {
                    let mut ed = expected_dest_for_handler.lock().unwrap();
                    *ed = Some(dest.clone());
                }
                println!("[downloader] saving to: {:?}", dest);
                true // allow download
            }
            DownloadEvent::Finished { url: _, path, success } => {
                println!("[downloader] download finished, success={}, path={:?}", success, path);
                let mut guard = downloaded_path_clone.lock().unwrap();
                if success {
                    // Use the provided path if available; fall back to expected_dest
                    // (macOS always returns None for path due to API limitations)
                    if let Some(p) = path {
                        *guard = Some(Ok(p.to_path_buf()));
                    } else {
                        // Signal success with an empty PathBuf — the caller will
                        // look up the expected destination via `expected_dest`.
                        *guard = Some(Ok(PathBuf::new()));
                    }
                } else {
                    *guard = Some(Err("Download failed (WebView reported failure)".to_string()));
                }
            }
            _ => {}
        }
        true
    })
    .build()
    .map_err(|e| e.to_string())?;

    // Poll for up to 60 seconds (5s countdown + download time)
    let timeout = Duration::from_secs(60);
    let poll_interval = Duration::from_millis(500);
    let start = std::time::Instant::now();

    loop {
        tokio::time::sleep(poll_interval).await;

        let result = {
            let guard = downloaded_path.lock().unwrap();
            guard.clone()
        };

        if let Some(res) = result {
            // Clean up the downloader WebView
            if let Some(win) = app.get_webview_window(label) {
                let _ = win.destroy();
            }

            let mut path = res?;

            // macOS: path may be empty PathBuf — resolve from expected_dest
            if path.as_os_str().is_empty() {
                let ed = expected_dest.lock().unwrap();
                match ed.as_ref() {
                    Some(p) => path = p.clone(),
                    None => return Err("Download path not recorded".to_string()),
                }
            }

            println!("[downloader] extracting zip from {:?}", path);

            // Read file bytes and extract
            let bytes = fs::read(&path).map_err(|e| format!("Failed to read downloaded file {:?}: {}", path, e))?;
            let folders = extract_zip_bytes(&bytes, &addons_path)?;

            // Clean up temp file
            let _ = fs::remove_file(&path);

            println!("[downloader] extracted folders: {:?}", folders);
            return Ok(folders);
        }

        if start.elapsed() > timeout {
            // Clean up
            if let Some(win) = app.get_webview_window(label) {
                let _ = win.destroy();
            }
            return Err("Download timed out after 60 seconds".to_string());
        }
    }
}

/// Remove an addon folder from the AddOns directory.
#[tauri::command]
fn remove_addon(addons_path: String, folder: String) -> Result<(), String> {
    // Sanitize - no path traversal
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }

    let target = Path::new(&addons_path).join(&folder);
    if !target.exists() {
        return Ok(()); // Already gone
    }

    fs::remove_dir_all(&target).map_err(|e| e.to_string())
}

/// Create a test addon directory structure for development/testing.
#[tauri::command]
fn create_test_addon_dir(base_path: String) -> Result<String, String> {
    let addons_dir = Path::new(&base_path).join("TestAddOns");
    fs::create_dir_all(&addons_dir).map_err(|e| e.to_string())?;

    // Classic interface numbers: 11502 = Classic Era 1.15.2, 40402 = Cataclysm Classic
    let test_addons = vec![
        ("WeakAuras",   "3.5.1",     "WeakAuras",           "Powerful and versatile display addon",  "The WeakAuras Team", "11502"),
        ("Details",     "9.8.18.2",  "Details! Damage Meter","Damage meter and combat analyzer",      "Tercioo",            "11502"),
        ("Questie",     "7.2.0",     "Questie",             "Quest helper for Classic",               "Questie Team",       "11502"),
    ];

    for (folder, version, title, notes, author, interface) in &test_addons {
        let addon_dir = addons_dir.join(folder);
        fs::create_dir_all(&addon_dir).map_err(|e| e.to_string())?;

        // Use the Classic-specific key so the TOC is valid for WoW Classic Era.
        let toc_content = format!(
            "## Interface-Classic: {}\n## Title: {}\n## Notes: {}\n## Author: {}\n## Version: {}\n\n{}.lua\n",
            interface, title, notes, author, version, folder
        );

        let toc_path = addon_dir.join(format!("{}.toc", folder));
        let mut f = fs::File::create(toc_path).map_err(|e| e.to_string())?;
        f.write_all(toc_content.as_bytes()).map_err(|e| e.to_string())?;

        // Create empty lua file
        let lua_path = addon_dir.join(format!("{}.lua", folder));
        let mut lf = fs::File::create(lua_path).map_err(|e| e.to_string())?;
        lf.write_all(b"-- placeholder\n").map_err(|e| e.to_string())?;
    }

    Ok(addons_dir.to_string_lossy().to_string())
}

// ── Post-MVP: scan with disabled addons ───────────────────────────────────────

/// Scan addons including those disabled via the `-disabled` folder-suffix convention.
/// Returns addons with `disabled: true` for ones whose folder ends with `-disabled`.
#[tauri::command]
fn scan_addons_with_disabled(path: String) -> Result<Vec<serde_json::Value>, String> {
    let dir = Path::new(&path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }

    let mut addons: Vec<serde_json::Value> = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        if !entry_path.is_dir() { continue; }

        let raw_name = entry_path.file_name().unwrap().to_string_lossy().to_string();
        let (canonical_name, is_disabled) = if let Some(base) = raw_name.strip_suffix("-disabled") {
            (base.to_string(), true)
        } else {
            (raw_name.clone(), false)
        };

        let toc_path = entry_path.join(format!("{}.toc", canonical_name));
        if let Ok(toc_content) = fs::read_to_string(&toc_path) {
            let meta = parse_toc(&toc_content);
            let interface_str = meta.get("interface_canonical").cloned().unwrap_or_default();
            let deps: Vec<String> = meta.get("dependencies")
                .map(|s| s.split(',').map(|d| d.trim().to_string()).filter(|d| !d.is_empty()).collect())
                .unwrap_or_default();

            addons.push(serde_json::json!({
                "folder": canonical_name,
                "title": meta.get("title").cloned().unwrap_or_else(|| canonical_name.clone()),
                "version": meta.get("version").cloned().unwrap_or_default(),
                "notes": meta.get("notes").cloned().unwrap_or_default(),
                "author": meta.get("author").cloned().unwrap_or_default(),
                "interface": interface_str,
                "updateAvailable": null,
                "dependencies": deps,
                "lastUpdated": null,
                "disabled": is_disabled,
                "userNotes": null,
            }));
        }
    }

    addons.sort_by(|a, b| {
        let ta = a["title"].as_str().unwrap_or("").to_lowercase();
        let tb = b["title"].as_str().unwrap_or("").to_lowercase();
        ta.cmp(&tb)
    });

    Ok(addons)
}

/// Disable an addon by renaming its folder to `<name>-disabled`.
#[tauri::command]
fn disable_addon(addons_path: String, folder: String) -> Result<(), String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }
    let src = Path::new(&addons_path).join(&folder);
    if !src.exists() {
        return Err(format!("Addon folder not found: {}", folder));
    }
    let dst = Path::new(&addons_path).join(format!("{}-disabled", folder));
    fs::rename(&src, &dst).map_err(|e| e.to_string())
}

/// Re-enable an addon by renaming `<name>-disabled` back to `<name>`.
#[tauri::command]
fn enable_addon(addons_path: String, folder: String) -> Result<(), String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }
    let src = Path::new(&addons_path).join(format!("{}-disabled", folder));
    if !src.exists() {
        return Err(format!("Disabled addon folder not found: {}-disabled", folder));
    }
    let dst = Path::new(&addons_path).join(&folder);
    fs::rename(&src, &dst).map_err(|e| e.to_string())
}

// ── Post-MVP: profiles ────────────────────────────────────────────────────────

fn profiles_path(addons_path: &str) -> PathBuf {
    Path::new(addons_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("grimoire-profiles.json")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
struct AddonProfiles {
    profiles: HashMap<String, Vec<String>>,
}

/// Load saved addon profiles from disk.
#[tauri::command]
fn load_profiles(addons_path: String) -> Result<HashMap<String, Vec<String>>, String> {
    let path = profiles_path(&addons_path);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let p: AddonProfiles = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(p.profiles)
}

/// Save a named profile (list of addon folder names).
#[tauri::command]
fn save_profile(addons_path: String, name: String, folders: Vec<String>) -> Result<(), String> {
    let path = profiles_path(&addons_path);
    let mut p: AddonProfiles = if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        AddonProfiles::default()
    };
    p.profiles.insert(name, folders);
    let json = serde_json::to_string_pretty(&p).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

/// Delete a named profile.
#[tauri::command]
fn delete_profile(addons_path: String, name: String) -> Result<bool, String> {
    let path = profiles_path(&addons_path);
    if !path.exists() {
        return Ok(false);
    }
    let mut p: AddonProfiles = {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).unwrap_or_default()
    };
    let removed = p.profiles.remove(&name).is_some();
    let json = serde_json::to_string_pretty(&p).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(removed)
}

// ── Post-MVP: user notes ──────────────────────────────────────────────────────

fn notes_path(addons_path: &str) -> PathBuf {
    Path::new(addons_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("grimoire-notes.json")
}

/// Load user notes for all addons.
#[tauri::command]
fn load_notes(addons_path: String) -> Result<HashMap<String, String>, String> {
    let path = notes_path(&addons_path);
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let map: HashMap<String, String> = serde_json::from_str(&json).unwrap_or_default();
    Ok(map)
}

/// Save a user note for a specific addon.
#[tauri::command]
fn save_note(addons_path: String, folder: String, note: String) -> Result<(), String> {
    let path = notes_path(&addons_path);
    let mut map: HashMap<String, String> = if path.exists() {
        let json = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&json).unwrap_or_default()
    } else {
        HashMap::new()
    };
    if note.is_empty() {
        map.remove(&folder);
    } else {
        map.insert(folder, note);
    }
    let json = serde_json::to_string_pretty(&map).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ── Post-MVP: export addon list ───────────────────────────────────────────────

/// Export the installed addon list as a JSON manifest string.
/// The frontend can offer this as a file download or display it for copy-paste.
#[tauri::command]
fn export_addon_list(addons_path: String) -> Result<String, String> {
    let dir = Path::new(&addons_path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", addons_path));
    }

    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    let mut addons = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() { continue; }

        let folder_name = path.file_name().unwrap().to_string_lossy().to_string();
        // Skip disabled folders
        if folder_name.ends_with("-disabled") { continue; }

        let toc_path = path.join(format!("{}.toc", folder_name));
        if let Ok(toc_content) = fs::read_to_string(&toc_path) {
            let meta = parse_toc(&toc_content);
            addons.push(serde_json::json!({
                "folder": folder_name,
                "title": meta.get("title").cloned().unwrap_or_else(|| folder_name.clone()),
                "version": meta.get("version").cloned().unwrap_or_default(),
                "curseforgeid": null,
            }));
        }
    }

    addons.sort_by(|a, b| {
        let ta = a["title"].as_str().unwrap_or("").to_lowercase();
        let tb = b["title"].as_str().unwrap_or("").to_lowercase();
        ta.cmp(&tb)
    });

    let manifest = serde_json::json!({
        "version": 1,
        "exported_at": chrono_now_iso(),
        "addons": addons,
    });

    serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())
}

fn chrono_now_iso() -> String {
    // Use std::time rather than pulling in chrono
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{}", secs)
}

// ── Post-MVP: auto-backup ─────────────────────────────────────────────────────

/// Zip an addon folder into a backup directory before updating.
/// Returns the path to the created zip file.
#[tauri::command]
fn backup_addon(addons_path: String, folder: String, backup_dir: String) -> Result<String, String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }

    let src_dir = Path::new(&addons_path).join(&folder);
    if !src_dir.exists() {
        return Err(format!("Addon folder not found: {}", folder));
    }

    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let zip_name = format!("{}-{}.zip", folder, ts);
    let zip_path = Path::new(&backup_dir).join(&zip_name);
    let zip_file = fs::File::create(&zip_path).map_err(|e| e.to_string())?;

    let mut writer = zip::ZipWriter::new(zip_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_dir_to_zip(&mut writer, &src_dir, &src_dir, options)?;
    writer.finish().map_err(|e| e.to_string())?;

    Ok(zip_path.to_string_lossy().to_string())
}

fn add_dir_to_zip(
    writer: &mut zip::ZipWriter<fs::File>,
    base: &Path,
    current: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        let relative = entry_path.strip_prefix(base).map_err(|e| e.to_string())?;
        let zip_name = relative.to_string_lossy().replace('\\', "/");

        if entry_path.is_dir() {
            writer.add_directory(&zip_name, options).map_err(|e| e.to_string())?;
            add_dir_to_zip(writer, base, &entry_path, options)?;
        } else {
            writer.start_file(&zip_name, options).map_err(|e| e.to_string())?;
            let mut f = fs::File::open(&entry_path).map_err(|e| e.to_string())?;
            io::copy(&mut f, writer).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// ── App entry ─────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // Scraper
            scrape_in_webview,
            handle_scrape_result,
            handle_scrape_error,
            // Addon management (MVP)
            scan_addons,
            validate_wow_path,
            install_addon_zip,
            download_addon_via_webview,
            remove_addon,
            create_test_addon_dir,
            // Post-MVP: disable/enable
            scan_addons_with_disabled,
            disable_addon,
            enable_addon,
            // Post-MVP: profiles
            load_profiles,
            save_profile,
            delete_profile,
            // Post-MVP: user notes
            load_notes,
            save_note,
            // Post-MVP: export & backup
            export_addon_list,
            backup_addon,
        ])
        .setup(|_app| {
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
