<template>
  <main class="p-4" style="max-width: 500px; margin: 0 auto;">
    <Search
      v-model="search"
      class="mb-3"
    />

    <VersionChips
      :versions="availableVersions"
      :selectedVersion="selectedVersion"
      :showInstalled="showInstalled"
      @update:selectedVersion="selectedVersion = $event"
      @update:showInstalled="showInstalled = $event"
      class="mb-4"
    />

    <!-- Loading skeletons -->
    <template v-if="loading">
      <AddonSkeleton
        v-for="n in 5"
        :key="`skeleton-${n}`"
        class="mb-3"
      />
    </template>

    <!-- Actual addons -->
    <template v-else>
      <Addon
        v-for="addon in addons"
        :key="addon.name"
        :image="addon.image"
        :author="addon.author"
        :title="addon.title"
        :description="addon.description"
        class="mb-3"
      />

      <!-- Loading more indicator -->
      <AddonSkeleton
        v-if="loadingMore"
        class="mb-3"
      />
    </template>

  </main>

  <ScrollToTopButton />
</template>

<script setup>
import {
  onMounted,
  onUnmounted,
  watch,
  ref,
} from "vue";

import {browse, versions} from "@/lib/curseforge.js";
import Addon              from "@/components/Addon.vue";
import AddonSkeleton      from "@/components/AddonSkeleton.vue";
import ScrollToTopButton  from "@/components/ScrollToTopButton.vue";
import Search             from "@/components/Search.vue";
import VersionChips       from "@/components/VersionChips.vue";

const addons = ref([]);
const search = ref("");
const loading = ref(false);
const loadingMore = ref(false);
const currentPage = ref(1);
const hasMoreResults = ref(true);
const availableVersions = ref(versions());
const selectedVersion = ref(517); // Default to Retail
const showInstalled = ref(false);

let debounceTimer = null;

const performSearch = async (resetResults = true) => {
  if (resetResults) {
    loading.value = true;
    currentPage.value = 1;
    hasMoreResults.value = true;
  } else {
    loadingMore.value = true;
  }

  try {
    const results = await browse(currentPage.value, 20, search.value || undefined);

    if (resetResults) {
      addons.value = results;
    } else {
      addons.value = [...addons.value, ...results];
    }

    // If we got fewer results than requested, we've reached the end
    hasMoreResults.value = results.length === 20;

  } catch (error) {
    console.error("Search error:", error);
  } finally {
    loading.value = false;
    loadingMore.value = false;
  }
};

const loadMore = async () => {
  if (loadingMore.value || loading.value || !hasMoreResults.value) {
    return;
  }

  currentPage.value++;
  await performSearch(false);
};

// Scroll detection for infinite scroll
const handleScroll = () => {
  const scrollHeight = document.documentElement.scrollHeight;
  const scrollTop = document.documentElement.scrollTop;
  const clientHeight = document.documentElement.clientHeight;

  // Trigger load more when user is 200px from bottom
  if (scrollTop + clientHeight >= scrollHeight - 200) {
    loadMore();
  }
};

watch(search, () => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = setTimeout(() => performSearch(true), 300);
}, { immediate: false });

watch(selectedVersion, () => {
  performSearch(true);
});

watch(showInstalled, () => {
  performSearch(true);
});

onMounted(() => {
  performSearch(true);
  window.addEventListener('scroll', handleScroll);
});

onUnmounted(() => {
  window.removeEventListener('scroll', handleScroll);
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
});
</script>

<style>
@import "tailwindcss";
</style>
