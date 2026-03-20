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
                 transition-all duration-200"
        >
          <span v-if="checkingUpdates">Checking…</span>
          <span v-else>Check for Updates</span>
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

    <!-- Update progress -->
    <div
      v-if="updateProgress"
      class="mb-3 px-3 py-2 text-xs text-zinc-600 dark:text-zinc-400 bg-zinc-50 dark:bg-zinc-800 rounded-lg"
    >
      {{ updateProgress }}
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
        :key="addon.folder"
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
import { ref, computed, onMounted, reactive } from "vue";
import { invoke }                              from "@tauri-apps/api/core";
import { Store }                               from "@tauri-apps/plugin-store";
import { getLatestVersion, getAddonDownloadUrl, DEFAULT_GAME_VERSION } from "@/lib/curseforge.js";

import Addon from "@/components/Addon.vue";

const emit = defineEmits(["refresh"]);

const addons          = ref([]);
const checkingUpdates = ref(false);
const updatingAll     = ref(false);
const updateProgress  = ref("");

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
    // Use scan_addons_with_disabled so we can show enabled/disabled state
    const result = await invoke("scan_addons_with_disabled", { path: addonsPath });
    addons.value = result;
  } catch (e) {
    // Fall back to basic scan if post-mvp command not available
    try {
      const result = await invoke("scan_addons", { path: addonsPath });
      addons.value = result;
    } catch (e2) {
      console.error("scan_addons error:", e2);
    }
  }
};

const checkUpdates = async () => {
  if (!addons.value.length) return;
  checkingUpdates.value = true;
  updateProgress.value  = "Checking for updates…";

  try {
    // We don't have a mapping from folder name → curseforge slug automatically.
    // This is a known limitation. We check by trying the lowercase folder name
    // as a slug — works for most popular addons (WeakAuras, Details, ElvUI…)
    const checks = addons.value.map(async (addon) => {
      const slug    = addon.folder.toLowerCase().replace(/_/g, "-");
      const latest  = await getLatestVersion(slug, gameVersion).catch(() => null);
      if (latest && latest !== addon.version) {
        addon.updateAvailable = latest;
      } else {
        addon.updateAvailable = null;
      }
    });

    // Batch sequentially to avoid hammering the scraper queue
    for (const check of checks) {
      await check;
    }

    const count = addonsWithUpdates.value.length;
    updateProgress.value = count
      ? `${count} update${count !== 1 ? "s" : ""} available`
      : "All addons are up to date";
  } finally {
    checkingUpdates.value = false;
  }
};

const updateAddon = async (addon) => {
  const slug = addon.folder.toLowerCase().replace(/_/g, "-");
  try {
    const { downloadUrl } = await getAddonDownloadUrl(slug, gameVersion);
    await invoke("install_addon_zip", { url: downloadUrl, addonsPath });
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

  updateProgress.value = "All updates complete!";
  updatingAll.value    = false;
};

const removeAddon = async (addon) => {
  if (!confirm(`Remove ${addon.title}?`)) return;
  try {
    await invoke("remove_addon", { addonsPath, folder: addon.folder });
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
