<template>
  <div>
    <!-- Toolbar -->
    <div class="flex items-center justify-between mb-4 gap-2 flex-wrap">
      <h2 class="text-base font-semibold text-zinc-900 dark:text-zinc-100">
        Installed Addons
        <span
          v-if="addons.length"
          class="ml-1.5 text-xs font-normal text-zinc-400"
        >
          ({{ addons.length }})
        </span>
      </h2>

      <div class="flex gap-2 flex-wrap">
        <button
          @click="checkUpdates"
          :disabled="checkingUpdates || !addons.length"
          class="px-3 py-1.5 text-xs font-medium text-zinc-700 bg-white border border-zinc-200 rounded-lg
                 hover:bg-zinc-50 hover:border-zinc-300 disabled:opacity-50 disabled:cursor-not-allowed
                 dark:bg-zinc-800 dark:text-zinc-300 dark:border-zinc-700 dark:hover:bg-zinc-700
                 transition-all duration-200 flex items-center gap-1.5 cursor-pointer"
        >
          <Spinner v-if="checkingUpdates" />
          <span>{{ checkingUpdates ? 'Checking…' : 'Check for Updates' }}</span>
        </button>

        <button
          v-if="addonsWithUpdates.length"
          @click="updateAll"
          :disabled="updatingAll"
          class="px-3 py-1.5 text-xs font-medium text-white bg-amber-500 rounded-lg
                 hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed
                 transition-all duration-200"
        >
          <span v-if="updatingAll">Updating…</span>
          <span v-else>Update All ({{ addonsWithUpdates.length }})</span>
        </button>

        <!-- Export addon list -->
        <button
          v-if="addons.length"
          @click="exportList"
          class="px-3 py-1.5 text-xs font-medium text-zinc-700 bg-white border border-zinc-200 rounded-lg
                 hover:bg-zinc-50 hover:border-zinc-300
                 dark:bg-zinc-800 dark:text-zinc-300 dark:border-zinc-700 dark:hover:bg-zinc-700
                 transition-all duration-200"
          title="Export list of installed addons as JSON"
        >
          Export List
        </button>
      </div>
    </div>

    <!-- Update-all progress -->
    <div
      v-if="updatingAll && updateProgress"
      class="mb-3 px-3 py-2 text-xs text-zinc-600 dark:text-zinc-400 bg-zinc-50 dark:bg-zinc-800 rounded-lg"
    >
      {{ updateProgress }}
    </div>

    <!-- All up to date -->
    <div
      v-if="allUpToDate"
      class="mb-3 flex items-center gap-2 px-3 py-2 text-xs font-medium text-emerald-700 bg-emerald-50 border border-emerald-200 rounded-lg
             dark:text-emerald-400 dark:bg-emerald-900/20 dark:border-emerald-800"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5 flex-shrink-0" viewBox="0 0 20 20" fill="currentColor">
        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
      </svg>
      All addons are up to date
    </div>

    <!-- Empty state -->
    <div
      v-if="!addons.length"
      class="text-center py-12 text-zinc-400 dark:text-zinc-600"
    >
      <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mx-auto mb-3 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
        <path stroke-linecap="round" stroke-linejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4"/>
      </svg>
      <p class="text-sm">No addons installed</p>
      <p class="text-xs mt-1">Search for addons to install them</p>
    </div>

    <!-- Addon list -->
    <template v-else>
      <div
        v-for="addon in addons"
        :key="addon.slug"
        :installed="true"
        :image="addon.image"
        :title="addon.title"
        :folder="addon.slug"
        :version="addon.version"
        :notes="addon.notes"
        :author="addon.author"
        :update-available="addon.updateAvailable"
        :disabled="checkingUpdates"
        :on-remove="() => removeAddon(addon)"
        :on-update="addon.updateAvailable ? () => updateAddon(addon) : null"
        class="mb-3"
        :class="{ 'opacity-50': addon.disabled }"
      >
        <Addon
          :installed="true"
          :title="addon.title"
          :folder="addon.folder"
          :version="addon.version"
          :notes="addon.notes"
          :author="addon.author"
          :update-available="addon.updateAvailable"
          :on-remove="() => removeAddon(addon)"
          :on-update="addon.updateAvailable ? () => updateAddon(addon) : null"
        />

        <!-- Dependency info -->
        <div
          v-if="addon.dependencies && addon.dependencies.length"
          class="mt-1 ml-3 text-xs text-zinc-400 dark:text-zinc-600"
        >
          Requires: {{ addon.dependencies.join(', ') }}
        </div>

        <!-- Disable / Enable + Notes row -->
        <div class="mt-1 ml-3 flex items-center gap-2 flex-wrap">
          <button
            @click="toggleDisable(addon)"
            class="text-xs text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 transition-colors"
          >
            {{ addon.disabled ? '▶ Enable' : '⏸ Disable' }}
          </button>

          <span class="text-zinc-200 dark:text-zinc-700">|</span>

          <!-- Inline notes -->
          <span
            v-if="!editingNotes[addon.folder]"
            @click="startEditNote(addon.folder)"
            class="text-xs text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 cursor-pointer italic transition-colors"
          >
            {{ addonNotes[addon.folder] || '+ add note' }}
          </span>

          <span v-else class="flex items-center gap-1">
            <input
              v-model="noteInputs[addon.folder]"
              type="text"
              placeholder="Add a personal note…"
              class="text-xs px-2 py-0.5 rounded border border-zinc-300 dark:border-zinc-600
                     bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100
                     focus:outline-none focus:ring-1 focus:ring-amber-400"
              @keydown.enter="saveNote(addon.folder)"
              @keydown.escape="cancelEditNote(addon.folder)"
              @blur="saveNote(addon.folder)"
              style="min-width: 180px"
            />
          </span>
        </div>
      </div>
    </template>

    <!-- Export modal / display area -->
    <div
      v-if="exportedJson"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      @click.self="exportedJson = ''"
    >
      <div class="bg-white dark:bg-zinc-900 rounded-2xl shadow-2xl p-6 max-w-xl w-full mx-4">
        <h3 class="text-sm font-semibold text-zinc-900 dark:text-zinc-100 mb-3">Exported Addon List</h3>
        <textarea
          :value="exportedJson"
          readonly
          class="w-full h-64 text-xs font-mono bg-zinc-50 dark:bg-zinc-800 rounded-lg p-3 border border-zinc-200 dark:border-zinc-700 resize-none focus:outline-none"
        />
        <div class="mt-3 flex justify-end gap-2">
          <button
            @click="copyExport"
            class="px-3 py-1.5 text-xs font-medium text-white bg-amber-500 rounded-lg hover:bg-amber-600 transition-all"
          >
            Copy to Clipboard
          </button>
          <button
            @click="exportedJson = ''"
            class="px-3 py-1.5 text-xs font-medium text-zinc-600 dark:text-zinc-400 hover:text-zinc-900 dark:hover:text-zinc-100 transition-all"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke }                   from "@tauri-apps/api/core";
