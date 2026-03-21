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

      <div class="flex gap-2">
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
      <Addon
        v-for="addon in addons"
        :key="addon.folder"
        :installed="true"
        :image="addonImages[addon.folder]"
        :title="addon.title"
        :folder="addon.folder"
        :version="addon.version"
        :notes="addon.notes"
        :author="addon.author"
        :update-available="addon.updateAvailable"
        :on-remove="() => removeAddon(addon)"
        :on-update="addon.updateAvailable ? () => updateAddon(addon) : null"
        class="mb-3"
      />
    </template>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from "vue";
import { invoke }                   from "@tauri-apps/api/core";
import { Store }                    from "@tauri-apps/plugin-store";
import { ask }                      from "@tauri-apps/plugin-dialog";
import { getLatestVersion, getAddonDownloadUrl, DEFAULT_GAME_VERSION } from "@/lib/curseforge.js";

import Addon from "@/components/Addon.vue";

const emit = defineEmits(["refresh"]);

const addons          = ref([]);
const addonImages     = ref({});
const checkingUpdates = ref(false);
const updatingAll     = ref(false);
const updateProgress  = ref("");

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
  addonImages.value = (await store.get("addonImages")) ?? {};
};

const scan = async () => {
  if (!addonsPath) return;
  try {
    const result = await invoke("scan_addons", { path: addonsPath });
    addons.value = result;
  } catch (e) {
    console.error("scan_addons error:", e);
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
    // Use the WebView-based downloader so the 5-second CurseForge countdown
    // fires naturally and we intercept the actual file download.
    await invoke("download_addon_via_webview", {
      downloadPageUrl: downloadUrl,
      addonsPath,
    });
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
  const yes = await ask(`Remove ${addon.title}?`, { title: "Confirm Removal", kind: "warning" });
  if (!yes) return;
  try {
    await invoke("remove_addon", { addonsPath, folder: addon.folder });
    delete addonImages.value[addon.folder];
    await store.set("addonImages", addonImages.value);
    await store.save();
    await scan();
  } catch (e) {
    console.error("Remove error:", e);
    alert(`Failed to remove ${addon.title}: ${e.message || e}`);
  }
};

onMounted(async () => {
  await loadSettings();
  await scan();
});

// Expose refresh for parent
defineExpose({ refresh: scan });
</script>
