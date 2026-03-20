//! addon-core — pure logic for the Grimoire WoW addon manager.
//!
//! This crate has no Tauri or GTK dependencies, making it fully testable
//! on any platform without a GUI environment.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ── Addon types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Comma-separated list of addon dependencies from .toc
    pub dependencies: Vec<String>,
    /// Last modified time of the .toc file (Unix timestamp, seconds)
    pub last_updated: Option<u64>,
    /// Whether the addon is currently disabled
    pub disabled: bool,
    /// User-added personal notes/tags
    pub user_notes: Option<String>,
}

// ── TOC parsing ───────────────────────────────────────────────────────────────

/// Parse a WoW .toc file into a key/value map.
///
/// Keys are lowercased; the `interface_canonical` key is set to whichever
/// interface variant should be considered authoritative (Classic-specific keys
/// win over the generic `Interface:` key).
pub fn parse_toc(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("##") {
            let rest = line[2..].trim();
            if let Some(colon_pos) = rest.find(':') {
                let raw_key = rest[..colon_pos].trim();
                let value = rest[colon_pos + 1..].trim().to_string();
                let key = raw_key.to_lowercase();
                map.insert(key.clone(), value.clone());

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

/// Parse a comma/space-separated dependency list from a toc map.
pub fn parse_dependencies(map: &HashMap<String, String>) -> Vec<String> {
    map.get("dependencies")
        .or_else(|| map.get("requiredeps"))
        .map(|s| {
            s.split(',')
                .map(|d| d.trim().to_string())
                .filter(|d| !d.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Read a single addon from its folder, returning None if not a valid addon.
pub fn read_addon_from_folder(folder_path: &Path) -> Option<InstalledAddon> {
    let folder_name = folder_path.file_name()?.to_string_lossy().to_string();
    let toc_path = folder_path.join(format!("{}.toc", folder_name));
    let toc_content = fs::read_to_string(&toc_path).ok()?;
    let meta = parse_toc(&toc_content);

    let interface_str = meta
        .get("interface_canonical")
        .cloned()
        .unwrap_or_default();

    let dependencies = parse_dependencies(&meta);

    // Last-modified time from filesystem
    let last_updated = toc_path
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs());

    Some(InstalledAddon {
        folder: folder_name.clone(),
        title: meta.get("title").cloned().unwrap_or_else(|| folder_name),
        version: meta.get("version").cloned().unwrap_or_default(),
        notes: meta.get("notes").cloned().unwrap_or_default(),
        author: meta.get("author").cloned().unwrap_or_default(),
        interface: interface_str,
        update_available: None,
        dependencies,
        last_updated,
        disabled: false,
        user_notes: None,
    })
}

// ── scan_addons ───────────────────────────────────────────────────────────────

/// Scan the AddOns directory and return parsed addon metadata.
pub fn scan_addons(path: &str) -> Result<Vec<InstalledAddon>, String> {
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }

    let mut addons = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if let Some(addon) = read_addon_from_folder(&entry_path) {
                addons.push(addon);
            }
        }
    }

    addons.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(addons)
}

// ── validate_wow_path ─────────────────────────────────────────────────────────

/// Validate that a given path looks like an AddOns directory.
pub fn validate_wow_path(path: &str) -> Result<String, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }

    let name_lc = p
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    if name_lc == "addons" {
        return Ok("addons_dir".to_string());
    }

    if name_lc == "interface" {
        let addons_candidate = p.join("AddOns");
        if addons_candidate.exists() {
            return Ok("addons_dir".to_string());
        }
    }

    let variants: &[(&str, &str)] = &[
        ("_classic_", "classic"),
        ("_classic_era_", "classic_era"),
        ("_classic_ptr_", "classic_ptr"),
        ("_retail_", "retail"),
        ("_ptr_", "ptr"),
    ];

    for (folder, label) in variants {
        let candidate = p.join(folder).join("Interface").join("AddOns");
        if candidate.exists() {
            return Ok(label.to_string());
        }
    }

    Err("Could not find a WoW AddOns folder at this path. \
         Please point to your WoW installation root or directly to \
         the Interface/AddOns directory."
        .to_string())
}

// ── remove_addon ──────────────────────────────────────────────────────────────

/// Remove an addon folder from the AddOns directory.
pub fn remove_addon(addons_path: &str, folder: &str) -> Result<(), String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }
    let target = Path::new(addons_path).join(folder);
    if !target.exists() {
        return Ok(());
    }
    fs::remove_dir_all(&target).map_err(|e| e.to_string())
}

