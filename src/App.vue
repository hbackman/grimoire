<template>
  <main class="p-4" style="max-width: 500px; margin: 0 auto;">
    <Search
      v-model="search"
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
        :name="addon.name"
        :description="addon.description"
        class="mb-3"
      />
    </template>
  </main>
</template>

<script setup>
import {onMounted, watch} from "vue";
import {ref}              from "vue";
import {browse}           from "@/lib/curseforge.js";
import Addon              from "@/components/Addon.vue";
import AddonSkeleton      from "@/components/AddonSkeleton.vue";
import Search             from "@/components/Search.vue";

const addons = ref([]);
const search = ref("");
const loading = ref(false);

let debounceTimer = null;

const performSearch = async () => {
  loading.value = true;
  try {
    const results = await browse(1, 20, search.value || undefined);
    addons.value = results;
  } catch (error) {
    console.error("Search error:", error);
  } finally {
    loading.value = false;
  }
};

watch(search, () => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }

  debounceTimer = setTimeout(performSearch, 300);
}, { immediate: false });

onMounted(() => {
  performSearch();
});
</script>

<style>
@import "tailwindcss";
</style>
