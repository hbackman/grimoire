<template>
  <main class="p-4" style="max-width: 500px; margin: 0 auto;">
    <Search
      v-model="search"
      class="mb-4"
    />
    <Addon
      v-for="addon in addons"
      :key="addon.name"
      :image="addon.image"
      :author="addon.author"
      :name="addon.name"
      :description="addon.description"
      class="mb-3"
    />
  </main>
</template>

<script setup>
import {onMounted, watch} from "vue";
import {ref}              from "vue";
import {browse}           from "@/lib/curseforge.js";
import Addon              from "@/components/Addon.vue";
import Search             from "@/components/Search.vue";

const addons = ref([]);
const search = ref("");

let debounceTimer = null;

const performSearch = async () => {
  try {
    const results = await browse(1, 20, search.value || undefined);
    addons.value = results;
  } catch (error) {
    console.error("Search error:", error);
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
