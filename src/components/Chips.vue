<template>
  <div class="flex gap-2 flex-wrap items-center justify-between">
    <!-- Tab chips -->
    <div class="flex gap-2 flex-wrap">
      <button
        @click="setView('browse')"
        :class="[
          'px-3 py-1.5 text-xs font-medium rounded-full transition-all duration-200 ease-in-out flex items-center gap-1.5',
          currentView === 'browse'
            ? 'bg-blue-500 text-white shadow-sm'
            : 'bg-white text-zinc-700 ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50 dark:bg-zinc-800 dark:text-zinc-300 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700'
        ]">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"/>
        </svg>
        Browse
      </button>

      <button
        @click="setView('installed')"
        :class="[
          'px-3 py-1.5 text-xs font-medium rounded-full transition-all duration-200 ease-in-out flex items-center gap-1.5',
          currentView === 'installed'
            ? 'bg-green-500 text-white shadow-sm'
            : 'bg-white text-zinc-700 ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50 dark:bg-zinc-800 dark:text-zinc-300 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700'
        ]">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
        </svg>
        Installed
      </button>

      <button
        @click="setView('profiles')"
        :class="[
          'px-3 py-1.5 text-xs font-medium rounded-full transition-all duration-200 ease-in-out flex items-center gap-1.5',
          currentView === 'profiles'
            ? 'bg-purple-500 text-white shadow-sm'
            : 'bg-white text-zinc-700 ring-1 ring-zinc-200 hover:ring-zinc-300 hover:bg-zinc-50 dark:bg-zinc-800 dark:text-zinc-300 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700'
        ]">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="currentColor">
          <path d="M3 18h18v-2H3v2zm0-5h18v-2H3v2zm0-7v2h18V6H3z"/>
        </svg>
        Profiles
      </button>
    </div>

    <!-- Settings gear -->
    <div class="flex gap-2 items-center">
      <button
        @click="openSettings"
        class="w-7 h-7 flex items-center justify-center rounded-full bg-white text-zinc-600 ring-1 ring-zinc-200
               hover:ring-zinc-300 hover:bg-zinc-50 hover:text-zinc-700
               dark:bg-zinc-800 dark:text-zinc-400 dark:ring-zinc-700 dark:hover:ring-zinc-600 dark:hover:bg-zinc-700 dark:hover:text-zinc-300
               transition-all duration-200 ease-in-out"
        title="Settings">
        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { defineProps, defineEmits } from 'vue';

const props = defineProps({
  currentView: {
    type: String,
    default: 'browse',
  },
});

const emit = defineEmits([
  'update:currentView',
  'open-settings',
]);

const setView = (view) => {
  emit('update:currentView', view);
};

const openSettings = () => {
  emit('open-settings');
};
</script>
