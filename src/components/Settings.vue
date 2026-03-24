<template>
  <main class="p-4" style="max-width: 500px; margin: 0 auto;">
    <!-- Header with back button -->
    <div class="flex items-center gap-3 mb-6">
      <button
        @click="$emit('back')"
        class="flex items-center justify-center w-8 h-8 rounded-full bg-white ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50
               dark:bg-zinc-800 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700
               transition-all duration-200 ease-in-out">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-zinc-700 dark:text-zinc-300" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <h1 class="text-xl font-semibold text-zinc-900 dark:text-zinc-100">Settings</h1>
    </div>

    <div class="space-y-6">

      <!-- WoW AddOns Path -->
      <div class="bg-white rounded-xl p-4 shadow ring-1 ring-black/5 dark:bg-zinc-900 dark:ring-white/10">
        <label class="block text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-2">
          WoW AddOns Path
        </label>

        <!-- Validation indicator -->
        <div
          v-if="pathValidation"
          :class="[
            'mb-2 flex items-center gap-1.5 text-xs font-medium',
            pathValid ? 'text-green-600 dark:text-green-400' : 'text-red-500 dark:text-red-400',
          ]"
        >
          <span v-if="pathValid">✓ {{ pathValidation }}</span>
          <span v-else>✗ {{ pathValidation }}</span>
        </div>

        <div class="flex gap-2">
          <input
            v-model="gameAddonPath"
            type="text"
            placeholder="e.g., /Applications/World of Warcraft/_classic_/Interface/AddOns"
            class="flex-1 px-3 py-2 text-sm bg-white border border-zinc-200 rounded-lg
                   focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
                   dark:bg-zinc-800 dark:border-zinc-700 dark:text-zinc-100 dark:placeholder-zinc-400
                   transition-all duration-200"
          />
          <button
            @click="browsePath"
            class="px-3 py-2 text-xs font-medium text-zinc-700 bg-white border border-zinc-200 rounded-lg
                   hover:bg-zinc-50 hover:border-zinc-300 focus:outline-none focus:ring-2 focus:ring-blue-500
                   dark:bg-zinc-800 dark:text-zinc-300 dark:border-zinc-700 dark:hover:bg-zinc-700 dark:hover:border-zinc-600
                   transition-all duration-200">
            Browse
          </button>
        </div>

        <p class="mt-2 text-xs text-zinc-500 dark:text-zinc-400">
          Point to your <code class="font-mono">_classic_/Interface/AddOns</code> folder, or your WoW installation root (Grimoire will find the Classic folder automatically).
        </p>

        <!-- Dev helper: create test dir -->
        <div class="mt-3 pt-3 border-t border-zinc-100 dark:border-zinc-800">
          <p class="text-xs text-zinc-400 dark:text-zinc-600 mb-2">Developer: create a test AddOns directory</p>
          <div class="flex gap-2">
            <input
              v-model="testDirBase"
              type="text"
              placeholder="/tmp"
              class="flex-1 px-3 py-2 text-sm bg-white border border-zinc-200 rounded-lg
                     focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
                     dark:bg-zinc-800 dark:border-zinc-700 dark:text-zinc-100 dark:placeholder-zinc-400
                     transition-all duration-200"
            />
            <button
              @click="createTestDir"
              class="px-3 py-2 text-xs font-medium text-zinc-700 bg-zinc-50 border border-zinc-200 rounded-lg
                     hover:bg-zinc-100 hover:border-zinc-300
                     dark:bg-zinc-800 dark:text-zinc-300 dark:border-zinc-700 dark:hover:bg-zinc-700
                     transition-all duration-200">
              Create
            </button>
          </div>
          <p v-if="testDirResult" class="mt-1 text-xs text-green-600 dark:text-green-400 font-mono">
            {{ testDirResult }}
          </p>
        </div>
      </div>

      <!-- Game Version -->
      <div class="bg-white rounded-xl p-4 shadow ring-1 ring-black/5 dark:bg-zinc-900 dark:ring-white/10">
        <label class="block text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-2">
          Game Version
        </label>
        <div class="relative">
          <select
            v-model="gameVersion"
            class="w-full px-3 py-2 pr-10 text-sm bg-white border border-zinc-200 rounded-lg
                   focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
                   dark:bg-zinc-800 dark:border-zinc-700 dark:text-zinc-100
                   transition-all duration-200 appearance-none"
          >
            <option value="">Select a version</option>
            <option
              v-for="v in gameVersions"
              :key="v.value"
              :value="v.value"
            >
              {{ v.label }}
            </option>
          </select>
          <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-zinc-400" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>
        <p class="mt-2 text-xs text-zinc-500 dark:text-zinc-400">
          Select which WoW version to download addons for.
        </p>
      </div>

      <!-- Save button -->
      <button
        @click="saveSettings"
        class="w-full px-4 py-2 text-sm font-medium text-white bg-blue-500 rounded-lg
               hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
               dark:focus:ring-offset-zinc-900 transition-all duration-200">
        Save Settings
      </button>

    </div>
  </main>
