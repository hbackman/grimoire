<template>
  <!-- Settings page -->
  <Settings
    v-if="currentView === 'settings'"
    @back="onSettingsBack"
  />

  <!-- Main app -->
  <div v-else>
    <main class="p-4" style="max-width: 540px; margin: 0 auto;">

      <!-- First-run banner: no WoW path configured -->
      <div
        v-if="!addonsPath"
        class="mb-4 p-4 rounded-xl bg-amber-50 ring-1 ring-amber-200 dark:bg-amber-900/20 dark:ring-amber-700/50"
      >
        <p class="text-sm font-medium text-amber-800 dark:text-amber-300">
          👋 Welcome to Grimoire!
        </p>
        <p class="mt-1 text-xs text-amber-700 dark:text-amber-400">
          Configure your WoW AddOns path to get started.
        </p>
        <button
          @click="currentView = 'settings'"
          class="mt-2 px-3 py-1.5 text-xs font-medium text-white bg-amber-500 rounded-lg hover:bg-amber-600 transition-all duration-200"
        >
          Open Settings
        </button>
      </div>

      <!-- Search bar (only on browse tab) -->
      <Search
        v-if="mainView === 'browse'"
        v-model="search"
        class="mb-3"
      />

      <!-- Tab chips -->
      <Chips
        :current-view="mainView"
        @update:current-view="mainView = $event"
        @open-settings="currentView = 'settings'"
        class="mb-4"
      />

      <!-- ── Browse tab ── -->
      <template v-if="mainView === 'browse'">
        <!-- Loading skeletons -->
        <template v-if="loading">
          <AddonSkeleton v-for="n in 5" :key="`sk-${n}`" class="mb-3" />
        </template>

        <!-- Search results -->
        <template v-else>
          <Addon
            v-for="addon in searchResults"
            :key="addon.name"
            :image="addon.image"
            :author="addon.author"
            :title="addon.title"
            :description="addon.description"
            :is-installed="installedSlugs.has(addon.name)"
            :installing="installingSlug === addon.name"
            :on-install="addonsPath && !installedSlugs.has(addon.name) ? () => installAddon(addon) : null"
            class="mb-3"
          />

          <!-- Loading more -->
          <AddonSkeleton v-if="loadingMore" class="mb-3" />

          <!-- End of results -->
          <p
            v-if="!hasMoreResults && searchResults.length"
            class="text-center text-xs text-zinc-400 dark:text-zinc-600 py-4"
          >
            No more results
          </p>
        </template>
      </template>

      <!-- ── Installed tab ── -->
      <InstalledView
        v-else-if="mainView === 'installed'"
        ref="installedView"
      />

    </main>

    <ScrollToTopButton />
  </div>
</template>

<script setup>
import {
  onMounted,
  onUnmounted,
  watch,
  ref,
} from "vue";

import { Store }       from "@tauri-apps/plugin-store";

import {
  browse,
  installAddon as cfInstallAddon,
  getLatestVersion,
  fetchGameVersions,
  DEFAULT_GAME_VERSION,
} from "@/lib/curseforge.js";

import Addon          from "@/components/Addon.vue";
import AddonSkeleton  from "@/components/AddonSkeleton.vue";
import ScrollToTopButton from "@/components/ScrollToTopButton.vue";
import Search         from "@/components/Search.vue";
import Settings       from "@/components/Settings.vue";
import Chips          from "@/components/Chips.vue";
import InstalledView  from "@/components/InstalledView.vue";

// ── State ──────────────────────────────────────────────────────────────────

const currentView    = ref("main");   // main | settings
const mainView       = ref("browse"); // browse | installed

const search         = ref("");
const searchResults  = ref([]);
const loading        = ref(false);
const loadingMore    = ref(false);
const currentPage    = ref(1);
const hasMoreResults = ref(true);
const installingSlug = ref(null);
const installedSlugs = ref(new Set());

const addonsPath  = ref("");
const gameVersion = ref(DEFAULT_GAME_VERSION);

