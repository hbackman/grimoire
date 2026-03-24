import {invoke} from "@tauri-apps/api/core";
import {listen} from "@tauri-apps/api/event";
import {parse}  from "node-html-parser";

// ── Request queue (serialises scrape calls) ────────────────────────────────

let requestQueue = [];
let isProcessing = false;

export async function scrape(url) {
  return new Promise((resolve, reject) => {
    requestQueue.push({ url, resolve, reject });
    processQueue();
  });
}

async function processQueue() {
  if (isProcessing || requestQueue.length === 0) return;
  isProcessing = true;

  while (requestQueue.length > 0) {
    const { url, resolve, reject } = requestQueue.shift();
    try {
      resolve(await scrapeInternal(url));
    } catch (error) {
      reject(error);
    }
  }

  isProcessing = false;
}

async function scrapeInternal(url) {
  let unlistenOk;
  let unlistenErr;

  const resultP = new Promise(async (resolve, reject) => {
    unlistenOk  = await listen("scraper:result", (evt) => resolve(evt.payload));
    unlistenErr = await listen("scraper:error",  (evt) => reject(new Error(evt.payload || "unknown error")));
  });

  await invoke("scrape_in_webview", { url });

  const data = await resultP;

  if (unlistenOk)  unlistenOk();
  if (unlistenErr) unlistenErr();

  return data;
}

// ── HTML extraction helpers ────────────────────────────────────────────────

async function extractAddonsFromHtml(html) {
  return parse(html).querySelectorAll(".project-card")
    .map(e => {
      let name = e.querySelector(".name")
        .getAttribute("href")
        .split("/");
      name = name[name.length - 1];

      return {
        name,
        image:       e.querySelector(".art img")?.getAttribute("src") ?? "",
        title:       e.querySelector(".name")?.text.trim() ?? name,
        description: e.querySelector(".description")?.text.trim() ?? "",
        author:      e.querySelector(".author")?.text.trim() ?? "",
      };
    });
}

/**
 * Extract the most-recent file download link for a given gameVersionTypeId.
 *
 * @param {string} html
 * @param {number} gameVersionTypeId  e.g. 517 (retail), 79434 (MoP), 67408 (classic era)
 * @returns {string|null}  A CurseForge file page URL or null
 */
export function extractLatestFileUrl(html, gameVersionTypeId) {
  const root = parse(html);

  // .file-card links on the /files/all page
  const cards = root.querySelectorAll(".file-card");
  if (cards.length === 0) return null;

  // Return href of the first match
  const href = cards[0].getAttribute("href");
  return href ? `https://www.curseforge.com${href}` : null;
}

/**
 * From a /files/<id> detail page, extract the direct download URL.
 * CurseForge renders it as a <a> with data-action="cf-download" or a
 * Download button.
 */
export function extractDirectDownloadUrl(html) {
  const root = parse(html);

  // Try data-action download button
  let el = root.querySelector('a[data-action="cf-download"]');
  if (el) return el.getAttribute("href");

  // Fallback: look for /download in links
  const links = root.querySelectorAll("a");
  for (const link of links) {
    const href = link.getAttribute("href") || "";
    if (href.includes("/download") && href.includes("curseforge.com")) {
      return href;
    }
  }

  return null;
}

/**
 * Extract version string from a file card page (from the file name or title).
 */
export function extractVersionFromFilesPage(html) {
  const root = parse(html);

  // File cards usually have a version in the file name displayed
  const firstCard = root.querySelector(".file-card");
  if (!firstCard) return null;

  // Try the file display name
  const nameEl = firstCard.querySelector(".name") || firstCard.querySelector(".file-name");
  if (nameEl) {
    const text = nameEl.text.trim();
    // Extract semver-ish pattern
    const match = text.match(/(\d+\.\d+[\.\d]*)/);
    if (match) return match[1];
  }

  return null;
}

// ── Public API ─────────────────────────────────────────────────────────────

/**
 * Browse/search CurseForge addons.
 */
export async function browse(options = {}) {
  const page   = options.page   ?? 1;
  const size   = options.size   ?? 20;
  const search = options.search ?? null;

  const data = await scrape(
    search
      ? `https://www.curseforge.com/wow/search?page=${page}&pageSize=${size}&sortBy=relevancy&search=${encodeURIComponent(search)}`
      : `https://www.curseforge.com/wow/search?page=${page}&pageSize=${size}&sortBy=relevancy&class=addons`
  );
  return extractAddonsFromHtml(data.html);
}

/**
 * Get the download URL for a CurseForge addon slug + game version type ID.
 * Returns { downloadUrl, version } or throws.
 */