// ── create_test_addon_dir ─────────────────────────────────────────────────────

/// Create a test addon directory structure for development/testing.
pub fn create_test_addon_dir(base_path: &str) -> Result<String, String> {
    let addons_dir = Path::new(base_path).join("TestAddOns");
    fs::create_dir_all(&addons_dir).map_err(|e| e.to_string())?;

    let test_addons = vec![
        (
            "WeakAuras",
            "3.5.1",
            "WeakAuras",
            "Powerful and versatile display addon",
            "The WeakAuras Team",
            "11502",
        ),
        (
            "Details",
            "9.8.18.2",
            "Details! Damage Meter",
            "Damage meter and combat analyzer",
            "Tercioo",
            "11502",
        ),
        (
            "Questie",
            "7.2.0",
            "Questie",
            "Quest helper for Classic",
            "Questie Team",
            "11502",
        ),
    ];

    for (folder, version, title, notes, author, interface) in &test_addons {
        let addon_dir = addons_dir.join(folder);
        fs::create_dir_all(&addon_dir).map_err(|e| e.to_string())?;

        let toc_content = format!(
            "## Interface-Classic: {}\n## Title: {}\n## Notes: {}\n## Author: {}\n## Version: {}\n\n{}.lua\n",
            interface, title, notes, author, version, folder
        );

        let toc_path = addon_dir.join(format!("{}.toc", folder));
        let mut f = fs::File::create(toc_path).map_err(|e| e.to_string())?;
        f.write_all(toc_content.as_bytes()).map_err(|e| e.to_string())?;

        let lua_path = addon_dir.join(format!("{}.lua", folder));
        let mut lf = fs::File::create(lua_path).map_err(|e| e.to_string())?;
        lf.write_all(b"-- placeholder\n").map_err(|e| e.to_string())?;
    }

    Ok(addons_dir.to_string_lossy().to_string())
}

// ── zip extraction ────────────────────────────────────────────────────────────

/// Extract a zip archive (as raw bytes) into `addons_path`.
/// Returns the list of top-level folder names extracted.
pub fn extract_zip_bytes(bytes: &[u8], addons_path: &str) -> Result<Vec<String>, String> {
    use std::io::Cursor;

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    let dest = Path::new(addons_path);
    let mut extracted_folders: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = file.name().to_string();

        if name.contains("__MACOSX") || name.starts_with('.') {
            continue;
        }

        let out_path = dest.join(&name);

        if let Some(first_component) = PathBuf::from(&name).components().next() {
            if let std::path::Component::Normal(s) = first_component {
                extracted_folders.insert(s.to_string_lossy().to_string());
            }
        }

        if file.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut out_file = fs::File::create(&out_path).map_err(|e| e.to_string())?;
            io::copy(&mut file, &mut out_file).map_err(|e| e.to_string())?;
        }
    }

    Ok(extracted_folders.into_iter().collect())
}

// ── disable/enable addons ─────────────────────────────────────────────────────
//
// WoW doesn't have a built-in "disabled" state via a flat file per se — the
// recommended community approach is to rename the folder with a `-disabled`
// suffix (which WoW ignores during loading) or to create a `disabled.txt`
// inside the addon folder. Grimoire uses the rename approach because it is
// clean, reversible, and doesn't require modifying addon files.

/// Disable an addon by renaming its folder to `<name>-disabled`.
pub fn disable_addon(addons_path: &str, folder: &str) -> Result<(), String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }
    let src = Path::new(addons_path).join(folder);
    if !src.exists() {
        return Err(format!("Addon folder not found: {}", folder));
    }
    let dst = Path::new(addons_path).join(format!("{}-disabled", folder));
    fs::rename(&src, &dst).map_err(|e| e.to_string())
}

/// Re-enable a previously disabled addon by renaming `<name>-disabled` back.
pub fn enable_addon(addons_path: &str, folder: &str) -> Result<(), String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }
    let src = Path::new(addons_path).join(format!("{}-disabled", folder));
    if !src.exists() {
        return Err(format!("Disabled addon folder not found: {}-disabled", folder));
    }
    let dst = Path::new(addons_path).join(folder);
    fs::rename(&src, &dst).map_err(|e| e.to_string())
}

