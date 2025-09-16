<template>
  <main class="p-4" style="max-width: 500px; margin: 0 auto;">
    <!-- Header with back button -->
    <div class="flex items-center gap-3 mb-6">
      <button
        @click="$emit('back')"
        class="flex items-center justify-center w-8 h-8 rounded-full bg-white ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50
               dark:bg-zinc-800 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700
               transition-all duration-200 ease-in-out"
      >
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
            :value="addonPath"
            @input="$emit('update:addonPath', $event.target.value)"
            type="text"
            placeholder="e.g., /Applications/World of Warcraft/Interface/AddOns"
            class="flex-1 px-3 py-2 text-sm bg-white border border-zinc-200 rounded-lg
                   focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent
                   dark:bg-zinc-800 dark:border-zinc-700 dark:text-zinc-100 dark:placeholder-zinc-400
                   transition-all duration-200"
          />
          <button
            @click="browseForPath"
            class="px-3 py-2 text-xs font-medium text-zinc-700 bg-white border border-zinc-200 rounded-lg
                   hover:bg-zinc-50 hover:border-zinc-300 focus:outline-none focus:ring-2 focus:ring-blue-500
                   dark:bg-zinc-800 dark:text-zinc-300 dark:border-zinc-700 dark:hover:bg-zinc-700 dark:hover:border-zinc-600
                   transition-all duration-200"
          >
            Browse
          </button>
        </div>
        <p class="mt-2 text-xs text-zinc-500 dark:text-zinc-400">
          Path to your World of Warcraft AddOns folder where installed addons are stored.
        </p>
      </div>

      <!-- Save button -->
      <button
        @click="saveSettings"
        class="w-full px-4 py-2 text-sm font-medium text-white bg-blue-500 rounded-lg
               hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2
               dark:focus:ring-offset-zinc-900 transition-all duration-200"
      >
        Save Settings
      </button>
    </div>
  </main>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'

defineProps({
  addonPath: {
    type: String,
    default: ''
  }
})

const emit = defineEmits(['back', 'update:addonPath', 'save'])

const browseForPath = async () => {
  // This would use Tauri's file dialog API
  try {
    // For now, just emit an event - the parent can handle the actual file dialog
    emit('browse-path')
  } catch (error) {
    console.error('Failed to browse for path:', error)
  }
}

const saveSettings = () => {
  emit('save')
}
</script>