<template>
  <main>
    <Addon
      v-for="addon in addons"
      :key="addon.name"
      :image="addon.image"
      :author="addon.author"
      :name="addon.name"
      :description="addon.description"
      class="mb-2"
    />
  </main>
</template>

<script setup>
import {onMounted} from "vue";
import {ref}       from "vue";
import {browse}    from "@/lib/curseforge.js";
import Addon       from "@/components/Addon.vue";

const addons = ref([]);

onMounted(() => {
  browse()
    .then(a => {
      addons.value = a;
    })
    .catch(() => {
      console.error("error");
    })
});
</script>

<style>
@import "tailwindcss";
</style>
