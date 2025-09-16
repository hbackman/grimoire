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

    <!-- Installed toggle -->
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

const emit = defineEmits(['update:selectedVersion', 'update:showInstalled'])

const selectVersion = (value) => {
  emit('update:selectedVersion', value)
}

const toggleInstalled = () => {
  emit('update:showInstalled', !props.showInstalled)
}
</script>