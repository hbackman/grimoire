<template>
  <div class="flex gap-2 flex-wrap items-center justify-between">
    <!-- Version chips -->
    <div class="flex gap-2 flex-wrap">
      <button
        v-for="version in versions"
        :key="version.value"
        @click="selectVersion(version.value)"
        :class="[
          'px-3 py-1.5 text-xs font-medium rounded-full transition-all duration-200 ease-in-out',
          selectedVersion === version.value
            ? 'bg-blue-500 text-white shadow-sm'
            : 'bg-white text-zinc-700 ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50 dark:bg-zinc-800 dark:text-zinc-300 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700'
        ]"
      >
        {{ version.label }}
      </button>
    </div>

    <!-- Installed toggle and settings -->
    <div class="flex gap-2 items-center">
      <button
        @click="toggleInstalled"
        :class="[
          'px-3 py-1.5 text-xs font-medium rounded-full transition-all duration-200 ease-in-out flex items-center gap-1.5',
          showInstalled
            ? 'bg-green-500 text-white shadow-sm'
            : 'bg-white text-zinc-700 ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50 dark:bg-zinc-800 dark:text-zinc-300 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700'
        ]"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
        </svg>
        Installed
      </button>

      <!-- Settings gear icon -->
      <button
        @click="openSettings"
        class="w-7 h-7 flex items-center justify-center rounded-full bg-white text-zinc-600 ring-1 ring-zinc-200
               hover:ring-zinc-300 hover:bg-zinc-50 hover:text-zinc-700
               dark:bg-zinc-800 dark:text-zinc-400 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700 dark:hover:text-zinc-300
               transition-all duration-200 ease-in-out"
        title="Settings"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue'

const props = defineProps({
  versions: {
    type: Array,
    required: true
  },
  selectedVersion: {
    type: Number,
    default: null
  },
  showInstalled: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:selectedVersion', 'update:showInstalled', 'open-settings'])

const selectVersion = (value) => {
  emit('update:selectedVersion', value)
}

const toggleInstalled = () => {
  emit('update:showInstalled', !props.showInstalled)
}

const openSettings = () => {
  emit('open-settings')
}
</script>