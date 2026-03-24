<template>
  <div
    class="flex items-start gap-3 rounded-xl bg-white p-3 shadow ring-1 ring-black/5
           dark:bg-zinc-900 dark:ring-white/10
           transition-all duration-200 ease-in-out"
    :class="{ 'hover:ring-2 hover:ring-blue-500': !installed }"
  >
    <!-- Addon image (only for search results) -->
    <img
      v-if="image"
      :src="image"
      class="h-14 w-14 flex-shrink-0 rounded-md object-cover"
      @error="imageError = true"
    />

    <!-- Placeholder icon for installed addons without image -->
    <div
      v-else
      class="h-14 w-14 flex-shrink-0 rounded-md bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center text-zinc-400 dark:text-zinc-600 text-xl font-bold"
    >
      {{ (title || folder || '?')[0].toUpperCase() }}
    </div>

    <!-- Content -->
    <div class="flex flex-col flex-1 min-w-0">
      <div class="flex items-center justify-between gap-2">
        <h3 class="text-sm font-semibold text-zinc-900 dark:text-zinc-100 truncate">
          {{ title || folder }}
        </h3>

        <!-- Version badge -->
        <span
          v-if="version"
          class="text-xs text-zinc-400 dark:text-zinc-500 whitespace-nowrap shrink-0"
        >
          v{{ version }}
        </span>
      </div>

      <p class="mt-0.5 text-xs leading-5 text-zinc-600 dark:text-zinc-300 line-clamp-2">
        {{ description || notes }}
      </p>

      <div class="mt-1.5 flex items-center gap-2 flex-wrap">
        <!-- Author -->
        <div
          v-if="author"
          class="flex items-center gap-1 text-xs text-zinc-500 dark:text-zinc-400"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 12c2.21 0 4-1.79 4-4S14.21 4 12 4s-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"/>
          </svg>
          <span>{{ author }}</span>
        </div>

        <!-- Update available badge -->
        <span
          v-if="updateAvailable"
          class="px-1.5 py-0.5 text-xs font-medium bg-amber-100 text-amber-700 rounded-full dark:bg-amber-900/40 dark:text-amber-400"
        >
          Update: v{{ updateAvailable }}
        </span>
      </div>
    </div>

    <!-- Action buttons -->
    <div class="flex flex-col gap-1.5 shrink-0 ml-1">
      <!-- Installed badge (search results, already installed) -->
      <span
        v-if="!installed && isInstalled"
        class="px-3 py-1.5 text-xs font-medium text-green-600 bg-green-50 rounded-lg
               dark:text-green-400 dark:bg-green-900/30 whitespace-nowrap"
      >
        Installed
      </span>

      <!-- Install button (search results) -->
      <button
        v-else-if="!installed && onInstall"
        @click.stop="onInstall()"
        :disabled="installing"
        class="px-3 py-1.5 text-xs font-medium text-white bg-blue-500 rounded-lg
               hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed
               transition-all duration-200 whitespace-nowrap flex items-center gap-1.5 cursor-pointer"
      >
        <Spinner v-if="installing" />
        <span>{{ installing ? 'Installing…' : 'Install' }}</span>
      </button>

      <!-- Update button (installed with update) -->
      <button
        v-if="installed && updateAvailable && onUpdate"
        @click.stop="onUpdate()"
        :disabled="updating"
        class="px-3 py-1.5 text-xs font-medium text-white bg-amber-500 rounded-lg
               hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed
               transition-all duration-200 whitespace-nowrap"
      >
        <span v-if="updating">Updating…</span>
        <span v-else>Update</span>
      </button>

      <!-- Remove button (installed) -->
      <button
        v-if="installed && onRemove"
        @click.stop="onRemove()"
        :disabled="disabled"
        class="px-3 py-1.5 text-xs font-medium text-zinc-500 bg-zinc-100 rounded-lg
               hover:bg-zinc-200 hover:text-zinc-700 disabled:opacity-50 disabled:cursor-not-allowed
               dark:bg-zinc-800 dark:text-zinc-400 dark:hover:bg-zinc-700 dark:hover:text-zinc-300
               transition-all duration-200 whitespace-nowrap"
      >
        Remove
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, defineProps } from "vue";
import Spinner from "@/components/Spinner.vue";

const props = defineProps({
  // Common
  image:       String,
  title:       String,
  description: String,
  author:      String,

  // Browse results
  isInstalled:     Boolean,
  installing:      Boolean,

  // Installed addons
  installed:       Boolean,
  folder:          String,
  version:         String,
  notes:           String,
  updateAvailable: String,

  // State
  disabled:  Boolean,

  // Callbacks
  onInstall: Function,
  onUpdate:  Function,
  onRemove:  Function,
});

const imageError  = ref(false);
const updating    = ref(false);
</script>