const installedView = ref(null); // ref to InstalledView component

let debounceTimer = null;
let store         = null;

// ── Settings ───────────────────────────────────────────────────────────────

const loadSettings = async () => {
  store         = await Store.load("settings.json");
  addonsPath.value  = await store.get("gameAddonPath") ?? "";
  const storedVersion = await store.get("gameVersion");
  gameVersion.value = (storedVersion !== null && storedVersion !== undefined)
    ? storedVersion
    : DEFAULT_GAME_VERSION;

  const manifest = (await store.get("addonManifest")) ?? {};
  installedSlugs.value = new Set(Object.keys(manifest));
};

const onSettingsBack = async () => {
  await loadSettings();
  currentView.value = "main";
  // Refresh installed list if on that tab
  if (mainView.value === "installed") {
    installedView.value?.refresh();
  }
};

// ── Browse / search ────────────────────────────────────────────────────────

const performSearch = async (resetResults = true) => {
  if (resetResults) {
    loading.value        = true;
    currentPage.value    = 1;
    hasMoreResults.value = true;
  } else {
    loadingMore.value = true;
  }

  try {
    const results = await browse({
      page:   currentPage.value,
      size:   20,
      search: search.value || undefined,
    });

    if (resetResults) {
      searchResults.value = results;
    } else {
      searchResults.value = [...searchResults.value, ...results];
    }

    hasMoreResults.value = results.length === 20;
  } catch (error) {
    console.error("Search error:", error);
  } finally {
    loading.value     = false;
    loadingMore.value = false;
  }
};

const loadMore = async () => {
  if (loadingMore.value || loading.value || !hasMoreResults.value) return;
  currentPage.value++;
  await performSearch(false);
};

const handleScroll = () => {
  const { scrollHeight, scrollTop, clientHeight } = document.documentElement;
  if (scrollTop + clientHeight >= scrollHeight - 200) {
    loadMore();
  }
};

watch(search, () => {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => performSearch(true), 300);
});

watch(mainView, async (v) => {
  if (v === "installed") {
    installedView.value?.refresh();
  }
  if (v === "browse") {
    // Refresh installed slugs in case addons were removed on the installed tab
    const manifest = (await store.get("addonManifest")) ?? {};
    installedSlugs.value = new Set(Object.keys(manifest));
  }
});

// ── Install from search results ────────────────────────────────────────────

const installAddon = async (addon) => {
  if (!addonsPath.value) {
    alert("Please configure your WoW AddOns path in Settings first.");
    return;
  }

  installingSlug.value = addon.name;
  try {
    // cfInstallAddon handles the CurseForge countdown via WebView download interception
    const folders = await cfInstallAddon(addon.name, gameVersion.value, addonsPath.value);
    console.log("Installed folders:", folders);

    // Persist addon manifest so installed view shows one entry per addon
    if (folders?.length) {
      // Fetch the CurseForge version so we can detect updates later
      const cfVersion = await getLatestVersion(addon.name, gameVersion.value).catch(() => null);

      const manifest = (await store.get("addonManifest")) ?? {};
      manifest[addon.name] = {
        slug:    addon.name,
        title:   addon.title,
        image:   addon.image || "",
        folders: folders,
        cfVersion: cfVersion || "",
      };
      await store.set("addonManifest", manifest);
      await store.save();
      installedSlugs.value = new Set(Object.keys(manifest));
    }
  } catch (e) {
    console.error("Install error:", e);
    alert(`Failed to install ${addon.title}:\n${e.message || e}`);
  } finally {
    installingSlug.value = null;
  }
};

// ── Lifecycle ──────────────────────────────────────────────────────────────

onMounted(async () => {
  await loadSettings();
  fetchGameVersions();  // warm the cache; don't await — runs in background
  performSearch(true);
  window.addEventListener("scroll", handleScroll);
});

onUnmounted(() => {
  window.removeEventListener("scroll", handleScroll);
  clearTimeout(debounceTimer);
});
</script>

<style>
@import "tailwindcss";
</style>