/// Scan addons and mark disabled ones (those whose folder ends with `-disabled`).
pub fn scan_addons_with_disabled(path: &str) -> Result<Vec<InstalledAddon>, String> {
    let dir = Path::new(path);
    if !dir.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }

    let mut addons = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }

        let raw_name = entry_path.file_name().unwrap().to_string_lossy().to_string();
        let (canonical_name, is_disabled) = if let Some(base) = raw_name.strip_suffix("-disabled") {
            (base.to_string(), true)
        } else {
            (raw_name.clone(), false)
        };

        // For disabled addons, look for the .toc using the canonical (non-disabled) name
        let toc_path = entry_path.join(format!("{}.toc", canonical_name));
        if let Ok(toc_content) = fs::read_to_string(&toc_path) {
            let meta = parse_toc(&toc_content);
            let interface_str = meta.get("interface_canonical").cloned().unwrap_or_default();
            let dependencies = parse_dependencies(&meta);
            let last_updated = toc_path
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs());

            addons.push(InstalledAddon {
                folder: canonical_name.clone(),
                title: meta.get("title").cloned().unwrap_or_else(|| canonical_name),
                version: meta.get("version").cloned().unwrap_or_default(),
                notes: meta.get("notes").cloned().unwrap_or_default(),
                author: meta.get("author").cloned().unwrap_or_default(),
                interface: interface_str,
                update_available: None,
                dependencies,
                last_updated,
                disabled: is_disabled,
                user_notes: None,
            });
        }
    }

    addons.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(addons)
}

// ── profiles ──────────────────────────────────────────────────────────────────

/// A named profile is simply a saved list of addon folder names.
/// Profiles are stored as JSON in `<addons_path>/../grimoire-profiles.json`
/// (one level above the AddOns folder, i.e. in Interface/).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddonProfiles {
    pub profiles: HashMap<String, Vec<String>>,
}

impl AddonProfiles {
    pub fn load(addons_path: &str) -> Self {
        let path = profiles_path(addons_path);
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, addons_path: &str) -> Result<(), String> {
        let path = profiles_path(addons_path);
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn create_profile(&mut self, name: &str, folders: Vec<String>) {
        self.profiles.insert(name.to_string(), folders);
    }

    pub fn delete_profile(&mut self, name: &str) -> bool {
        self.profiles.remove(name).is_some()
    }

    pub fn get_profile(&self, name: &str) -> Option<&Vec<String>> {
        self.profiles.get(name)
    }

    /// Snapshot the currently-installed addons into a new profile.
    pub fn snapshot_from_addons(
        &mut self,
        name: &str,
        addons: &[InstalledAddon],
    ) {
        let folders: Vec<String> = addons.iter().map(|a| a.folder.clone()).collect();
        self.create_profile(name, folders);
    }
}

fn profiles_path(addons_path: &str) -> PathBuf {
    Path::new(addons_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("grimoire-profiles.json")
}

// ── user notes ────────────────────────────────────────────────────────────────

/// Persistent per-addon user notes/tags.
/// Stored alongside profiles in `grimoire-notes.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddonNotes {
    pub notes: HashMap<String, String>,
}

impl AddonNotes {
    pub fn load(addons_path: &str) -> Self {
        let path = notes_path(addons_path);
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, addons_path: &str) -> Result<(), String> {
        let path = notes_path(addons_path);
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }

    pub fn set(&mut self, folder: &str, note: &str) {
        self.notes.insert(folder.to_string(), note.to_string());
    }

    pub fn get(&self, folder: &str) -> Option<&str> {
        self.notes.get(folder).map(|s| s.as_str())
    }