export async function getAddonDownloadUrl(addonSlug, gameVersionTypeId) {
  const filesUrl = `https://www.curseforge.com/wow/addons/${addonSlug}/files/all?page=1&pageSize=20&gameVersionTypeId=${gameVersionTypeId}`;
  const data     = await scrape(filesUrl);

  const fileUrl  = extractLatestFileUrl(data.html, gameVersionTypeId);
  if (!fileUrl) throw new Error(`No files found for ${addonSlug}`);

  const version  = extractVersionFromFilesPage(data.html);

  // Navigate to the file detail page to get the real download link
  const fileData = await scrape(fileUrl);
  let downloadUrl = extractDirectDownloadUrl(fileData.html);

  // CurseForge direct download link pattern: /api/v1/mods/{modId}/files/{fileId}/download
  // If still not found, try to construct from the file page URL
  if (!downloadUrl) {
    // fileUrl looks like /wow/addons/<slug>/files/<fileId>
    const m = fileUrl.match(/\/files\/(\d+)$/);
    if (m) {
      // Try the standard download endpoint
      downloadUrl = `https://www.curseforge.com/wow/addons/${addonSlug}/download/${m[1]}`;
    }
  }

  if (!downloadUrl) throw new Error("Could not find download URL");

  return { downloadUrl, version };
}

/**
 * Install an addon by slug into the given AddOns directory.
 *
 * CurseForge download pages have a 5-second countdown before the actual file
 * download starts. We use `download_addon_via_webview` which opens a hidden
 * WebView, navigates to the download page, and intercepts the file download
 * via Tauri's native download event API. This avoids the need to reverse-engineer
 * the direct CDN URL.
 *
 * Returns the list of folders extracted.
 */
export async function installAddon(addonSlug, gameVersionTypeId, addonsPath) {
  const { downloadUrl } = await getAddonDownloadUrl(addonSlug, gameVersionTypeId);

  // downloadUrl is typically a CurseForge download page URL like:
  // https://www.curseforge.com/wow/addons/<slug>/download/<fileId>
  // We navigate the hidden WebView to this page; Tauri intercepts the resulting
  // file download (triggered by the 5s countdown) and extracts it automatically.
  return await invoke("download_addon_via_webview", {
    downloadPageUrl: downloadUrl,
    addonsPath,
  });
}

/**
 * Check the latest version of an addon on CurseForge.
 * Returns a version string or null.
 */
export async function getLatestVersion(addonSlug, gameVersionTypeId) {
  try {
    const filesUrl = `https://www.curseforge.com/wow/addons/${addonSlug}/files/all?page=1&pageSize=5&gameVersionTypeId=${gameVersionTypeId}`;
    const data     = await scrape(filesUrl);
    return extractVersionFromFilesPage(data.html);
  } catch {
    return null;
  }
}

/**
 * Hardcoded fallback in case scraping fails.
 */
const FALLBACK_VERSIONS = [
  { label: "Classic Era (1.x)",     value: 67408 },
  { label: "Cataclysm Classic",     value: 73246 },
  { label: "MoP Classic",           value: 79434 },
  { label: "Retail",                value: 517   },
];

/**
 * Extract game version filter options from a CurseForge files page.
 * Looks for links or options containing gameVersionTypeId in the HTML.
 */
export function extractGameVersions(html) {
  const root = parse(html);
  const versions = [];
  const seen = new Set();

  // Look for links with gameVersionTypeId in href
  for (const el of root.querySelectorAll("a[href]")) {
    const href = el.getAttribute("href") || "";
    const match = href.match(/gameVersionTypeId=(\d+)/);
    if (match) {
      const value = parseInt(match[1], 10);
      if (!seen.has(value)) {
        seen.add(value);
        versions.push({ label: el.text.trim(), value });
      }
    }
  }

  // Also look for <option> elements with gameVersionTypeId values
  if (versions.length === 0) {
    for (const el of root.querySelectorAll("option[value]")) {
      const val = el.getAttribute("value") || "";
      const match = val.match(/gameVersionTypeId=(\d+)/) || (val.match(/^\d+$/) ? [null, val] : null);
      if (match) {
        const value = parseInt(match[1], 10);
        if (!seen.has(value) && value > 0) {
          seen.add(value);
          versions.push({ label: el.text.trim(), value });
        }
      }
    }
  }

  return versions;
}

let cachedVersions = null;

/**
 * Fetch available game versions by scraping a CurseForge addon files page.
 * Results are cached for the session. Falls back to hardcoded list on failure.
 */
export async function fetchGameVersions() {
  if (cachedVersions) return cachedVersions;

  try {
    // Use a popular addon that will always have files across all game versions
    const data = await scrape(
      "https://www.curseforge.com/wow/addons/weakauras-2/files/all"
    );
    const versions = extractGameVersions(data.html);
    if (versions.length > 0) {
      cachedVersions = versions;
      return versions;
    }
  } catch {
    // Fall through to fallback
  }

  cachedVersions = FALLBACK_VERSIONS;
  return cachedVersions;
}

/**
 * Synchronous fallback for immediate use before async fetch completes.
 */
export function versions() {
  return cachedVersions || FALLBACK_VERSIONS;
}

/**
 * The default/fallback game version type ID to use when none is stored.
 * Classic Era is our primary target.
 */
export const DEFAULT_GAME_VERSION = 67408;
