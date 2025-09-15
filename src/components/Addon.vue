<template>
  <div class="flex items-start gap-3 rounded-xl bg-white p-3 shadow ring-1 ring-black/5
             dark:bg-zinc-900 dark:ring-white/10
             hover:ring-2 hover:ring-blue-500
             transition-all duration-200 ease-in-out">
    <img
      :src="image"
      class="h-14 w-14 flex-shrink-0 rounded-md object-cover"
    />

    <div class="flex flex-col">
      <h3 class="text-sm font-semibold text-zinc-900 dark:text-zinc-100">
        {{ name }}
      </h3>

      <p class="mt-0.5 text-xs leading-5 text-zinc-600 dark:text-zinc-300">
        {{ description }}
      </p>

      <div class="mt-1 flex items-center gap-1 text-xs text-zinc-500 dark:text-zinc-400">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 12c2.21 0 4-1.79 4-4S14.21 4 12 4s-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
        </svg>
        <span>{{ author }}</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import {defineProps} from "vue";

defineProps({
  image: String,
  author: String,
  name: String,
  description: String,
});
</script>
