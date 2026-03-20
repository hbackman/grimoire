<template>
  <div>
    <h2 class="text-base font-semibold text-zinc-900 dark:text-zinc-100 mb-4">
      Addon Profiles
    </h2>

    <!-- Create new profile -->
    <div class="mb-5 p-4 rounded-xl bg-zinc-50 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700">
      <p class="text-xs font-medium text-zinc-600 dark:text-zinc-400 mb-2">Save current addons as a profile</p>
      <div class="flex gap-2">
        <input
          v-model="newProfileName"
          type="text"
          placeholder="Profile name (e.g. Raiding, Leveling)"
          class="flex-1 px-3 py-1.5 text-sm rounded-lg border border-zinc-200 dark:border-zinc-700
                 bg-white dark:bg-zinc-900 text-zinc-900 dark:text-zinc-100
                 focus:outline-none focus:ring-2 focus:ring-amber-400"
          @keydown.enter="createProfile"
        />
        <button
          @click="createProfile"
          :disabled="!newProfileName.trim() || saving"
          class="px-3 py-1.5 text-xs font-medium text-white bg-amber-500 rounded-lg
                 hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
        >
          Save
        </button>
      </div>
    </div>

    <!-- Profile list -->
    <div v-if="Object.keys(profiles).length === 0" class="text-center py-8 text-zinc-400 dark:text-zinc-600">
      <p class="text-sm">No profiles saved yet</p>
      <p class="text-xs mt-1">Create a profile to snapshot your current addon setup</p>
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="(folders, name) in profiles"
        :key="name"
        class="p-4 rounded-xl border border-zinc-200 dark:border-zinc-700 bg-white dark:bg-zinc-800"
      >
        <div class="flex items-start justify-between">
          <div>
            <p class="text-sm font-medium text-zinc-900 dark:text-zinc-100">{{ name }}</p>
            <p class="text-xs text-zinc-500 dark:text-zinc-400 mt-0.5">
              {{ folders.length }} addon{{ folders.length !== 1 ? 's' : '' }}:
              {{ folders.slice(0, 5).join(', ') }}{{ folders.length > 5 ? ` +${folders.length - 5} more` : '' }}
            </p>
          </div>
          <div class="flex gap-2 ml-3">
            <button
              @click="deleteProfile(name)"
              class="px-2 py-1 text-xs font-medium text-red-600 dark:text-red-400
                     hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-all"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { Store } from '@tauri-apps/plugin-store';

const profiles      = ref({});
const newProfileName = ref('');
const saving        = ref(false);
let addonsPath       = '';

const loadSettings = async () => {
  const store = await Store.load('settings.json');
  addonsPath  = await store.get('gameAddonPath') ?? '';
};

const loadProfiles = async () => {
  if (!addonsPath) return;
  try {
    profiles.value = await invoke('load_profiles', { addonsPath });
  } catch (e) {
    console.error('load_profiles error:', e);
  }
};

const createProfile = async () => {
  const name = newProfileName.value.trim();
  if (!name || !addonsPath) return;
  saving.value = true;
  try {
    // Snapshot current installed addons
    const addons = await invoke('scan_addons', { path: addonsPath });
    const folders = addons.map(a => a.folder);
    await invoke('save_profile', { addonsPath, name, folders });
    newProfileName.value = '';
    await loadProfiles();
  } catch (e) {
    console.error('save_profile error:', e);
    alert(`Failed to save profile: ${e}`);
  } finally {
    saving.value = false;
  }
};

const deleteProfile = async (name) => {
  if (!confirm(`Delete profile "${name}"?`)) return;
  try {
    await invoke('delete_profile', { addonsPath, name });
    await loadProfiles();
  } catch (e) {
    console.error('delete_profile error:', e);
  }
};

onMounted(async () => {
  await loadSettings();
  await loadProfiles();
});
</script>