    pub fn remove(&mut self, folder: &str) {
        self.notes.remove(folder);
    }
}

fn notes_path(addons_path: &str) -> PathBuf {
    Path::new(addons_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("grimoire-notes.json")
}

// ── import / export ───────────────────────────────────────────────────────────

/// Export installed addons to a simple JSON manifest for backup/restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonManifest {
    pub version: u32,
    pub addons: Vec<AddonManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonManifestEntry {
    pub folder: String,
    pub title: String,
    pub addon_version: String,
    /// CurseForge project ID if known (optional — populated by the UI layer)
    pub curseforge_id: Option<u32>,
}

pub fn export_addon_list(addons: &[InstalledAddon]) -> AddonManifest {
    AddonManifest {
        version: 1,
        addons: addons
            .iter()
            .map(|a| AddonManifestEntry {
                folder: a.folder.clone(),
                title: a.title.clone(),
                addon_version: a.version.clone(),
                curseforge_id: None,
            })
            .collect(),
    }
}

// ── auto-backup ───────────────────────────────────────────────────────────────

/// Zip an addon folder into `<backup_dir>/<folder>-<timestamp>.zip`.
/// Returns the path of the created zip file.
///
/// NOTE: Requires the `zip` crate (already a dependency).
pub fn backup_addon(
    addons_path: &str,
    folder: &str,
    backup_dir: &str,
) -> Result<String, String> {
    if folder.contains('/') || folder.contains('\\') || folder.contains("..") {
        return Err("Invalid folder name".to_string());
    }

    let src_dir = Path::new(addons_path).join(folder);
    if !src_dir.exists() {
        return Err(format!("Addon folder not found: {}", folder));
    }

    fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let zip_name = format!("{}-{}.zip", folder, ts);
    let zip_path = Path::new(backup_dir).join(&zip_name);
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
            writer
                .add_directory(&zip_name, options)
                .map_err(|e| e.to_string())?;
            add_dir_to_zip(writer, base, &entry_path, options)?;
        } else {
            writer
                .start_file(&zip_name, options)
                .map_err(|e| e.to_string())?;
            let mut f = fs::File::open(&entry_path).map_err(|e| e.to_string())?;
            io::copy(&mut f, writer).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ── parse_toc ──────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_toc_basic_fields() {
        let toc = "## Title: WeakAuras\n## Version: 3.5.1\n## Interface: 110002\n## Notes: A powerful display addon\n## Author: The WeakAuras Team\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("WeakAuras"));
        assert_eq!(meta.get("version").map(|s| s.as_str()), Some("3.5.1"));
        assert_eq!(meta.get("interface").map(|s| s.as_str()), Some("110002"));
        assert_eq!(meta.get("notes").map(|s| s.as_str()), Some("A powerful display addon"));
        assert_eq!(meta.get("author").map(|s| s.as_str()), Some("The WeakAuras Team"));
    }

    #[test]
    fn test_parse_toc_interface_classic() {
        let toc = "## Interface-Classic: 11502\n## Title: Questie\n## Version: 7.0.0\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("interface-classic").map(|s| s.as_str()), Some("11502"));
        assert_eq!(meta.get("interface_canonical").map(|s| s.as_str()), Some("11502"));
    }

    #[test]
    fn test_parse_toc_classic_wins_over_generic_interface() {
        // When both Interface and Interface-Classic are present, Classic wins
        let toc = "## Interface: 110002\n## Interface-Classic: 11502\n## Title: SomeAddon\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("interface_canonical").map(|s| s.as_str()), Some("11502"));
    }

    #[test]
    fn test_parse_toc_generic_interface_sets_canonical_if_no_classic() {
        let toc = "## Interface: 110002\n## Title: RetailAddon\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("interface_canonical").map(|s| s.as_str()), Some("110002"));
    }

    #[test]
    fn test_parse_toc_missing_fields() {
        let toc = "## Title: MinimalAddon\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("MinimalAddon"));
        assert!(meta.get("version").is_none());
        assert!(meta.get("author").is_none());
        assert!(meta.get("notes").is_none());
    }

    #[test]
    fn test_parse_toc_empty() {
        let meta = parse_toc("");
        assert!(meta.is_empty());
    }

    #[test]
    fn test_parse_toc_extra_whitespace() {
        let toc = "##   Title  :   WeakAuras With Spaces   \n##  Version :  1.2.3  \n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("WeakAuras With Spaces"));
        assert_eq!(meta.get("version").map(|s| s.as_str()), Some("1.2.3"));
    }

    #[test]
    fn test_parse_toc_ignores_non_metadata_lines() {
        let toc = "# This is a normal comment\n## Title: MyAddon\nSomeFile.lua\nAnotherFile.lua\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("MyAddon"));
        // title + interface_canonical won't be here since no interface; only "title"
        assert_eq!(meta.len(), 1);
    }

    #[test]
    fn test_parse_toc_interface_classic_era() {
        let toc = "## Interface-Classic-Era: 11502\n## Title: EraAddon\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("interface-classic-era").map(|s| s.as_str()), Some("11502"));
        assert_eq!(meta.get("interface_canonical").map(|s| s.as_str()), Some("11502"));
    }

    #[test]
    fn test_parse_toc_keys_are_lowercased() {
        let toc = "## TITLE: AllCaps\n## Author: Someone\n";
        let meta = parse_toc(toc);
        assert_eq!(meta.get("title").map(|s| s.as_str()), Some("AllCaps"));
        assert_eq!(meta.get("author").map(|s| s.as_str()), Some("Someone"));
    }

    #[test]
    fn test_parse_toc_dependencies() {
        let toc = "## Title: MyAddon\n## Dependencies: LibStub, CallbackHandler-1.0, AceComm-3.0\n";
        let meta = parse_toc(toc);
        let deps = parse_dependencies(&meta);
        assert_eq!(deps, vec!["LibStub", "CallbackHandler-1.0", "AceComm-3.0"]);
    }

    #[test]
    fn test_parse_toc_no_dependencies() {
        let toc = "## Title: StandaloneAddon\n## Version: 1.0\n";
        let meta = parse_toc(toc);
        let deps = parse_dependencies(&meta);
        assert!(deps.is_empty());
    }

    // ── validate_wow_path ──────────────────────────────────────────────────────

    #[test]
    fn test_validate_wow_path_direct_addons_dir() {
        let tmp = TempDir::new().unwrap();
        let addons_dir = tmp.path().join("AddOns");
        fs::create_dir_all(&addons_dir).unwrap();
        let result = validate_wow_path(&addons_dir.to_string_lossy());
        assert_eq!(result, Ok("addons_dir".to_string()));
    }

    #[test]
    fn test_validate_wow_path_case_insensitive_addons() {
        let tmp = TempDir::new().unwrap();
        let addons_dir = tmp.path().join("addons");
        fs::create_dir_all(&addons_dir).unwrap();
        let result = validate_wow_path(&addons_dir.to_string_lossy());
        assert_eq!(result, Ok("addons_dir".to_string()));
    }

    #[test]
    fn test_validate_wow_path_interface_subdir() {
        let tmp = TempDir::new().unwrap();
        let interface_dir = tmp.path().join("Interface");
        let addons_dir = interface_dir.join("AddOns");
        fs::create_dir_all(&addons_dir).unwrap();
        let result = validate_wow_path(&interface_dir.to_string_lossy());
        assert_eq!(result, Ok("addons_dir".to_string()));
    }

    #[test]
    fn test_validate_wow_path_classic_root() {
        let tmp = TempDir::new().unwrap();
        let classic_addons = tmp.path().join("_classic_").join("Interface").join("AddOns");
        fs::create_dir_all(&classic_addons).unwrap();
        let result = validate_wow_path(&tmp.path().to_string_lossy());
        assert_eq!(result, Ok("classic".to_string()));
    }

    #[test]
    fn test_validate_wow_path_classic_era_root() {
        let tmp = TempDir::new().unwrap();
        let era_addons = tmp.path().join("_classic_era_").join("Interface").join("AddOns");
        fs::create_dir_all(&era_addons).unwrap();
        let result = validate_wow_path(&tmp.path().to_string_lossy());
        assert_eq!(result, Ok("classic_era".to_string()));
    }

    #[test]
    fn test_validate_wow_path_retail_root() {
        let tmp = TempDir::new().unwrap();
        let retail_addons = tmp.path().join("_retail_").join("Interface").join("AddOns");
        fs::create_dir_all(&retail_addons).unwrap();
        let result = validate_wow_path(&tmp.path().to_string_lossy());
        assert_eq!(result, Ok("retail".to_string()));
    }

    #[test]
    fn test_validate_wow_path_nonexistent() {
        let result = validate_wow_path("/nonexistent/path/that/does/not/exist");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not exist"));
    }

    #[test]
    fn test_validate_wow_path_wrong_dir() {
        let tmp = TempDir::new().unwrap();
        let result = validate_wow_path(&tmp.path().to_string_lossy());
        assert!(result.is_err());
    }

    // ── scan_addons ────────────────────────────────────────────────────────────

    fn make_fake_addon(addons_dir: &Path, folder: &str, title: &str, version: &str, interface: &str) {
        let addon_dir = addons_dir.join(folder);
        fs::create_dir_all(&addon_dir).unwrap();
        let toc = format!(
            "## Title: {}\n## Version: {}\n## Interface: {}\n## Author: TestAuthor\n## Notes: Test addon\n",
            title, version, interface
        );
        fs::write(addon_dir.join(format!("{}.toc", folder)), toc).unwrap();
    }

    #[test]
    fn test_scan_addons_basic() {
        let tmp = TempDir::new().unwrap();
        make_fake_addon(tmp.path(), "WeakAuras", "WeakAuras", "3.5.1", "110002");
        make_fake_addon(tmp.path(), "Details", "Details! Damage Meter", "9.0.0", "110002");

        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        assert_eq!(addons.len(), 2);
        // Sorted alphabetically by title
        assert_eq!(addons[0].title, "Details! Damage Meter");
        assert_eq!(addons[1].title, "WeakAuras");
    }

    #[test]
    fn test_scan_addons_metadata() {
        let tmp = TempDir::new().unwrap();
        make_fake_addon(tmp.path(), "Questie", "Questie", "7.2.0", "11502");

        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        assert_eq!(addons.len(), 1);

        let addon = &addons[0];
        assert_eq!(addon.folder, "Questie");
        assert_eq!(addon.title, "Questie");
        assert_eq!(addon.version, "7.2.0");
        assert_eq!(addon.author, "TestAuthor");
        assert_eq!(addon.notes, "Test addon");
        assert_eq!(addon.interface, "11502");
        assert!(addon.update_available.is_none());
    }

    #[test]
    fn test_scan_addons_skips_folders_without_toc() {
        let tmp = TempDir::new().unwrap();
        make_fake_addon(tmp.path(), "ValidAddon", "ValidAddon", "1.0", "110002");
        fs::create_dir_all(tmp.path().join("NoTocAddon")).unwrap();

        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        assert_eq!(addons.len(), 1);
        assert_eq!(addons[0].folder, "ValidAddon");
    }

    #[test]
    fn test_scan_addons_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        assert!(addons.is_empty());
    }

    #[test]
    fn test_scan_addons_nonexistent_path() {
        let result = scan_addons("/no/such/directory");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_addons_uses_folder_name_as_fallback_title() {
        let tmp = TempDir::new().unwrap();
        let addon_dir = tmp.path().join("MyAddon");
        fs::create_dir_all(&addon_dir).unwrap();
        fs::write(addon_dir.join("MyAddon.toc"), "## Version: 1.0\n").unwrap();

        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        assert_eq!(addons.len(), 1);
        assert_eq!(addons[0].title, "MyAddon");
    }

    // ── create_test_addon_dir ──────────────────────────────────────────────────

    #[test]
    fn test_create_test_addon_dir_creates_folder() {
        let tmp = TempDir::new().unwrap();
        let result = create_test_addon_dir(&tmp.path().to_string_lossy());
        assert!(result.is_ok());
        assert!(tmp.path().join("TestAddOns").exists());
    }

    #[test]
    fn test_create_test_addon_dir_contains_expected_addons() {
        let tmp = TempDir::new().unwrap();
        create_test_addon_dir(&tmp.path().to_string_lossy()).unwrap();
        let addons_dir = tmp.path().join("TestAddOns");
        for folder in &["WeakAuras", "Details", "Questie"] {
            let addon_dir = addons_dir.join(folder);
            assert!(addon_dir.exists(), "Addon folder {} should exist", folder);
            assert!(addon_dir.join(format!("{}.toc", folder)).exists());
            assert!(addon_dir.join(format!("{}.lua", folder)).exists());
        }
    }

    #[test]
    fn test_create_test_addon_dir_toc_is_parseable() {
        let tmp = TempDir::new().unwrap();
        create_test_addon_dir(&tmp.path().to_string_lossy()).unwrap();
        let toc_path = tmp.path().join("TestAddOns").join("WeakAuras").join("WeakAuras.toc");
        let toc_content = fs::read_to_string(toc_path).unwrap();
        let meta = parse_toc(&toc_content);
        assert!(meta.get("title").is_some());
        assert!(meta.get("version").is_some());
    }

    #[test]
    fn test_create_test_addon_dir_scannable() {
        let tmp = TempDir::new().unwrap();
        create_test_addon_dir(&tmp.path().to_string_lossy()).unwrap();
        let addons_dir = tmp.path().join("TestAddOns");
        let addons = scan_addons(&addons_dir.to_string_lossy()).unwrap();
        assert_eq!(addons.len(), 3);
    }

    // ── remove_addon ───────────────────────────────────────────────────────────

    #[test]
    fn test_remove_addon_rejects_path_traversal_with_dots() {
        let tmp = TempDir::new().unwrap();
        let result = remove_addon(&tmp.path().to_string_lossy(), "../../../etc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid folder name");
    }

    #[test]
    fn test_remove_addon_rejects_forward_slash() {
        let tmp = TempDir::new().unwrap();
        let result = remove_addon(&tmp.path().to_string_lossy(), "some/nested/path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid folder name");
    }

    #[test]
    fn test_remove_addon_rejects_backslash() {
        let tmp = TempDir::new().unwrap();
        let result = remove_addon(&tmp.path().to_string_lossy(), r"some\path");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid folder name");
    }

    #[test]
    fn test_remove_addon_rejects_double_dots() {
        let tmp = TempDir::new().unwrap();
        let result = remove_addon(&tmp.path().to_string_lossy(), "..");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid folder name");
    }

    #[test]
    fn test_remove_addon_removes_existing_folder() {
        let tmp = TempDir::new().unwrap();
        let addon_dir = tmp.path().join("WeakAuras");
        fs::create_dir_all(&addon_dir).unwrap();
        fs::write(addon_dir.join("WeakAuras.toc"), "## Title: WeakAuras\n").unwrap();

        assert!(addon_dir.exists());
        let result = remove_addon(&tmp.path().to_string_lossy(), "WeakAuras");
        assert!(result.is_ok());
        assert!(!addon_dir.exists());
    }

    #[test]
    fn test_remove_addon_ok_when_already_gone() {
        let tmp = TempDir::new().unwrap();
        let result = remove_addon(&tmp.path().to_string_lossy(), "NonExistentAddon");
        assert!(result.is_ok());
    }

    // ── disable / enable ───────────────────────────────────────────────────────

    #[test]
    fn test_disable_addon() {
        let tmp = TempDir::new().unwrap();
        let addon_dir = tmp.path().join("WeakAuras");
        fs::create_dir_all(&addon_dir).unwrap();
        fs::write(addon_dir.join("WeakAuras.toc"), "## Title: WeakAuras\n").unwrap();

        disable_addon(&tmp.path().to_string_lossy(), "WeakAuras").unwrap();

        assert!(!addon_dir.exists());
        assert!(tmp.path().join("WeakAuras-disabled").exists());
    }

    #[test]
    fn test_enable_addon() {
        let tmp = TempDir::new().unwrap();
        let disabled_dir = tmp.path().join("WeakAuras-disabled");
        fs::create_dir_all(&disabled_dir).unwrap();
        fs::write(disabled_dir.join("WeakAuras.toc"), "## Title: WeakAuras\n").unwrap();

        enable_addon(&tmp.path().to_string_lossy(), "WeakAuras").unwrap();

        assert!(!disabled_dir.exists());
        assert!(tmp.path().join("WeakAuras").exists());
    }

    #[test]
    fn test_disable_addon_rejects_traversal() {
        let tmp = TempDir::new().unwrap();
        let result = disable_addon(&tmp.path().to_string_lossy(), "../evil");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_addons_with_disabled_marks_disabled() {
        let tmp = TempDir::new().unwrap();
        // Normal addon
        make_fake_addon(tmp.path(), "Questie", "Questie", "7.0.0", "11502");
        // Disabled addon folder
        let dis_dir = tmp.path().join("WeakAuras-disabled");
        fs::create_dir_all(&dis_dir).unwrap();
        fs::write(
            dis_dir.join("WeakAuras.toc"),
            "## Title: WeakAuras\n## Version: 3.5.1\n## Interface: 11502\n",
        )
        .unwrap();

        let addons = scan_addons_with_disabled(&tmp.path().to_string_lossy()).unwrap();
        assert_eq!(addons.len(), 2);

        let wa = addons.iter().find(|a| a.folder == "WeakAuras").unwrap();
        assert!(wa.disabled);

        let q = addons.iter().find(|a| a.folder == "Questie").unwrap();
        assert!(!q.disabled);
    }

    // ── profiles ───────────────────────────────────────────────────────────────

    #[test]
    fn test_profiles_create_and_retrieve() {
        let tmp = TempDir::new().unwrap();
        let addons_path = tmp.path().to_string_lossy().to_string();

        let mut profiles = AddonProfiles::default();
        profiles.create_profile("raiding", vec!["WeakAuras".into(), "Details".into()]);
        profiles.save(&addons_path).unwrap();

        let loaded = AddonProfiles::load(&addons_path);
        let raiding = loaded.get_profile("raiding").unwrap();
        assert_eq!(raiding, &vec!["WeakAuras".to_string(), "Details".to_string()]);
    }

    #[test]
    fn test_profiles_delete() {
        let tmp = TempDir::new().unwrap();
        let addons_path = tmp.path().to_string_lossy().to_string();

        let mut profiles = AddonProfiles::default();
        profiles.create_profile("leveling", vec!["Questie".into()]);
        assert!(profiles.delete_profile("leveling"));
        assert!(profiles.get_profile("leveling").is_none());
        profiles.save(&addons_path).unwrap();

        let loaded = AddonProfiles::load(&addons_path);
        assert!(loaded.get_profile("leveling").is_none());
    }

    #[test]
    fn test_profiles_snapshot_from_addons() {
        let tmp = TempDir::new().unwrap();
        make_fake_addon(tmp.path(), "WeakAuras", "WeakAuras", "3.5.1", "110002");
        make_fake_addon(tmp.path(), "Details", "Details", "9.0.0", "110002");

        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        let mut profiles = AddonProfiles::default();
        profiles.snapshot_from_addons("current", &addons);

        let folders = profiles.get_profile("current").unwrap();
        assert_eq!(folders.len(), 2);
        assert!(folders.contains(&"WeakAuras".to_string()));
        assert!(folders.contains(&"Details".to_string()));
    }

    // ── user notes ─────────────────────────────────────────────────────────────

    #[test]
    fn test_user_notes_set_get_save_load() {
        let tmp = TempDir::new().unwrap();
        let addons_path = tmp.path().to_string_lossy().to_string();

        let mut notes = AddonNotes::default();
        notes.set("WeakAuras", "Essential for progression");
        notes.save(&addons_path).unwrap();

        let loaded = AddonNotes::load(&addons_path);
        assert_eq!(loaded.get("WeakAuras"), Some("Essential for progression"));
        assert!(loaded.get("Details").is_none());
    }

    #[test]
    fn test_user_notes_remove() {
        let mut notes = AddonNotes::default();
        notes.set("Questie", "Quest helper");
        notes.remove("Questie");
        assert!(notes.get("Questie").is_none());
    }

    // ── export ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_export_addon_list() {
        let tmp = TempDir::new().unwrap();
        make_fake_addon(tmp.path(), "WeakAuras", "WeakAuras", "3.5.1", "110002");
        let addons = scan_addons(&tmp.path().to_string_lossy()).unwrap();
        let manifest = export_addon_list(&addons);
        assert_eq!(manifest.version, 1);
        assert_eq!(manifest.addons.len(), 1);
        assert_eq!(manifest.addons[0].folder, "WeakAuras");
        assert_eq!(manifest.addons[0].addon_version, "3.5.1");
        assert!(manifest.addons[0].curseforge_id.is_none());
    }

    // ── backup ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_backup_addon_creates_zip() {
        let tmp = TempDir::new().unwrap();
        make_fake_addon(tmp.path(), "WeakAuras", "WeakAuras", "3.5.1", "110002");

        let backup_dir = tmp.path().join("backups");
        let result = backup_addon(
            &tmp.path().to_string_lossy(),
            "WeakAuras",
            &backup_dir.to_string_lossy(),
        );
        assert!(result.is_ok(), "Backup should succeed: {:?}", result.err());

        let zip_path = result.unwrap();
        assert!(Path::new(&zip_path).exists(), "Zip file should exist");
        assert!(zip_path.ends_with(".zip"));
    }

    #[test]
    fn test_backup_addon_rejects_traversal() {
        let tmp = TempDir::new().unwrap();
        let result = backup_addon(&tmp.path().to_string_lossy(), "../evil", "/tmp");
        assert!(result.is_err());
    }
}