</template>

<script setup>
import { ref, watch }  from "vue";
import { onMounted }   from "vue";
import { Store }       from "@tauri-apps/plugin-store";
import { open }        from "@tauri-apps/plugin-dialog";
import { invoke }      from "@tauri-apps/api/core";
import { versions, DEFAULT_GAME_VERSION } from "../lib/curseforge.js";

const emit = defineEmits(["back"]);

let store = null;

const gameAddonPath   = ref("");
const gameVersion     = ref(DEFAULT_GAME_VERSION);
const gameVersions    = ref(versions());
const pathValidation  = ref("");
const pathValid       = ref(false);
const testDirBase     = ref("/tmp");
const testDirResult   = ref("");

// Validate path on change (debounced)
let validateTimer = null;
watch(gameAddonPath, (val) => {
  clearTimeout(validateTimer);
  if (!val) { pathValidation.value = ""; return; }
  validateTimer = setTimeout(() => validatePath(val), 500);
});

const VARIANT_LABELS = {
  addons_dir:   "Valid AddOns directory",
  classic:      "WoW Classic detected ✓",
  classic_era:  "WoW Classic Era detected ✓",
  classic_ptr:  "WoW Classic PTR detected",
  retail:       "WoW Retail detected (note: this is a Classic addon manager)",
  ptr:          "WoW PTR detected",
};

const validatePath = async (path) => {
  try {
    const result = await invoke("validate_wow_path", { path });
    pathValidation.value = VARIANT_LABELS[result] ?? `Detected: ${result}`;
    pathValid.value = true;
  } catch (e) {
    pathValidation.value = typeof e === "string" ? e : (e?.message ?? String(e));
    pathValid.value = false;
  }
};

const browsePath = async () => {
  const selected = await open({ directory: true, multiple: false });
  if (selected) {
    gameAddonPath.value = selected;
  }
};

const saveSettings = async () => {
  await store.set("gameAddonPath", gameAddonPath.value ?? "");
  await store.set("gameVersion",   gameVersion.value   ?? 517);
  await store.save();
  emit("back");
};

const createTestDir = async () => {
  try {
    const path = await invoke("create_test_addon_dir", { basePath: testDirBase.value });
    testDirResult.value   = path;
    gameAddonPath.value   = path;
  } catch (e) {
    testDirResult.value = "Error: " + (e.message || e);
  }
};

onMounted(async () => {
  store = await Store.load("settings.json");
  const storedPath    = await store.get("gameAddonPath");
  const storedVersion = await store.get("gameVersion");
  if (storedPath)    gameAddonPath.value = storedPath;
  if (storedVersion !== null && storedVersion !== undefined)
    gameVersion.value = storedVersion;
  if (gameAddonPath.value) validatePath(gameAddonPath.value);

  // Use cached versions (populated at app boot)
  gameVersions.value = versions();
});
</script>