import { Store }                    from "@tauri-apps/plugin-store";
import { ask }                      from "@tauri-apps/plugin-dialog";
import { getLatestVersion, getAddonDownloadUrl, DEFAULT_GAME_VERSION } from "@/lib/curseforge.js";

import Addon   from "@/components/Addon.vue";
import Spinner from "@/components/Spinner.vue";

const emit = defineEmits(["refresh"]);

const addons          = ref([]);
const checkingUpdates = ref(false);
const updatingAll     = ref(false);
const updateProgress  = ref("");
const allUpToDate     = ref(false);

// Post-MVP state
const addonNotes    = ref({});  // { folder: noteText }
const editingNotes  = reactive({});
const noteInputs    = reactive({});
const exportedJson  = ref('');

let store       = null;
let addonsPath  = "";
let gameVersion = DEFAULT_GAME_VERSION;

const addonsWithUpdates = computed(() =>
  addons.value.filter(a => a.updateAvailable)
);

const loadSettings = async () => {
  store       = await Store.load("settings.json");
  addonsPath  = await store.get("gameAddonPath") ?? "";
  const stored = await store.get("gameVersion");
  gameVersion = (stored !== null && stored !== undefined) ? stored : DEFAULT_GAME_VERSION;
};

const loadNotes = async () => {
  if (!addonsPath) return;
  try {
    addonNotes.value = await invoke("load_notes", { addonsPath });
  } catch (e) {
    console.error("load_notes error:", e);
  }
};

const scan = async () => {
  if (!addonsPath) return;
  try {
    // Get manifest (slug -> { slug, title, image, folders })
    const manifest = (await store.get("addonManifest")) ?? {};

    // Scan filesystem for version/author/notes from .toc files
    const scanned = await invoke("scan_addons", { path: addonsPath });
    const scannedMap = {};
    for (const a of scanned) {
      scannedMap[a.folder] = a;
    }

    // Build one entry per manifest addon, using primary folder's .toc data
    addons.value = Object.values(manifest)
      .map(entry => {
        const primary = scannedMap[entry.folders[0]];
        return {
          slug:    entry.slug,
          title:   entry.title,
          image:   entry.image,
          folders: entry.folders,
          cfVersion: entry.cfVersion || "",
          version: primary?.version ?? "",
          notes:   primary?.notes ?? "",
          author:  primary?.author ?? "",
          updateAvailable: null,
        };
      })
      .filter(a => {
        // Only show addons whose primary folder still exists on disk
        return scannedMap[a.folders?.[0]];
      })
      .sort((a, b) => a.title.localeCompare(b.title, undefined, { sensitivity: "base" }));
  } catch (e) {
    console.error("scan error:", e);
  }
};

