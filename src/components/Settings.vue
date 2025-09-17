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

    <!-- Settings form -->
    <div class="space-y-6">
      <!-- WoW Addon File Path -->
      <div class="bg-white rounded-xl p-4 shadow ring-1 ring-black/5 dark:bg-zinc-900 dark:ring-white/10">
        <label class="block text-sm font-medium text-zinc-900 dark:text-zinc-100 mb-2">
          WoW Addon File Path
        </label>
        <div class="flex gap-2">
          <input
            v-model="gameAddonPath"
            type="text"
            placeholder="e.g., /Applications/World of Warcraft/Interface/AddOns"
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
          Path to your World of Warcraft AddOns folder where installed addons are stored.
        </p>
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
                   dark:bg-zinc-800 dark:border-zinc-700 dark:text-zinc-100 dark:placeholder-zinc-400
                   transition-all duration-200 appearance-none"
          >
            <option value="">Select a version</option>
            <option
              v-for="version in gameVersions"
              :key="version.value"
              :value="version.value"
            >
              {{ version.label }}
            </option>
          </select>
          <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 text-zinc-400 dark:text-zinc-500" viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>
        <p class="mt-2 text-xs text-zinc-500 dark:text-zinc-400">
          Select which World of Warcraft version to download addons for.
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
import {Store}     from "@tauri-apps/plugin-store";
import {open}      from "@tauri-apps/plugin-dialog";
import {ref}       from "vue";
import {onMounted} from "vue";
import {versions}  from "../lib/curseforge.js";

const emit = defineEmits([
  "back",
]);

let store = null;

const gameAddonPath = ref("");
const gameVersion = ref(null);
const gameVersions = ref(versions());

/**
 * Browse for the addon path.
 */
const browsePath = async () => {
  // Browse path.
  const selected = await open({
    directory: true,
    multiple: false,
  });

  gameAddonPath.value = selected;
};

/**
 * Save the settings.
 */
const saveSettings = async () => {
  await store.set("gameAddonPath", gameAddonPath.value ?? "");
  await store.set("gameVersion",   gameVersion.value   ?? "");

  await store.save();

  emit("back");
};

onMounted(async () => {
  store = await Store.load("settings.json");

  gameAddonPath.value = await store.get("gameAddonPath");
  gameVersion.value   = await store.get("gameVersion");
});
</script>