const checkUpdates = async () => {
  if (!addons.value.length) return;
  checkingUpdates.value = true;
  allUpToDate.value     = false;

  try {
    const checks = addons.value.map(async (addon) => {
      const slug    = addon.slug;
      const latest  = await getLatestVersion(slug, gameVersion).catch(() => null);
      // Compare against the CurseForge version stored at install/update time,
      // not the .toc version (which uses a different format).
      const installed = addon.cfVersion || addon.version;
      if (latest && latest !== installed) {
        addon.updateAvailable = latest;
      } else {
        addon.updateAvailable = null;
      }
    });

    // Batch sequentially to avoid hammering the scraper queue
    for (const check of checks) {
      await check;
    }

    allUpToDate.value = addonsWithUpdates.value.length === 0;
  } finally {
    checkingUpdates.value = false;
  }
};

const updateAddon = async (addon) => {
  const slug = addon.slug;
  try {
    const { downloadUrl, version: cfVersion } = await getAddonDownloadUrl(slug, gameVersion);
    // Use the WebView-based downloader so the 5-second CurseForge countdown
    // fires naturally and we intercept the actual file download.
    const folders = await invoke("download_addon_via_webview", {
      downloadPageUrl: downloadUrl,
      addonsPath,
    });

    // Update manifest with new CurseForge version and folders
    const manifest = (await store.get("addonManifest")) ?? {};
    if (manifest[slug]) {
      if (cfVersion) manifest[slug].cfVersion = cfVersion;
      if (folders?.length) manifest[slug].folders = folders;
      await store.set("addonManifest", manifest);
      await store.save();
    }

    addon.updateAvailable = null;
    await scan();
  } catch (e) {
    console.error("Update error:", e);
    alert(`Failed to update ${addon.title}: ${e.message || e}`);
  }
};

const updateAll = async () => {
  updatingAll.value = true;
  const toUpdate    = [...addonsWithUpdates.value];

  for (let i = 0; i < toUpdate.length; i++) {
    const addon = toUpdate[i];
    updateProgress.value = `Updating ${addon.title} (${i + 1}/${toUpdate.length})…`;
    await updateAddon(addon);
  }

  updatingAll.value = false;
  allUpToDate.value = true;
};

const removeAddon = async (addon) => {
  const yes = await ask(`Remove ${addon.title}?`, { title: "Confirm Removal", kind: "warning" });
  if (!yes) return;
  try {
    // Remove all folders belonging to this addon
    for (const folder of addon.folders) {
      await invoke("remove_addon", { addonsPath, folder });
    }
    // Remove from manifest
    const manifest = (await store.get("addonManifest")) ?? {};
    delete manifest[addon.slug];
    await store.set("addonManifest", manifest);
    await store.save();
    await scan();
  } catch (e) {
    console.error("Remove error:", e);
    alert(`Failed to remove ${addon.title}: ${e.message || e}`);
  }
};

// ── Post-MVP: disable / enable ───────────────────────────────────────────────
const toggleDisable = async (addon) => {
  if (!addonsPath) return;
  try {
    if (addon.disabled) {
      await invoke("enable_addon", { addonsPath, folder: addon.folder });
    } else {
      await invoke("disable_addon", { addonsPath, folder: addon.folder });
    }
    await scan();
  } catch (e) {
    console.error("toggle disable error:", e);
    alert(`Failed: ${e}`);
  }
};

// ── Post-MVP: notes ──────────────────────────────────────────────────────────
const startEditNote = (folder) => {
  noteInputs[folder]   = addonNotes.value[folder] || '';
  editingNotes[folder] = true;
};

const saveNote = async (folder) => {
  if (!editingNotes[folder]) return;
  editingNotes[folder] = false;
  const note = noteInputs[folder] ?? '';
  try {
    await invoke("save_note", { addonsPath, folder, note });
    await loadNotes();
  } catch (e) {
    console.error("save_note error:", e);
  }
};

const cancelEditNote = (folder) => {
  editingNotes[folder] = false;
};

// ── Post-MVP: export ─────────────────────────────────────────────────────────
const exportList = async () => {
  if (!addonsPath) return;
  try {
    exportedJson.value = await invoke("export_addon_list", { addonsPath });
  } catch (e) {
    console.error("export error:", e);
    alert(`Export failed: ${e}`);
  }
};

const copyExport = async () => {
  try {
    await navigator.clipboard.writeText(exportedJson.value);
  } catch {
    // fallback
    const ta = document.createElement('textarea');
    ta.value = exportedJson.value;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
  }
};

onMounted(async () => {
  await loadSettings();
  await scan();
  await loadNotes();
});

// Expose refresh for parent
defineExpose({ refresh: scan });
</script>
